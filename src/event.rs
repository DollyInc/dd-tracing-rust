use std::{collections::HashMap, fmt::Debug, time::{SystemTime, UNIX_EPOCH}};

#[derive(Default)]
pub struct Event {
  event: String,
  function: String,
  span_id: String,
  trace_id: String,
  data: HashMap<String, String>
}

impl Event {
  pub fn new(event: &str, function: Option<&str>, span_id: Option<u64>, trace_id: Option<u64>) -> Self {
    Self {
      // event and function are overridden in record_str with values passed to the event macro
      event: event.to_string(),
      function: function.map(|f| f.to_string()).unwrap_or_default(),
      span_id: span_id.map(|s| s.to_string()).unwrap_or_default(),
      trace_id: trace_id.map(|t| t.to_string()).unwrap_or_default(),
      ..Self::default()
    }
  }
  pub fn data(&self) -> &HashMap<String, String> {
    &self.data
  }
  // todo event metadata
  pub fn log(&self, level: &tracing::Level, logger: &slog::Logger) {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)
      .unwrap_or_default().as_millis();
    let kv = o!(
      "event" => self.event.as_str(),
      "function" => self.function.as_str(),
      "dd.span_id" => self.span_id.as_str(),
      "dd.trace_id" => self.trace_id.as_str(),
      "metadata.time" => time
    );
    let message = serde_json::to_string(&self.data).unwrap_or_default();
    match *level {
      tracing::Level::ERROR => error!(logger, "{}", message; o!(kv, "status" => "error")),
      tracing::Level::WARN => warn!(logger, "{}", message; o!(kv, "status" => "warn")),
      tracing::Level::INFO => info!(logger, "{}", message; o!(kv, "status" => "info")),
      tracing::Level::DEBUG => debug!(logger, "{}", message; o!(kv, "status" => "debug")),
      tracing::Level::TRACE => trace!(logger, "{}", message; o!(kv, "status" => "trace"))
    }
  }
}

impl tracing::field::Visit for Event {
  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    let name = field.name();
    match name {
      "event" => self.event = value.to_string(),
      "function" => self.function = value.to_string(),
      _ => {
       let _ = self.data.insert(name.to_string(), value.to_string());
      }
    };
  }
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
    self.data.insert(field.name().to_string(), format!("{:#?}", value));
  }
}