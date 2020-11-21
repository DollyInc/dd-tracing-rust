#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;

mod collector;
mod event;
mod span;
pub use collector::Collector;
pub use datadog_apm::Config;