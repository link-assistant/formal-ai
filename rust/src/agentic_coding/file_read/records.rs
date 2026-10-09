//! Current-window receipt matching; source status stays outside file bytes.
use super::{Capability, tool_capability};
use crate::protocol::{ChatMessage, ToolCall};

pub(super) struct ToolResultRecord {
    pub(super) capability: Option<Capability>,
    pub(super) arguments: serde_json::Value,
    pub(super) content: String,
    pub(super) is_error: bool,
}

pub(super) fn tool_result_records(messages: &[ChatMessage]) -> Vec<ToolResultRecord> {
    // Only results produced *within the current user turn* ground a read. A
    // client loops tool calls without a new user message, so the current turn is
    // everything after the last user message; a `cat 1.txt` result from an
    // earlier turn must never be replayed as the answer to a later `read 1.txt`
    // (issue #755 — read/list determinism). Without this bound the recipe short
    // -circuits on a stale, unrelated result and skips the fresh read entirely.
    let turn_start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    let mut records = Vec::new();
    for (index, message) in messages.iter().enumerate().skip(turn_start) {
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let (capability, arguments) =
            tool_result_call(messages, index).map_or((None, serde_json::Value::Null), |call| {
                (
                    tool_capability(&call.function.name),
                    serde_json::from_str(&call.function.arguments)
                        .unwrap_or(serde_json::Value::Null),
                )
            });
        records.push(ToolResultRecord {
            capability,
            arguments,
            content: message.content.plain_text(),
            is_error: message.is_error,
        });
    }
    records
}

fn tool_result_call(messages: &[ChatMessage], index: usize) -> Option<&ToolCall> {
    let message = &messages[index];
    let call_id = message.tool_call_id.as_ref()?;
    let start = messages[..index]
        .iter()
        .rposition(|prior| prior.role.eq_ignore_ascii_case("user"))
        .map_or(0, |at| at + 1);
    let call = messages[start..index]
        .iter()
        .rev()
        .flat_map(|prior| prior.tool_calls.iter().rev())
        .find(|call| &call.id == call_id)?;
    (message
        .name
        .as_deref()
        .is_none_or(|name| name.eq_ignore_ascii_case(&call.function.name)))
    .then_some(call)
}

pub(super) fn read_record_for_path<'a>(
    records: &'a [ToolResultRecord],
    path: &str,
) -> Option<&'a ToolResultRecord> {
    records.iter().rev().find(|record| {
        let fields = ["path", "filePath", "file_path", "absolute_path"]
            .iter()
            .filter_map(|key| record.arguments.get(*key))
            .collect::<Vec<_>>();
        record.capability == Some(Capability::Read)
            && fields
                .first()
                .and_then(|field| field.as_str())
                .is_some_and(|first| {
                    !first.trim().is_empty()
                        && fields.iter().all(|field| field.as_str() == Some(first))
                        && same_path(first, path)
                })
    })
}
pub(super) fn read_result_for_path<'a>(
    records: &'a [ToolResultRecord],
    path: &str,
) -> Option<&'a str> {
    read_record_for_path(records, path).map(|record| record.content.as_str())
}

pub(super) fn grep_result_for_path<'a>(
    records: &'a [ToolResultRecord],
    path: &str,
    pattern: &str,
) -> Option<&'a str> {
    records
        .iter()
        .find(|record| {
            record.capability == Some(Capability::Grep)
                && path_argument(&record.arguments)
                    .is_some_and(|recorded| same_path(&recorded, path))
                && record
                    .arguments
                    .get("pattern")
                    .and_then(serde_json::Value::as_str)
                    == Some(pattern)
        })
        .map(|record| record.content.as_str())
}

/// Whether a recorded path argument denotes the path the planner asked for.
///
/// The transcript holds the argument the *client's* schema accepted, not the one
/// the planner planned with: [`crate::protocol_responses`] rewrites `path` to
/// Gemini's `absolute_path` and absolutises it, so a recorded
/// `/work/alpha.txt` has to answer a planned `alpha.txt`.
pub(super) fn same_path(recorded: &str, planned: &str) -> bool {
    if recorded == planned {
        return true;
    }
    let planned = planned.trim_start_matches("./");
    recorded
        .trim_start_matches("./")
        .strip_suffix(planned)
        .is_some_and(|prefix| prefix.is_empty() || prefix.ends_with('/'))
}

/// The recorded run result for `command`, exactly as the harness reported it.
///
/// Callers that only want the command's own text strip the transport envelope
/// themselves; callers that have to judge whether the step *succeeded* need the
/// envelope intact, because the `Exit Code:` field is what it carries (rung
/// `R916-01`).
pub(super) fn run_record_for_command<'a>(
    records: &'a [ToolResultRecord],
    command: &str,
) -> Option<&'a str> {
    records
        .iter()
        .find(|record| {
            record.capability == Some(Capability::Run)
                && super::super::tool_result::command_argument(&record.arguments.to_string())
                    .as_deref()
                    == Some(command)
        })
        .map(|record| record.content.as_str())
}

fn path_argument(arguments: &serde_json::Value) -> Option<String> {
    ["filePath", "path", "file_path", "absolute_path"]
        .iter()
        .find_map(|key| arguments.get(*key).and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
}
