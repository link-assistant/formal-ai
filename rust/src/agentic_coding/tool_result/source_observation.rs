//! Lossless source observations separated from friendly result presentation.

use super::{
    ShellStep, StepOutcome, looks_like_error, nonempty_text, normalize, parse_shell_envelope,
    step_outcome,
};
use serde_json::Value;

/// Only explicit successful receipts of raw string stdout certify exact bytes.
pub(in crate::agentic_coding) fn observed_bytes_match(raw: &str, expected: &str) -> bool {
    let result = normalize(raw);
    if result.exit_code != Some(0) || result.error.is_some() || incomplete_receipt(raw) {
        return false;
    }
    let inner = raw
        .split_once("<untrusted_context>")
        .and_then(|(_, rest)| rest.split_once("</untrusted_context>"))
        .map_or(raw, |(inside, _)| inside);
    let inner = inner
        .strip_prefix("Command: ")
        .and_then(|rest| rest.split_once('\n').map(|(_, stdout)| stdout))
        .unwrap_or(inner);
    if let Some(stdout) = inner
        .strip_prefix("Output: ")
        .and_then(|stdout| stdout.strip_suffix("\nExit Code: 0"))
    {
        return stdout == expected;
    }
    let Ok(value) = serde_json::from_str::<Value>(raw.trim()) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    if ["is_error", "isError"]
        .iter()
        .filter_map(|key| object.get(*key))
        .any(|value| value.as_bool() == Some(true))
        || ["ok", "success"]
            .iter()
            .filter_map(|key| object.get(*key))
            .any(|value| value.as_bool() == Some(false))
        || ["error", "stderr", "failure"]
            .iter()
            .filter_map(|key| object.get(*key))
            .any(|value| nonempty_text(value).is_some())
    {
        return false;
    }
    ["output", "stdout", "content", "result"]
        .iter()
        .find_map(|key| object.get(*key))
        .and_then(Value::as_str)
        == Some(expected)
}

pub(in crate::agentic_coding) fn observed_digest_matches(raw: &str, expected: &str) -> bool {
    !incomplete_receipt(raw)
        && step_outcome(raw) != StepOutcome::Failed
        && observed_payload(raw)
            .as_deref()
            .and_then(|payload| payload.split_whitespace().next())
            == Some(expected)
}

/// The process exit status the harness reported for `raw`, when it reported one.
pub(in crate::agentic_coding) fn reported_exit_code(raw: &str) -> Option<i64> {
    normalize(raw).exit_code
}

/// Whether the *harness itself* judged this step a failure: it named an error field, or it reported a non-zero exit status (rung `R916-01`).
pub(in crate::agentic_coding) fn harness_reported_failure(raw: &str) -> bool {
    normalize(raw).error.is_some()
}

/// Read a shell-harness envelope, if the result is one.
pub(in crate::agentic_coding) fn shell_step(raw: &str) -> Option<ShellStep> {
    let result = parse_shell_envelope(raw.trim())?.into_result();
    Some(ShellStep {
        text: result.error.unwrap_or(result.payload),
    })
}

/// Remove client transport wrappers while preserving the tool's actual text.
pub(in crate::agentic_coding) fn normalized_payload(raw: &str) -> Option<String> {
    let result = normalize(raw);
    let succeeded = result.exit_code == Some(0);
    (result.error.is_none() && (succeeded || !looks_like_error(&result.payload)))
        .then_some(result.payload)
}

/// Return output after transport normalization when the transport itself succeeded.
pub(in crate::agentic_coding) fn observed_payload(raw: &str) -> Option<String> {
    if let Some(envelope) = parse_shell_envelope(raw)
        && envelope.exit_code == Some(0)
    {
        let inner = raw
            .split_once("<untrusted_context>")
            .and_then(|(_, rest)| rest.split_once("</untrusted_context>"))
            .map_or(raw, |(inside, _)| inside);
        let exact = inner
            .strip_prefix("Output: ")
            .and_then(|output| output.strip_suffix("\nExit Code: 0"));
        return Some(exact.map_or(envelope.output, str::to_owned));
    }
    let result = normalize(raw);
    if result.error.is_some() {
        return None;
    }
    if result.exit_code == Some(0)
        && let Ok(value) = serde_json::from_str::<Value>(raw.trim())
        && let Some(object) = value.as_object()
        && let Some(payload) = ["output", "stdout", "content", "result"]
            .iter()
            .find_map(|key| object.get(*key))
        && let Some(text) = payload.as_str()
    {
        return Some(text.to_owned());
    }
    Some(result.payload)
}

/// Declared partial or interrupted payloads do not certify bytes.
pub(in crate::agentic_coding) fn incomplete_receipt(raw: &str) -> bool {
    let Ok(Value::Object(object)) = serde_json::from_str::<Value>(raw.trim()) else {
        return false;
    };
    ["complete", "stream_complete"]
        .iter()
        .any(|key| object.get(*key).and_then(Value::as_bool) == Some(false))
        || ["truncated", "timed_out", "aborted"]
            .iter()
            .any(|key| object.get(*key).and_then(Value::as_bool) == Some(true))
        || object
            .get("signal")
            .and_then(Value::as_str)
            .is_some_and(|signal| !signal.is_empty())
}

/// Source-read status is independent of process exit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceReadStatus {
    ReportedSuccess,
    ReportedFailure,
    Unknown,
}

/// Parsed payload metadata; callers still own exact call/window/path binding.
#[derive(Debug)]
pub struct SourceReadObservation {
    pub status: SourceReadStatus,
    pub complete: bool,
    pub source: Option<String>,
    pub error: Option<String>,
}

/// A bare payload never declares status or complete-file observation.
#[must_use]
pub fn source_read_observation(
    raw: &str,
    explicitly_failed: bool,
    metadata: Option<&Value>,
    expected_path: &str,
) -> SourceReadObservation {
    let bound = metadata
        .filter(|metadata| metadata.get("path").and_then(Value::as_str) == Some(expected_path));
    let receipt = bound.map(Value::to_string);
    let exit = receipt.as_deref().and_then(reported_exit_code);
    let error = if explicitly_failed {
        Some(raw.to_owned())
    } else {
        receipt.as_deref().and_then(|receipt| {
            super::failure_message(receipt, false, false)
                .or_else(|| exit.filter(|exit| *exit != 0).map(|_| receipt.to_owned()))
        })
    };
    let status = if error.is_some() {
        SourceReadStatus::ReportedFailure
    } else if bound
        .and_then(|value| value.get("success"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        SourceReadStatus::ReportedSuccess
    } else {
        SourceReadStatus::Unknown
    };
    let framed = metadata
        .is_none()
        .then(|| super::super::code_artifact::source_from_agent_read_result(raw))
        .flatten();
    let whole_frame = framed.is_some()
        && raw
            .strip_prefix("<file>\n")
            .and_then(|body| body.rsplit_once("\n\n("))
            .is_some_and(|(numbered, footer)| {
                let rows = numbered.split('\n').collect::<Vec<_>>();
                footer
                    .split_whitespace()
                    .nth(5)
                    .and_then(|count| count.parse::<usize>().ok())
                    == Some(rows.len())
                    && rows.iter().enumerate().all(|(index, line)| {
                        line.split_once("| ")
                            .and_then(|(number, _)| number.parse::<usize>().ok())
                            == Some(index + 1)
                    })
            });
    let complete = error.is_none()
        && (whole_frame
            || (status == SourceReadStatus::ReportedSuccess
                && bound
                    .and_then(|value| value.get("format"))
                    .and_then(Value::as_str)
                    == Some("raw")
                && bound
                    .and_then(|value| value.get("complete"))
                    .and_then(Value::as_bool)
                    == Some(true)
                && receipt
                    .as_deref()
                    .is_some_and(|receipt| !incomplete_receipt(receipt))));
    let source = error
        .is_none()
        .then(|| framed.unwrap_or_else(|| raw.to_owned()));
    SourceReadObservation {
        status,
        complete,
        source,
        error,
    }
}
