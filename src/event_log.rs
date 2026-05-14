//! Append-only event log for the universal solver.
//!
//! Every step the solver takes is recorded as a content-addressed event
//! before the user-facing answer is built. The answer is then a projection
//! of the log — see `VISION.md` and `GOALS.md` for the rationale.
//!
//! The log is intentionally small: it lives in-process, holds plain Rust
//! records, and uses the same FNV-1a 64-bit hash that `engine::stable_id`
//! uses so identifiers stay stable across surfaces.

use crate::engine::stable_id;

/// A single event in the append-only log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: String,
    pub kind: &'static str,
    pub payload: String,
}

/// In-process append-only event log.
#[derive(Debug, Default, Clone)]
pub struct EventLog {
    events: Vec<Event>,
}

impl EventLog {
    #[must_use]
    pub const fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Append a new event with content-addressed id.
    ///
    /// The id is derived from the kind, the payload, and the current log
    /// length so that repeated events produce stable, distinct ids.
    pub fn append(&mut self, kind: &'static str, payload: impl Into<String>) -> String {
        let payload = payload.into();
        let seed = format!("{kind}:{}:{payload}", self.events.len());
        let id = stable_id(kind, &seed);
        self.events.push(Event {
            id: id.clone(),
            kind,
            payload,
        });
        id
    }

    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Returns the first event of the given kind, if any.
    #[must_use]
    pub fn first_of(&self, kind: &str) -> Option<&Event> {
        self.events.iter().find(|event| event.kind == kind)
    }

    /// Returns the most recent event of the given kind, if any.
    #[must_use]
    pub fn last_of(&self, kind: &str) -> Option<&Event> {
        self.events.iter().rev().find(|event| event.kind == kind)
    }

    /// Project the log to a list of `<kind>:<id>` links for the user-facing
    /// evidence array. Each link points back to a distinct event.
    #[must_use]
    pub fn evidence_links(&self) -> Vec<String> {
        self.events
            .iter()
            .map(|event| format!("{}:{}", event.kind, event.id))
            .collect()
    }

    /// Build a Links Notation `steps` block listing every event in order.
    /// Used by trace serialization in [`crate::solver`].
    #[must_use]
    pub fn steps_block(&self) -> String {
        use std::fmt::Write as _;
        let mut buffer = String::from("steps:");
        for (index, event) in self.events.iter().enumerate() {
            let _ = write!(
                buffer,
                "\n  step_{index} {} {}",
                event.kind,
                sanitize_payload(&event.payload)
            );
        }
        buffer
    }
}

fn sanitize_payload(value: &str) -> String {
    value
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::EventLog;

    #[test]
    fn append_returns_stable_ids_for_distinct_events() {
        let mut log = EventLog::new();
        let first = log.append("impulse", "hi");
        let second = log.append("impulse", "hi");
        assert_ne!(first, second, "appending twice must produce distinct ids");
        assert_eq!(log.events().len(), 2);
    }

    #[test]
    fn evidence_links_round_trip_event_kinds() {
        let mut log = EventLog::new();
        log.append("impulse", "hello");
        log.append("intent", "greeting");
        let links = log.evidence_links();
        assert_eq!(links.len(), 2);
        assert!(links[0].starts_with("impulse:"));
        assert!(links[1].starts_with("intent:"));
    }

    #[test]
    fn steps_block_lists_events_in_insertion_order() {
        let mut log = EventLog::new();
        log.append("impulse", "x");
        log.append("trace", "y");
        let block = log.steps_block();
        assert!(block.contains("step_0 impulse x"));
        assert!(block.contains("step_1 trace y"));
    }
}
