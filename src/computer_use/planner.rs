use serde_json::Value;

use crate::agentic_coding::{AgenticPlan, PlannedToolCall};
use crate::protocol::ChatMessage;

use super::seed::{completion_message, missing_primitive_message, verification_failed_message};
use super::{ComputerUsePrimitive, capability_gap_for_request, plan_request};

#[must_use]
pub fn tool_for_primitive<'a>(
    tool_names: &[&'a str],
    primitive: ComputerUsePrimitive,
) -> Option<&'a str> {
    tool_names
        .iter()
        .copied()
        .find(|name| ComputerUsePrimitive::from_tool_name(name) == Some(primitive))
}

#[must_use]
pub fn plan_agentic_step(messages: &[ChatMessage], tool_names: &[&str]) -> Option<AgenticPlan> {
    let user_index = messages
        .iter()
        .rposition(|message| message.role == "user")?;
    let task = messages[user_index].content.plain_text();
    if let Some(gap) = capability_gap_for_request(&task) {
        return Some(AgenticPlan::Final(gap.response));
    }
    let plan = plan_request(&task)?;
    let tool_results = messages
        .iter()
        .enumerate()
        .skip(user_index + 1)
        .filter(|(_, message)| message.role == "tool")
        .collect::<Vec<_>>();
    for (step_index, (message_index, message)) in tool_results.iter().copied().enumerate() {
        let Some(step) = plan.steps.get(step_index) else {
            return Some(AgenticPlan::Final(verification_failed_message(
                &plan.locale,
                &plan.id,
                "unexpected-result",
                message.name.as_deref().unwrap_or("unknown"),
            )));
        };
        let tool_name = message
            .name
            .as_deref()
            .or_else(|| tool_name_for_result(messages, user_index, message_index));
        let matches_primitive =
            tool_name.and_then(ComputerUsePrimitive::from_tool_name) == Some(step.primitive);
        let verified = serde_json::from_str::<Value>(&message.content.plain_text())
            .ok()
            .and_then(|value| value.get("verified").and_then(Value::as_bool))
            == Some(true);
        if !matches_primitive || !verified {
            return Some(AgenticPlan::Final(verification_failed_message(
                &plan.locale,
                &plan.id,
                &step.id,
                step.primitive.name(),
            )));
        }
    }
    let completed = tool_results.len();
    let Some(step) = plan.steps.get(completed) else {
        return Some(AgenticPlan::Final(completion_message(
            &plan.locale,
            &plan.id,
            plan.steps.len(),
        )));
    };
    let Some(tool) = tool_for_primitive(tool_names, step.primitive) else {
        // A seed-defined computer-use plan is executed by the client that
        // advertises the primitives, and a client that advertises none of them
        // is not this route's client at all. Reporting the gap to it claims the
        // request and ends the turn with a sentence about `fs.write`, while the
        // ordinary write route sitting directly below could have written the
        // file the caller asked for -- the Agent CLI names its write tool
        // `write`, so every ladder node whose task the computer-use recogniser
        // matched came back with a capability gap and an empty workspace
        // (issue #1066).
        //
        // The message is still the right answer for a client that *is* running
        // computer-use plans and happens to be missing one primitive: there the
        // gap is real and naming it is what lets the client advertise it.
        if !advertises_computer_use(tool_names) {
            return None;
        }
        return Some(AgenticPlan::Final(missing_primitive_message(
            &plan.locale,
            &plan.id,
            &step.id,
            step.primitive,
        )));
    };
    let mut arguments = step.arguments.clone();
    if let Some(map) = arguments.as_object_mut() {
        map.insert("plan_id".to_owned(), Value::String(plan.id));
        map.insert("step_id".to_owned(), Value::String(step.id.clone()));
        map.insert(
            "precondition".to_owned(),
            Value::String(step.precondition.clone()),
        );
        map.insert(
            "postcondition".to_owned(),
            Value::String(step.postcondition.clone()),
        );
    }
    Some(AgenticPlan::ToolCalls(vec![PlannedToolCall {
        tool: tool.to_owned(),
        arguments: arguments.to_string(),
    }]))
}

/// Whether `tool_names` advertises any computer-use primitive at all.
///
/// The distinction this draws is between a client that speaks the computer-use
/// contract and one that does not. Only the first can act on a report that one
/// primitive is missing; for the second, the report is a refusal dressed as an
/// answer.
fn advertises_computer_use(tool_names: &[&str]) -> bool {
    tool_names
        .iter()
        .any(|name| ComputerUsePrimitive::from_tool_name(name).is_some())
}

fn tool_name_for_result(
    messages: &[ChatMessage],
    user_index: usize,
    result_index: usize,
) -> Option<&str> {
    let call_id = messages.get(result_index)?.tool_call_id.as_deref()?;
    messages[user_index + 1..result_index]
        .iter()
        .rev()
        .flat_map(|message| message.tool_calls.iter().rev())
        .find(|call| call.id == call_id)
        .map(|call| call.function.name.as_str())
}
