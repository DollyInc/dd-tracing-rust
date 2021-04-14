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

The `Config` struct is set up to work with the `config` crate, and has a function `create_global_subscriber` that can be called to start the subscriber.

## Example setup
```
  // set up config, or load from environment via config crate
  let config = Config {
    level: "info".to_string(),
    prefix: "foo".to_string(),
    dd: Dd {
      service: "foo".to_string(),
      env: "dev".to_string(),
      ..Default::default()
    }
    ..Default::default()
  };
  // configure the subscriber
  config.create_global_subscriber();

  // create a span whenever foo is called
  [#tracing::instrument(level = "info")]
  async fn foo() {
    //create an event inside the span
    tracing::info!(event = "bar");
  }
```


# Running DD Agent
In order to submit logs to Datadog you must be running the DD agent. The DD agent is available as a Docker image.
[Click Here](https://docs.datadoghq.com/agent/docker/?tab=standard) to see the official documentation of running a DD agent for running on various platforms.

The standard command to get the Agent up and running locally will be the following

```
docker run -d -v /var/run/docker.sock:/var/run/docker.sock:ro \
              -v /proc/:/host/proc/:ro \
              -v /sys/fs/cgroup/:/host/sys/fs/cgroup:ro \
              -p 127.0.0.1:8126:8126/tcp \
              -e DD_API_KEY="<INSERT DD API KEY HERE>" \
              -e DD_APM_ENABLED=true \
              --name dd-agent-local\
              gcr.io/datadoghq/agent:latest
```

In order to retrieve your `DD_API_KEY` go to [Datadog Account API Settings](https://app.datadoghq.com/account/settings#api)

By default once your DD agent container is up and runing, the request for logs will be sent to the localhost address on port 8126
