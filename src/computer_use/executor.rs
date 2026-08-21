use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{
    COMPUTER_USE_PRIMITIVES, ComputerPlanStep, ComputerStepRecord, ComputerUsePrimitive,
    VerificationEvent, VerificationPhase,
};

static SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum ComputerUseError {
    Io(io::Error),
    UnknownPlan(String),
}

impl fmt::Display for ComputerUseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::UnknownPlan(prompt) => write!(formatter, "unknown computer-use plan: {prompt}"),
        }
    }
}

impl Error for ComputerUseError {}

impl From<io::Error> for ComputerUseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerUsePolicy {
    pub agent_mode: bool,
    grants: BTreeSet<String>,
}

impl ComputerUsePolicy {
    #[must_use]
    pub const fn deny_all() -> Self {
        Self {
            agent_mode: false,
            grants: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn agent_mode_all() -> Self {
        Self {
            agent_mode: true,
            grants: COMPUTER_USE_PRIMITIVES
                .into_iter()
                .map(ComputerUsePrimitive::permission_key)
                .collect(),
        }
    }

    #[must_use]
    pub fn with_grants(agent_mode: bool, grants: impl IntoIterator<Item = String>) -> Self {
        Self {
            agent_mode,
            grants: grants.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn permits(&self, primitive: ComputerUsePrimitive) -> bool {
        self.agent_mode && self.grants.contains(&primitive.permission_key())
    }
}

pub struct ComputerUseSession {
    plan_id: String,
    root: PathBuf,
    policy: ComputerUsePolicy,
}

impl ComputerUseSession {
    pub fn new(plan_id: &str, policy: ComputerUsePolicy) -> Result<Self, ComputerUseError> {
        let base = std::env::temp_dir().join("formal-ai-computer-use");
        Self::in_base(plan_id, policy, &base)
    }

    pub fn in_base(
        plan_id: &str,
        policy: ComputerUsePolicy,
        base: &Path,
    ) -> Result<Self, ComputerUseError> {
        let sequence = SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let safe_id = plan_id
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                    character
                } else {
                    '-'
                }
            })
            .collect::<String>();
        let root = base.join(format!("{safe_id}-{}-{sequence}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(Self {
            plan_id: plan_id.to_owned(),
            root,
            policy,
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn execute_step(&mut self, step: &ComputerPlanStep) -> ComputerStepRecord {
        self.execute_primitive(
            &step.id,
            step.primitive,
            step.arguments.clone(),
            &step.precondition,
            &step.postcondition,
        )
    }

    #[must_use]
    pub fn execute_primitive(
        &mut self,
        step_id: &str,
        primitive: ComputerUsePrimitive,
        arguments: Value,
        precondition: &str,
        postcondition: &str,
    ) -> ComputerStepRecord {
        let mut events = Vec::with_capacity(3);
        let precondition_error = self.precondition_error(primitive, &arguments);
        events.push(event(
            &self.plan_id,
            step_id,
            primitive,
            VerificationPhase::Precondition,
            precondition_error.is_none(),
            precondition_error
                .as_deref()
                .unwrap_or(precondition)
                .to_owned(),
        ));

        let effect = precondition_error.map_or_else(|| self.apply(primitive, &arguments), Err);
        let (output, effect_passed, effect_detail) = match effect {
            Ok(output) => (
                output,
                true,
                format!(
                    "executed={};changed={}",
                    primitive.name(),
                    primitive.changes_state()
                        || arguments.get("save_as").and_then(Value::as_str).is_some()
                ),
            ),
            Err(error) => (json!({"error": error}), false, error),
        };
        events.push(event(
            &self.plan_id,
            step_id,
            primitive,
            VerificationPhase::Effect,
            effect_passed,
            effect_detail,
        ));

        let post_passed =
            effect_passed && self.verify_postcondition(primitive, &arguments, &output);
        events.push(event(
            &self.plan_id,
            step_id,
            primitive,
            VerificationPhase::Postcondition,
            post_passed,
            if post_passed {
                postcondition.to_owned()
            } else {
                format!("postcondition_failed:{postcondition}")
            },
        ));
        ComputerStepRecord {
            plan_id: self.plan_id.clone(),
            step_id: step_id.to_owned(),
            primitive: primitive.name().to_owned(),
            arguments,
            output,
            verified: events.iter().all(|item| item.passed),
            events,
        }
    }

    fn precondition_error(
        &self,
        primitive: ComputerUsePrimitive,
        arguments: &Value,
    ) -> Option<String> {
        if !self.policy.agent_mode {
            return Some("policy_refusal: agent_mode_required".to_owned());
        }
        if !self.policy.permits(primitive) {
            return Some(format!(
                "policy_refusal:permission_required:{}",
                primitive.permission_key()
            ));
        }
        if primitive.changes_state()
            && arguments.get("confirmed").and_then(Value::as_bool) != Some(true)
        {
            return Some(format!(
                "confirmation_required:destructive_or_effectful:{}",
                primitive.name()
            ));
        }
        self.validate_arguments(primitive, arguments).err()
    }

    fn validate_arguments(
        &self,
        primitive: ComputerUsePrimitive,
        arguments: &Value,
    ) -> Result<(), String> {
        let paths: Vec<&str> = match primitive {
            ComputerUsePrimitive::FsRead
            | ComputerUsePrimitive::FsWrite
            | ComputerUsePrimitive::FsList => vec![arg(arguments, "path")?],
            ComputerUsePrimitive::FsMove => {
                vec![arg(arguments, "from")?, arg(arguments, "to")?]
            }
            ComputerUsePrimitive::ShellRun => {
                let operation = arg(arguments, "operation")?;
                if !matches!(operation, "count_lines" | "filter_csv" | "unique_csv") {
                    return Err(format!("unsupported_allowlisted_operation:{operation}"));
                }
                vec![arg(arguments, "input")?, arg(arguments, "output")?]
            }
            ComputerUsePrimitive::HttpFetch | ComputerUsePrimitive::HttpPost => {
                let url = arg(arguments, "url")?;
                if !url.starts_with("fixture://") {
                    return Err(format!("network_fixture_not_permitted:{url}"));
                }
                vec![arg(arguments, "save_as")?]
            }
            ComputerUsePrimitive::DomQuery | ComputerUsePrimitive::DomExtract => {
                vec![arg(arguments, "source")?, arg(arguments, "save_as")?]
            }
            ComputerUsePrimitive::ArchivePack => {
                let mut paths = string_array(arguments, "paths")?;
                paths.push(arg(arguments, "archive")?);
                paths
            }
            ComputerUsePrimitive::ArchiveUnpack => {
                vec![arg(arguments, "archive")?, arg(arguments, "destination")?]
            }
            ComputerUsePrimitive::ProcessStatus => vec![arg(arguments, "save_as")?],
        };
        for path in paths {
            self.path(path)?;
        }
        let required_inputs: Vec<&str> = match primitive {
            ComputerUsePrimitive::FsRead | ComputerUsePrimitive::FsList => {
                vec![arg(arguments, "path")?]
            }
            ComputerUsePrimitive::FsMove => vec![arg(arguments, "from")?],
            ComputerUsePrimitive::ShellRun => vec![arg(arguments, "input")?],
            ComputerUsePrimitive::DomQuery | ComputerUsePrimitive::DomExtract => {
                vec![arg(arguments, "source")?]
            }
            ComputerUsePrimitive::ArchivePack => string_array(arguments, "paths")?,
            ComputerUsePrimitive::ArchiveUnpack => vec![arg(arguments, "archive")?],
            _ => Vec::new(),
        };
        for input in required_inputs {
            if !self.path(input)?.exists() {
                return Err(format!("precondition_failed:input_not_found:{input}"));
            }
        }
        Ok(())
    }

    fn apply(&self, primitive: ComputerUsePrimitive, arguments: &Value) -> Result<Value, String> {
        match primitive {
            ComputerUsePrimitive::FsRead => self.read_file(arg(arguments, "path")?),
            ComputerUsePrimitive::FsWrite => {
                self.write_file(arg(arguments, "path")?, arg(arguments, "content")?)
            }
            ComputerUsePrimitive::FsList => self.list_directory(arg(arguments, "path")?),
            ComputerUsePrimitive::FsMove => {
                self.move_file(arg(arguments, "from")?, arg(arguments, "to")?)
            }
            ComputerUsePrimitive::ShellRun => self.run_allowlisted(arguments),
            ComputerUsePrimitive::HttpFetch => self.http_fetch(arguments),
            ComputerUsePrimitive::HttpPost => self.http_post(arguments),
            ComputerUsePrimitive::DomQuery => self.dom_query(arguments),
            ComputerUsePrimitive::DomExtract => self.dom_extract(arguments),
            ComputerUsePrimitive::ArchivePack => self.archive_pack(arguments),
            ComputerUsePrimitive::ArchiveUnpack => self.archive_unpack(arguments),
            ComputerUsePrimitive::ProcessStatus => self.process_status(arguments),
        }
    }

    fn verify_postcondition(
        &self,
        primitive: ComputerUsePrimitive,
        arguments: &Value,
        output: &Value,
    ) -> bool {
        match primitive {
            ComputerUsePrimitive::FsRead => self
                .path(arg_or_empty(arguments, "path"))
                .ok()
                .and_then(|path| fs::read(path).ok())
                .is_some_and(|bytes| {
                    output.get("sha256").and_then(Value::as_str) == Some(digest(&bytes).as_str())
                        && output.get("content").and_then(Value::as_str)
                            == std::str::from_utf8(&bytes).ok()
                }),
            ComputerUsePrimitive::FsWrite => self
                .path(arg_or_empty(arguments, "path"))
                .ok()
                .and_then(|path| fs::read(path).ok())
                .is_some_and(|bytes| {
                    let expected = arg_or_empty(arguments, "content").as_bytes();
                    bytes == expected
                        && output.get("sha256").and_then(Value::as_str)
                            == Some(digest(&bytes).as_str())
                }),
            ComputerUsePrimitive::FsList => {
                let observed = self.directory_entries(arg_or_empty(arguments, "path")).ok();
                let reported = output
                    .get("entries")
                    .and_then(Value::as_array)
                    .map(|entries| {
                        entries
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToOwned::to_owned)
                            .collect::<Vec<_>>()
                    });
                observed.is_some() && observed == reported
            }
            ComputerUsePrimitive::FsMove => {
                let from_missing = self
                    .path(arg_or_empty(arguments, "from"))
                    .is_ok_and(|path| !path.exists());
                let destination_digest = self.file_digest(arg_or_empty(arguments, "to"));
                from_missing
                    && destination_digest.as_deref() == output.get("sha256").and_then(Value::as_str)
            }
            ComputerUsePrimitive::ArchiveUnpack => self.verify_archive_unpack(arguments, output),
            ComputerUsePrimitive::ProcessStatus => {
                output.get("scope").and_then(Value::as_str) == Some("isolated_workspace")
                    && output.get("state").and_then(Value::as_str) == Some("running")
                    && self
                        .file_digest(arg_or_empty(arguments, "save_as"))
                        .as_deref()
                        == output.get("sha256").and_then(Value::as_str)
            }
            ComputerUsePrimitive::ShellRun => {
                self.file_digest(arg_or_empty(arguments, "output"))
                    .as_deref()
                    == output.get("sha256").and_then(Value::as_str)
            }
            ComputerUsePrimitive::HttpFetch
            | ComputerUsePrimitive::HttpPost
            | ComputerUsePrimitive::DomQuery
            | ComputerUsePrimitive::DomExtract => {
                self.file_digest(arg_or_empty(arguments, "save_as"))
                    .as_deref()
                    == output.get("sha256").and_then(Value::as_str)
            }
            ComputerUsePrimitive::ArchivePack => {
                self.file_digest(arg_or_empty(arguments, "archive"))
                    .as_deref()
                    == output.get("sha256").and_then(Value::as_str)
            }
        }
    }

    fn path(&self, value: &str) -> Result<PathBuf, String> {
        let relative = Path::new(value.trim());
        if relative.as_os_str().is_empty() || relative.is_absolute() {
            return Err(format!("path_escapes_workspace:{value}"));
        }
        if relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(format!("path_escapes_workspace:{value}"));
        }
        Ok(self.root.join(relative))
    }

    fn read_file(&self, relative: &str) -> Result<Value, String> {
        let path = self.path(relative)?;
        let bytes = fs::read(&path).map_err(|error| format!("read_failed:{relative}:{error}"))?;
        let content = String::from_utf8(bytes.clone())
            .map_err(|_| format!("read_failed:{relative}:content_not_utf8"))?;
        Ok(json!({
            "path": relative,
            "content": content,
            "bytes": bytes.len(),
            "sha256": digest(&bytes)
        }))
    }

    fn write_file(&self, relative: &str, content: &str) -> Result<Value, String> {
        self.write_bytes(relative, content.as_bytes())?;
        Ok(json!({
            "path": relative,
            "bytes": content.len(),
            "sha256": digest(content.as_bytes())
        }))
    }

    fn write_bytes(&self, relative: &str, bytes: &[u8]) -> Result<(), String> {
        let path = self.path(relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create_directory_failed:{relative}:{error}"))?;
        }
        fs::write(path, bytes).map_err(|error| format!("write_failed:{relative}:{error}"))
    }

    fn list_directory(&self, relative: &str) -> Result<Value, String> {
        let entries = self.directory_entries(relative)?;
        let encoded = serde_json::to_vec(&entries).map_err(|error| error.to_string())?;
        Ok(json!({
            "path": relative,
            "entries": entries,
            "sha256": digest(&encoded)
        }))
    }

    fn directory_entries(&self, relative: &str) -> Result<Vec<String>, String> {
        let path = self.path(relative)?;
        let mut entries = fs::read_dir(path)
            .map_err(|error| format!("list_failed:{relative}:{error}"))?
            .map(|entry| {
                entry
                    .map(|item| item.file_name().to_string_lossy().into_owned())
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort();
        Ok(entries)
    }

    fn move_file(&self, from: &str, to: &str) -> Result<Value, String> {
        let source = self.path(from)?;
        let destination = self.path(to)?;
        let bytes =
            fs::read(&source).map_err(|error| format!("move_source_failed:{from}:{error}"))?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("move_directory_failed:{to}:{error}"))?;
        }
        fs::rename(source, destination)
            .map_err(|error| format!("move_failed:{from}->{to}:{error}"))?;
        Ok(json!({"from":from,"to":to,"moved":true,"sha256":digest(&bytes)}))
    }

    fn run_allowlisted(&self, arguments: &Value) -> Result<Value, String> {
        let operation = arg(arguments, "operation")?;
        let input = arg(arguments, "input")?;
        let output = arg(arguments, "output")?;
        let content = fs::read_to_string(self.path(input)?)
            .map_err(|error| format!("shell_input_failed:{input}:{error}"))?;
        let generated = match operation {
            "count_lines" => format!("{}\n", content.lines().count()),
            "filter_csv" => filter_csv(
                &content,
                arg(arguments, "column")?,
                arg(arguments, "equals")?,
            )?,
            "unique_csv" => unique_csv(&content, arg(arguments, "column")?)?,
            _ => return Err(format!("unsupported_allowlisted_operation:{operation}")),
        };
        self.write_bytes(output, generated.as_bytes())?;
        Ok(json!({
            "operation": operation,
            "output": output,
            "stdout": generated,
            "sha256": digest(generated.as_bytes())
        }))
    }

    fn http_fetch(&self, arguments: &Value) -> Result<Value, String> {
        let url = arg(arguments, "url")?;
        let save_as = arg(arguments, "save_as")?;
        let body = fixture(url).ok_or_else(|| format!("fixture_not_found:{url}"))?;
        self.write_bytes(save_as, body.as_bytes())?;
        Ok(http_output("GET", url, 200, body, save_as))
    }

    fn http_post(&self, arguments: &Value) -> Result<Value, String> {
        let url = arg(arguments, "url")?;
        let save_as = arg(arguments, "save_as")?;
        let request_body = arg(arguments, "body")?;
        if url != "fixture://submit" {
            return Err(format!("fixture_not_found:{url}"));
        }
        let (status, body) = if request_body == "token=fixture-token" {
            (
                200,
                include_str!("../../data/fixtures/computer-use/submission.json").trim(),
            )
        } else {
            (400, r#"{"accepted":false,"error":"invalid token"}"#)
        };
        self.write_bytes(save_as, body.as_bytes())?;
        Ok(http_output("POST", url, status, body, save_as))
    }

    fn dom_query(&self, arguments: &Value) -> Result<Value, String> {
        let source = arg(arguments, "source")?;
        let selector = arg(arguments, "selector")?;
        let save_as = arg(arguments, "save_as")?;
        let html = fs::read_to_string(self.path(source)?)
            .map_err(|error| format!("dom_source_failed:{source}:{error}"))?;
        let matches = query_html(&html, selector)?;
        let text = matches.join("\n");
        self.write_bytes(save_as, text.as_bytes())?;
        Ok(json!({
            "selector": selector,
            "matches": matches,
            "text": text,
            "sha256": digest(text.as_bytes())
        }))
    }

    fn dom_extract(&self, arguments: &Value) -> Result<Value, String> {
        let source = arg(arguments, "source")?;
        let pointer = arg(arguments, "pointer")?;
        let save_as = arg(arguments, "save_as")?;
        let bytes = fs::read(self.path(source)?)
            .map_err(|error| format!("dom_source_failed:{source}:{error}"))?;
        let document: Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("json_parse_failed:{source}:{error}"))?;
        let value = document
            .pointer(pointer)
            .ok_or_else(|| format!("json_pointer_not_found:{pointer}"))?;
        let text = value
            .as_str()
            .map_or_else(|| value.to_string(), ToOwned::to_owned);
        self.write_bytes(save_as, text.as_bytes())?;
        Ok(json!({"pointer":pointer,"value":value,"text":text,"sha256":digest(text.as_bytes())}))
    }

    fn archive_pack(&self, arguments: &Value) -> Result<Value, String> {
        let archive_path = arg(arguments, "archive")?;
        let mut paths = string_array(arguments, "paths")?;
        paths.sort_unstable();
        let entries = paths
            .iter()
            .map(|relative| {
                let bytes = fs::read(self.path(relative)?)
                    .map_err(|error| format!("archive_input_failed:{relative}:{error}"))?;
                Ok(ArchiveEntry {
                    path: (*relative).to_owned(),
                    content_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let archive = DeterministicArchive {
            format: "formal-ai-archive-v1".to_owned(),
            entries,
        };
        let encoded = serde_json::to_vec(&archive).map_err(|error| error.to_string())?;
        self.write_bytes(archive_path, &encoded)?;
        Ok(json!({
            "archive":archive_path,
            "entries":paths,
            "sha256":digest(&encoded)
        }))
    }

    fn archive_unpack(&self, arguments: &Value) -> Result<Value, String> {
        let archive_path = arg(arguments, "archive")?;
        let destination = arg(arguments, "destination")?;
        let bytes = fs::read(self.path(archive_path)?)
            .map_err(|error| format!("archive_read_failed:{archive_path}:{error}"))?;
        let archive: DeterministicArchive =
            serde_json::from_slice(&bytes).map_err(|error| format!("archive_invalid:{error}"))?;
        if archive.format != "formal-ai-archive-v1" {
            return Err(format!("archive_format_unsupported:{}", archive.format));
        }
        let mut restored = Vec::with_capacity(archive.entries.len());
        for entry in archive.entries {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&entry.content_base64)
                .map_err(|error| format!("archive_content_invalid:{error}"))?;
            let target = format!("{destination}/{}", entry.path);
            self.write_bytes(&target, &bytes)?;
            restored.push(entry.path);
        }
        restored.sort();
        Ok(json!({
            "destination":destination,
            "entries":restored,
            "sha256":digest(&bytes)
        }))
    }

    fn verify_archive_unpack(&self, arguments: &Value, output: &Value) -> bool {
        let archive_path = arg_or_empty(arguments, "archive");
        let destination = arg_or_empty(arguments, "destination");
        let Ok(bytes) = self.path(archive_path).and_then(|path| {
            fs::read(path).map_err(|error| format!("archive_read_failed:{error}"))
        }) else {
            return false;
        };
        let Ok(archive) = serde_json::from_slice::<DeterministicArchive>(&bytes) else {
            return false;
        };
        if archive.format != "formal-ai-archive-v1"
            || output.get("sha256").and_then(Value::as_str) != Some(digest(&bytes).as_str())
        {
            return false;
        }

        let mut observed_entries = Vec::with_capacity(archive.entries.len());
        for entry in archive.entries {
            let Ok(expected) =
                base64::engine::general_purpose::STANDARD.decode(&entry.content_base64)
            else {
                return false;
            };
            let target = format!("{destination}/{}", entry.path);
            let Ok(actual) = self
                .path(&target)
                .and_then(|path| fs::read(path).map_err(|error| error.to_string()))
            else {
                return false;
            };
            if actual != expected {
                return false;
            }
            observed_entries.push(entry.path);
        }
        observed_entries.sort();
        output
            .get("entries")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .as_ref()
            == Some(&observed_entries)
    }

    fn process_status(&self, arguments: &Value) -> Result<Value, String> {
        let save_as = arg(arguments, "save_as")?;
        let status = json!({
            "plan_id": self.plan_id,
            "state": "running",
            "scope": "isolated_workspace"
        });
        let bytes = serde_json::to_vec(&status).map_err(|error| error.to_string())?;
        self.write_bytes(save_as, &bytes)?;
        Ok(json!({
            "state":"running",
            "scope":"isolated_workspace",
            "save_as":save_as,
            "sha256":digest(&bytes)
        }))
    }

    fn file_digest(&self, relative: &str) -> Option<String> {
        let path = self.path(relative).ok()?;
        fs::read(path).ok().map(|bytes| digest(&bytes))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DeterministicArchive {
    format: String,
    entries: Vec<ArchiveEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArchiveEntry {
    path: String,
    content_base64: String,
}

fn event(
    plan_id: &str,
    step_id: &str,
    primitive: ComputerUsePrimitive,
    phase: VerificationPhase,
    passed: bool,
    detail: String,
) -> VerificationEvent {
    let phase_name = match phase {
        VerificationPhase::Precondition => "precondition",
        VerificationPhase::Effect => "effect",
        VerificationPhase::Postcondition => "postcondition",
    };
    VerificationEvent {
        id: format!("{plan_id}:{step_id}:{phase_name}"),
        step_id: step_id.to_owned(),
        primitive: primitive.name().to_owned(),
        phase,
        passed,
        detail,
    }
}

fn arg<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("invalid_argument:{key}:expected_non_empty_string"))
}

fn arg_or_empty<'a>(arguments: &'a Value, key: &str) -> &'a str {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
}

fn string_array<'a>(arguments: &'a Value, key: &str) -> Result<Vec<&'a str>, String> {
    let values = arguments
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("invalid_argument:{key}:expected_array"))?;
    let parsed = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|item| !item.is_empty())
                .ok_or_else(|| format!("invalid_argument:{key}:expected_string_entries"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if parsed.is_empty() {
        return Err(format!("invalid_argument:{key}:expected_non_empty_array"));
    }
    Ok(parsed)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture(url: &str) -> Option<&'static str> {
    match url {
        "fixture://orders.json" => {
            Some(include_str!("../../data/fixtures/computer-use/orders.json").trim())
        }
        "fixture://status.html" => {
            Some(include_str!("../../data/fixtures/computer-use/status.html").trim())
        }
        "fixture://form.html" => {
            Some(include_str!("../../data/fixtures/computer-use/form.html").trim())
        }
        "fixture://inventory.csv" => {
            Some(include_str!("../../data/fixtures/computer-use/inventory.csv").trim())
        }
        _ => None,
    }
}

fn http_output(method: &str, url: &str, status: u16, body: &str, cache_path: &str) -> Value {
    let sha256 = digest(body.as_bytes());
    json!({
        "method":method,
        "url":url,
        "status":status,
        "body":body,
        "headers":{"content-type":content_type(url)},
        "sha256":sha256,
        "cache_path":cache_path,
        "cached":true,
        "provenance":{
            "method":method,
            "url":url,
            "status":status,
            "sha256":sha256,
            "cache_path":cache_path
        }
    })
}

fn content_type(url: &str) -> &'static str {
    let extension = Path::new(url).extension();
    if extension.is_some_and(|value| value.eq_ignore_ascii_case("html")) {
        "text/html"
    } else if extension.is_some_and(|value| value.eq_ignore_ascii_case("csv")) {
        "text/csv"
    } else {
        "application/json"
    }
}

fn filter_csv(content: &str, column: &str, expected: &str) -> Result<String, String> {
    let mut rows = content.lines();
    let header = rows.next().ok_or_else(|| "csv_header_missing".to_owned())?;
    let headers = header.split(',').collect::<Vec<_>>();
    let index = headers
        .iter()
        .position(|name| *name == column)
        .ok_or_else(|| format!("csv_column_missing:{column}"))?;
    let mut selected = vec![header.to_owned()];
    selected.extend(rows.filter_map(|row| {
        let cells = row.split(',').collect::<Vec<_>>();
        (cells.get(index) == Some(&expected)).then(|| row.to_owned())
    }));
    Ok(format!("{}\n", selected.join("\n")))
}

fn unique_csv(content: &str, column: &str) -> Result<String, String> {
    let mut rows = content.lines();
    let header = rows.next().ok_or_else(|| "csv_header_missing".to_owned())?;
    let headers = header.split(',').collect::<Vec<_>>();
    let index = headers
        .iter()
        .position(|name| *name == column)
        .ok_or_else(|| format!("csv_column_missing:{column}"))?;
    let mut values = rows
        .filter_map(|row| row.split(',').nth(index).map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Ok(format!("{}\n", values.join("\n")))
}

fn query_html(html: &str, selector: &str) -> Result<Vec<String>, String> {
    if let Some(id) = selector.strip_prefix('#') {
        for quote in ['"', '\''] {
            let marker = format!("id={quote}{id}{quote}");
            if let Some(attribute) = html.find(&marker) {
                let open = html[..attribute]
                    .rfind('<')
                    .ok_or_else(|| format!("selector_not_found:{selector}"))?;
                let tag_end = html[open + 1..]
                    .find(|character: char| character.is_whitespace() || character == '>')
                    .map(|index| open + 1 + index)
                    .ok_or_else(|| format!("selector_not_found:{selector}"))?;
                let tag = &html[open + 1..tag_end];
                return extract_tag_contents(&html[open..], tag)
                    .map(|value| vec![strip_tags(value).trim().to_owned()]);
            }
        }
        return Err(format!("selector_not_found:{selector}"));
    }
    if selector.starts_with('.') {
        return Err(format!("selector_unsupported_in_fixture_parser:{selector}"));
    }
    extract_tag_contents(html, selector).map(|value| vec![strip_tags(value).trim().to_owned()])
}

fn extract_tag_contents<'a>(html: &'a str, tag: &str) -> Result<&'a str, String> {
    let open = format!("<{tag}");
    let start = html
        .find(&open)
        .ok_or_else(|| format!("selector_not_found:{tag}"))?;
    let content_start = html[start..]
        .find('>')
        .map(|offset| start + offset + 1)
        .ok_or_else(|| format!("selector_not_found:{tag}"))?;
    let close = format!("</{tag}>");
    let content_end = html[content_start..]
        .find(&close)
        .map(|offset| content_start + offset)
        .ok_or_else(|| format!("selector_not_found:{tag}"))?;
    Ok(&html[content_start..content_end])
}

fn strip_tags(value: &str) -> String {
    let mut output = String::new();
    let mut inside = false;
    for character in value.chars() {
        match character {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => output.push(character),
            _ => {}
        }
    }
    output
}
