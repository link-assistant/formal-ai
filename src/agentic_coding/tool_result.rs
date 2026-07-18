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

pub(super) fn latest_turn_answer(messages: &[ChatMessage], prompt: &str) -> Option<String> {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let (index, result) = messages
        .iter()
        .enumerate()
        .skip(start + 1)
        .rev()
        .find(|(_, message)| message.role.eq_ignore_ascii_case("tool"))?;
    let label = result_label(messages, index);
    Some(render(&label, &result.content.plain_text(), prompt))
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
        return from_payload(strip_transport_envelope(trimmed), None);
    };
    let Some(object) = value.as_object() else {
        return from_payload(pretty_json(&value), None);
    };
    let nonzero_exit = ["exit_code", "exitCode"]
        .iter()
        .filter_map(|key| object.get(*key))
        .any(nonzero_status);
    let failed_http = object
        .get("status_code")
        .and_then(Value::as_u64)
        .is_some_and(|status| status >= 400);
    let failed_status = object.get("status").is_some_and(nonzero_status);
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
        return from_payload(String::new(), Some(error));
    }
    if let Some(payload) = ["output", "stdout", "content", "result"]
        .iter()
        .find_map(|key| object.get(*key))
    {
        let payload = payload
            .as_str()
            .map(strip_transport_envelope)
            .unwrap_or_else(|| pretty_json(payload));
        return from_payload(payload, None);
    }
    from_payload(pretty_json(&value), None)
}

fn from_payload(payload: String, error: Option<String>) -> NormalizedResult {
    let trimmed = payload.trim().to_owned();
    if let Ok(json) = serde_json::from_str::<Value>(&trimmed) {
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

fn strip_transport_envelope(text: &str) -> String {
    let inner = text
        .split_once("<untrusted_context>")
        .and_then(|(_, rest)| rest.split_once("</untrusted_context>"))
        .map_or(text, |(inside, _)| inside);
    inner
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("Process Group PGID:") {
                None
            } else if let Some(output) = trimmed.strip_prefix("Output:") {
                let output = output.trim();
                (!matches!(output, "(empty)" | "(no output)")).then_some(output)
            } else {
                Some(line)
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
    crate::seed::response_for(intent, language)
        .or_else(|| crate::seed::response_for(intent, "en"))
        .unwrap_or_default()
        .replace("{tool}", tool)
        .replace("{format}", format)
        .replace("{payload}", payload)
        .replace("{error}", error)
}
