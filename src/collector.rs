use std::{thread, str::FromStr, collections::HashMap,
  sync::{Mutex, atomic::{AtomicU64, Ordering}}, cell::RefCell};
use slog::Drain;
use super::{event::Event, span::Span, Config};

// Tracks the currently executing span on a per-thread basis.
// adapted from https://github.com/tokio-rs/tracing/blob/master/examples/examples/sloggish/sloggish_subscriber.rs
#[derive(Clone)]
struct CurrentSpanPerThread {
  current: &'static thread::LocalKey<RefCell<Vec<tracing::Id>>>,
}

impl CurrentSpanPerThread {
  pub fn new() -> Self {
    thread_local! {
      static CURRENT: RefCell<Vec<tracing::Id>> = RefCell::new(vec![]);
    };
    Self { current: &CURRENT }
  }

  // Returns the id of the span in which the current thread is
  // executing, or `None` if it is not inside of a span.
  pub fn id(&self) -> Option<tracing::Id> {
    self.current
      .with(|current| current.borrow().last().cloned())
  }

  pub fn enter(&self, span: tracing::Id) {
    self.current.with(|current| {
      current.borrow_mut().push(span);
    })
  }

  pub fn exit(&self) {
    self.current.with(|current| {
      let _ = current.borrow_mut().pop();
    })
  }
}

impl Config {
  pub fn create_global_subscriber(&self) {
    let collector = Collector::new(self);
    tracing::subscriber::set_global_default(collector)
      .expect("setting tracing default failed");
  }
}

const AUTO_REJECT: u32 = 0;
const AUTO_KEEP: u32 = 1;

pub struct Collector {
  level: tracing::Level,
  prefix: String,
  spans: Mutex<HashMap<tracing::Id, Span>>,
  traces: Mutex<HashMap<u64, Vec<tracing::Id>>>,
  current: CurrentSpanPerThread,
  span_id: AtomicU64,
  logger: slog::Logger,
  dd_client: datadog_apm::Client,
  sample_rate: f32
}

impl Collector {
  pub fn new(config: &Config) -> Self {
    config.validate().unwrap();
    let level = tracing::Level::from_str(&config.level).expect("Invalid level.");
    let drain = slog_json::Json::new(std::io::stdout())
      .build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, 
      o!("dd.service" => config.dd.service.clone(), "dd.version" => config.dd.version.clone(), "dd.env" => config.dd.env.clone(),
        "metadata.environment" => config.metadata.env.clone(), "metadata.image" => config.metadata.image.clone())
    );
    let dd_apm_config = datadog_apm::Config {
      env: Some(config.dd.env.clone()),
      service: config.dd.service.clone(),
      host: config.dd.host.clone(),
      port: config.dd.port.clone(),
      ..Default::default()
    };
    Self {
      level,
      prefix: config.prefix.clone(),
      spans: Mutex::new(HashMap::new()),
      traces: Mutex::new(HashMap::new()),
      span_id: AtomicU64::new(1),
      current: CurrentSpanPerThread::new(),
      logger,
      sample_rate: config.sample_rate,
      dd_client: datadog_apm::Client::new(dd_apm_config)
    }
  }

  fn get_next_span_id(&self) -> u64 {
    self.span_id.fetch_add(1, Ordering::SeqCst)
  }

  fn get_next_trace_id(&self) -> u64 {
    rand::random()
  }

  fn sample(&self) -> u32 {
    let val: f32 = rand::random();
    if val > self.sample_rate {
      AUTO_REJECT
    }
    else {
      AUTO_KEEP
    }
  }
}

impl tracing::Subscriber for Collector {
  fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
    *metadata.level() <= self.level 
      && metadata.target().starts_with(&self.prefix)
  }
  fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::Id {
    let parent = self.current.id();
    let mut spans = self.spans.lock().unwrap();
    let trace_id = parent.as_ref().map(|parent_id| 
      spans.get(parent_id).map(|parent_span| parent_span.trace_id)
    ).flatten().unwrap_or_else(|| {
      self.get_next_trace_id()
    });
    let mut traces = self.traces.lock().unwrap();
    let trace_spans = traces.entry(trace_id)
      .or_insert_with(|| vec![]);
    let span_id = self.get_next_span_id();
    let span_id = tracing::Id::from_u64(span_id);
    let span = Span::new(parent, trace_id, span);
    spans.insert(span_id.clone(), span);
    trace_spans.push(span_id.clone());
    span_id
  }
  fn record(&self, span_id: &tracing::Id, values: &tracing::span::Record<'_>) {
    self.spans.lock().unwrap().get_mut(span_id)
      .map(|span| values.record(span));
  }
  fn record_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id) {
    // unimplemented
  }
  fn event(&self, event: &tracing::Event<'_>) {
    let parent_id = self.current.id();
    let mut spans = self.spans.lock().unwrap();
    // Option<Span>
    let parent = parent_id.as_ref().map(|p| spans.get_mut(p)).flatten();
    let metadata = event.metadata();
    let parent_ref = parent.as_ref();
    let mut ev = Event::new(metadata.target(), parent_ref.map(|p| p.name()), 
      parent_id.map(|p| p.into_u64()), parent_ref.map(|p| p.trace_id));
    event.record(&mut ev);
    let level = metadata.level();
    ev.log(level, &self.logger);

    // if event is an error, propagate its info to containing span
    if let Some(parent) = parent {
      if *level <= tracing::Level::WARN {
        for (key, value) in ev.data() {
          if key.starts_with("error") {
            parent.set_tag(key.to_owned(), value.to_owned())
          }
        }
      }
    }
  }
  fn enter(&self, span_id: &tracing::Id) {
    self.current.enter(span_id.clone());
    // set time when first entering the span
    self.spans.lock().unwrap()
      .get_mut(span_id).map(|span| span.set_time());
  }
  fn exit(&self, _span_id: &tracing::Id) {
    self.current.exit();
  }
  fn clone_span(&self, span_id: &tracing::Id) -> tracing::Id {
    self.spans.lock().unwrap()
      .get_mut(span_id).map(|span| span.increment_handlers());
    span_id.clone()
  }
  fn try_close(&self, span_id: tracing::Id) -> bool {
    let mut spans = self.spans.lock().unwrap();
    if let Some(span) = spans.get_mut(&span_id) {
      span.decrement_handlers();
      if span.is_closed() {
        span.set_duration();
        if span.parent == None { // if closing a parent, its trace is done
          // todo clear the current stack just in case?
          let mut traces = self.traces.lock().unwrap();
          let trace_id = span.trace_id;
          if let Some(trace) = traces.remove(&trace_id) {
            let trace_spans = trace.into_iter().filter_map(|span_id| 
              spans.remove(&span_id).map(|span| Span::into(span_id, span))
            ).collect();
            let trace = datadog_apm::Trace {
              id: trace_id,
              priority: self.sample(),
              spans: trace_spans
            };
            let client = self.dd_client.clone();
            client.send_trace(trace);     
          }
        }
        return true
      }
    }
    false
  }
}