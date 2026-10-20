//! Finite transcript-derived action budgets, independent of result/capability recognition.
use crate::protocol::ChatMessage;
use std::collections::BTreeMap;

pub const MAXIMUM_IDENTICAL_ACTION_ATTEMPTS: usize = crate::bounded_retry::MAXIMUM_FAILED_ATTEMPTS;

/// Stop even if errors are unrecognized, results are missing, or the server restarted.
pub fn repeated_action_stop(messages: &[ChatMessage]) -> Option<String> {
    if messages
        .iter()
        .filter(|message| message.role.eq_ignore_ascii_case("assistant"))
        .map(|message| message.tool_calls.len())
        .sum::<usize>()
        < MAXIMUM_IDENTICAL_ACTION_ATTEMPTS
    {
        return None;
    }
    let last_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"));
    let prior_calls = messages.iter().take(last_user.unwrap_or(0)).any(|message| {
        message.role.eq_ignore_ascii_case("assistant") && !message.tool_calls.is_empty()
    });
    let messages = if prior_calls
        && super::file_read::file_read_task_for(
            &crate::protocol::latest_user_request(messages).unwrap_or_default(),
        )
        .is_some()
    {
        super::file_read::file_read_evidence(messages)
    } else {
        messages
    };
    // With no earlier calls, classifying the initial request cannot change the counted window.
    let window = if prior_calls {
        super::planner::evidence_window_start(messages)
    } else {
        last_user.map_or(0, |index| index + 1)
    };
    let mut counts = BTreeMap::new();
    let mut repeated = None;
    let mut total = 0usize;
    for message in messages.iter().skip(window) {
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
            if *count >= MAXIMUM_IDENTICAL_ACTION_ATTEMPTS {
                repeated = Some((call, *count));
            }
        }
    }
    if repeated.is_none() && total >= crate::bounded_retry::MAXIMUM_ACTIONS_PER_OBJECTIVE {
        return Some(crate::seed::agentic_message(
            "action_budget_exhausted",
            &[
                ("total", &total.to_string()),
                (
                    "maximum",
                    &crate::bounded_retry::MAXIMUM_ACTIONS_PER_OBJECTIVE.to_string(),
                ),
            ],
        ));
    }
    let (call, attempts) = repeated?;
    let receipt = messages.iter().rev().find(|message| {
        message.role.eq_ignore_ascii_case("tool")
            && message.tool_call_id.as_deref() == Some(call.id.as_str())
    });
    let result = receipt.map_or_else(
        || crate::seed::agentic_message("action_receipt_missing", &[]),
        |message| message.content.plain_text(),
    );
    let detail: String = result.chars().take(2048).collect();
    Some(crate::seed::agentic_message(
        "action_repetition_stopped",
        &[
            ("attempts", &attempts.to_string()),
            ("maximum", &MAXIMUM_IDENTICAL_ACTION_ATTEMPTS.to_string()),
            ("tool", &call.function.name),
            ("arguments", &call.function.arguments),
            ("result", &detail),
        ],
    ))
}
