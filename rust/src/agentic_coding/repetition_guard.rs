//! Finite transcript-derived action budgets, independent of result/capability recognition.
use crate::protocol::ChatMessage;
use std::collections::BTreeMap;

pub const MAX_IDENTICAL_ACTION_ATTEMPTS: usize = crate::bounded_retry::MAX_FAILED_ATTEMPTS;

/// Stop even if errors are unrecognized, results are missing, or the server restarted.
pub fn repeated_action_stop(messages: &[ChatMessage]) -> Option<String> {
    if messages
        .iter()
        .filter(|message| message.role.eq_ignore_ascii_case("assistant"))
        .map(|message| message.tool_calls.len())
        .sum::<usize>()
        < MAX_IDENTICAL_ACTION_ATTEMPTS
    {
        return None;
    }
    let request = crate::protocol::latest_user_request(messages).unwrap_or_default();
    let messages = if super::file_read::file_read_task_for(&request).is_some() {
        super::file_read::file_read_evidence(messages)
    } else {
        messages
    };
    let mut counts = BTreeMap::new();
    let mut repeated = None;
    let mut total = 0usize;
    for message in messages
        .iter()
        .skip(super::planner::evidence_window_start(messages))
    {
        if !message.role.eq_ignore_ascii_case("assistant") {
            continue;
        }
        for call in &message.tool_calls {
            total = total.saturating_add(1);
            // serde_json objects use sorted keys, independent of the wire's key order.
            let arguments = serde_json::from_str::<serde_json::Value>(&call.function.arguments)
                .map_or_else(
                    |_| call.function.arguments.clone(),
                    |value| value.to_string(),
                );
            let count = counts
                .entry((call.function.name.to_lowercase(), arguments))
                .or_insert(0usize);
            *count += 1;
            if *count >= MAX_IDENTICAL_ACTION_ATTEMPTS {
                repeated = Some((call, *count));
            }
        }
    }
    if repeated.is_none() && total >= crate::bounded_retry::MAX_ACTIONS_PER_OBJECTIVE {
        return Some(format!(
            "Action budget exhausted after {total} tool declarations (limit {}). The task is incomplete; inspect progress before starting a new request.",
            crate::bounded_retry::MAX_ACTIONS_PER_OBJECTIVE
        ));
    }
    let (call, attempts) = repeated?;
    let receipt = messages.iter().rev().find(|message| {
        message.role.eq_ignore_ascii_case("tool")
            && message.tool_call_id.as_deref() == Some(call.id.as_str())
    });
    let result = receipt.map_or_else(
        || "No bound tool result was received.".to_owned(),
        |message| message.content.plain_text(),
    );
    let detail: String = result.chars().take(2048).collect();
    Some(format!(
        "Repeated action stopped after {attempts} attempts (limit {MAX_IDENTICAL_ACTION_ATTEMPTS}): {} {}. Last result: {detail}. The task is incomplete; inspect the tool contract or change strategy.",
        call.function.name, call.function.arguments
    ))
}
