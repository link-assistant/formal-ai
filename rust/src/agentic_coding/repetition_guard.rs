//! Finite transcript-derived action budgets, independent of result/capability recognition.
use super::planner::{AgenticPlan, PlannedToolCall};
use crate::protocol::ChatMessage;
use std::collections::BTreeMap;

pub const MAXIMUM_IDENTICAL_FAILED_ACTIONS: usize = 2;

pub const MAXIMUM_IDENTICAL_ACTION_ATTEMPTS: usize = crate::bounded_retry::MAXIMUM_FAILED_ATTEMPTS;

/// Stop even if errors are unrecognized, results are missing, or the server restarted.
#[must_use]
pub fn repeated_action_stop(messages: &[ChatMessage]) -> Option<String> {
    repeated_action_stop_for_candidates(messages, None)
}

fn canonical_action_key(tool: &str, arguments: &str) -> (String, String) {
    let arguments = serde_json::from_str::<serde_json::Value>(arguments)
        .map_or_else(|_| arguments.to_owned(), |value| value.to_string());
    (tool.to_lowercase(), arguments)
}

/// Proposed calls filter only the recognized two-failure budget, never global action caps.
#[must_use]
pub fn repeated_action_stop_for_candidates(
    messages: &[ChatMessage],
    candidates: Option<&[PlannedToolCall]>,
) -> Option<String> {
    if messages
        .iter()
        .filter(|message| message.role.eq_ignore_ascii_case("assistant"))
        .map(|message| message.tool_calls.len())
        .sum::<usize>()
        < MAXIMUM_IDENTICAL_FAILED_ACTIONS
    {
        return None;
    }
    let candidate_keys = candidates.map(|calls| {
        calls
            .iter()
            .map(|call| canonical_action_key(&call.tool, &call.arguments))
            .collect::<std::collections::BTreeSet<_>>()
    });
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
    // Consume matching receipts once, including clients that reuse IDs sequentially.
    let mut pending = BTreeMap::new();
    let mut failed_counts = BTreeMap::new();
    let mut failed = None;
    for message in messages.iter().skip(window) {
        if message.role.eq_ignore_ascii_case("assistant") {
            let mut identifiers = std::collections::BTreeSet::new();
            for call in &message.tool_calls {
                if call.id.is_empty() {
                    continue;
                }
                let qualified = identifiers.insert(call.id.as_str()).then_some(call);
                pending.insert(call.id.as_str(), qualified);
            }
        } else if message.role.eq_ignore_ascii_case("tool") {
            let Some(call) = message
                .tool_call_id
                .as_deref()
                .and_then(|identifier| pending.remove(identifier))
                .flatten()
            else {
                continue;
            };
            if message
                .name
                .as_ref()
                .is_some_and(|name| !name.eq_ignore_ascii_case(&call.function.name))
            {
                continue;
            }
            let result = message.content.plain_text();
            if super::tool_result::failure_message(&result, message.is_error, false).is_none() {
                continue;
            }
            let key = canonical_action_key(&call.function.name, &call.function.arguments);
            let count = failed_counts.entry(key.clone()).or_insert(0usize);
            *count += 1;
            if *count >= MAXIMUM_IDENTICAL_FAILED_ACTIONS
                && candidate_keys
                    .as_ref()
                    .is_none_or(|keys| keys.contains(&key))
            {
                failed = Some((call, *count, result));
            }
        }
    }
    if let Some((call, attempts, result)) = failed {
        let detail: String = result.chars().take(2048).collect();
        return Some(crate::seed::agentic_message(
            "action-repetition-stopped",
            &[
                ("attempts", &attempts.to_string()),
                ("maximum", &MAXIMUM_IDENTICAL_FAILED_ACTIONS.to_string()),
                ("tool", &call.function.name),
                ("arguments", &call.function.arguments),
                ("result", &detail),
            ],
        ));
    }
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
            let key = canonical_action_key(&call.function.name, &call.function.arguments);
            let count = counts.entry(key).or_insert(0usize);
            *count += 1;
            if *count >= MAXIMUM_IDENTICAL_ACTION_ATTEMPTS {
                repeated = Some((call, *count));
            }
        }
    }
    if repeated.is_none() && total >= crate::bounded_retry::MAXIMUM_ACTIONS_PER_OBJECTIVE {
        return Some(crate::seed::agentic_message(
            "action-budget-exhausted",
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
        || crate::seed::agentic_message("action-receipt-missing", &[]),
        |message| message.content.plain_text(),
    );
    let detail: String = result.chars().take(2048).collect();
    Some(crate::seed::agentic_message(
        "action-repetition-stopped",
        &[
            ("attempts", &attempts.to_string()),
            ("maximum", &MAXIMUM_IDENTICAL_ACTION_ATTEMPTS.to_string()),
            ("tool", &call.function.name),
            ("arguments", &call.function.arguments),
            ("result", &detail),
        ],
    ))
}

/// Apply the failure budget after a route is chosen, including null and terminal routes.
pub(super) fn guard_plan(
    messages: &[ChatMessage],
    plan: Option<AgenticPlan>,
    result: &mut Option<super::final_result::FinalResult>,
) -> Option<AgenticPlan> {
    if crate::protocol::latest_user_request(messages).is_none() {
        return plan;
    }
    let candidates = match &plan {
        Some(AgenticPlan::ToolCalls(calls)) => Some(calls.as_slice()),
        _ => None,
    };
    if let Some(reason) = repeated_action_stop_for_candidates(messages, candidates) {
        Some(super::final_result::record(
            AgenticPlan::Final(reason),
            super::final_result::FinalDisposition::Failure,
            "repeated_action_budget",
            result,
        ))
    } else {
        plan
    }
}
