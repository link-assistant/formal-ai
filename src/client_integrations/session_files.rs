use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::seed::ClientIntegration;

pub(super) struct TempConfigDir {
    pub(super) path: PathBuf,
    remove_on_drop: bool,
}

impl TempConfigDir {
    pub(super) fn new(tool: &str) -> Result<Self, Box<dyn Error>> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "formal-ai-{tool}-config-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path)?;
        Ok(Self {
            path,
            remove_on_drop: true,
        })
    }

    pub(super) fn preserve(mut self) {
        self.remove_on_drop = false;
    }
}

impl Drop for TempConfigDir {
    fn drop(&mut self) {
        if self.remove_on_drop {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

pub(super) type SessionSnapshot = HashMap<PathBuf, (SystemTime, u64)>;

pub(super) fn user_home_dir() -> Result<PathBuf, Box<dyn Error>> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set; cannot resolve session path".into())
}

pub(super) fn session_file_snapshot(root: &Path, suffix: &str) -> SessionSnapshot {
    collect_session_files(root, suffix)
        .into_iter()
        .filter_map(|path| {
            let metadata = fs::metadata(&path).ok()?;
            Some((path, (metadata.modified().ok()?, metadata.len())))
        })
        .collect()
}

fn collect_session_files(path: &Path, suffix: &str) -> Vec<PathBuf> {
    fn visit(path: &Path, suffix: &str, files: &mut Vec<PathBuf>) {
        let Ok(metadata) = fs::symlink_metadata(path) else {
            return;
        };
        if metadata.file_type().is_symlink() {
            return;
        }
        if metadata.is_file() {
            if suffix.is_empty() || path.as_os_str().to_string_lossy().ends_with(suffix) {
                files.push(path.to_path_buf());
            }
            return;
        }
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            visit(&entry.path(), suffix, files);
        }
    }

    let mut files = Vec::new();
    visit(path, suffix, &mut files);
    files
}

pub(super) fn newest_changed_session_file(
    root: &Path,
    suffix: &str,
    before: &SessionSnapshot,
) -> Option<PathBuf> {
    collect_session_files(root, suffix)
        .into_iter()
        .filter_map(|path| {
            let metadata = fs::metadata(&path).ok()?;
            let modified = metadata.modified().ok()?;
            let changed = before
                .get(&path)
                .is_none_or(|previous| *previous != (modified, metadata.len()));
            changed.then_some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

pub(super) fn print_session_files(
    integration: &ClientIntegration,
    session_file: Option<&Path>,
    server_log: Option<&Path>,
) {
    if session_file.is_none() && server_log.is_none() {
        return;
    }
    eprintln!("formal-ai: session files for debugging:");
    if let Some(path) = session_file {
        let resume = session_id(path)
            .or_else(|| query_session_id(integration))
            .and_then(|id| {
                (!integration.invocation.resume_command.is_empty()).then(|| {
                    integration
                        .invocation
                        .resume_command
                        .replace("{session_id}", &id)
                })
            });
        if let Some(resume) = resume {
            eprintln!(
                "  {}: {}   (resume: {resume})",
                integration.id,
                path.display()
            );
        } else {
            eprintln!("  {}: {}", integration.id, path.display());
        }
    }
    if let Some(path) = server_log {
        eprintln!("  server log: {}", path.display());
    }
}

fn query_session_id(integration: &ClientIntegration) -> Option<String> {
    if integration.invocation.session_id_query_args.is_empty() {
        return None;
    }
    let output = Command::new(&integration.command)
        .args(&integration.invocation.session_id_query_args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = serde_json::from_slice::<Value>(&output.stdout).ok()?;
    value
        .as_array()?
        .first()
        .and_then(|session| session.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn session_id(path: &Path) -> Option<String> {
    let mut contents = String::new();
    fs::File::open(path)
        .ok()?
        .take(256 * 1024)
        .read_to_string(&mut contents)
        .ok()?;
    for line in contents.lines().take(64) {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(id) = find_session_id(&value) {
            return Some(id);
        }
    }
    path.file_stem()
        .and_then(OsStr::to_str)
        .filter(|stem| stem.starts_with("ses_") || looks_like_uuid(stem))
        .map(str::to_string)
}

fn find_session_id(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    for key in ["sessionId", "session_id"] {
        if let Some(id) = object.get(key).and_then(Value::as_str) {
            return Some(id.to_string());
        }
    }
    if value.get("type").and_then(Value::as_str) == Some("session_meta") {
        if let Some(id) = value.pointer("/payload/id").and_then(Value::as_str) {
            return Some(id.to_string());
        }
    }
    object.values().find_map(find_session_id)
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .enumerate()
            .all(|(index, character)| match index {
                8 | 13 | 18 | 23 => character == '-',
                _ => character.is_ascii_hexdigit(),
            })
}
