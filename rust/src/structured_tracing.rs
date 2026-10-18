//! `tracing` events enter the same redacting typed-Lino sink as native diagnostics.
//! Span payloads are emitted once and subsequent events refer to their identifiers.
use crate::dialog_log::structured_log;
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

type Sink = Arc<dyn Fn(&str, &str, String, Value) + Send + Sync>;
static NEXT_SPAN: AtomicU64 = AtomicU64::new(1);
thread_local! { static ENTERED: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) }; }
#[derive(Default)]
struct Fields(Map<String, Value>);
impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().into(), json!(format!("{value:?}")));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().into(), json!(value));
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().into(), json!(value));
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(
            field.name().into(),
            if value.unsigned_abs() <= 9_007_199_254_740_991 {
                json!(value)
            } else {
                json!({"type":"integer","decimal":value.to_string()})
            },
        );
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(
            field.name().into(),
            if value <= 9_007_199_254_740_991 {
                json!(value)
            } else {
                json!({"type":"integer","decimal":value.to_string()})
            },
        );
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(
            field.name().into(),
            json!({"type":"float64","bits":format!("{:016x}",value.to_bits())}),
        );
    }
    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.0.insert(field.name().into(), json!(value.to_string()));
    }
}
struct SpanState {
    references: usize,
    level: &'static str,
    target: &'static str,
}
/// Subscriber for library consumers; the CLI installs exactly one global instance.
pub struct LinoSubscriber {
    spans: Mutex<HashMap<u64, SpanState>>,
    sink: Sink,
    maximum_level: Option<tracing::Level>,
}
fn level(metadata: &Metadata<'_>) -> &'static str {
    match *metadata.level() {
        tracing::Level::ERROR => "error",
        tracing::Level::WARN => "warn",
        tracing::Level::INFO => "info",
        tracing::Level::DEBUG => "debug",
        tracing::Level::TRACE => "trace",
    }
}
impl Default for LinoSubscriber {
    fn default() -> Self {
        Self {
            spans: Mutex::new(HashMap::new()),
            maximum_level: None,
            sink: Arc::new(|level, target, message, fields| {
                structured_log::event(level, target, || message, || fields)
            }),
        }
    }
}
impl LinoSubscriber {
    fn active(&self) -> Option<u64> {
        let spans = self
            .spans
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ENTERED.with(|entered| {
            entered
                .borrow()
                .iter()
                .rev()
                .find(|id| spans.contains_key(id))
                .copied()
        })
    }
    fn emit(&self, level: &str, target: &str, message: String, fields: Value) {
        // The production sink redacts independently; custom test sinks observe the same data.
        let safe = structured_log::redact(&fields);
        let safe_message = structured_log::redact(&json!(message));
        (self.sink)(
            level,
            target,
            safe_message.as_str().unwrap_or("redaction_failed").into(),
            safe,
        );
    }
}
impl Subscriber for LinoSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.maximum_level.map_or_else(
            || structured_log::enabled(level(metadata)),
            |maximum| *metadata.level() <= maximum,
        )
    }
    fn new_span(&self, attributes: &Attributes<'_>) -> Id {
        let identity = NEXT_SPAN.fetch_add(1, Ordering::Relaxed);
        let metadata = attributes.metadata();
        let parent = attributes.parent().map(Id::into_u64).or_else(|| {
            if attributes.is_contextual() {
                self.active()
            } else {
                None
            }
        });
        let mut fields = Fields::default();
        attributes.record(&mut fields);
        self.spans
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                identity,
                SpanState {
                    references: 1,
                    level: level(metadata),
                    target: metadata.target(),
                },
            );
        self.emit(level(metadata),metadata.target(),"tracing_span_created".into(),json!({"kind":"tracing_span","span_id":identity,"parent_span_id":parent,"name":metadata.name(),"fields":fields.0}));
        Id::from_u64(identity)
    }
    fn record(&self, span: &Id, values: &Record<'_>) {
        let metadata = self
            .spans
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&span.into_u64())
            .map(|state| (state.level, state.target));
        if let Some((level, target)) = metadata {
            if !structured_log::enabled(level) {
                return;
            }
            let mut fields = Fields::default();
            values.record(&mut fields);
            self.emit(
                level,
                target,
                "tracing_span_updated".into(),
                json!({"kind":"tracing_span_update","span_id":span.into_u64(),"fields":fields.0}),
            );
        }
    }
    fn record_follows_from(&self, span: &Id, follows: &Id) {
        if structured_log::enabled("debug") {
            self.emit("debug","tracing","tracing_span_follows".into(),json!({"kind":"tracing_span_follows","span_id":span.into_u64(),"follows_span_id":follows.into_u64()}));
        }
    }
    fn event(&self, event: &Event<'_>) {
        if !self.enabled(event.metadata()) {
            return;
        }
        let mut fields = Fields::default();
        event.record(&mut fields);
        let message = fields
            .0
            .remove("message")
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string())
            })
            .unwrap_or_else(|| event.metadata().name().into());
        let span = event.parent().map(Id::into_u64).or_else(|| {
            if event.is_contextual() {
                self.active()
            } else {
                None
            }
        });
        self.emit(
            level(event.metadata()),
            event.metadata().target(),
            message,
            json!({"kind":"tracing_event","span_id":span,"fields":fields.0}),
        );
    }
    fn enter(&self, span: &Id) {
        ENTERED.with(|entered| entered.borrow_mut().push(span.into_u64()));
    }
    fn exit(&self, span: &Id) {
        ENTERED.with(|entered| {
            let mut stack = entered.borrow_mut();
            if let Some(index) = stack.iter().rposition(|value| *value == span.into_u64()) {
                stack.remove(index);
            }
        });
    }
    fn clone_span(&self, span: &Id) -> Id {
        if let Some(state) = self
            .spans
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_mut(&span.into_u64())
        {
            state.references += 1;
        }
        span.clone()
    }
    fn try_close(&self, span: Id) -> bool {
        let metadata = {
            let mut spans = self
                .spans
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(state) = spans.get_mut(&span.into_u64()) {
                state.references -= 1;
                if state.references == 0 {
                    spans
                        .remove(&span.into_u64())
                        .map(|state| (state.level, state.target))
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some((level, target)) = metadata {
            if structured_log::enabled(level) {
                self.emit(
                    level,
                    target,
                    "tracing_span_closed".into(),
                    json!({"kind":"tracing_span_close","span_id":span.into_u64()}),
                );
            }
            true
        } else {
            false
        }
    }
}
/// Returns a fixed installation result; a foreign global subscriber is never replaced.
pub fn install() -> Result<(), &'static str> {
    static INSTALLED: OnceLock<Result<(), &'static str>> = OnceLock::new();
    *INSTALLED.get_or_init(|| {
        tracing::subscriber::set_global_default(LinoSubscriber::default())
            .map_err(|_| "tracing_subscriber_already_installed")
    })
}
#[cfg(test)]
#[path = "../tests/unit/structured_tracing.rs"]
mod tests;
