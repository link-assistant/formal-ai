//! Read native client transcripts from the configuration directory of the wrapper.

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use serde_json::{Value, json};

const CODEX_ROLLOUT_PREFIX: &str = "rollout-";

pub(super) enum Harness {
    Claude(PathBuf),
    Codex(PathBuf),
    Agent(PathBuf),
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
        _ => None,
    }
}

impl Harness {
    pub(super) fn latest(&self) -> Result<String, Box<dyn Error>> {
        if let Self::Agent(_) = self
            && let Ok(identifier) = std::env::var("FORMAL_AI_REPORT_DIALOG_ID")
            && valid_identifier(&identifier)
        {
            return Ok(identifier);
        }
        self.latest_native()
    }

    fn latest_native(&self) -> Result<String, Box<dyn Error>> {
        let (root, extension) = match self {
            Self::Claude(root) => (root.clone(), "jsonl"),
            Self::Agent(root) => (root.join("session"), "json"),
            Self::Codex(root) => {
                let path = codex_rollout(root, None)?;
                let identifier = rollout_identifier(&path).ok_or(io::ErrorKind::InvalidData)?;
                return Ok(identifier.to_owned());
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
            return Err(io::ErrorKind::InvalidInput.into());
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
        };
        Ok(document(tool, identifier, messages))
    }
}

fn rollout_identifier(path: &Path) -> Option<&str> {
    let name = path.file_stem()?.to_str()?;
    let identifier = name.get(name.len().checked_sub(36)?..)?;
    (name.starts_with(CODEX_ROLLOUT_PREFIX) && valid_identifier(identifier)).then_some(identifier)
}

fn entries(root: &Path) -> impl Iterator<Item = fs::DirEntry> {
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
        .filter(|text| !text.trim().is_empty())
        .collect()
}

fn matching_agent_context(root: &Path, server: &Value) -> Result<Value, Box<dyn Error>> {
    let turns = user_turns(server["messages"].as_array().map_or(&[], Vec::as_slice));
    if turns.is_empty() {
        return Err(io::ErrorKind::NotFound.into());
    }
    let first_capture = server["metadata"]["first_timestamp_unix_ms"]
        .as_u64()
        .ok_or(io::ErrorKind::InvalidData)?;
    let earliest_write = UNIX_EPOCH + Duration::from_millis(first_capture.saturating_sub(30_000));
    let directories = root.join("message");
    let mut matching = None;
    for directory in subdirectories(&directories) {
        let Some(identifier) = directory.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !valid_identifier(identifier)
            || !directory
                .metadata()
                .and_then(|file| file.modified())
                .is_ok_and(|time| time >= earliest_write)
        {
            continue;
        }
        let Ok(messages) = agent_messages(root, identifier) else {
            continue;
        };
        let native_turns = user_turns(&messages);
        if !native_turns.ends_with(&turns) && !turns.ends_with(&native_turns) {
            continue;
        }
        if native_turns.is_empty() {
            continue;
        }
        if matching.is_some() {
            return Err(
                io::Error::other("multiple Agent transcripts match this conversation").into(),
            );
        }
        matching = Some(document("agent", identifier, messages));
    }
    matching.ok_or_else(|| io::Error::other("no Agent transcript matches this conversation").into())
}

fn subdirectories(root: &Path) -> impl Iterator<Item = PathBuf> {
    entries(root)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
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
