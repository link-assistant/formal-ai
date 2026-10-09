//! Lossless source observations separated from friendly result presentation.

use super::{
    ShellStep, StepOutcome, looks_like_error, nonempty_text, normalize, parse_shell_envelope,
    step_outcome,
};
use serde_json::Value;

/// Only explicit successful receipts of raw string stdout certify exact bytes.
pub(crate) fn observed_bytes_match(raw: &str, expected: &str) -> bool {
    let result = normalize(raw);
    if result.exit_code != Some(0) || result.error.is_some() {
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

pub(crate) fn observed_digest_matches(raw: &str, expected: &str) -> bool {
    step_outcome(raw) != StepOutcome::Failed
        && observed_payload(raw)
            .as_deref()
            .and_then(|payload| payload.split_whitespace().next())
            == Some(expected)
}

/// The process exit status the harness reported for `raw`, when it reported one.
///
/// Callers that build their own report of a stopped step need the status the
/// workspace answered with, not a rendering of it: a recipe that stops on a
/// precondition says which check stopped it and with what code (issue #944).
pub(crate) fn reported_exit_code(raw: &str) -> Option<i64> {
    normalize(raw).exit_code
}

/// Whether the *harness itself* judged this step a failure: it named an error
/// field, or it reported a non-zero exit status (rung `R916-01`).
///
/// The failure lexicon [`step_outcome`] falls back on is deliberately not
/// consulted here. Callers that hold bytes which are legitimately file contents
/// use this: a file that merely *reads* like an error message is still that
/// file's contents, and only the harness can say the read failed.
pub(crate) fn harness_reported_failure(raw: &str) -> bool {
    normalize(raw).error.is_some()
}

/// Read a shell-harness envelope, if the result is one. Callers that report a
/// step's outcome use this to quote the command's own text instead of the
/// transport wrapper; [`step_outcome`] reads the status the harness observed.
pub(in crate::agentic_coding) fn shell_step(raw: &str) -> Option<ShellStep> {
    let result = parse_shell_envelope(raw.trim())?.into_result();
    Some(ShellStep {
        text: result.error.unwrap_or(result.payload),
    })
}

/// Remove client transport wrappers while preserving the tool's actual text.
/// Agentic planners consume this form; durable protocol recording still keeps
/// the original result byte-for-byte.
pub(crate) fn normalized_payload(raw: &str) -> Option<String> {
    let result = normalize(raw);
    let succeeded = result.exit_code == Some(0);
    (result.error.is_none() && (succeeded || !looks_like_error(&result.payload)))
        .then_some(result.payload)
}

/// Return output after transport normalization when the transport itself
/// succeeded. Unlike [`normalized_payload`], this does not classify arbitrary
/// output vocabulary: verification targets are allowed to contain words such
/// as `error` or `failed` when those are the requested bytes.
pub(crate) fn observed_payload(raw: &str) -> Option<String> {
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
