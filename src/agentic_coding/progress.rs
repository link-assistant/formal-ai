//! Reading a turn's tool results back out of the transcript (issue #468).
//!
//! The planner is stateless: it re-derives what to do next from the conversation
//! alone, so "what has already been tried this turn" has to be *read* rather than
//! remembered. That reading is this module, and it is a separate concern from
//! choosing the next step — [`Progress::scan`] answers what happened, and
//! `planner` decides what happens next.

use super::planner::{classify_tool, Capability};
use crate::protocol::ChatMessage;

/// One observed client-owned tool execution in transcript order.
#[derive(Debug)]
pub(super) struct ToolAttempt {
    pub(super) capability: Capability,
    pub(super) succeeded: bool,
    pub(super) detail: String,
    pub(super) arguments: Option<String>,
}

/// Tool results produced since the current user turn began.
pub struct Progress {
    /// Observed results, including failures. Existing recipes use this to move
    /// to their fallback or rendering phase after a client-owned attempt.
    completed: Vec<Capability>,
    /// Results that actually satisfied their planned step.
    successful: Vec<Capability>,
    attempts: Vec<ToolAttempt>,
    pub(super) fetched_text: Option<String>,
    pub(super) fetched_pages: Vec<(String, String)>,
    pub(super) attempted_fetches: Vec<String>,
    pub(super) search_output: Option<String>,
    /// Every shell result of this turn, in arrival order.
    ///
    /// A report runs one command per destination (#839), so keeping only the
    /// last one would drop the export results the moment the issue was filed.
    pub(super) run_outputs: Vec<String>,
    pub(super) fetch_result: Option<String>,
    pub(super) search_result: Option<String>,
}

impl Progress {
    pub(super) fn scan(messages: &[ChatMessage]) -> Self {
        let mut completed = Vec::new();
        let mut successful = Vec::new();
        let mut attempts = Vec::new();
        let mut fetched_text = None;
        let mut fetched_pages = Vec::new();
        let mut attempted_fetches = Vec::new();
        let mut search_output = None;
        let mut run_outputs = Vec::new();
        let mut fetch_result = None;
        let mut search_result = None;
        // Ignore results from earlier user turns.
        let current_turn = messages
            .iter()
            .rposition(|message| message.role.eq_ignore_ascii_case("user"))
            .map_or(0, |index| index + 1);
        for (index, message) in messages.iter().enumerate().skip(current_turn) {
            if !message.role.eq_ignore_ascii_case("tool") {
                continue;
            }
            let Some(capability) = result_capability(messages, index) else {
                continue;
            };
            let raw = message.content.plain_text();
            let failure = super::tool_result::failure_message(
                &raw,
                message.is_error,
                capability != Capability::Run,
            );
            let arguments =
                result_tool_call(messages, index).map(|call| call.function.arguments.clone());
            attempts.push(ToolAttempt {
                capability,
                succeeded: failure.is_none(),
                detail: failure.clone().unwrap_or_else(|| raw.clone()),
                arguments,
            });
            if capability == Capability::Fetch {
                let payload = super::tool_result::normalized_payload(&raw);
                fetch_result = Some(payload.clone().unwrap_or_default());
                let fetch_url = result_tool_call(messages, index).and_then(fetch_call_url);
                if let Some(url) = fetch_url.as_ref()
                    && !attempted_fetches.contains(url) {
                        attempted_fetches.push(url.clone());
                    }
                if let Some(text) = payload.filter(|text| !text.trim().is_empty()) {
                    if let Some(url) = fetch_url {
                        fetched_pages.push((url, text.clone()));
                    }
                    fetched_text = Some(text);
                }
            }
            if capability == Capability::Search {
                let payload = super::tool_result::normalized_payload(&raw);
                search_result = Some(payload.clone().unwrap_or_default());
                if let Some(text) = payload.filter(|text| !text.trim().is_empty()) {
                    search_output = Some(text);
                }
            }
            if capability == Capability::Run {
                run_outputs.push(raw);
            }
            completed.push(capability);
            if failure.is_none() {
                successful.push(capability);
            }
        }
        Self {
            completed,
            successful,
            attempts,
            fetched_text,
            fetched_pages,
            attempted_fetches,
            search_output,
            run_outputs,
            fetch_result,
            search_result,
        }
    }

    /// Whether a prior tool result already covered `capability`.
    pub(super) fn done(&self, capability: Capability) -> bool {
        self.completed.contains(&capability)
    }

    pub(super) fn count(&self, capability: Capability) -> usize {
        self.completed
            .iter()
            .filter(|done| **done == capability)
            .count()
    }

    /// Number of observations that satisfied the planned capability.
    pub(super) fn successful_count(&self, capability: Capability) -> usize {
        self.successful
            .iter()
            .filter(|done| **done == capability)
            .count()
    }

    /// Most recent successful result payload for one capability.
    pub(super) fn latest_successful_output(&self, capability: Capability) -> Option<&str> {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| attempt.capability == capability && attempt.succeeded)
            .map(|attempt| attempt.detail.as_str())
    }

    pub(super) fn attempted_write_for(&self, path: &str) -> bool {
        self.attempts.iter().any(|attempt| {
            attempt.capability == Capability::Write
                && attempt
                    .arguments
                    .as_deref()
                    .is_some_and(|arguments| argument_targets(arguments, path))
        })
    }

    pub(super) fn successful_write_for(&self, path: &str) -> bool {
        self.attempts.iter().any(|attempt| {
            attempt.capability == Capability::Write
                && attempt.succeeded
                && attempt
                    .arguments
                    .as_deref()
                    .is_some_and(|arguments| argument_targets(arguments, path))
        })
    }

    /// Content supplied to the latest successful write of `path`.
    ///
    /// A composed request can deliver one observation to more than one file.
    /// The later delivery must recover the observation from the earlier write,
    /// not use the earlier writer's human-facing completion status as data.
    pub(super) fn successful_write_content_for(&self, path: &str) -> Option<String> {
        self.attempts.iter().rev().find_map(|attempt| {
            (attempt.capability == Capability::Write && attempt.succeeded)
                .then_some(attempt.arguments.as_deref())
                .flatten()
                .filter(|arguments| argument_targets(arguments, path))
                .and_then(argument_content)
        })
    }

    /// The capability of the most recent tool result in this turn.
    ///
    /// `completed` is in arrival order, so this distinguishes *which phase* a
    /// multi-round loop is in — a search that has not been read yet, versus a
    /// completed read — which [`Progress::done`] alone cannot, since it stays
    /// true for every later round.
    pub(super) fn last(&self) -> Option<Capability> {
        self.completed.last().copied()
    }

    /// The latest observed result when it failed. Successful observations do
    /// not leave an older failure active.
    pub(super) fn latest_failure(&self) -> Option<&ToolAttempt> {
        self.attempts.last().filter(|attempt| !attempt.succeeded)
    }

    pub(super) fn previous_attempt(&self) -> Option<&ToolAttempt> {
        self.attempts.iter().rev().nth(1)
    }

    /// Bound retries per concrete write target, rather than globally across a
    /// multi-file recipe.
    pub(super) fn failed_write_count_for(&self, path: &str) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| {
                attempt.capability == Capability::Write
                    && !attempt.succeeded
                    && attempt
                        .arguments
                        .as_deref()
                        .is_some_and(|arguments| argument_targets(arguments, path))
            })
            .count()
    }

    pub(super) fn fetch_result(&self) -> Option<&str> {
        self.fetch_result.as_deref()
    }

    pub(super) fn search_result(&self) -> Option<&str> {
        self.search_result.as_deref()
    }
}

fn argument_path(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    ["path", "filePath", "file_path"]
        .iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_str))
        .map(str::to_owned)
}

fn argument_content(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    ["content", "contents", "text", "new_string"]
        .iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_str))
        .map(str::to_owned)
}

fn argument_targets(arguments: &str, path: &str) -> bool {
    argument_path(arguments).is_some_and(|observed| {
        observed == path
            || (std::path::Path::new(path).is_relative()
                && std::path::Path::new(&observed).ends_with(path))
    })
}

/// Resolve which capability the tool result at `index` answers. Prefer the
/// result's own `name`; otherwise map its `tool_call_id` back to the tool name in
/// a prior assistant `tool_calls` turn.
pub(super) fn result_capability(messages: &[ChatMessage], index: usize) -> Option<Capability> {
    let message = &messages[index];
    if let Some(name) = &message.name
        && let Some(capability) = classify_tool(name) {
            return Some(capability);
        }
    result_tool_call(messages, index).and_then(|call| classify_tool(&call.function.name))
}

fn result_tool_call(messages: &[ChatMessage], index: usize) -> Option<&crate::protocol::ToolCall> {
    let call_id = messages[index].tool_call_id.as_ref()?;
    messages[..index]
        .iter()
        .rev()
        .flat_map(|prior| prior.tool_calls.iter())
        .find(|call| &call.id == call_id)
}

fn fetch_call_url(call: &crate::protocol::ToolCall) -> Option<String> {
    let arguments: serde_json::Value = serde_json::from_str(&call.function.arguments).ok()?;
    arguments
        .get("url")
        .and_then(serde_json::Value::as_str)
        .filter(|url| !url.trim().is_empty())
        .map(str::to_owned)
}
