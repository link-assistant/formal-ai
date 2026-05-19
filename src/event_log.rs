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

/// Build the evidence links array for a symbolic answer.
///
/// Translates each log event into a typed link, appending `response_link` at
/// the end when it is not already present.
#[must_use]
pub fn build_evidence_links(prompt: &str, log: &EventLog, response_link: &str) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    links.push(format!("prompt:{}", stable_id("prompt", prompt)));
    for event in log.events() {
        let evidence = match event.kind {
            "trace:execution_failure" => format!("trace:execution_failure:{}", event.id),
            "language" => format!("language:{}", event.payload),
            "language_from" => format!("language_from:{}", event.payload),
            "language_to" => format!("language_to:{}", event.payload),
            "definition_merge:language" => format!("definition_merge:language:{}", event.payload),
            "meaning" => format!("meaning:{}", event.payload),
            "translation_gap" => format!("translation_gap:{}", event.payload),
            "wikidata" => format!("wikidata:{}", event.payload),
            // Structured fact_query trace events (Issue #127): preserve the
            // payload verbatim so memory consumers can render the parsed
            // relation, subject, and cache decision (rather than a hash id).
            "fact_query:request" => format!("fact_query:request:{}", event.id),
            "fact_query:relation" => format!("fact_query:relation:{}", event.payload),
            "fact_query:subject" => format!("fact_query:subject:{}", event.payload),
            "fact_query:cache:hit" => format!("fact_query:cache:hit:{}", event.payload),
            "fact_query:cache:miss" => String::from("fact_query:cache:miss"),
            "fact_query:cache:bypass" => String::from("fact_query:cache:bypass"),
            "fact_query:force_fresh" => String::from("fact_query:force_fresh"),
            "fact_query:subject_qid" => format!("fact_query:subject_qid:{}", event.payload),
            "fact_query:value_qid" => format!("fact_query:value_qid:{}", event.payload),
            "search:local" => format!("search:local:{}", event.id),
            "search:external" => format!("search:external:{}", event.id),
            "source:http" => format!("source:http:{}", event.payload.replace(' ', ":")),
            "source_refresh" => format!("source_refresh:{}", event.payload),
            "conflict:source_disagreement" => {
                format!("conflict:source_disagreement:{}", event.id)
            }
            "cache_hit" => format!("cache_hit:{}", event.payload),
            "network_fetch" => format!("network_fetch:{}", event.id),
            "calculation:engine" => format!("calculation:engine:{}", event.payload),
            "calculation:lino" => format!("calculation:lino:{}", event.payload),
            "intent" => format!("intent:{}", event.payload),
            "response" => event.payload.clone(),
            "agent_mode:opted_in" => format!("agent_mode:opted_in:{}", event.id),
            "agent_mode:active" => format!("agent_mode:active:{}", event.id),
            "policy:chat_bounded_autonomy" => String::from("policy:chat_bounded_autonomy"),
            "policy:add_only_history" => String::from("policy:add_only_history"),
            "policy:destructive_action_requires_confirmation" => {
                String::from("policy:destructive_action_requires_confirmation")
            }
            "policy:agent_time_budget" => format!("policy:agent_time_budget:{}", event.id),
            "policy:cache_flush_requires_confirmation" => {
                String::from("policy:cache_flush_requires_confirmation")
            }
            "policy:inappropriate_content" => String::from("policy:inappropriate_content"),
            "error" => format!("error:{}", event.id),
            "filter:user" => format!("filter:user:{}", event.payload),
            "diagnostic_mode" => format!("diagnostic_mode:{}", event.payload),
            "execution_status" => format!("execution_status:{}", event.id),
            "execution_environment" => format!("execution_environment:{}", event.id),
            _ => format!("{}:{}", event.kind, event.id),
        };
        links.push(evidence);
    }
    if !links.iter().any(|link| link == response_link) {
        links.push(response_link.to_owned());
    }
    links
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
