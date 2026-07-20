//! Friendly, lossless presentation of client-owned tool results (issue #750).

use serde_json::Value;

use crate::protocol::ChatMessage;
use crate::seed::{
    ROLE_TOOL_RESULT_DETAIL_REQUEST, ROLE_TOOL_RESULT_FIRST_REFERENCE,
    ROLE_TOOL_RESULT_LINE_REQUEST, ROLE_TOOL_RESULT_SECOND_REFERENCE, ROLE_TOOL_RESULT_URL_REQUEST,
};

struct NormalizedResult {
    payload: String,
    error: Option<String>,
    format: &'static str,
}

/// Remove client transport wrappers while preserving the tool's actual text.
/// Agentic planners consume this form; durable protocol recording still keeps
/// the original result byte-for-byte.
pub(super) fn normalized_payload(raw: &str) -> String {
    let result = normalize(raw);
    result
        .error
        .map_or(result.payload, |error| format!("Error: {error}"))
}

pub(super) fn render(label: &str, raw: &str, prompt: &str) -> String {
    let result = normalize(raw);
    let language = response_language(prompt);
    if let Some(error) = result.error {
        return fill("tool_result_failed", language, label, "", "", &error);
    }
    if result.payload.trim().is_empty() {
        let intent = if is_listing(label) {
            "tool_result_empty_list"
        } else if is_search(label) {
            "tool_result_empty_search"
        } else {
            "tool_result_empty_generic"
        };
        return fill(intent, language, label, "", "", "");
    }
    fill(
        "tool_result_completed",
        language,
        label,
        result.format,
        &result.payload,
        "",
    )
}

pub(super) fn latest_turn_answer(
    messages: &[ChatMessage],
    tool_names: &[&str],
    prompt: &str,
) -> Option<String> {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let (index, result) = messages
        .iter()
        .enumerate()
        .skip(start + 1)
        .rev()
        .find(|(_, message)| message.role.eq_ignore_ascii_case("tool"))?;
    if is_write_run_recipe(messages, tool_names) {
        return None;
    }
    let label = result_label(messages, index);
    Some(render(&label, &result.content.plain_text(), prompt))
}

pub(super) fn has_latest_turn_result(messages: &[ChatMessage]) -> bool {
    let Some(start) = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
    else {
        return false;
    };
    messages
        .iter()
        .skip(start + 1)
        .any(|message| message.role.eq_ignore_ascii_case("tool"))
}

fn is_write_run_recipe(messages: &[ChatMessage], tool_names: &[&str]) -> bool {
    let is_write = |name: &str| {
        let lower = name.to_ascii_lowercase();
        lower.contains("write") || lower.contains("create_file")
    };
    let is_run = |name: &str| {
        let lower = name.to_ascii_lowercase();
        lower.contains("run")
            || lower.contains("bash")
            || lower.contains("command")
            || lower.contains("exec")
            || lower.contains("shell")
    };
    tool_names.iter().copied().any(is_write)
        && tool_names.iter().copied().any(is_run)
        && messages
            .iter()
            .flat_map(|message| &message.tool_calls)
            .any(|call| is_write(&call.function.name))
}

pub(super) fn follow_up_answer(messages: &[ChatMessage], prompt: &str) -> Option<String> {
    let normalized_prompt = crate::engine::normalize_prompt(prompt);
    let lexicon = crate::seed::lexicon();
    let wants_url = lexicon.mentions_role(ROLE_TOOL_RESULT_URL_REQUEST, &normalized_prompt);
    let wants_line = lexicon.mentions_role(ROLE_TOOL_RESULT_LINE_REQUEST, &normalized_prompt);
    let wants_detail = lexicon.mentions_role(ROLE_TOOL_RESULT_DETAIL_REQUEST, &normalized_prompt);
    if !wants_url && !wants_line && !wants_detail {
        return None;
    }
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let result = messages[..latest_user]
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("tool"))?;
    let result = normalize(&result.content.plain_text());
    if wants_url {
        let urls = extract_urls(&result.payload);
        return urls
            .get(requested_index(lexicon, &normalized_prompt))
            .cloned();
    }
    if wants_line {
        return result
            .payload
            .lines()
            .nth(requested_index(lexicon, &normalized_prompt))
            .map(str::to_owned);
    }
    Some(result.payload)
}

fn requested_index(lexicon: &crate::seed::Lexicon, prompt: &str) -> usize {
    if lexicon.mentions_role(ROLE_TOOL_RESULT_SECOND_REFERENCE, prompt) {
        return 1;
    }
    if lexicon.mentions_role(ROLE_TOOL_RESULT_FIRST_REFERENCE, prompt) {
        return 0;
    }
    prompt
        .split(|character: char| !character.is_ascii_digit())
        .find_map(|digits| digits.parse::<usize>().ok())
        .unwrap_or(1)
        .saturating_sub(1)
}

fn normalize(raw: &str) -> NormalizedResult {
    let trimmed = raw.trim();
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return from_payload(&strip_transport_envelope(trimmed), None);
    };
    let Some(object) = value.as_object() else {
        return from_payload(&pretty_json(&value), None);
    };
    let nonzero_exit = ["exit_code", "exitCode"]
        .iter()
        .filter_map(|key| object.get(*key))
        .any(nonzero_status);
    let failed_http = object
        .get("status_code")
        .and_then(Value::as_u64)
        .is_some_and(|status| status >= 400);
    let failed_status = object.get("status").is_some_and(failed_status);
    let explicitly_unsuccessful = object.get("success").and_then(Value::as_bool) == Some(false);
    let explicit_error = ["error", "stderr", "failure"]
        .iter()
        .filter_map(|key| object.get(*key))
        .find_map(nonempty_text);
    if nonzero_exit
        || failed_http
        || failed_status
        || explicitly_unsuccessful
        || explicit_error.is_some()
    {
        let error = explicit_error
            .or_else(|| object.get("output").and_then(nonempty_text))
            .or_else(|| {
                ["exit_code", "exitCode", "status_code", "status"]
                    .iter()
                    .find_map(|key| object.get(*key).map(|value| format!("{key}={value}")))
            })
            .unwrap_or_default();
        return from_payload("", Some(error));
    }
    if let Some(payload) = ["output", "stdout", "content", "result"]
        .iter()
        .find_map(|key| object.get(*key))
    {
        let payload = payload
            .as_str()
            .map_or_else(|| pretty_json(payload), strip_transport_envelope);
        return from_payload(&payload, None);
    }
    from_payload(&pretty_json(&value), None)
}

fn from_payload(payload: &str, error: Option<String>) -> NormalizedResult {
    let trimmed = payload.trim().to_owned();
    if let Ok(json) = serde_json::from_str::<Value>(&trimmed) {
        if let Some(text) = mcp_text_content(&json) {
            return NormalizedResult {
                payload: text,
                error,
                format: "text",
            };
        }
        return NormalizedResult {
            payload: pretty_json(&json),
            error,
            format: "json",
        };
    }
    let format = if trimmed.starts_with("#!/bin/bash") || trimmed.starts_with("#!/usr/bin/env bash")
    {
        "bash"
    } else if trimmed.starts_with("#!/usr/bin/env python") {
        "python"
    } else {
        "text"
    };
    NormalizedResult {
        payload: trimmed,
        error,
        format,
    }
}

fn mcp_text_content(value: &Value) -> Option<String> {
    let content = value
        .as_object()
        .and_then(|object| object.get("content"))
        .unwrap_or(value)
        .as_array()?;
    if content.is_empty() {
        return None;
    }
    let text = content
        .iter()
        .map(|item| {
            let object = item.as_object()?;
            (object.get("type").and_then(Value::as_str) == Some("text"))
                .then(|| object.get("text").and_then(Value::as_str))
                .flatten()
        })
        .collect::<Option<Vec<_>>>()?;
    Some(text.join("\n"))
}

fn strip_transport_envelope(text: &str) -> String {
    let inner = text
        .split_once("<untrusted_context>")
        .and_then(|(_, rest)| rest.split_once("</untrusted_context>"))
        .map_or(text, |(inside, _)| inside);
    let lines = inner.lines().collect::<Vec<_>>();
    let output = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed == "Output:" || trimmed.starts_with("Output: ")
    });
    lines
        .iter()
        .enumerate()
        .filter_map(|line| {
            let (index, line) = line;
            if output.is_some_and(|output| index < output) {
                return None;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("Process Group PGID:") {
                None
            } else if let Some(output) = trimmed.strip_prefix("Output:") {
                let output = output.trim();
                (!matches!(output, "(empty)" | "(no output)")).then_some(output)
            } else {
                Some(*line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

fn nonzero_status(value: &Value) -> bool {
    value.as_i64().is_some_and(|status| status != 0)
        || value.as_str().is_some_and(|status| {
            status.parse::<i64>().map_or_else(
                |_| !matches!(status, "ok" | "success" | "completed"),
                |status| status != 0,
            )
        })
}

fn failed_status(value: &Value) -> bool {
    let numeric = value.as_i64().or_else(|| value.as_str()?.parse().ok());
    numeric.map_or_else(
        || {
            value
                .as_str()
                .is_some_and(|status| !matches!(status, "ok" | "success" | "completed" | "passed"))
        },
        |status| status < 0 || (0 < status && status < 100) || status >= 400,
    )
}

fn nonempty_text(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(text) => (!text.trim().is_empty()).then(|| text.trim().to_owned()),
        Value::Object(object) => object
            .get("message")
            .and_then(nonempty_text)
            .or_else(|| serde_json::to_string(value).ok()),
        other => serde_json::to_string(other).ok(),
    }
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn result_tool(messages: &[ChatMessage], index: usize) -> Option<&str> {
    let message = &messages[index];
    message.name.as_deref().or_else(|| {
        let id = message.tool_call_id.as_deref()?;
        messages[..index]
            .iter()
            .flat_map(|prior| &prior.tool_calls)
            .find(|call| call.id == id)
            .map(|call| call.function.name.as_str())
    })
}

fn result_label(messages: &[ChatMessage], index: usize) -> String {
    let message = &messages[index];
    let call = message.tool_call_id.as_deref().and_then(|id| {
        messages[..index]
            .iter()
            .flat_map(|prior| &prior.tool_calls)
            .find(|call| call.id == id)
    });
    if let Some(command) = call
        .and_then(|call| serde_json::from_str::<Value>(&call.function.arguments).ok())
        .as_ref()
        .and_then(|arguments| arguments.get("command").or_else(|| arguments.get("cmd")))
        .and_then(Value::as_str)
    {
        return command.to_owned();
    }
    result_tool(messages, index).unwrap_or("tool").to_owned()
}

fn extract_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut rest = text;
    while let Some(start) = [rest.find("https://"), rest.find("http://")]
        .into_iter()
        .flatten()
        .min()
    {
        let candidate = &rest[start..];
        let end = candidate
            .find(|character: char| character.is_whitespace() || "\"'<>)]}".contains(character))
            .unwrap_or(candidate.len());
        urls.push(candidate[..end].to_owned());
        rest = &candidate[end..];
    }
    urls
}

fn is_listing(label: &str) -> bool {
    let lower = label.to_ascii_lowercase();
    lower.split_whitespace().next() == Some("ls")
        || lower.contains("list")
        || lower.contains("glob")
}

fn is_search(label: &str) -> bool {
    let lower = label.to_ascii_lowercase();
    ["grep", "find", "search"]
        .iter()
        .any(|kind| lower.contains(kind))
}

fn response_language(prompt: &str) -> &'static str {
    match crate::language::detect(prompt).slug() {
        "ru" => "ru",
        "hi" => "hi",
        "zh" => "zh",
        _ => "en",
    }
}

fn fill(
    intent: &str,
    language: &str,
    tool: &str,
    format: &str,
    payload: &str,
    error: &str,
) -> String {
    const TOOL_PLACEHOLDER: &str = "{tool}";
    const FORMAT_PLACEHOLDER: &str = "{format}";
    const PAYLOAD_PLACEHOLDER: &str = "{payload}";
    const ERROR_PLACEHOLDER: &str = "{error}";

    crate::seed::response_for(intent, language)
        .or_else(|| crate::seed::response_for(intent, "en"))
        .unwrap_or_default()
        .replace(TOOL_PLACEHOLDER, tool)
        .replace(FORMAT_PLACEHOLDER, format)
        .replace(PAYLOAD_PLACEHOLDER, payload)
        .replace(ERROR_PLACEHOLDER, error)
}
