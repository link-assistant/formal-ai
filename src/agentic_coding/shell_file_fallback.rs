//! Adapt a bounded append intent for clients that expose file tools but no shell.
//!
//! Some real coding CLIs (notably Gemini) advertise `read_file` and `write_file`
//! in auto-edit mode without a shell capability. A seed-backed intent may still
//! lower naturally to a shell append. This module preserves that intent by
//! reading the existing file and writing the exact appended bytes instead of
//! returning misleading prose that merely says a shell tool is unavailable.

use std::path::{Component, Path};

use serde_json::json;

use super::planner::{plan_one, tool_for, write_arguments, AgenticPlan, Capability, Progress};
use super::progress::result_capability;
use super::tool_result;
use crate::protocol::ChatMessage;

struct AppendOperation<'a> {
    path: &'a str,
    payload: &'a str,
}

pub(super) fn plan_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    command: &str,
) -> Option<AgenticPlan> {
    let operation = parse_append(command)?;
    if tool_for(tool_names, Capability::Run).is_some() {
        return None;
    }
    let read_tool = tool_for(tool_names, Capability::Read);
    let write_tool = tool_for(tool_names, Capability::Write);
    if read_tool.is_none() || write_tool.is_none() {
        let discovery = tool_names
            .iter()
            .copied()
            .find(|name| name.eq_ignore_ascii_case("tool_search"))?;
        if has_tool_result(messages, discovery) {
            return None;
        }
        return Some(plan_one(
            discovery,
            json!({
                "query": "select:write_file,run_shell_command",
                "max_results": 2,
            })
            .to_string(),
        ));
    }
    let read_tool = read_tool?;
    let write_tool = write_tool?;
    let progress = Progress::scan(messages);

    if !progress.done(Capability::Read) {
        return Some(plan_one(
            read_tool,
            json!({
                "path": operation.path,
                "filePath": operation.path,
                "file_path": operation.path,
            })
            .to_string(),
        ));
    }
    if !progress.done(Capability::Write) {
        let read_result = latest_result(messages, Capability::Read)?;
        let mut content = tool_result::normalized_payload(&read_result)?;
        content.push('\n');
        content.push_str(operation.payload);
        content.push('\n');
        return Some(plan_one(
            write_tool,
            write_arguments(operation.path, &content),
        ));
    }

    let result = latest_result(messages, Capability::Write).unwrap_or_default();
    Some(AgenticPlan::Final(tool_result::render(
        command, &result, task,
    )))
}

fn parse_append(command: &str) -> Option<AppendOperation<'_>> {
    let (printf, destination) = command.split_once(" >> ")?;
    if !printf.starts_with("printf ") {
        return None;
    }
    let path = destination.split_whitespace().next()?;
    if !safe_relative_path(path) {
        return None;
    }
    let payload_end = printf.rfind('\'')?;
    let payload_start = printf[..payload_end].rfind('\'')?;
    let payload = &printf[payload_start + 1..payload_end];
    (!payload.is_empty()).then_some(AppendOperation { path, payload })
}

fn safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn latest_result(messages: &[ChatMessage], capability: Capability) -> Option<String> {
    let current_turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    messages
        .iter()
        .enumerate()
        .skip(current_turn)
        .rev()
        .find(|(index, message)| {
            message.role.eq_ignore_ascii_case("tool")
                && result_capability(messages, *index) == Some(capability)
        })
        .map(|(_, message)| message.content.plain_text())
}

fn has_tool_result(messages: &[ChatMessage], tool: &str) -> bool {
    messages.iter().enumerate().any(|(index, message)| {
        if !message.role.eq_ignore_ascii_case("tool") {
            return false;
        }
        if message
            .name
            .as_deref()
            .is_some_and(|name| name.eq_ignore_ascii_case(tool))
        {
            return true;
        }
        message.tool_call_id.as_ref().is_some_and(|call_id| {
            messages[..index]
                .iter()
                .flat_map(|prior| &prior.tool_calls)
                .any(|call| &call.id == call_id && call.function.name.eq_ignore_ascii_case(tool))
        })
    })
}
