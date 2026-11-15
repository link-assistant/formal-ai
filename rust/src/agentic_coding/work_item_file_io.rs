//! Explicit actual-client file IO transport. Physical Run provenance is retained;
//! these observations never fabricate protocol Read/Edit receipts or exit codes.
use super::capability_router::{is_workspace_creation_tool, shell_command_tool};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for, write_arguments};
use super::progress::Progress;
use super::tool_result::{
    SourceReadObservation, SourceReadStatus, command_argument, incomplete_receipt,
    observed_payload, reported_exit_code,
};
use crate::protocol::{ChatMessage, client_working_directory};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const PROGRAM: &str = include_str!("../../embedded/data/meta/work-item-file-io-program.cjs");
const FRAME: &str = "FORMAL_AI_FILE_IO_V1 ";
mod search;
pub(super) use search::FileIoSearchObservation;
fn digest(text: &str) -> String {
    crate::source_fetch::sha256_hex(text.as_bytes())
}
fn canonical(value: &Value) -> Option<String> {
    let sorted = value
        .as_object()?
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    serde_json::to_string(&sorted).ok()
}
fn encode(text: &str) -> String {
    STANDARD.encode(text.as_bytes())
}
// Keep canonical decoding explicit without accepting alternate padding spellings.
fn decoded(text: &str) -> Option<String> {
    let bytes = STANDARD.decode(text).ok()?;
    if STANDARD.encode(&bytes) != text {
        return None;
    }
    String::from_utf8(bytes).ok()
}
fn relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && !path.starts_with('/')
        && !path.contains(['\\', '\0'])
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}
fn root_path(root: &str) -> bool {
    root.starts_with('/')
        && !root.ends_with('/')
        && root
            .split('/')
            .skip(1)
            .all(|part| !matches!(part, "" | "." | ".."))
}
pub(super) fn canonical_file_io_path(path: &str, root: Option<&str>) -> Option<String> {
    if relative(path) {
        return Some(path.to_owned());
    }
    let root = root.filter(|root| root_path(root))?;
    let suffix = path.strip_prefix(&(root.to_owned() + "/"))?;
    relative(suffix).then(|| suffix.to_owned())
}
fn file_io_caller_root(messages: &[ChatMessage]) -> Option<String> {
    client_working_directory(
        &messages
            .iter()
            .filter(|message| {
                matches!(
                    message.role.to_lowercase().as_str(),
                    "system" | "developer" | "user"
                )
            })
            .cloned()
            .collect::<Vec<_>>(),
    )
}
fn request(
    operation: &str,
    root: Option<&str>,
    path: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
) -> Option<Value> {
    if !matches!(operation, "root" | "read" | "create" | "replace") {
        return None;
    }
    if operation != "root" && (!root.is_some_and(root_path) || !path.is_some_and(relative)) {
        return None;
    }
    Some(
        json!({"afterBase64":after.map(encode),"beforeSha256":before.map(digest),"operation":operation,"path":path,"root":root,"schema":"formal-ai-owned-file-request/v1"}),
    )
}
pub(super) fn file_io_command(
    operation: &str,
    root: Option<&str>,
    path: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
) -> Option<String> {
    if PROGRAM.len() > 32768 {
        return None;
    }
    let request = request(operation, root, path, before, after)?;
    let payload = encode(&canonical(&request)?);
    let quote = crate::portable_string_contracts::quote_shell_word;
    let command = format!("node -e {} {}", quote(PROGRAM), quote(&payload));
    (command.len() <= 196608).then_some(command)
}
struct Request {
    value: Value,
    identity: String,
    command: String,
}
fn owned_command(command: &str) -> Option<Request> {
    if command.len() > 196608 {
        return None;
    }
    let prefix = format!(
        "node -e {} ",
        crate::portable_string_contracts::quote_shell_word(PROGRAM)
    );
    let payload = command
        .strip_prefix(&prefix)?
        .strip_prefix('\'')?
        .strip_suffix('\'')?;
    if !payload
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "+/=".contains(character))
    {
        return None;
    }
    let input = decoded(payload)?;
    let value: Value = serde_json::from_str(&input).ok()?;
    if canonical(&value)? != input {
        return None;
    }
    if search::valid_request(&value) {
        return Some(Request {
            value,
            identity: digest(&input),
            command: command.to_owned(),
        });
    }
    if value["schema"] != "formal-ai-owned-file-request/v1" {
        return None;
    }
    let keys = value.as_object()?.keys().cloned().collect::<BTreeSet<_>>();
    if keys
        != [
            "afterBase64",
            "beforeSha256",
            "operation",
            "path",
            "root",
            "schema",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    {
        return None;
    }
    let operation = value["operation"].as_str()?;
    if !matches!(operation, "root" | "read" | "create" | "replace") {
        return None;
    }
    if operation == "root" {
        if ["root", "path", "beforeSha256", "afterBase64"]
            .iter()
            .any(|key| !value[*key].is_null())
        {
            return None;
        }
    } else if !value["root"].as_str().is_some_and(root_path)
        || !value["path"].as_str().is_some_and(relative)
    {
        return None;
    }
    Some(Request {
        value,
        identity: digest(&input),
        command: command.to_owned(),
    })
}
struct Parts {
    output: Option<String>,
    code: Option<i64>,
    session: Option<u64>,
}
fn process_parts(raw: &str) -> Parts {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        if let Some(output) = value["output"]
            .as_str()
            .or_else(|| value["stdout"].as_str())
        {
            return Parts {
                output: Some(output.to_owned()),
                code: value["exit_code"]
                    .as_i64()
                    .or_else(|| value["exitCode"].as_i64()),
                session: value["session_id"]
                    .as_u64()
                    .filter(|value| *value > 0 && *value <= 9_007_199_254_740_991),
            };
        }
    }
    Parts {
        output: observed_payload(raw),
        code: reported_exit_code(raw),
        session: None,
    }
}
fn incomplete_cleanup(raw: &str) -> bool {
    serde_json::from_str::<Value>(raw)
        .is_ok_and(|value| value["cleanup_complete"] == false || value["cleanupComplete"] == false)
}
fn receipt(request: &Request, output: &str) -> Option<Value> {
    if output.len() > 262144 {
        return None;
    }
    let encoded = output
        .strip_prefix(FRAME)?
        .strip_suffix('\n')
        .unwrap_or(output.strip_prefix(FRAME)?);
    if encoded.contains(['\r', '\n']) {
        return None;
    }
    let text = decoded(encoded)?;
    let value: Value = serde_json::from_str(&text).ok()?;
    if canonical(&value)? != text
        || value["schema"] != "formal-ai-owned-file-receipt/v1"
        || value["requestIdentity"] != request.identity
        || value["operation"] != request.value["operation"]
        || value["path"] != request.value["path"]
        || request.value["operation"] != "root" && value["root"] != request.value["root"]
    {
        return None;
    }
    Some(value)
}
struct Frame {
    id: String,
    tool: String,
    argument_values: String,
    raw: String,
    error: bool,
    index: usize,
}
fn raw_frames(messages: &[ChatMessage]) -> Vec<Frame> {
    let mut calls = BTreeMap::new();
    let mut duplicates = BTreeSet::new();
    let mut frames: Vec<Frame> = Vec::new();
    for (index, message) in messages
        .iter()
        .enumerate()
        .skip(super::planner::evidence_window_start(messages))
    {
        if message.role.eq_ignore_ascii_case("assistant") {
            for call in &message.tool_calls {
                if calls.contains_key(&call.id) {
                    duplicates.insert(call.id.clone());
                }
                calls.insert(call.id.clone(), call);
            }
        }
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let Some(id) = message.tool_call_id.as_ref() else {
            continue;
        };
        if frames.iter().any(|frame| &frame.id == id) {
            duplicates.insert(id.clone());
            continue;
        }
        let Some(call) = calls.get(id).filter(|_| !duplicates.contains(id)) else {
            continue;
        };
        if message
            .name
            .as_ref()
            .is_some_and(|name| name != &call.function.name)
        {
            continue;
        }
        frames.push(Frame {
            id: id.clone(),
            tool: call.function.name.clone(),
            argument_values: call.function.arguments.clone(),
            raw: message.content.plain_text(),
            error: message.is_error,
            index,
        });
    }
    frames
        .into_iter()
        .filter(|frame| !duplicates.contains(&frame.id))
        .collect()
}
pub(super) struct Pending {
    session: u64,
    polls: usize,
}
struct Completed {
    code: Option<i64>,
    output: String,
    at: usize,
    polls: usize,
    pending: Option<Pending>,
    gap: Option<String>,
}
fn completed_run(frame: &Frame, frames: &[Frame]) -> Completed {
    let mut parts = process_parts(&frame.raw);
    let mut output = parts.output.clone().unwrap_or_default();
    let mut polls = 0;
    let mut at = frame.index;
    if frame.error
        || (incomplete_receipt(&frame.raw) || incomplete_cleanup(&frame.raw))
            && parts.session.is_none()
    {
        return Completed {
            code: parts.code,
            output,
            at,
            polls,
            pending: None,
            gap: Some(
                if frame.error {
                    "FileIoProviderReportedFailure"
                } else {
                    "FileIoIncompleteProcessReceipt"
                }
                .to_owned(),
            ),
        };
    }
    while parts.code.is_none() && parts.session.is_some() {
        let session = parts.session.unwrap_or_default();
        let next = frames.iter().find(|next| {
            next.index > at
                && next
                    .tool
                    .rsplit("__")
                    .next()
                    .unwrap_or("")
                    .rsplit(['.', ':', '/'])
                    .next()
                    == Some("write_stdin")
                && serde_json::from_str::<Value>(&next.argument_values).is_ok_and(
                    |argument_values| {
                        argument_values["session_id"].as_u64() == Some(session)
                            && (argument_values.get("chars").is_none()
                                || argument_values["chars"] == "")
                    },
                )
        });
        let Some(next) = next else {
            return Completed {
                code: None,
                output,
                at,
                polls,
                pending: Some(Pending { session, polls }),
                gap: None,
            };
        };
        polls += 1;
        if polls > 3 {
            return Completed {
                code: None,
                output,
                at,
                polls,
                pending: None,
                gap: Some("FileIoPollingBudgetExceeded".to_owned()),
            };
        }
        parts = process_parts(&next.raw);
        at = next.index;
        if next.error
            || (incomplete_receipt(&next.raw) || incomplete_cleanup(&next.raw))
                && parts.session.is_none()
        {
            return Completed {
                code: parts.code,
                output,
                at,
                polls,
                pending: None,
                gap: Some("FileIoPollingFailed".to_owned()),
            };
        }
        output.push_str(parts.output.as_deref().unwrap_or(""));
        if output.len() > 262144 {
            return Completed {
                code: parts.code,
                output: String::new(),
                at,
                polls,
                pending: None,
                gap: Some("FileIoOutputLimit".to_owned()),
            };
        }
    }
    Completed {
        code: parts.code,
        output,
        at,
        polls,
        pending: None,
        gap: None,
    }
}
pub(super) struct FileIoAttempt {
    pub capability: Capability,
    pub succeeded: bool,
    pub detail: String,
    pub arguments: Option<String>,
    pub source_read: Option<SourceReadObservation>,
    pub transport: Value,
}
impl FileIoAttempt {
    pub(super) fn source_observation(&self) -> Option<&SourceReadObservation> {
        self.source_read.as_ref()
    }
}
pub(super) struct WorkItemFileProgress {
    attempts: Vec<FileIoAttempt>,
    searches: Vec<FileIoSearchObservation>,
    pub root: Option<String>,
    pub pending: Option<Pending>,
    pub gap: Option<String>,
    transport_diagnostics: Vec<Value>,
}
impl WorkItemFileProgress {
    pub(super) fn scan(messages: &[ChatMessage]) -> Self {
        let original = Progress::scan(messages);
        let frames = raw_frames(messages);
        let owned = frames
            .iter()
            .filter_map(|frame| {
                Some((
                    frame,
                    owned_command(&command_argument(&frame.argument_values)?)?,
                ))
            })
            .collect::<Vec<_>>();
        let mut root = file_io_caller_root(messages);
        let mut pending = None;
        let mut gap = root
            .as_deref()
            .filter(|root| !root_path(root))
            .map(|_| "FileIoCallerRootUnsupported".to_owned());
        let mut events: Vec<(usize, FileIoAttempt)> = Vec::new();
        let mut transport_diagnostics = Vec::new();
        let mut searches = Vec::new();
        for (frame, request) in &owned {
            let completed = completed_run(frame, &frames);
            transport_diagnostics.push(json!({"kind":"registered-process","actualTool":frame.tool,"callId":frame.id,"requestIdentity":request.identity,
                "operation":request.value["operation"],"path":request.value["path"],"actualExitCode":completed.code,
                "pendingSession":completed.pending.as_ref().map(|pending|pending.session),"polls":completed.polls,"reason":completed.gap}));
            if request.value["operation"] == "search" {
                searches.push(search::observation(frame, request, &completed));
            }
            if completed.pending.is_some() {
                pending = completed.pending;
                continue;
            }
            if completed.gap.is_some() {
                gap = completed.gap;
                continue;
            }
            if request.value["operation"] == "search" {
                if root
                    .as_deref()
                    .is_some_and(|root| request.value["root"] != root)
                {
                    gap = Some("FileIoRootContextContradicted".to_owned());
                }
                continue;
            }
            let observed = receipt(request, &completed.output);
            let code = completed.code;
            if request.value["operation"] == "root" {
                if code == Some(0)
                    && observed.as_ref().is_some_and(|value| {
                        value["state"] == "observed-root"
                            && value["root"].as_str().is_some_and(root_path)
                    })
                {
                    let observed_root = observed
                        .as_ref()
                        .and_then(|value| value["root"].as_str())
                        .unwrap_or_default();
                    if root.as_deref().is_some_and(|root| root != observed_root) {
                        gap = Some("FileIoRootContextContradicted".to_owned());
                    } else {
                        root = Some(observed_root.to_owned());
                    }
                } else {
                    gap = Some("FileIoRootObservationUnavailable".to_owned());
                }
                continue;
            }
            if root
                .as_deref()
                .is_some_and(|root| request.value["root"] != root)
            {
                gap = Some("FileIoRootContextContradicted".to_owned());
                continue;
            }
            let operation = request.value["operation"].as_str().unwrap_or_default();
            let path = request.value["path"].as_str().unwrap_or_default();
            let capability = if operation == "read" {
                Capability::Read
            } else if operation == "create" {
                Capability::Write
            } else {
                Capability::Edit
            };
            let mut argument_values = json!({"filePath":path});
            let mut source_read = None;
            let mut succeeded = false;
            if operation == "read" {
                let mut read = SourceReadObservation {
                    status: if code == Some(0) {
                        SourceReadStatus::ReportedSuccess
                    } else if code.is_none() {
                        SourceReadStatus::Unknown
                    } else {
                        SourceReadStatus::ReportedFailure
                    },
                    complete: false,
                    absent: false,
                    source: None,
                    error: Some(
                        if code.is_none() {
                            "FileIoProcessStatusUnknown"
                        } else if code != Some(0) {
                            "FileIoProcessFailed"
                        } else {
                            "FileIoReceiptInvalid"
                        }
                        .to_owned(),
                    ),
                };
                if code == Some(0)
                    && observed
                        .as_ref()
                        .is_some_and(|value| value["state"] == "absent")
                {
                    read.absent = true;
                    read.error = Some("ENOENT".to_owned());
                }
                if code == Some(0)
                    && observed
                        .as_ref()
                        .is_some_and(|value| value["state"] == "observed-source")
                {
                    if let Some(source) = observed
                        .as_ref()
                        .and_then(|value| value["contentBase64"].as_str())
                        .and_then(decoded)
                    {
                        if source.len() <= 131072
                            && observed.as_ref().is_some_and(|value| {
                                value["byteLength"].as_u64() == u64::try_from(source.len()).ok()
                                    && value["sha256"] == digest(&source)
                            })
                        {
                            read.complete = true;
                            read.error = None;
                            read.source = Some(source);
                            succeeded = true;
                        }
                    }
                }
                source_read = Some(read);
            } else if let Some(after) = request.value["afterBase64"].as_str().and_then(decoded) {
                if operation == "create" {
                    argument_values["content"] = json!(after);
                } else {
                    argument_values["newString"] = json!(after);
                    argument_values["beforeSha256"] = request.value["beforeSha256"].clone();
                }
                succeeded = code == Some(0)
                    && observed.as_ref().is_some_and(|value| {
                        value["state"] == "effect-applied"
                            && value["beforeSha256"] == request.value["beforeSha256"]
                            && value["afterSha256"] == digest(&after)
                    });
            }
            let transport = json!({"kind":"registered-run-file-io","actualTool":frame.tool,"callId":frame.id,"requestIdentity":request.identity,"command":request.command,"exitCode":code,"completedAt":completed.at,"originalReceipt":frame.raw});
            events.push((
                completed.at,
                FileIoAttempt {
                    capability,
                    succeeded,
                    detail: frame.raw.clone(),
                    arguments: Some(argument_values.to_string()),
                    source_read,
                    transport,
                },
            ));
            if code.is_none() {
                gap = Some("FileIoProcessStatusUnknown".to_owned());
            }
        }
        let mut used = BTreeSet::new();
        for attempt in original.research_attempts() {
            if attempt
                .arguments
                .as_deref()
                .and_then(command_argument)
                .and_then(|command| owned_command(&command))
                .is_some()
            {
                continue;
            }
            let Some((index, frame)) = frames.iter().enumerate().find(|(index, frame)| {
                !used.contains(index)
                    && attempt.tool.as_deref() == Some(frame.tool.as_str())
                    && attempt.arguments.as_deref() == Some(frame.argument_values.as_str())
            }) else {
                continue;
            };
            used.insert(index);
            let mut arguments = attempt.arguments.clone();
            let mut transport = Value::Null;
            if matches!(
                attempt.capability,
                Capability::Read | Capability::Edit | Capability::Write
            ) {
                if let Some(mut argument_values) = arguments
                    .as_deref()
                    .and_then(|text| serde_json::from_str::<Value>(text).ok())
                {
                    let keys = ["path", "filePath", "file_path", "absolute_path"]
                        .iter()
                        .filter(|key| argument_values.get(**key).is_some())
                        .copied()
                        .collect::<Vec<_>>();
                    let paths = keys
                        .iter()
                        .map(|key| {
                            argument_values[*key]
                                .as_str()
                                .and_then(|path| canonical_file_io_path(path, root.as_deref()))
                        })
                        .collect::<Vec<_>>();
                    if paths.is_empty()
                        || paths.iter().any(|path| path.is_none() || path != &paths[0])
                    {
                        arguments = None;
                        transport = json!({"kind":"client-file-tool","root":root,"canonicalPath":null,
                            "actualTool":attempt.tool,"physicalArguments":attempt.arguments,
                            "reason":"AmbiguousOrUnownedPathAliases"});
                    } else {
                        let path = paths[0].as_deref().unwrap_or_default();
                        for key in keys {
                            argument_values[key] = json!(path);
                        }
                        argument_values["filePath"] = json!(path);
                        arguments = Some(argument_values.to_string());
                        transport = json!({"kind":"client-file-tool","root":root,"canonicalPath":path,"actualTool":attempt.tool,"physicalArguments":attempt.arguments});
                    }
                }
            }
            if transport["kind"] == "client-file-tool" {
                transport["callId"] = json!(frame.id);
                transport["completedAt"] = json!(frame.index);
            }
            let source_read = attempt
                .source_observation()
                .map(|read| SourceReadObservation {
                    status: read.status,
                    complete: read.complete,
                    absent: read.absent,
                    source: read.source.clone(),
                    error: read.error.clone(),
                });
            events.push((
                frame.index,
                FileIoAttempt {
                    capability: attempt.capability,
                    succeeded: attempt.succeeded,
                    detail: attempt.detail.clone(),
                    arguments,
                    source_read,
                    transport,
                },
            ));
        }
        events.sort_by_key(|(index, _)| *index);
        for index in 0..events.len() {
            if events[index].1.capability != Capability::Edit
                || events[index].1.transport["kind"] != "registered-run-file-io"
            {
                continue;
            }
            let mut argument_values: Value =
                serde_json::from_str(events[index].1.arguments.as_deref().unwrap_or(""))
                    .unwrap_or(Value::Null);
            let prior = events[..index].iter().rev().find_map(|(_, attempt)| {
                let arguments: Value = serde_json::from_str(attempt.arguments.as_deref()?).ok()?;
                let read = attempt.source_observation()?;
                (attempt.capability == Capability::Read
                    && arguments["filePath"] == argument_values["filePath"]
                    && read.complete
                    && read.error.is_none())
                .then(|| read.source.clone())
                .flatten()
            });
            if let Some(prior) =
                prior.filter(|source| argument_values["beforeSha256"] == digest(source))
            {
                argument_values["oldString"] = json!(prior);
                events[index].1.arguments = Some(argument_values.to_string());
            } else {
                events[index].1.succeeded = false;
                gap = Some("FileIoObservedPreimageUnavailable".to_owned());
            }
        }
        Self {
            attempts: events.into_iter().map(|(_, attempt)| attempt).collect(),
            root,
            pending,
            gap,
            transport_diagnostics,
            searches,
        }
    }
    pub(super) fn search_for(
        &self,
        directory: &str,
        identifiers: &[String],
    ) -> Option<&FileIoSearchObservation> {
        search::find(&self.searches, directory, identifiers)
    }
    pub(super) fn research_attempts(&self) -> &[FileIoAttempt] {
        &self.attempts
    }
    pub(super) fn transport_diagnostics(&self) -> &[Value] {
        &self.transport_diagnostics
    }
    pub(super) fn source_read_for(&self, path: &str) -> Option<&SourceReadObservation> {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| {
                attempt.capability == Capability::Read
                    && attempt
                        .arguments
                        .as_deref()
                        .and_then(|text| serde_json::from_str::<Value>(text).ok())
                        .is_some_and(|argument_values| argument_values["filePath"] == path)
            })
            .and_then(FileIoAttempt::source_observation)
    }
}
pub(super) enum FileIoPlan {
    Pending(AgenticPlan),
    Gap(String),
}
pub(super) fn file_io_pending_step(
    progress: &WorkItemFileProgress,
    tools: &[&str],
) -> Option<FileIoPlan> {
    if let Some(gap) = progress.gap.as_ref() {
        return Some(FileIoPlan::Gap(gap.clone()));
    }
    let pending = progress.pending.as_ref()?;
    if pending.polls >= 3 {
        return Some(FileIoPlan::Gap("FileIoPollingBudgetExceeded".to_owned()));
    }
    let tool = tools.iter().find(|name| {
        name.rsplit("__")
            .next()
            .unwrap_or("")
            .rsplit(['.', ':', '/'])
            .next()
            == Some("write_stdin")
    });
    Some(tool.map_or_else(||FileIoPlan::Gap("FileIoPollingToolUnavailable".to_owned()),|tool|FileIoPlan::Pending(plan_one(tool,json!({"session_id":pending.session,"chars":"","max_output_tokens":65536,"yield_time_ms":1000}).to_string()))))
}
fn run_step(
    messages: &[ChatMessage],
    tools: &[&str],
    operation: &str,
    path: &str,
    before: Option<&str>,
    after: Option<&str>,
) -> FileIoPlan {
    let progress = WorkItemFileProgress::scan(messages);
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return pending;
    }
    let Some(tool) = shell_command_tool(tools) else {
        return FileIoPlan::Gap("FileIoRunToolUnavailable".to_owned());
    };
    let command = if progress.root.is_some() {
        file_io_command(
            operation,
            progress.root.as_deref(),
            Some(path),
            before,
            after,
        )
    } else {
        file_io_command("root", None, None, None, None)
    };
    command.map_or_else(
        || FileIoPlan::Gap("FileIoCommandLimitOrUnavailable".to_owned()),
        |command| {
            FileIoPlan::Pending(plan_one(
                tool,
                json!({"command":command,"max_output_tokens":65536,"yield_time_ms":1000})
                    .to_string(),
            ))
        },
    )
}
pub(super) fn file_io_search_step(
    messages: &[ChatMessage],
    tools: &[&str],
    directory: &str,
    identifiers: &[String],
    task: &str,
) -> FileIoPlan {
    search::step(messages, tools, directory, identifiers, task)
}
pub(super) fn file_io_read_step(
    messages: &[ChatMessage],
    tools: &[&str],
    path: &str,
    task: &str,
) -> FileIoPlan {
    // Policy query only: never emit this conceptual Read or fabricate a receipt.
    if super::file_read::read_policy_blocks_plan(
        task,
        &plan_one("read", json!({"filePath":path}).to_string()),
    ) {
        return FileIoPlan::Gap("ReadPolicyRefused".to_owned());
    }
    let progress = WorkItemFileProgress::scan(messages);
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return pending;
    }
    tool_for(tools, Capability::Read).map_or_else(
        || run_step(messages, tools, "read", path, None, None),
        |tool| {
            FileIoPlan::Pending(plan_one(
                tool,
                super::workspace_change::read_arguments(path),
            ))
        },
    )
}
pub(super) fn file_io_effect_step(
    messages: &[ChatMessage],
    tools: &[&str],
    path: &str,
    before: Option<&str>,
    after: &str,
) -> FileIoPlan {
    if after.len() > 131072 {
        return FileIoPlan::Gap("FileIoCandidateLimit".to_owned());
    }
    let progress = WorkItemFileProgress::scan(messages);
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return pending;
    }
    match before {
        None => tool_for(tools, Capability::Write).map_or_else(
            || run_step(messages, tools, "create", path, None, Some(after)),
            |tool| FileIoPlan::Pending(plan_one(tool, write_arguments(path, after))),
        ),
        Some(before) => tool_for(tools, Capability::Edit)
            .filter(|tool| !is_workspace_creation_tool(tool))
            .map_or_else(
                || run_step(messages, tools, "replace", path, Some(before), Some(after)),
                |tool| {
                    FileIoPlan::Pending(plan_one(
                        tool,
                        json!({"filePath":path,"oldString":before,"newString":after}).to_string(),
                    ))
                },
            ),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/work_item_file_io.rs"]
mod tests;
