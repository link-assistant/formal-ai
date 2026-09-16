//! Reading this turn's tool results back out as execution records (#1138 B5).
//!
//! The planner is stateless: what has already been observed is *read* from the
//! transcript rather than remembered. `progress` reads it as "an attempt of this
//! capability succeeded"; this module reads the same transcript as the thing an
//! obligation can actually be discharged against — an
//! [`Evidence`](crate::execution_evidence::Evidence) carrying the command that
//! was issued, the exit status the harness reported (or an honest `None`) and a
//! digest of the bytes that came back.
//!
//! The distinction is the whole of bottleneck B5. "A write tool call returned
//! success" is an attempt; the bytes a read-back observed are evidence. A record
//! built here claims only [`EvidenceSource::Harness`], because a transcript that
//! *says* it ran a local process is still a transcript.

use crate::execution_evidence::{Evidence, EvidenceSource};
use crate::protocol::ChatMessage;

/// Argument keys whose value names the artifact or command a result is about.
///
/// A result binds to an obligation only by the path, command or check the
/// obligation expects (R710-R4), so the rendered command line carries exactly
/// those argument values and not, say, a file's contents.
const NAMING_KEYS: [&str; 8] = [
    "path",
    "filePath",
    "file_path",
    "file",
    "command",
    "cmd",
    "query",
    "url",
];

/// Every tool result of the current user turn, as an execution record.
#[must_use]
pub fn records(messages: &[ChatMessage]) -> Vec<Evidence> {
    let current_turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    let mut records = Vec::new();
    for (index, message) in messages.iter().enumerate().skip(current_turn) {
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let raw = message.content.plain_text();
        let Some(command) = command_for(messages, index, message) else {
            continue;
        };
        records.push(Evidence::from_tool_result(
            &command,
            &raw,
            EvidenceSource::Harness,
        ));
    }
    records
}

/// The canonical rendering of the tool call a result answers: the tool's name
/// followed by the argument values that name what it acted on.
fn command_for(messages: &[ChatMessage], index: usize, result: &ChatMessage) -> Option<String> {
    let call = preceding_call(messages, index, result);
    let tool = result
        .name
        .clone()
        .or_else(|| call.as_ref().map(|(name, _)| name.clone()))?;
    let mut rendered = tool;
    if let Some((_, arguments)) = call
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&arguments)
    {
        for key in NAMING_KEYS {
            if let Some(named) = value.get(key).and_then(serde_json::Value::as_str) {
                rendered.push(' ');
                rendered.push_str(named);
            }
        }
    }
    Some(rendered)
}

/// The assistant tool call this result answers: the one whose id matches, or the
/// nearest preceding call when the transcript carries no ids.
fn preceding_call(
    messages: &[ChatMessage],
    index: usize,
    result: &ChatMessage,
) -> Option<(String, String)> {
    for message in messages[..index].iter().rev() {
        let calls = &message.tool_calls;
        if calls.is_empty() {
            continue;
        }
        if let Some(call) = calls
            .iter()
            .find(|call| result.tool_call_id.as_ref().is_none_or(|id| &call.id == id))
        {
            return Some((call.function.name.clone(), call.function.arguments.clone()));
        }
    }
    None
}
