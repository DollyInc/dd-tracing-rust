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
  let my_subscriber = dd_tracing::Collector::new(tracing::Level::INFO, dd_config);
  tracing::subscriber::set_global_default(my_subscriber)
    .unwrap();
```