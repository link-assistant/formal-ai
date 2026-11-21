//! Native transcripts for Aider and Grok Build.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use formal_ai::protocol::MessageContent;
use serde_json::{Value, json};

use super::{modified, subdirectories, valid_identifier};

pub(super) fn grok_session(root: &Path, wanted: Option<&str>) -> io::Result<(String, PathBuf)> {
    subdirectories(root)
        .flat_map(|project| subdirectories(&project))
        .filter_map(|session| {
            let identifier = session.file_name()?.to_str()?;
            if !valid_identifier(identifier) || wanted.is_some_and(|wanted| wanted != identifier) {
                return None;
            }
            let path = session.join("chat_history.jsonl");
            Some((modified(&path).ok()?, identifier.to_owned(), path))
        })
        .max()
        .map(|(_, identifier, path)| (identifier, path))
        .ok_or_else(|| io::ErrorKind::NotFound.into())
}

pub(super) fn grok_messages(path: &Path) -> io::Result<Vec<Value>> {
    let mut messages = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let Ok(entry) = serde_json::from_str::<Value>(&line?) else {
            continue;
        };
        match entry["type"].as_str() {
            Some("user") if entry.get("prompt_index").is_some() => {
                let content = MessageContent::Text(crate::cli_report::message_text(&entry))
                    .user_request_text();
                messages.push(json!({"role": "user", "content": content}));
            }
            Some("assistant") => {
                let mut message = json!({"role": "assistant", "content": entry["content"]});
                if let Some(calls) = entry.get("tool_calls") {
                    message["tool_calls"] = calls.clone();
                }
                messages.push(message);
            }
            Some("tool" | "tool_result") => {
                messages.push(json!({"role": "tool", "content": entry["content"], "tool_call_id": entry["tool_call_id"]}));
            }
            _ => {}
        }
    }
    Ok(messages)
}

pub(super) fn aider_messages(path: &Path) -> io::Result<Vec<Value>> {
    let start = std::env::var("FORMAL_AI_REPORT_AIDER_HISTORY_START")
        .ok()
        .and_then(|offset| offset.parse::<u64>().ok())
        .unwrap_or(0);
    let mut file = File::open(path)?;
    if file.metadata()?.len() < start {
        return Err(io::ErrorKind::InvalidData.into());
    }
    file.seek(SeekFrom::Start(start))?;
    let mut messages = Vec::new();
    let mut current_role = "assistant";
    let mut content = String::new();
    let mut session_marker = false;
    for line in BufReader::new(file).lines() {
        let line = line?;
        const SESSION_START: &str = "# aider chat started at ";
        let (role, text) = if line.starts_with(SESSION_START) {
            if session_marker {
                return Err(io::ErrorKind::InvalidData.into());
            }
            session_marker = true;
            continue;
        } else if let Some(text) = line.strip_prefix("#### ") {
            ("user", text)
        } else if let Some(text) = line.strip_prefix("> ") {
            ("tool", text)
        } else {
            ("assistant", line.as_str())
        };
        if current_role != role {
            push_aider_message(&mut messages, current_role, &mut content);
            current_role = role;
        }
        content.push_str(text);
        content.push('\n');
    }
    push_aider_message(&mut messages, current_role, &mut content);
    Ok(messages)
}

fn push_aider_message(messages: &mut Vec<Value>, role: &str, content: &mut String) {
    if !content.trim().is_empty() {
        messages.push(json!({"role": role, "content": content.trim()}));
    }
    content.clear();
}
