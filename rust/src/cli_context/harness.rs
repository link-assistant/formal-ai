//! Read native client transcripts from the configuration directory of the wrapper.

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

mod terminal;

const CODEX_ROLLOUT_PREFIX: &str = "rollout-";

pub(super) enum Harness {
    Claude(PathBuf),
    Codex(PathBuf),
    Agent(PathBuf),
    Gemini(PathBuf),
    Qwen(PathBuf),
    Aider(PathBuf),
    Grok(PathBuf),
}

pub(super) fn current() -> Option<Harness> {
    let home = || std::env::var_os("HOME").map(PathBuf::from);
    match std::env::var("FORMAL_AI_REPORT_CLIENT").ok()?.as_str() {
        "claude" => {
            let configuration = std::env::var_os("CLAUDE_CONFIG_DIR")
                .map(PathBuf::from)
                .or_else(|| home().map(|path| path.join(".claude")))?;
            Some(Harness::Claude(configuration.join("projects")))
        }
        "agent" => {
            let data = std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| home().map(|path| path.join(".local/share")))?;
            Some(Harness::Agent(data.join("link-assistant-agent/storage")))
        }
        "codex" => {
            let configuration = std::env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .or_else(|| home().map(|path| path.join(".codex")))?;
            Some(Harness::Codex(configuration.join("sessions")))
        }
        "gemini" => {
            let configuration = std::env::var_os("GEMINI_CLI_HOME")
                .map(PathBuf::from)
                .or_else(home)?;
            Some(Harness::Gemini(configuration.join(".gemini/tmp")))
        }
        "qwen" => Some(Harness::Qwen(home()?.join(".qwen/projects"))),
        "aider" => Some(Harness::Aider(
            std::env::var_os("FORMAL_AI_REPORT_AIDER_HISTORY").map(PathBuf::from)?,
        )),
        "grok" => {
            let configuration = std::env::var_os("GROK_HOME")
                .map(PathBuf::from)
                .or_else(|| home().map(|path| path.join(".grok")))?;
            Some(Harness::Grok(configuration.join("sessions")))
        }
        _ => None,
    }
}

impl Harness {
    pub(super) fn latest(&self) -> io::Result<String> {
        if matches!(self, Self::Agent(_) | Self::Gemini(_) | Self::Aider(_))
            && let Ok(identifier) = std::env::var("FORMAL_AI_REPORT_DIALOG_ID")
            && valid_identifier(&identifier)
        {
            return Ok(identifier);
        }
        self.latest_native()
    }

    fn latest_native(&self) -> io::Result<String> {
        let (root, extension) = match self {
            Self::Claude(root) => (root.clone(), "jsonl"),
            Self::Agent(root) => (root.join("session"), "json"),
            Self::Codex(root) => {
                let path = codex_rollout(root, None)?;
                let identifier = rollout_identifier(&path).ok_or(io::ErrorKind::InvalidData)?;
                return Ok(identifier.to_owned());
            }
            Self::Gemini(root) => {
                return chat_sessions(root)
                    .max_by_key(|path| modified(path).ok())
                    .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
                    .and_then(|path| gemini_messages(&path).map(|(identifier, _)| identifier));
            }
            Self::Qwen(root) => {
                if let Ok(identifier) = std::env::var("QWEN_CODE_SESSION_ID")
                    && valid_identifier(&identifier)
                {
                    return Ok(identifier);
                }
                return chat_sessions(root)
                    .filter_map(|path| {
                        let identifier = path.file_stem()?.to_str()?;
                        valid_identifier(identifier)
                            .then(|| (modified(&path).ok(), identifier.to_owned()))
                    })
                    .max()
                    .map(|(_, identifier)| identifier)
                    .ok_or_else(|| io::ErrorKind::NotFound.into());
            }
            Self::Aider(_) => return Err(io::ErrorKind::NotFound.into()),
            Self::Grok(root) => {
                return terminal::grok_session(root, None)
                    .map(|(identifier, _)| identifier)
                    .or_else(|error| {
                        std::env::var("FORMAL_AI_REPORT_DIALOG_ID")
                            .ok()
                            .filter(|identifier| valid_identifier(identifier))
                            .ok_or(error)
                    });
            }
        };
        let identifier = find_session(&root, extension, None)?
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|name| valid_identifier(name))
            .ok_or(io::ErrorKind::InvalidData)?
            .to_owned();
        Ok(identifier)
    }

    pub(super) fn export(
        &self,
        identifier: &str,
        log_directory: Option<&Path>,
    ) -> Result<Value, Box<dyn Error>> {
        if !valid_identifier(identifier) {
            return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
        }
        let (tool, messages) = match self {
            Self::Claude(root) => {
                let path = find_session(root, "jsonl", Some(identifier))?;
                ("claude", claude_messages(&path, identifier)?)
            }
            Self::Codex(root) => (
                "codex",
                codex_messages(&codex_rollout(root, Some(identifier))?)?,
            ),
            Self::Agent(root) => {
                if std::env::var("FORMAL_AI_REPORT_DIALOG_ID").as_deref() == Ok(identifier) {
                    let server = super::load_server_context(identifier, log_directory)?;
                    return matching_agent_context(root, &server);
                }
                ("agent", agent_messages(root, identifier)?)
            }
            Self::Gemini(root) => {
                if std::env::var("FORMAL_AI_REPORT_DIALOG_ID").as_deref() == Ok(identifier) {
                    let server = super::load_server_context(identifier, log_directory)?;
                    return matching_gemini_context(root, &server);
                }
                let (_, messages) = chat_sessions(root)
                    .filter_map(|path| gemini_messages(&path).ok())
                    .find(|(candidate, _)| candidate == identifier)
                    .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
                ("gemini", messages)
            }
            Self::Qwen(root) => {
                let path = chat_sessions(root)
                    .filter(|path| {
                        path.file_stem().and_then(|name| name.to_str()) == Some(identifier)
                    })
                    .max_by_key(|path| modified(path).ok())
                    .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
                ("qwen", qwen_messages(&path, identifier)?)
            }
            Self::Aider(history) => {
                if std::env::var("FORMAL_AI_REPORT_DIALOG_ID").as_deref() != Ok(identifier) {
                    return Err(io::ErrorKind::NotFound.into());
                }
                ("aider", terminal::aider_messages(history)?)
            }
            Self::Grok(root) => {
                let (_, transcript) = terminal::grok_session(root, Some(identifier))?;
                ("grok", terminal::grok_messages(&transcript)?)
            }
        };
        Ok(document(tool, identifier, messages))
    }
}

fn rollout_identifier(path: &Path) -> Option<&str> {
    let name = path.file_stem()?.to_str()?;
    let identifier = name.get(name.len().checked_sub(36)?..)?;
    (name.starts_with(CODEX_ROLLOUT_PREFIX) && valid_identifier(identifier)).then_some(identifier)
}

fn entries(root: &Path) -> impl Iterator<Item = fs::DirEntry> + use<> {
    fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
}

fn document(tool: &str, identifier: &str, messages: Vec<Value>) -> Value {
    json!({
        "source": {"tool": tool, "storage": "native"},
        "session": {"id": identifier},
        "message_count": messages.len(),
        "messages": messages,
    })
}

fn user_turns(messages: &[Value]) -> Vec<String> {
    messages
        .iter()
        .filter(|message| message["role"] == "user")
        .map(crate::cli_report::message_text)
        .map(|text| {
            let trimmed = text.trim();
            if let Some(context) = trimmed.strip_prefix("<session_context>")
                && let Some((_, user)) = context.split_once("</session_context>")
            {
                return user.trim().to_owned();
            }
            trimmed.to_owned()
        })
        .filter(|text| !text.trim().is_empty())
        .collect()
}

fn earliest_transcript_write(server: &Value) -> io::Result<SystemTime> {
    let first_capture = server["metadata"]["first_timestamp_unix_ms"]
        .as_u64()
        .ok_or(io::ErrorKind::InvalidData)?;
    Ok(UNIX_EPOCH + Duration::from_millis(first_capture.saturating_sub(30_000)))
}

fn modified(path: &Path) -> io::Result<SystemTime> {
    path.metadata()?.modified()
}

fn matching_agent_context(root: &Path, server: &Value) -> Result<Value, Box<dyn Error>> {
    let earliest = earliest_transcript_write(server)?;
    let directories = root.join("message");
    let candidates = subdirectories(&directories)
        .filter(|path| modified(path).is_ok_and(|time| time >= earliest))
        .filter_map(|path| {
            let identifier = path.file_name()?.to_str()?.to_owned();
            valid_identifier(&identifier).then_some(identifier)
        })
        .filter_map(|identifier| {
            agent_messages(root, &identifier)
                .ok()
                .map(|messages| (identifier, messages))
        });
    matching_context("Agent", "agent", server, candidates)
}

fn matching_gemini_context(root: &Path, server: &Value) -> Result<Value, Box<dyn Error>> {
    let earliest = earliest_transcript_write(server)?;
    let candidates = chat_sessions(root)
        .filter(|path| modified(path).is_ok_and(|time| time >= earliest))
        .filter_map(|path| gemini_messages(&path).ok());
    matching_context("Gemini", "gemini", server, candidates)
}

fn matching_context(
    client: &str,
    tool: &str,
    server: &Value,
    candidates: impl Iterator<Item = (String, Vec<Value>)>,
) -> Result<Value, Box<dyn Error>> {
    let turns = user_turns(server["messages"].as_array().map_or(&[], Vec::as_slice));
    if turns.is_empty() {
        return Err(io::Error::from(io::ErrorKind::NotFound).into());
    }
    let mut matching = None;
    for (identifier, messages) in candidates {
        let native_turns = user_turns(&messages);
        if native_turns.is_empty()
            || (!native_turns.ends_with(&turns) && !turns.ends_with(&native_turns))
        {
            continue;
        }
        if matching.is_some() {
            return Err(
                transcript_match_error("context-transcript-multiple-matches", client).into(),
            );
        }
        matching = Some(document(tool, &identifier, messages));
    }
    matching.ok_or_else(|| transcript_match_error("context-transcript-no-match", client).into())
}

fn transcript_match_error(key: &str, client: &str) -> io::Error {
    io::Error::other(super::config(key).replace("{client}", client))
}

fn subdirectories(root: &Path) -> impl Iterator<Item = PathBuf> + use<> {
    entries(root)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
}

fn chat_sessions(root: &Path) -> impl Iterator<Item = PathBuf> {
    subdirectories(root)
        .flat_map(|project| entries(&project.join("chats")))
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "jsonl")
        })
}

fn qwen_messages(path: &Path, identifier: &str) -> io::Result<Vec<Value>> {
    let mut messages = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let Ok(entry) = serde_json::from_str::<Value>(&line?) else {
            continue;
        };
        if entry["sessionId"].as_str() != Some(identifier) {
            continue;
        }
        let role = match entry["type"].as_str() {
            Some("user") => "user",
            Some("assistant") => "assistant",
            Some("tool_result") => "tool",
            _ => continue,
        };
        messages.push(json!({"role": role, "parts": entry["message"]["parts"]}));
    }
    Ok(messages)
}

fn gemini_messages(path: &Path) -> io::Result<(String, Vec<Value>)> {
    let mut lines = BufReader::new(File::open(path)?).lines();
    let header: Value = serde_json::from_str(&lines.next().transpose()?.unwrap_or_default())
        .map_err(io::Error::other)?;
    let identifier = header["sessionId"]
        .as_str()
        .filter(|id| valid_identifier(id))
        .ok_or(io::ErrorKind::InvalidData)?;
    let mut messages = Vec::new();
    for line in lines {
        let line = line?;
        let Ok(mut entry) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let role = match entry["type"].as_str() {
            Some("user") => "user",
            Some("gemini") => "assistant",
            _ => continue,
        };
        entry["role"] = Value::String(role.to_owned());
        messages.push(entry);
    }
    Ok((identifier.to_owned(), messages))
}

fn codex_rollout(root: &Path, identifier: Option<&str>) -> io::Result<PathBuf> {
    subdirectories(root)
        .flat_map(|year| subdirectories(&year))
        .flat_map(|month| subdirectories(&month))
        .flat_map(|day| entries(&day))
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let path = entry.path();
            let found = rollout_identifier(&path)?;
            (path.extension().is_some_and(|ext| ext == "jsonl")
                && identifier.is_none_or(|identifier| identifier == found))
            .then(|| {
                (
                    entry
                        .metadata()
                        .and_then(|metadata| metadata.modified())
                        .ok(),
                    path,
                )
            })
        })
        .max()
        .map(|(_, path)| path)
        .ok_or_else(|| io::ErrorKind::NotFound.into())
}

fn codex_messages(path: &Path) -> io::Result<Vec<Value>> {
    let mut messages = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        let Ok(entry) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let payload = &entry["payload"];
        if entry["type"] == "response_item"
            && payload["type"] == "message"
            && matches!(payload["role"].as_str(), Some("user" | "assistant"))
        {
            messages.push(payload.clone());
        }
    }
    Ok(messages)
}

fn valid_identifier(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.len() <= 128
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn find_session(root: &Path, extension: &str, identifier: Option<&str>) -> io::Result<PathBuf> {
    fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|project| project.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|project| fs::read_dir(project.path()).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some(extension)
                && identifier.is_none_or(|identifier| {
                    path.file_stem().and_then(|name| name.to_str()) == Some(identifier)
                }))
            .then(|| {
                (
                    entry
                        .metadata()
                        .and_then(|metadata| metadata.modified())
                        .ok(),
                    path,
                )
            })
        })
        .max()
        .map(|(_, path)| path)
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
}

fn claude_messages(path: &Path, identifier: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut messages = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        // The live client can be appending a partial last record during export.
        let Ok(entry) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if entry["sessionId"].as_str() != Some(identifier)
            || entry["isSidechain"] == true
            || !matches!(entry["type"].as_str(), Some("user" | "assistant"))
        {
            continue;
        }
        messages.push(entry["message"].clone());
    }
    Ok(messages)
}

fn json_objects(directory: &Path) -> io::Result<Vec<Value>> {
    let mut files = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    files
        .into_iter()
        .map(|path| serde_json::from_reader(File::open(path)?).map_err(io::Error::other))
        .collect()
}

fn agent_messages(root: &Path, identifier: &str) -> io::Result<Vec<Value>> {
    let mut messages = Vec::new();
    for entry in json_objects(&root.join("message").join(identifier))? {
        let Some(message) = entry["id"]
            .as_str()
            .filter(|identifier| valid_identifier(identifier))
        else {
            continue;
        };
        if entry["sessionID"].as_str() != Some(identifier) {
            continue;
        }
        let parts = match json_objects(&root.join("part").join(message)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            result => result?,
        };
        messages.push(json!({"role": entry["role"], "parts": parts, "data": entry}));
    }
    Ok(messages)
}
