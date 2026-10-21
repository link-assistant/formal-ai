//! Typed terminal client status; twin: js/agentic/tool_result.mjs.
use super::{Value, source_observation};
/// Only a terminal typed provider part supplies nested status; source JSON does not.
pub(super) fn project(value: &Value) -> Option<Value> {
    let part = if value.get("type").and_then(Value::as_str) == Some("tool_use") {
        value.get("part").unwrap_or(value)
    } else {
        value
    };
    if !matches!(
        part.get("type").and_then(Value::as_str),
        Some("tool" | "tool_use")
    ) || part
        .get("tool")
        .or_else(|| part.get("name"))
        .and_then(Value::as_str)
        .is_none()
    {
        return None;
    }
    let state = part.get("state")?.as_object()?;
    if !matches!(
        state.get("status").and_then(Value::as_str),
        Some("completed" | "error")
    ) {
        return None;
    }
    let metadata = state.get("metadata").and_then(Value::as_object);
    if ["status", "outcome"]
        .iter()
        .filter_map(|key| value.get(*key))
        .any(super::expected_stop_status)
    {
        return None;
    }
    let mut projected = serde_json::Map::new();
    for key in [
        "exit_code",
        "exitCode",
        "status_code",
        "status",
        "outcome",
        "ok",
        "success",
        "is_error",
        "isError",
        "timed_out",
        "aborted",
        "signal",
        "error",
        "stderr",
        "failure",
        "output",
        "stdout",
        "content",
        "result",
    ] {
        if let Some(entry) = state.get(key) {
            projected.insert(key.to_owned(), entry.clone());
        }
    }
    let mut codes: Vec<i64> = [
        value.get("exit_code"),
        value.get("exitCode"),
        state.get("exit_code"),
        state.get("exitCode"),
        metadata.and_then(|record| record.get("exit")),
    ]
    .into_iter()
    .flatten()
    .filter_map(|entry| entry.as_i64().or_else(|| entry.as_str()?.parse().ok()))
    .collect();
    if let Some(code) = state
        .get("error")
        .and_then(Value::as_str)
        .and_then(source_observation::labelled_process_exit_code)
    {
        codes.push(code);
    }
    let code = codes
        .iter()
        .find(|code| **code != 0)
        .or_else(|| codes.first())
        .copied();
    if let Some(code) = code {
        projected.insert("exit_code".to_owned(), code.into());
    }
    let marked = |record: Option<&serde_json::Map<String, Value>>, key: &str| {
        record
            .and_then(|record| record.get(key))
            .and_then(Value::as_bool)
            == Some(true)
    };
    if marked(value.as_object(), "is_error") || marked(value.as_object(), "isError") {
        projected.insert("is_error".to_owned(), true.into());
    }
    if ["status", "outcome"]
        .iter()
        .filter_map(|key| value.get(*key))
        .any(super::failed_status)
        || value
            .get("status_code")
            .and_then(Value::as_u64)
            .is_some_and(|status| status >= 400)
    {
        projected.insert("is_error".to_owned(), true.into());
    }
    if ["success", "ok"]
        .iter()
        .any(|key| value.get(*key).and_then(Value::as_bool) == Some(false))
    {
        projected.insert("success".to_owned(), false.into());
    }
    for key in ["error", "stderr", "failure"] {
        if value.get(key).and_then(super::nonempty_text).is_some()
            && projected.get(key).and_then(super::nonempty_text).is_none()
            && let Some(error) = value.get(key)
        {
            projected.insert(key.to_owned(), error.clone());
        }
    }
    if marked(metadata, "timedOut") || marked(value.as_object(), "timed_out") {
        projected.insert("timed_out".to_owned(), true.into());
    }
    if marked(metadata, "aborted") || marked(value.as_object(), "aborted") {
        projected.insert("aborted".to_owned(), true.into());
    }
    if let Some(signal) = [
        state.get("signal"),
        metadata.and_then(|record| record.get("signal")),
        value.get("signal"),
    ]
    .into_iter()
    .flatten()
    .filter_map(Value::as_str)
    .find(|signal| !signal.is_empty())
    {
        projected.insert("signal".to_owned(), signal.into());
    }
    if !["output", "stdout", "content", "result"]
        .iter()
        .any(|key| projected.contains_key(*key))
        && let Some(output) = metadata.and_then(|record| record.get("output"))
    {
        projected.insert("output".to_owned(), output.clone());
    }
    // No type/part/state field is projected, so the normalizer cannot recurse.
    Some(Value::Object(projected))
}
