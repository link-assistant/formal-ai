//! Fresh digest evidence for the exact current whole-file mutation.
use super::{VerifiedChange, argument_matches_path, result_for_command};
use crate::agentic_coding::code_task::{render_seeded_change_with_lists, render_seeded_outcome};
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, record};
use crate::agentic_coding::planner::{
    AgenticPlan, Capability, plan_one, tool_capability, tool_for,
};
use crate::agentic_coding::tool_result::{StepOutcome, command_argument, step_outcome};
use crate::protocol::{ChatMessage, ToolCall};
use serde_json::{Value, json};

pub(in crate::agentic_coding) fn plan_digest_verification(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    change: &VerifiedChange<'_>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let command = ["sha256sum -- ", change.target].concat();
    let Some(observed) = result_for_command(current_turn, &command) else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(tool, json!({"command": command}).to_string()));
    };
    let digest = crate::source_fetch::sha256_hex(change.expected.as_bytes());
    if !crate::agentic_coding::tool_result::observed_digest_matches(&observed, &digest) {
        return failed(task, change.target, result);
    }
    Some(record(
        AgenticPlan::Final(render_seeded_change_with_lists(
            change.intent,
            task,
            change.target,
            change.slots,
            change.list_slots,
        )?),
        FinalDisposition::Finding,
        "workspace_change_observed",
        result,
    ))
}

fn receipt_call(messages: &[ChatMessage], index: usize) -> Option<&ToolCall> {
    let message = &messages[index];
    if !message.role.eq_ignore_ascii_case("tool") {
        return None;
    }
    let identity = message.tool_call_id.as_deref()?;
    messages[..index]
        .iter()
        .rev()
        .flat_map(|message| message.tool_calls.iter().rev())
        .find(|call| call.id == identity)
}

fn write_window<'a>(
    messages: &'a [ChatMessage],
    target: &str,
    expected: &str,
) -> Option<&'a [ChatMessage]> {
    for index in (0..messages.len()).rev() {
        let Some(call) = receipt_call(messages, index) else {
            continue;
        };
        if tool_capability(&call.function.name) != Some(Capability::Write) {
            continue;
        }
        let Ok(arguments) = serde_json::from_str::<Value>(&call.function.arguments) else {
            continue;
        };
        if !argument_matches_path(&arguments, target) {
            continue;
        }
        let message = &messages[index];
        if arguments.get("content").and_then(Value::as_str) != Some(expected)
            || message.is_error
            || step_outcome(&message.content.plain_text()) == StepOutcome::Failed
        {
            return None;
        }
        return Some(&messages[index + 1..]);
    }
    None
}

fn failed(task: &str, target: &str, result: &mut Option<FinalResult>) -> Option<AgenticPlan> {
    Some(record(
        AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            target,
        )?),
        FinalDisposition::Failure,
        "coding_workspace_verification_failed",
        result,
    ))
}

pub(in crate::agentic_coding) fn plan_write_digest_verification(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    change: &VerifiedChange<'_>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let Some(current) = write_window(messages, change.target, change.expected) else {
        return failed(task, change.target, result);
    };
    let command = ["sha256sum -- ", change.target].concat();
    for index in (0..current.len()).rev() {
        let Some(call) = receipt_call(current, index) else {
            continue;
        };
        if tool_capability(&call.function.name) != Some(Capability::Run)
            || command_argument(&call.function.arguments).as_deref() != Some(command.as_str())
        {
            continue;
        }
        if current[index].is_error {
            return failed(task, change.target, result);
        }
        break;
    }
    plan_digest_verification(task, current, tool_names, change, result)
}
