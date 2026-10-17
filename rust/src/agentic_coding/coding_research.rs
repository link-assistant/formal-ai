//! Thin receipt syntax adapter for canonical generated research decisions.
#[path = "../coding_research_kernel.rs"]
mod kernel;

use super::tool_result::{StepOutcome, incomplete_receipt, normalized_payload, step_outcome};

/// Terminal TAP evidence is decided by the same generated kernel as JavaScript.
#[must_use]
pub fn verification_receipt_gap(raw: &str) -> Option<String> {
    let complete = !incomplete_receipt(raw) && step_outcome(raw) == StepOutcome::Succeeded;
    let payload = normalized_payload(raw).unwrap_or_default();
    let mut counts = [-1.0, -1.0, -1.0, 0.0, 0.0];
    let mut unique = true;
    for (index, field) in ["tests", "pass", "fail", "skipped", "cancelled"].iter().enumerate() {
        let prefix = format!("# {field} ");
        let values: Vec<&str> = payload
            .split('\n')
            .filter_map(|line| line.strip_suffix('\r').unwrap_or(line).strip_prefix(&prefix))
            .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
            .collect();
        if values.len() > 1 {
            unique = false;
        }
        if let Some(value) = values.first() {
            counts[index] = value.parse::<f64>().unwrap_or(f64::NAN);
        }
    }
    let reason = kernel::tap_verification_decision(
        complete && unique,
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
    );
    if reason.is_empty() {
        None
    } else {
        Some(reason)
    }
}
