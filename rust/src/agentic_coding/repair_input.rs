//! Structured inputs for proposals and operator-reviewed changes.

use serde::de::DeserializeOwned;

const MAXIMUM_INPUT_BYTES: usize = 128 * 1024;

pub(super) fn intent(task: &str) -> &str {
    task.split_once("\n```json")
        .map_or(task, |(intent, _)| intent)
}

pub(super) fn data<T: DeserializeOwned>(task: &str) -> Option<T> {
    let (_, block) = task.split_once("\n```json")?;
    let block = block
        .strip_prefix("\r\n")
        .or_else(|| block.strip_prefix('\n'))?;
    let payload = block.trim_end().strip_suffix("\n```")?;
    if payload.len() > MAXIMUM_INPUT_BYTES {
        return None;
    }
    serde_json::from_str(payload).ok()
}

pub(super) fn owned_source(module: &str) -> Option<&'static str> {
    crate::self_source_links::owned_source_files()
        .iter()
        .find_map(|(path, source)| (*path == module).then_some(*source))
}

#[derive(serde::Deserialize)]
pub(super) struct TraceInput {
    prompt: String,
    events: Vec<TraceEventInput>,
}

#[derive(serde::Deserialize)]
struct TraceEventInput {
    kind: String,
    payload: String,
}

impl TraceInput {
    pub(super) fn into_trace(self) -> Option<crate::self_improvement::UnknownTrace> {
        if self.prompt.trim().is_empty() || self.events.is_empty() || self.events.len() > 128 {
            return None;
        }
        let mut log = crate::event_log::EventLog::new();
        for event in self.events {
            let kind = match event.kind.as_str() {
                "reasoning:unknown" => "reasoning:unknown",
                "selected_rule" => "selected_rule",
                "rule_synthesis_candidate" => "rule_synthesis_candidate",
                "rule_verification" => "rule_verification",
                "write_program_plan" => "write_program_plan",
                "lexicon_lookup" => "lexicon_lookup",
                "regression_check" => "regression_check",
                "answer:intent" => "answer:intent",
                "answer:evidence" => "answer:evidence",
                "answer:links_notation" => "answer:links_notation",
                _ => return None,
            };
            log.append(kind, event.payload);
        }
        Some(crate::self_improvement::UnknownTrace::new(
            self.prompt,
            log.events().to_vec(),
        ))
    }
}
