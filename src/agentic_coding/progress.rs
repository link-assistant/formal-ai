//! Reading a turn's tool results back out of the transcript (issue #468).
//!
//! The planner is stateless: it re-derives what to do next from the conversation
//! alone, so "what has already been tried this turn" has to be *read* rather than
//! remembered. That reading is this module, and it is a separate concern from
//! choosing the next step — [`Progress::scan`] answers what happened, and
//! `planner` decides what happens next.

use super::planner::{classify_tool, Capability};
use crate::protocol::ChatMessage;

/// Tool results produced since the current user turn began.
pub struct Progress {
    completed: Vec<Capability>,
    pub(super) fetched_text: Option<String>,
    pub(super) fetched_pages: Vec<(String, String)>,
    pub(super) attempted_fetches: Vec<String>,
    pub(super) search_output: Option<String>,
    pub(super) run_output: Option<String>,
    pub(super) fetch_result: Option<String>,
    pub(super) search_result: Option<String>,
}

impl Progress {
    pub(super) fn scan(messages: &[ChatMessage]) -> Self {
        let mut completed = Vec::new();
        let mut fetched_text = None;
        let mut fetched_pages = Vec::new();
        let mut attempted_fetches = Vec::new();
        let mut search_output = None;
        let mut run_output = None;
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
            if capability == Capability::Fetch {
                let payload = super::tool_result::normalized_payload(&message.content.plain_text());
                fetch_result = Some(payload.clone().unwrap_or_default());
                let fetch_url = result_tool_call(messages, index).and_then(fetch_call_url);
                if let Some(url) = fetch_url.as_ref() {
                    if !attempted_fetches.contains(url) {
                        attempted_fetches.push(url.clone());
                    }
                }
                if let Some(text) = payload.filter(|text| !text.trim().is_empty()) {
                    if let Some(url) = fetch_url {
                        fetched_pages.push((url, text.clone()));
                    }
                    fetched_text = Some(text);
                }
            }
            if capability == Capability::Search {
                let payload = super::tool_result::normalized_payload(&message.content.plain_text());
                search_result = Some(payload.clone().unwrap_or_default());
                if let Some(text) = payload.filter(|text| !text.trim().is_empty()) {
                    search_output = Some(text);
                }
            }
            if capability == Capability::Run {
                run_output = Some(message.content.plain_text());
            }
            completed.push(capability);
        }
        Self {
            completed,
            fetched_text,
            fetched_pages,
            attempted_fetches,
            search_output,
            run_output,
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

    /// The capability of the most recent tool result in this turn.
    ///
    /// `completed` is in arrival order, so this distinguishes *which phase* a
    /// multi-round loop is in — a search that has not been read yet, versus a
    /// completed read — which [`Progress::done`] alone cannot, since it stays
    /// true for every later round.
    pub(super) fn last(&self) -> Option<Capability> {
        self.completed.last().copied()
    }

    pub(super) fn fetch_result(&self) -> Option<&str> {
        self.fetch_result.as_deref()
    }

    pub(super) fn search_result(&self) -> Option<&str> {
        self.search_result.as_deref()
    }
}

/// Resolve which capability the tool result at `index` answers. Prefer the
/// result's own `name`; otherwise map its `tool_call_id` back to the tool name in
/// a prior assistant `tool_calls` turn.
fn result_capability(messages: &[ChatMessage], index: usize) -> Option<Capability> {
    let message = &messages[index];
    if let Some(name) = &message.name {
        if let Some(capability) = classify_tool(name) {
            return Some(capability);
        }
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
