use std::{collections::HashMap, fmt::Debug};

#[derive(Default)]
pub struct Event {
  event: String,
  function: String,
  span_id: u64,
  trace_id: u64,
  data: HashMap<&'static str, String>
}

impl Event {
  pub fn new(event: &str, function: &str, span_id: u64, trace_id: u64) -> Self {
    Self {
      // event and function are overridden in record_str with values passed to the event macro
      event: event.to_string(),
      function: function.to_string(),
      span_id,
      trace_id,
      ..Self::default()
    }
  }
  // todo event metadata
  pub fn log(&self, level: &tracing::Level, logger: &slog::Logger) {
    let kv = o!(
      "event" => self.event.as_str(),
      "function" => self.function.as_str(),
      "span_id" => self.span_id,
      "trace_id" => self.trace_id
    );
    let message = serde_json::to_string(&self.data).unwrap_or_default();
    match *level {
      tracing::Level::ERROR => error!(logger, "{}", message; o!(kv, "status" => "error")),
      tracing::Level::WARN => warn!(logger, "{}", message; o!(kv, "status" => "warn")),
      _ => info!(logger, "{}", message; o!(kv, "status" => "info"))
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
       let _ = self.data.insert(name, value.to_string());
      }
    };
  }
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
    self.data.insert(field.name(), format!("{:#?}", value));
  }
}