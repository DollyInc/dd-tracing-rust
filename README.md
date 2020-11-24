# dd_tracing
This is a `tracing` subscriber (see https://docs.rs/tracing) that collects traces and sends them to the datadog APM.

## Example setup
```
  let dd_config = dd_tracing::Config {
    env: None,
    service: "hello".to_string(),
    host: "localhost".to_string(),
    ..Default::default()
  };
  let collector = dd_tracing::Collector::new(tracing::Level::INFO,  "0.1".to_string(), dd_config);
  tracing::subscriber::set_global_default(collector)
    .unwrap();

  // create a span whenever foo is called
  [#tracing::instrument(level = "info")]
  async fn foo() {
    //create an event inside the span
    tracing::info!(event = "bar");
  }
```