//! Opt-in, per-dialog HTTP exchange logs for agentic CLI diagnosis (issue #781).
//!
//! The server already had a stderr request dump, but a request alone cannot
//! explain whether an empty CLI turn originated in the planner, a protocol
//! adapter, or the client. This module records the complete authenticated
//! request and response together as JSONL. It is disabled unless
//! `FORMAL_AI_DIALOG_LOG_DIR` is set because bodies can contain private prompts,
//! source text, and tool results.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::stable_id;
use crate::proxy::{summarize_proxy_exchange, ProxyExchangeLog};
use crate::server::ApiHttpResponse;

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static LOG_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Dump an inbound request to stderr when `FORMAL_AI_TRACE_REQUESTS=1`.
pub(crate) fn trace_request_if_enabled(method: &str, path: &str, body: &str) {
    if std::env::var("FORMAL_AI_TRACE_REQUESTS").as_deref() == Ok("1") {
        eprintln!("[trace] {method} {path} ({} byte body)\n{body}", body.len());
    }
}

/// One complete server exchange, stored as one JSONL record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogExchangeLog {
    pub timestamp_unix_ms: u128,
    pub dialog_id: String,
    pub request_id: String,
    #[serde(flatten)]
    pub exchange: ProxyExchangeLog,
}

/// Record an exchange when `FORMAL_AI_DIALOG_LOG_DIR` is configured.
///
/// Logging is best-effort: a filesystem problem is reported to stderr and never
/// prevents the protocol response from reaching the client.
pub(crate) fn record_api_exchange_if_enabled(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    request_body: &str,
    response: &ApiHttpResponse,
    authorized: bool,
) {
    if !authorized {
        return;
    }
    let Some(directory) = std::env::var_os("FORMAL_AI_DIALOG_LOG_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    else {
        return;
    };
    match write_dialog_exchange(
        &directory,
        method,
        path,
        headers,
        request_body,
        response.status_code,
        response.content_type,
        &response.body,
    ) {
        Ok(path) => eprintln!("[dialog-log] appended exchange to {}", path.display()),
        Err(error) => eprintln!("[dialog-log] failed to record exchange: {error}"),
    }
}

/// Append one complete exchange and return its per-dialog log path.
///
/// This explicit-directory form is public so an embedding application can use
/// the same recorder without mutating process-wide environment variables.
#[allow(clippy::too_many_arguments)]
pub fn write_dialog_exchange(
    directory: &Path,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    request_body: &str,
    status: u16,
    response_content_type: &str,
    response_body: &str,
) -> io::Result<PathBuf> {
    let timestamp_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    let dialog_id = dialog_id(headers, request_body, path);
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let request_id = stable_id(
        "request",
        &format!("{timestamp_unix_ms}|{sequence}|{dialog_id}|{path}"),
    );
    let exchange = summarize_proxy_exchange(
        method,
        path,
        request_body.as_bytes(),
        status,
        response_content_type,
        response_body.as_bytes(),
        true,
    );
    let record = DialogExchangeLog {
        timestamp_unix_ms,
        dialog_id: dialog_id.clone(),
        request_id,
        exchange,
    };

    fs::create_dir_all(directory)?;
    let path = directory.join(format!("{dialog_id}.jsonl"));
    let _guard = LOG_WRITE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    serde_json::to_writer(&mut file, &record).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(path)
}

fn dialog_id(headers: &[(&str, &str)], request_body: &str, path: &str) -> String {
    let explicit = headers.iter().find_map(|(name, value)| {
        name.eq_ignore_ascii_case("x-formal-ai-dialog-id")
            .then(|| value.trim())
            .filter(|value| !value.is_empty())
    });
    let basis = explicit
        .map(str::to_owned)
        .or_else(|| first_user_prompt(request_body))
        .unwrap_or_else(|| path.to_owned());
    stable_id("dialog", &basis)
}

fn first_user_prompt(body: &str) -> Option<String> {
    fn text(value: &Value) -> Option<String> {
        match value {
            Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_owned()),
            Value::Array(values) => values.iter().find_map(text),
            Value::Object(values) => ["text", "content", "input"]
                .iter()
                .find_map(|key| values.get(*key).and_then(text)),
            _ => None,
        }
    }

    fn user_content(value: &Value) -> Option<String> {
        match value {
            Value::Array(values) => values.iter().find_map(user_content),
            Value::Object(values) => {
                let is_user = values
                    .get("role")
                    .and_then(Value::as_str)
                    .is_some_and(|role| role.eq_ignore_ascii_case("user"));
                if is_user {
                    return values
                        .get("content")
                        .or_else(|| values.get("parts"))
                        .and_then(text);
                }
                values.values().find_map(user_content)
            }
            _ => None,
        }
    }

    let value = serde_json::from_str::<Value>(body).ok()?;
    user_content(&value).or_else(|| value.get("input").and_then(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_exchanges_from_one_dialog_append_to_one_file() {
        let directory = std::env::temp_dir().join(format!(
            "formal-ai-dialog-log-{}-{}",
            std::process::id(),
            REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let header = [("X-Formal-AI-Dialog-ID", "issue-781-reproduction")];
        let request =
            r#"{"model":"formal-ai","messages":[{"role":"user","content":"Find a charger"}]}"#;
        let response = r#"{"choices":[{"message":{"content":"I will search","tool_calls":[]}}]}"#;

        let first = write_dialog_exchange(
            &directory,
            "POST",
            "/v1/chat/completions",
            &header,
            request,
            200,
            "application/json",
            response,
        )
        .expect("first dialog log row");
        let second = write_dialog_exchange(
            &directory,
            "POST",
            "/v1/chat/completions",
            &header,
            request,
            200,
            "application/json",
            response,
        )
        .expect("second dialog log row");

        assert_eq!(first, second);
        let rows = fs::read_to_string(&first).expect("dialog log file");
        assert_eq!(rows.lines().count(), 2);
        for row in rows.lines() {
            let record: DialogExchangeLog =
                serde_json::from_str(row).expect("valid dialog JSONL row");
            assert_eq!(record.exchange.request_body.as_deref(), Some(request));
            assert_eq!(record.exchange.response_body.as_deref(), Some(response));
        }
        fs::remove_dir_all(directory).expect("remove isolated test directory");
    }

    #[test]
    fn expanded_histories_share_the_first_user_prompt_id() {
        let first = dialog_id(
            &[],
            r#"{"messages":[{"role":"user","content":"Find a charger"}]}"#,
            "/v1/chat/completions",
        );
        let next = dialog_id(
            &[],
            r#"{"messages":[{"role":"user","content":"Find a charger"},{"role":"assistant","content":"Searching"},{"role":"tool","content":"result"}]}"#,
            "/v1/chat/completions",
        );
        assert_eq!(first, next);
    }
}
