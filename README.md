# dd_tracing
This is a `tracing` subscriber (see https://docs.rs/tracing) that collects traces and sends them to the datadog APM. The `Collector` struct implements the `tracing::Subscriber` trait.

Events that are created using the `tracing::event` macros are automatically linked to their enclosing span and are logged in JSON format, following the standardized platform logging format. Fields named `event` and `function` are top-level fields; any other fields are logged in JSON format under `message`. Dd and metadata fields that are passed to the `Collector` constructor are also passed through to the event logs. For instance, `tracing::info!(event = "EventHappened", function = "doEvent", request_id = "000", a = 1, b = "b")` would write the log

```
{
  "event": "EventHappened",
  "function": "doEvent",
  "status": "info",
  "message": { 
    "request_id": "000",
    "a": "1",
    "b": "b"
  },
  "dd": {
    "env": ...
    "service": ...
    "version": ...
    "span_id": ...
    "trace_id": ...
  },
  "metadata": {
    "environment": ...,
    "image": ...,
    "time": ...
  }
}
```

Error events are automatically propagated to their parent span. If an error event is created, any fields that start with `error` are attached as tags to the parent span as well.

The collector requires a `tracing::Level` and `prefix` to use for filtering spans and events. Spans and events that do not match the minimum level or have a `target` that does not start with the `prefix` are filtered out. The `target` is typically the module path, but it can also be set explicitly when a span or event is constructed.

## Example setup
```
  // configure the dd agent info
  let dd_config = dd_tracing::Config {
    env: None,
    service: "hello".to_string(),
    host: "localhost".to_string(),
    ..Default::default()
  };
  // configure the subscriber
  let collector = dd_tracing::Collector::new(tracing::Level::INFO, "hello", "0.1".to_string(), "env", "image", dd_config);
  tracing::subscriber::set_global_default(collector)
    .unwrap();

  // create a span whenever foo is called
  [#tracing::instrument(level = "info")]
  async fn foo() {
    //create an event inside the span
    tracing::info!(event = "bar");
  }
```