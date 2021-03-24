use std::{
    collections::HashMap,
    fmt::Debug,
    time::{Duration, SystemTime},
};

#[derive(Debug, Default)]
struct SpanData {
    name: String,
    resource: String,
    start: Option<SystemTime>,
    duration: Option<Duration>,
    tags: HashMap<String, String>,
}

impl SpanData {
    fn new(name: &str, resource: &str) -> Self {
        Self {
            name: name.to_string(),
            resource: resource.to_string(),
            ..Self::default()
        }
    }
}

#[derive(Debug)]
pub struct Span {
    pub parent: Option<tracing::Id>,
    pub trace_id: u64,
    handlers: u64,
    data: SpanData,
}

impl Span {
    pub fn new(
        parent: Option<tracing::Id>,
        trace_id: u64,
        attrs: &tracing::span::Attributes,
    ) -> Self {
        let metadata = attrs.metadata();
        let name = metadata.name();
        let resource = metadata.target();
        let mut span = Span {
            parent,
            trace_id,
            handlers: 1,
            data: SpanData::new(name, resource),
        };
        attrs.record(&mut span);
        span
    }
    pub fn set_tag(&mut self, name: String, value: String) {
        self.data.tags.insert(name, value);
    }
    pub fn set_time(&mut self) {
        if self.data.start == None {
            self.data.start = Some(SystemTime::now())
        }
    }
    pub fn set_duration(&mut self) {
        let start = self.data.start.unwrap_or_else(SystemTime::now);
        let duration = SystemTime::now().duration_since(start).ok();
        self.data.duration = duration;
    }
    pub fn name(&self) -> &str {
        &self.data.name
    }
    pub fn increment_handlers(&mut self) {
        self.handlers += 1
    }
    pub fn decrement_handlers(&mut self) {
        if self.handlers > 0 {
            self.handlers -= 1
        }
    }
    pub fn is_closed(&self) -> bool {
        self.handlers == 0
    }
    pub fn into(id: tracing::Id, span: Span) -> datadog_apm::Span {
        let Span { parent, data, .. } = span;
        let start = data.start.unwrap_or_else(SystemTime::now);
        let duration = data.duration.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(start)
                .unwrap_or_else(|_| Duration::from_nanos(0))
        });
        datadog_apm::Span {
            id: id.into_u64(),
            parent_id: parent.map(|p| p.into_u64()),
            name: data.name,
            resource: data.resource,
            r#type: "web".to_string(),
            start,
            duration,
            tags: data.tags,
            http: None,
            error: None,
            sql: None,
        }
    }
}

impl tracing::field::Visit for Span {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let name = field.name();
        let name = if name.starts_with("grpc_") {
            str::replace(name, "grpc_", "grpc.")
        } else {
            name.to_string()
        };
        self.data.tags.insert(name, value.to_string());
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
        let name = field.name();
        if name != "callsite" {
            self.data
                .tags
                .insert(name.to_string(), format!("{:#?}", value));
        }
    }
}
