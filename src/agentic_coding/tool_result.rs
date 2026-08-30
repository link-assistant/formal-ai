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
    /// The process exit status the harness reported, when it reported one.
    /// This is the *primary* success signal (issues #905 and #908); the failure
    /// lexicon below is only consulted when no harness reported a status.
    exit_code: Option<i64>,
}

/// What a finished tool step actually did, read from the harness's own report.
///
/// Issues #905 and #908 are the two directions of one defect: the verdict was
/// keyed on whether the result *text* looked like a failure, so `cat: …: No such
/// file or directory` with `Exit Code: 1` was read as success and `Exit Code: 0`
/// with `Output: (empty)` / `Error: (none)` was read as failure. Every harness in
/// the #902–#909 corpus reports the status explicitly, so it is read, not inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    /// The harness reported a zero exit status.
    Succeeded,
    /// The harness reported a non-zero status, or an explicit error field.
    Failed,
    /// No harness status was reported and nothing in the text signals failure.
    Unreported,
}

/// Read the verdict of one finished tool result (issues #905 and #908).
#[must_use]
pub fn step_outcome(raw: &str) -> StepOutcome {
    let result = normalize(raw);
    if result.error.is_some() {
        return StepOutcome::Failed;
    }
    match result.exit_code {
        Some(0) => StepOutcome::Succeeded,
        Some(_) => StepOutcome::Failed,
        None if looks_like_error(&result.payload) => StepOutcome::Failed,
        None => StepOutcome::Unreported,
    }
}

/// The workspace's veto over a completion claim (issue #905).
///
/// "Completed … and verified it" is a claim about the workspace, so the
/// workspace gets the last word: issue #905 watched that sentence go out after
/// `cat hello.txt` had exited 1 because the file was never created. The
/// verification the plan itself named had already disproved the claim. When the
/// last observed step failed, the report of that failure replaces the claim.
pub(super) fn failed_verification(
    run_outputs: &[String],
    verification_command: &str,
    prompt: &str,
) -> Option<String> {
    let output = run_outputs
        .last()
        .filter(|output| step_outcome(output) == StepOutcome::Failed)?;
    Some(render(verification_command, output, prompt))
}

/// The process exit status the harness reported for `raw`, when it reported one.
///
/// Callers that build their own report of a stopped step need the status the
/// workspace answered with, not a rendering of it: a recipe that stops on a
/// precondition says which check stopped it and with what code (issue #944).
pub(super) fn reported_exit_code(raw: &str) -> Option<i64> {
    normalize(raw).exit_code
}

/// One shell step as the harness itself reported it: the command's own text,
/// with the transport envelope removed.
pub(super) struct ShellStep {
    pub(super) text: String,
}

/// Whether the *harness itself* judged this step a failure: it named an error
/// field, or it reported a non-zero exit status (rung `R916-01`).
///
/// The failure lexicon [`step_outcome`] falls back on is deliberately not
/// consulted here. Callers that hold bytes which are legitimately file contents
/// use this: a file that merely *reads* like an error message is still that
/// file's contents, and only the harness can say the read failed.
pub(super) fn harness_reported_failure(raw: &str) -> bool {
    normalize(raw).error.is_some()
}

/// Read a shell-harness envelope, if the result is one. Callers that report a
/// step's outcome use this to quote the command's own text instead of the
/// transport wrapper; [`step_outcome`] reads the status the harness observed.
pub(super) fn shell_step(raw: &str) -> Option<ShellStep> {
    let result = parse_shell_envelope(raw.trim())?.into_result();
    Some(ShellStep {
        text: result.error.unwrap_or(result.payload),
    })
}

/// Remove client transport wrappers while preserving the tool's actual text.
/// Agentic planners consume this form; durable protocol recording still keeps
/// the original result byte-for-byte.
pub(super) fn normalized_payload(raw: &str) -> Option<String> {
    let result = normalize(raw);
    let succeeded = result.exit_code == Some(0);
    (result.error.is_none() && (succeeded || !looks_like_error(&result.payload)))
        .then_some(result.payload)
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

/// Status-less adapters sometimes return only a provider's denial or error
/// notice. Inspect the leading diagnostic region for that fallback: successful
/// pages and documents can legitimately contain error vocabulary or HTTP codes
/// later in their contents.
const PROSE_FAILURE_PREFIX_CHARS: usize = 512;

fn looks_like_error(text: &str) -> bool {
    if quotes_a_located_line(text) {
        return false;
    }
    let diagnostic_prefix: String = text.chars().take(PROSE_FAILURE_PREFIX_CHARS).collect();
    crate::seed::lexicon().mentions_role(
        ROLE_TOOL_RESULT_FAILURE_SIGNAL,
        &crate::engine::normalize_prompt(&diagnostic_prefix),
    )
}

/// Whether the result opens with a line quoted from somewhere else.
///
/// A search answers with other files' text, and every line it returns carries
/// the place it was found: `./scripts/install.sh:260:    log "the 'code' CLI was
/// not found on PATH."`. The failure vocabulary in that line belongs to
/// `install.sh`, not to the search -- but the diagnostic prefix above cannot
/// tell them apart by wording, so issue #1066 watched a `grep` that had matched
/// fifty lines report itself as the command that failed, and the node whose only
/// evidence was that search recorded the failure as its proof.
///
/// The distinguishing property is not vocabulary and not the tool's name: it is
/// that a quotation says where it came from. A harness announcing its own
/// failure names no line number -- `grep: /etc/shadow: Permission denied` keeps
/// the old reading, because `/etc/shadow` is not a count of lines.
fn quotes_a_located_line(text: &str) -> bool {
    let Some(first) = text.lines().find(|line| !line.trim().is_empty()) else {
        return false;
    };
    let mut fields = first.splitn(3, ':');
    let (Some(origin), Some(line_number), Some(_)) = (fields.next(), fields.next(), fields.next())
    else {
        return false;
    };
    !origin.trim().is_empty()
        && !origin.contains(char::is_whitespace)
        && !line_number.is_empty()
        && line_number.bytes().all(|byte| byte.is_ascii_digit())
}

pub(super) fn render(label: &str, raw: &str, prompt: &str) -> String {
    let result = normalize(raw);
    let language = response_language(prompt);
    if let Some(error) = result.error {
        return failure_report(label, &error, result.exit_code, language);
    }
    // Some harnesses, including Agent CLI's shell adapter, return only the
    // process text and omit an exit-code field. Reuse the seed-backed failure
    // lexicon already used by `normalized_payload` so those genuine failures
    // reach the same opt-in report path without matching hardcoded prose here.
    // A harness that *did* report a zero status has already answered the
    // question, so its own verdict wins over the lexicon (issue #908).
    if result.exit_code != Some(0) && looks_like_error(&result.payload) {
        return failure_report(label, &result.payload, result.exit_code, language);
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
        return fill(intent, language, &[("{tool}", label)]);
    }
    fill(
        "tool_result_completed",
        language,
        &[
            ("{tool}", label),
            ("{format}", result.format),
            ("{payload}", &result.payload),
        ],
    )
}

/// Report a step the planner itself observed to have failed, naming the tool or
/// path it failed on. The plan's own execution state machine uses this when a
/// client-owned attempt came back marked as an error (issue #905).
pub(super) fn render_failure(label: &str, detail: &str, prompt: &str) -> String {
    let result = normalize(detail);
    let error = result.error.unwrap_or(result.payload);
    failure_report(label, &error, result.exit_code, response_language(prompt))
}

/// Report a failed step. When the harness reported the status, the status is
/// named: attributing the failure to the harness hid which command failed and
/// why (issue #908, suggested fix 3).
#[allow(clippy::literal_string_with_formatting_args)]
fn failure_report(label: &str, error: &str, exit_code: Option<i64>, language: &str) -> String {
    let code = exit_code.map(|code| code.to_string());
    let failure = code.as_ref().map_or_else(
        || {
            fill(
                "tool_result_failed",
                language,
                &[("{tool}", label), ("{error}", error.trim())],
            )
        },
        |code| {
            fill(
                "tool_result_failed_exit_code",
                language,
                &[
                    ("{tool}", label),
                    ("{exit_code}", code),
                    ("{error}", error.trim()),
                ],
            )
        },
    );
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
        if let Some(envelope) = parse_shell_envelope(trimmed) {
            return envelope.into_result();
        }
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
    let reported_exit = ["exit_code", "exitCode"]
        .iter()
        .filter_map(|key| object.get(*key))
        .find_map(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()));
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
        return NormalizedResult {
            exit_code: reported_exit.filter(|code| *code != 0),
            ..from_payload("", Some(error))
        };
    }
    if let Some(payload) = ["output", "stdout", "content", "result"]
        .iter()
        .find_map(|key| object.get(*key))
    {
        let payload = payload
            .as_str()
            .map_or_else(|| pretty_json(payload), strip_transport_envelope);
        return NormalizedResult {
            exit_code: reported_exit,
            ..from_payload(&payload, None)
        };
    }
    NormalizedResult {
        exit_code: reported_exit,
        ..from_payload(&pretty_json(&value), None)
    }
}

/// The fixed report every agent-CLI shell adapter wraps a command result in:
/// `Command:` / `Directory:` / `Output:` / `Error:` / `Exit Code:` / `Signal:` /
/// `Process Group PGID:`. The exit code is the harness's own verdict, so reading
/// it is what keeps `Error: (none)` from being read as an error (issue #908) and
/// `No such file or directory` with `Exit Code: 1` from being read as output
/// (issue #905).
struct ShellEnvelope {
    output: String,
    error: String,
    exit_code: i64,
}

impl ShellEnvelope {
    fn into_result(self) -> NormalizedResult {
        if self.exit_code == 0 {
            // A zero exit is success even when the command wrote to stderr, and
            // even when it wrote nothing at all: `python3 -m py_compile main.py`
            // succeeds silently.
            let payload = if self.output.is_empty() {
                self.error
            } else {
                self.output
            };
            return NormalizedResult {
                exit_code: Some(0),
                ..from_payload(&payload, None)
            };
        }
        let error = if self.error.is_empty() {
            self.output
        } else {
            self.error
        };
        NormalizedResult {
            payload: String::new(),
            error: Some(error),
            format: "text",
            exit_code: Some(self.exit_code),
        }
    }
}

fn parse_shell_envelope(text: &str) -> Option<ShellEnvelope> {
    let inner = text
        .split_once("<untrusted_context>")
        .and_then(|(_, rest)| rest.split_once("</untrusted_context>"))
        .map_or(text, |(inside, _)| inside);
    let lines = inner.lines().collect::<Vec<_>>();
    let mut exit_code = None;
    let mut end = lines.len();
    while end > 0 {
        let Some((field, value)) = lines[end - 1].trim().split_once(':') else {
            break;
        };
        match field.trim() {
            "Exit Code" => match value.trim().parse::<i64>() {
                Ok(code) => exit_code = Some(code),
                // A harness that could not report a status (a timeout, say) is
                // not an envelope this reader can trust.
                Err(_) => return None,
            },
            "Signal" | "Process Group PGID" => {}
            _ => break,
        }
        end -= 1;
    }
    let exit_code = exit_code?;
    let output_at = lines[..end]
        .iter()
        .position(|line| is_field(line, "Output"))?;
    let error_at = lines[..end]
        .iter()
        .enumerate()
        .skip(output_at + 1)
        .rev()
        .find(|(_, line)| is_field(line, "Error"))
        .map(|(index, _)| index);
    let output = envelope_section(&lines[output_at..error_at.unwrap_or(end)], "Output:");
    let error = error_at.map_or_else(String::new, |at| {
        envelope_section(&lines[at..end], "Error:")
    });
    Some(ShellEnvelope {
        output,
        error,
        exit_code,
    })
}

fn is_field(line: &str, field: &str) -> bool {
    let trimmed = line.trim();
    trimmed
        .strip_prefix(field)
        .and_then(|rest| rest.strip_prefix(':'))
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
}

/// Join one `Output:`/`Error:` section, dropping the field name and the
/// placeholders harnesses print for an absent section.
fn envelope_section(lines: &[&str], field: &str) -> String {
    let mut section = String::new();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            section.push('\n');
        }
        if index == 0 {
            section.push_str(line.trim().strip_prefix(field).unwrap_or(line).trim_start());
        } else {
            section.push_str(line);
        }
    }
    let trimmed = section.trim();
    if matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "" | "(empty)" | "(none)" | "(no output)" | "(no error)" | "none"
    ) {
        return String::new();
    }
    trimmed.to_owned()
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
                exit_code: None,
            };
        }
        return NormalizedResult {
            payload: pretty_json(&json),
            error,
            format: "json",
            exit_code: None,
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
        exit_code: None,
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

pub(super) fn response_language(prompt: &str) -> &'static str {
    crate::language::detect(prompt).slug()
}

fn fill(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut text = crate::seed::localized_response(intent, language).unwrap_or_default();
    for (placeholder, value) in values {
        text = text.replace(placeholder, value);
    }
    text
}
