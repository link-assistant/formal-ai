//! Friendly, lossless presentation of client-owned tool results (issue #750).

use serde_json::Value;

use super::local_search;
use crate::protocol::ChatMessage;
use crate::seed::{
    ROLE_TOOL_RESULT_DETAIL_REQUEST, ROLE_TOOL_RESULT_FAILURE_SIGNAL,
    ROLE_TOOL_RESULT_FIRST_REFERENCE, ROLE_TOOL_RESULT_LINE_REQUEST,
    ROLE_TOOL_RESULT_SECOND_REFERENCE, ROLE_TOOL_RESULT_URL_REQUEST,
};

struct NormalizedResult {
    payload: String,
    error: Option<String>,
    format: &'static str,
}

/// Remove client transport wrappers while preserving the tool's actual text.
/// Agentic planners consume this form; durable protocol recording still keeps
/// the original result byte-for-byte.
pub(super) fn normalized_payload(raw: &str) -> Option<String> {
    let result = normalize(raw);
    (result.error.is_none() && !looks_like_error(&result.payload)).then_some(result.payload)
}

/// Return output after transport normalization when the transport itself
/// succeeded. Unlike [`normalized_payload`], this does not classify arbitrary
/// output vocabulary: verification targets are allowed to contain words such
/// as `error` or `failed` when those are the requested bytes.
pub(super) fn observed_payload(raw: &str) -> Option<String> {
    let result = normalize(raw);
    result.error.is_none().then_some(result.payload)
}

/// Return the client-owned failure detail when a result failed, whether the
/// signal arrived as protocol metadata, a structured transport envelope, or a
/// raw adapter message.
pub(super) fn failure_message(
    raw: &str,
    explicitly_failed: bool,
    infer_from_prose: bool,
) -> Option<String> {
    let result = normalize(raw);
    if let Some(error) = result.error {
        return Some(error);
    }
    if explicitly_failed || (infer_from_prose && looks_like_error(&result.payload)) {
        return Some(if result.payload.trim().is_empty() {
            raw.trim().to_owned()
        } else {
            result.payload
        });
    }
    None
}

fn looks_like_error(text: &str) -> bool {
    crate::seed::lexicon().mentions_role(
        ROLE_TOOL_RESULT_FAILURE_SIGNAL,
        &crate::engine::normalize_prompt(text),
    )
}

pub(super) fn render(label: &str, raw: &str, prompt: &str) -> String {
    let result = normalize(raw);
    let language = response_language(prompt);
    if let Some(error) = result.error {
        let failure = fill("tool_result_failed", language, label, "", "", &error);
        return crate::failure_reporting::append_invitation(&failure, language);
    }
    // Some harnesses, including Agent CLI's shell adapter, return only the
    // process text and omit an exit-code field. Reuse the seed-backed failure
    // lexicon already used by `normalized_payload` so those genuine failures
    // reach the same opt-in report path without matching hardcoded prose here.
    if looks_like_error(&result.payload) {
        let failure = fill(
            "tool_result_failed",
            language,
            label,
            "",
            "",
            &result.payload,
        );
        return crate::failure_reporting::append_invitation(&failure, language);
    }
    if result.payload.trim().is_empty() {
        let intent = if local_search::request_for(prompt).is_some() {
            "tool_result_empty_local_path_search"
        } else if is_listing(label) {
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

pub(super) fn render_failure(label: &str, detail: &str, prompt: &str) -> String {
    let language = response_language(prompt);
    let failure = fill("tool_result_failed", language, label, "", "", detail);
    crate::failure_reporting::append_invitation(&failure, language)
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
    let is_write = super::capability_router::is_workspace_creation_tool;
    let is_run =
        |name: &str| super::planner::tool_capability(name) == Some(super::planner::Capability::Run);
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
        let payload = strip_transport_envelope(trimmed);
        let error = plain_nonzero_status(&payload).then(|| payload.clone());
        return from_payload(&payload, error);
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
    let status = ["status", "state", "outcome"]
        .iter()
        .find_map(|key| object.get(*key));
    let expected_stop = status.is_some_and(expected_stop_status);
    let failed_status = status.is_some_and(failed_status);
    let explicitly_unsuccessful = ["ok", "success"]
        .iter()
        .filter_map(|key| object.get(*key))
        .any(|value| value.as_bool() == Some(false));
    let explicitly_failed = ["is_error", "isError"]
        .iter()
        .filter_map(|key| object.get(*key))
        .any(|value| value.as_bool() == Some(true));
    let explicit_error = ["error", "stderr", "failure"]
        .iter()
        .filter_map(|key| object.get(*key))
        .find_map(nonempty_text);
    if !expected_stop
        && (nonzero_exit
            || failed_http
            || failed_status
            || explicitly_unsuccessful
            || explicitly_failed
            || explicit_error.is_some())
    {
        let error = explicit_error
            .or_else(|| object.get("output").and_then(nonempty_text))
            .or_else(|| object.get("content").and_then(nonempty_text))
            .or_else(|| object.get("result").and_then(nonempty_text))
            .or_else(|| {
                [
                    "exit_code",
                    "exitCode",
                    "status_code",
                    "status",
                    "state",
                    "outcome",
                ]
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

fn plain_nonzero_status(text: &str) -> bool {
    text.lines().any(|line| {
        let normalized = line.trim().to_ascii_lowercase();
        ["exit code:", "exit code =", "exited with status"]
            .iter()
            .find_map(|marker| normalized.strip_prefix(marker))
            .and_then(|suffix| {
                suffix
                    .trim()
                    .split(|character: char| !character.is_ascii_digit())
                    .find(|part| !part.is_empty())
            })
            .and_then(|digits| digits.parse::<u32>().ok())
            .is_some_and(|code| code != 0)
    })
}

fn from_payload(payload: &str, error: Option<String>) -> NormalizedResult {
    let trimmed = payload.trim();
    let trimmed = if matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "(empty)"
            | "(no output)"
            | "(bash completed with no output)"
            | "no output"
            | "command completed with no output"
            | "command completed without output"
            | "command produced no output"
    ) {
        String::new()
    } else {
        trimmed.to_owned()
    };
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

/// Drop the wrapper a client puts around a shell result — Codex's
/// `exec_command` prefixes `Chunk ID` / `Wall time` / `Process exited with code`
/// lines before the real `Output:` — so a recipe that answers with the command's
/// output quotes the file, not the transport (issue #671).
pub(super) fn strip_transport_envelope(text: &str) -> String {
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
            value.as_str().is_some_and(|status| {
                !matches!(
                    status.to_ascii_lowercase().as_str(),
                    "ok" | "success" | "succeeded" | "completed" | "passed"
                ) && !expected_stop_status(value)
            })
        },
        |status| status < 0 || (0 < status && status < 100) || status >= 400,
    )
}

fn expected_stop_status(value: &Value) -> bool {
    value.as_str().is_some_and(|status| {
        matches!(
            status.to_ascii_lowercase().as_str(),
            "refused"
                | "denied"
                | "cancelled"
                | "canceled"
                | "aborted"
                | "pending"
                | "awaiting_approval"
                | "not_granted"
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
    crate::language::detect(prompt).slug()
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

    crate::seed::localized_response(intent, language)
        .unwrap_or_default()
        .replace(TOOL_PLACEHOLDER, tool)
        .replace(FORMAT_PLACEHOLDER, format)
        .replace(PAYLOAD_PLACEHOLDER, payload)
        .replace(ERROR_PLACEHOLDER, error)
}
