//! Paired search protocol projection. Traversal exists once in the shared data program.
use super::*;
fn specification(directory: &str, identifiers: &[String]) -> Option<Vec<String>> {
    if !relative(directory)
        || directory.split('/').any(|part| part.starts_with('.'))
        || identifiers.is_empty()
        || identifiers.len() > 8
    {
        return None;
    }
    for identifier in identifiers {
        let mut characters = identifier.chars();
        let first = characters.next()?;
        if identifier.len() > 128
            || !(first.is_ascii_alphabetic() || first == '_')
            || characters.any(|character| !(character.is_ascii_alphanumeric() || character == '_'))
        {
            return None;
        }
    }
    let mut sorted = identifiers.to_vec();
    sorted.sort();
    sorted.dedup();
    (sorted.len() == identifiers.len()).then_some(sorted)
}
pub(super) fn valid_request(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    if object.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != ["directory", "identifiers", "operation", "root", "schema"]
            .into_iter()
            .collect()
        || value["schema"] != "formal-ai-owned-search-request/v1"
        || value["operation"] != "search"
        || !value["root"].as_str().is_some_and(root_path)
    {
        return false;
    }
    let Some(directory) = value["directory"].as_str() else {
        return false;
    };
    let Some(identifiers) = value["identifiers"].as_array().and_then(|values| {
        values
            .iter()
            .map(|value| value.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()
    }) else {
        return false;
    };
    specification(directory, &identifiers).is_some_and(|sorted| sorted == identifiers)
}
pub(super) fn command(root: &str, directory: &str, identifiers: &[String]) -> Option<String> {
    if PROGRAM.len() > 32768 || !root_path(root) {
        return None;
    }
    let identifiers = specification(directory, identifiers)?;
    let request = json!({"schema":"formal-ai-owned-search-request/v1","operation":"search","root":root,"directory":directory,"identifiers":identifiers});
    let quote = crate::portable_string_contracts::quote_shell_word;
    let command = format!(
        "node -e {} {}",
        quote(PROGRAM),
        quote(&encode(&canonical(&request)?))
    );
    (command.len() <= 196608).then_some(command)
}
fn receipt_for(request: &Request, output: &str) -> Option<Value> {
    if output.len() > 262144 {
        return None;
    }
    let payload = output.strip_prefix(FRAME)?;
    let encoded = payload.strip_suffix('\n').unwrap_or(payload);
    if encoded.contains(['\r', '\n']) {
        return None;
    }
    let text = decoded(encoded)?;
    let receipt: Value = serde_json::from_str(&text).ok()?;
    let value = &request.value;
    if canonical(&receipt)? != text
        || receipt["schema"] != "formal-ai-owned-search-receipt/v1"
        || receipt["operation"] != "search"
        || receipt["requestIdentity"] != request.identity
        || receipt["root"] != value["root"]
        || receipt["directory"] != value["directory"]
        || receipt["identifiers"] != value["identifiers"]
        || receipt["state"] != "observed-search"
        || !receipt["complete"].is_boolean()
        || !receipt["visitedFiles"]
            .as_u64()
            .is_some_and(|count| count <= 1025)
        || !receipt["visitedDirectories"]
            .as_u64()
            .is_some_and(|count| count <= 129)
        || !receipt["bytesRead"]
            .as_u64()
            .is_some_and(|count| count <= 16777216)
        || receipt["scope"]
            != "bounded-nonhidden-rust-source-candidates; text matches are not bindings; fresh Read and CST validation required"
    {
        return None;
    }
    let files = receipt["files"].as_array()?;
    let gaps = receipt["gaps"].as_array()?;
    if files.len() > 8 || gaps.len() > 32 || receipt["complete"] == true && !gaps.is_empty() {
        return None;
    }
    let mut paths = BTreeSet::new();
    let directory = value["directory"].as_str()?;
    let identifiers = value["identifiers"].as_array()?;
    for file in files {
        let path = file["path"].as_str()?;
        let hash = file["sha256"].as_str()?;
        let hits = file["identifiers"].as_array()?;
        if !relative(path)
            || !path.starts_with(&(directory.to_owned() + "/"))
            || !path.ends_with(".rs")
            || path.split('/').any(|part| part.starts_with('.'))
            || !paths.insert(path)
            || hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || !file["byteLength"]
                .as_u64()
                .is_some_and(|count| count <= 131072)
            || hits.is_empty()
            || hits.iter().any(|hit| !identifiers.contains(hit))
            || hits
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>()
                .len()
                != hits.len()
        {
            return None;
        }
    }
    Some(receipt)
}
pub(crate) struct FileIoSearchObservation {
    pub directory: String,
    pub identifiers: Vec<String>,
    pub root: String,
    pub request_identity: String,
    pub status: &'static str,
    pub complete: bool,
    pub files: Vec<Value>,
    pub receipt: Option<Value>,
    pub error: Option<String>,
    pub transport: Value,
}
pub(super) fn observation(
    frame: &Frame,
    request: &Request,
    completed: &Completed,
) -> FileIoSearchObservation {
    let receipt = receipt_for(request, &completed.output);
    let succeeded = completed.code == Some(0)
        && receipt.is_some()
        && completed.gap.is_none()
        && completed.pending.is_none();
    let status = if completed.pending.is_some() {
        "pending"
    } else if completed.code == Some(0) {
        "reported-success"
    } else if completed.code.is_none() {
        "unknown"
    } else {
        "reported-failure"
    };
    let identifiers = request.value["identifiers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(str::to_owned))
        .collect();
    FileIoSearchObservation {
        directory: request.value["directory"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        identifiers,
        root: request.value["root"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        request_identity: request.identity.clone(),
        status,
        complete: succeeded
            && receipt
                .as_ref()
                .is_some_and(|value| value["complete"] == true),
        files: if succeeded {
            receipt
                .as_ref()
                .and_then(|value| value["files"].as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            Vec::new()
        },
        error: completed.gap.clone().or_else(|| {
            (!succeeded && completed.pending.is_none())
                .then(|| "FileIoSearchObservationUnavailable".to_owned())
        }),
        receipt: if succeeded { receipt } else { None },
        transport: json!({"kind":"registered-run-search","actualTool":frame.tool,"callId":frame.id,"command":request.command,
            "actualExitCode":completed.code,"pendingSession":completed.pending.as_ref().map(|pending|pending.session),
            "completedAt":completed.at,"originalReceipt":frame.raw,"root":request.value["root"]}),
    }
}
pub(super) fn find<'a>(
    searches: &'a [FileIoSearchObservation],
    directory: &str,
    identifiers: &[String],
) -> Option<&'a FileIoSearchObservation> {
    let identifiers = specification(directory, identifiers)?;
    searches.iter().rev().find(|observation| {
        observation.directory == directory && observation.identifiers == identifiers
    })
}
pub(super) fn step(
    messages: &[ChatMessage],
    tools: &[&str],
    directory: &str,
    identifiers: &[String],
    task: &str,
) -> FileIoPlan {
    let Some(identifiers) = specification(directory, identifiers) else {
        return FileIoPlan::Gap("FileIoSearchScopeRequired".to_owned());
    };
    if super::super::file_read::read_policy_blocks_plan(
        task,
        &plan_one("read", json!({"filePath":directory}).to_string()),
    ) {
        return FileIoPlan::Gap("ReadPolicyRefused".to_owned());
    }
    let progress = WorkItemFileProgress::scan(messages);
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return pending;
    }
    let Some(tool) = shell_command_tool(tools) else {
        return FileIoPlan::Gap("FileIoRunToolUnavailable".to_owned());
    };
    let command = match progress.root.as_deref() {
        Some(root) => command(root, directory, &identifiers),
        None => file_io_command("root", None, None, None, None),
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
