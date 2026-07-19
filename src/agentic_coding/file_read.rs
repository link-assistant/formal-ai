//! File-reading agentic recipe for local workspace prompts (issue #627).

use serde_json::json;

use super::general_planner::has_file_write_intent;
use super::planner::{tool_capability, AgenticPlan, Capability, PlannedToolCall};
use crate::protocol::{ChatMessage, ToolCall};
use crate::seed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FileReadTask {
    Direct {
        path: String,
        mode: FileReadMode,
        prefer_run: bool,
    },
    ListThenRead {
        directory: String,
        selection: FileSelection,
        mode: FileReadMode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FileReadMode {
    Full,
    FirstLine,
    ExtractValue(String),
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FileSelection {
    First,
    Last,
    All,
}

struct ToolResultRecord {
    capability: Option<Capability>,
    arguments: serde_json::Value,
    content: String,
}

/// The issue-#627 file-reading recipe: direct filename requests use an advertised
/// `read` tool, shell-shaped `cat` requests use the run tool, and multi-step
/// "list then read" prompts walk `bash(ls)` → `read(file)` → final content.
pub(super) fn plan_file_read_step(
    task: &FileReadTask,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> AgenticPlan {
    let read_tool = tool_for(tool_names, Capability::Read);
    let run_tool = tool_for(tool_names, Capability::Run);
    let records = tool_result_records(messages);

    match task {
        FileReadTask::Direct {
            path,
            mode,
            prefer_run,
        } => plan_direct_file_read(path, mode, *prefer_run, read_tool, run_tool, &records),
        FileReadTask::ListThenRead {
            directory,
            selection,
            mode,
        } => plan_list_then_read(directory, *selection, mode, read_tool, run_tool, &records),
    }
}

fn plan_direct_file_read(
    path: &str,
    mode: &FileReadMode,
    prefer_run: bool,
    read_tool: Option<&str>,
    run_tool: Option<&str>,
    records: &[ToolResultRecord],
) -> AgenticPlan {
    if let Some(content) = read_result_for_path(records, path)
        .or_else(|| run_result_for_command(records, &read_command_for(path, mode)))
    {
        return AgenticPlan::Final(file_read_final_answer(
            mode,
            &[(path.to_owned(), content.to_owned())],
        ));
    }

    if prefer_run {
        if let Some(tool) = run_tool {
            return plan_one(
                tool,
                json!({ "command": read_command_for(path, mode) }).to_string(),
            );
        }
    }

    if let Some(tool) = read_tool {
        return plan_one(tool, read_arguments(path));
    }
    if let Some(tool) = run_tool {
        return plan_one(
            tool,
            json!({ "command": read_command_for(path, mode) }).to_string(),
        );
    }

    AgenticPlan::Final(format!(
        "I can read `{path}` when the client advertises a file read tool or a shell tool."
    ))
}

fn plan_list_then_read(
    directory: &str,
    selection: FileSelection,
    mode: &FileReadMode,
    read_tool: Option<&str>,
    run_tool: Option<&str>,
    records: &[ToolResultRecord],
) -> AgenticPlan {
    let list_command = list_files_command(directory);
    let Some(listing) = run_result_for_command(records, &list_command) else {
        if let Some(tool) = run_tool {
            return plan_one(tool, json!({ "command": list_command }).to_string());
        }
        return AgenticPlan::Final(
            "I can resolve that file selection when the client advertises a shell tool for listing files."
                .to_owned(),
        );
    };

    let paths = selected_paths_from_listing(directory, listing, selection);
    if paths.is_empty() {
        return AgenticPlan::Final(format!("No files were listed in `{directory}`."));
    }

    if let Some(contents) = read_results_for_paths(records, &paths) {
        return AgenticPlan::Final(file_read_final_answer(mode, &contents));
    }

    if selection == FileSelection::All {
        if let Some(tool) = read_tool {
            let calls = paths
                .iter()
                .filter(|path| read_result_for_path(records, path).is_none())
                .map(|path| PlannedToolCall {
                    tool: tool.to_owned(),
                    arguments: read_arguments(path),
                })
                .collect();
            return AgenticPlan::ToolCalls(calls);
        }
    } else if let Some(path) = paths.first() {
        if let Some(tool) = read_tool {
            return plan_one(tool, read_arguments(path));
        }
    }

    if let Some(tool) = run_tool {
        let command = if selection == FileSelection::All {
            cat_many_command(&paths)
        } else {
            read_command_for(&paths[0], mode)
        };
        if let Some(content) = run_result_for_command(records, &command) {
            return AgenticPlan::Final(file_read_final_answer(
                mode,
                &[(paths.join(", "), content.to_owned())],
            ));
        }
        return plan_one(tool, json!({ "command": command }).to_string());
    }

    AgenticPlan::Final(
        "I can read the selected file after listing when the client advertises a read or shell tool."
            .to_owned(),
    )
}

pub(super) fn file_read_task_for(prompt: &str) -> Option<FileReadTask> {
    let lower = prompt.to_lowercase();

    // Issue #681: a file-creation / write request must never be routed to the read
    // recipe — the target does not exist yet, so reading it is always the wrong
    // tool. When the request is a write intent, decline here and let the router
    // fall through to the general write/create planner. This is the general rule
    // ("write intent beats read intent"), not a per-phrase special case: the same
    // gate catches create/write/save/generate across every supported language.
    if has_file_write_intent(&lower) {
        return None;
    }

    if let Some(path) = leading_cat_path(prompt) {
        return Some(FileReadTask::Direct {
            path,
            mode: FileReadMode::Full,
            prefer_run: false,
        });
    }

    if asks_to_read_every_file(&lower) {
        return Some(FileReadTask::ListThenRead {
            directory: String::from("."),
            selection: FileSelection::All,
            mode: FileReadMode::Summary,
        });
    }

    if asks_to_list_then_read(&lower) {
        return Some(FileReadTask::ListThenRead {
            directory: directory_for_list_read(prompt).unwrap_or_else(|| String::from(".")),
            selection: selection_for_prompt(&lower),
            mode: mode_for_prompt(prompt),
        });
    }

    if asks_to_read_file_in_folder(&lower) {
        return Some(FileReadTask::ListThenRead {
            directory: directory_for_list_read(prompt).unwrap_or_else(|| String::from(".")),
            selection: FileSelection::First,
            mode: mode_for_prompt(prompt),
        });
    }

    if has_file_read_intent(&lower) {
        if let Some(path) = first_local_file_path(prompt) {
            return Some(FileReadTask::Direct {
                path,
                mode: mode_for_prompt(prompt),
                prefer_run: false,
            });
        }
    }

    None
}

fn has_file_read_intent(lower: &str) -> bool {
    seed::lexicon().mentions_role(seed::ROLE_FILE_READ_ACTION_CUE, lower)
}

fn asks_to_read_every_file(lower: &str) -> bool {
    (lower.contains("read every file") || lower.contains("read all files"))
        && (lower.contains("summarize") || lower.contains("summary") || lower.contains("here"))
}

fn asks_to_list_then_read(lower: &str) -> bool {
    let lists_files = lower.contains("list the files")
        || lower.contains("list files")
        || lower.contains("ls the folder")
        || lower.contains("ls ");
    let reads_after = lower.contains("read")
        || lower.contains("contents")
        || lower.contains("content")
        || lower.contains("show me");
    lists_files && reads_after
}

fn asks_to_read_file_in_folder(lower: &str) -> bool {
    lower.contains("read the file") && (lower.contains(" folder") || lower.contains(" directory"))
}

fn selection_for_prompt(lower: &str) -> FileSelection {
    if lower.contains("last") {
        FileSelection::Last
    } else {
        FileSelection::First
    }
}

fn mode_for_prompt(prompt: &str) -> FileReadMode {
    let lower = prompt.to_ascii_lowercase();
    if lower.contains("first line") {
        return FileReadMode::FirstLine;
    }
    if let Some(key) = extract_value_key(prompt) {
        return FileReadMode::ExtractValue(key);
    }
    if lower.contains("summarize") || lower.contains("summary") {
        return FileReadMode::Summary;
    }
    FileReadMode::Full
}

fn extract_value_key(prompt: &str) -> Option<String> {
    let lower = prompt.to_ascii_lowercase();
    let marker = "value of ";
    let start = lower.find(marker)? + marker.len();
    let rest = &prompt[start..];
    let key = rest
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|c: char| !is_file_path_char(c));
    (!key.is_empty()).then(|| key.to_owned())
}

fn leading_cat_path(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim().trim_matches('`').trim();
    let mut parts = trimmed.split_whitespace();
    let command = parts.next()?;
    if !command.eq_ignore_ascii_case("cat") {
        return None;
    }
    parts
        .next()
        .map(clean_file_token)
        .filter(|path| !path.is_empty())
}

fn first_local_file_path(prompt: &str) -> Option<String> {
    prompt
        .split_whitespace()
        .map(clean_file_token)
        .find(|token| looks_like_local_file_path(token))
}

fn clean_file_token(token: &str) -> String {
    token
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches(|c: char| {
            matches!(
                c,
                ',' | ';' | ':' | '!' | '?' | ')' | '(' | '[' | ']' | '{' | '}'
            )
        })
        .to_owned()
}

fn looks_like_local_file_path(token: &str) -> bool {
    if token.is_empty()
        || token.contains("://")
        || token.starts_with("http:")
        || token.starts_with("https:")
    {
        return false;
    }
    if token.contains('/') {
        return token.chars().all(is_file_path_char);
    }
    let Some((stem, extension)) = token.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && !extension.is_empty()
        && extension.len() <= 12
        && token.chars().all(is_file_path_char)
}

const fn is_file_path_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/' | '\\' | '@')
}

fn directory_for_list_read(prompt: &str) -> Option<String> {
    let tokens = prompt
        .split_whitespace()
        .map(clean_file_token)
        .collect::<Vec<_>>();
    for window in tokens.windows(3) {
        if window[0].eq_ignore_ascii_case("the")
            && !window[1].is_empty()
            && (window[2].eq_ignore_ascii_case("folder")
                || window[2].eq_ignore_ascii_case("directory"))
        {
            return Some(window[1].clone());
        }
        if (window[0].eq_ignore_ascii_case("in") || window[0].eq_ignore_ascii_case("inside"))
            && window[1].eq_ignore_ascii_case("the")
            && !window[2].is_empty()
        {
            return Some(window[2].clone());
        }
    }
    for window in tokens.windows(2) {
        let lower = window[0].to_ascii_lowercase();
        if !window[0].is_empty()
            && (window[1].eq_ignore_ascii_case("folder")
                || window[1].eq_ignore_ascii_case("directory"))
            && !["the", "a", "this", "current"].contains(&lower.as_str())
        {
            return Some(window[0].clone());
        }
    }
    None
}

fn tool_result_records(messages: &[ChatMessage]) -> Vec<ToolResultRecord> {
    // Only results produced *within the current user turn* ground a read. A
    // client loops tool calls without a new user message, so the current turn is
    // everything after the last user message; a `cat 1.txt` result from an
    // earlier turn must never be replayed as the answer to a later `read 1.txt`
    // (issue #755 — read/list determinism). Without this bound the recipe short
    // -circuits on a stale, unrelated result and skips the fresh read entirely.
    let turn_start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    let mut records = Vec::new();
    for (index, message) in messages.iter().enumerate().skip(turn_start) {
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let (capability, arguments) =
            tool_result_call(messages, index).map_or((None, serde_json::Value::Null), |call| {
                (
                    tool_capability(&call.function.name),
                    serde_json::from_str(&call.function.arguments)
                        .unwrap_or(serde_json::Value::Null),
                )
            });
        records.push(ToolResultRecord {
            capability,
            arguments,
            content: message.content.plain_text(),
        });
    }
    records
}

fn tool_result_call(messages: &[ChatMessage], index: usize) -> Option<&ToolCall> {
    let call_id = messages[index].tool_call_id.as_ref()?;
    messages[..index]
        .iter()
        .flat_map(|prior| prior.tool_calls.iter())
        .find(|call| &call.id == call_id)
}

fn read_result_for_path<'a>(records: &'a [ToolResultRecord], path: &str) -> Option<&'a str> {
    records
        .iter()
        .find(|record| {
            record.capability == Some(Capability::Read)
                && path_argument(&record.arguments).as_deref() == Some(path)
        })
        .map(|record| record.content.as_str())
}

fn read_results_for_paths(
    records: &[ToolResultRecord],
    paths: &[String],
) -> Option<Vec<(String, String)>> {
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let content = read_result_for_path(records, path)?;
        out.push((path.clone(), content.to_owned()));
    }
    Some(out)
}

fn run_result_for_command<'a>(records: &'a [ToolResultRecord], command: &str) -> Option<&'a str> {
    records
        .iter()
        .find(|record| {
            record.capability == Some(Capability::Run)
                && record
                    .arguments
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    == Some(command)
        })
        .map(|record| record.content.as_str())
}

fn path_argument(arguments: &serde_json::Value) -> Option<String> {
    ["filePath", "path", "file_path"]
        .iter()
        .find_map(|key| arguments.get(*key).and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
}

fn read_arguments(path: &str) -> String {
    json!({
        "filePath": path,
        "path": path,
        "file_path": path,
    })
    .to_string()
}

fn read_command_for(path: &str, mode: &FileReadMode) -> String {
    match mode {
        FileReadMode::FirstLine => format!("head -n 1 {}", shell_path(path)),
        FileReadMode::Full | FileReadMode::ExtractValue(_) | FileReadMode::Summary => {
            format!("cat {}", shell_path(path))
        }
    }
}

fn list_files_command(directory: &str) -> String {
    if directory == "." {
        String::from("find . -maxdepth 1 -type f | sed 's#^./##' | sort")
    } else {
        format!(
            "find {} -maxdepth 1 -type f | sed 's#^.*/##' | sort",
            shell_path(directory)
        )
    }
}

fn cat_many_command(paths: &[String]) -> String {
    paths
        .iter()
        .map(|path| {
            format!(
                "printf '==> %s <==\\n' {}; cat {}",
                shell_path(path),
                shell_path(path)
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn shell_path(path: &str) -> String {
    if path.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
    }) {
        path.to_owned()
    } else {
        format!("'{}'", path.replace('\'', "'\\''"))
    }
}

fn selected_paths_from_listing(
    directory: &str,
    listing: &str,
    selection: FileSelection,
) -> Vec<String> {
    let mut entries = listing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_start_matches("./").to_owned())
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();

    let selected = match selection {
        FileSelection::First => entries.into_iter().take(1).collect(),
        FileSelection::Last => entries.into_iter().rev().take(1).collect(),
        FileSelection::All => entries,
    };

    selected
        .into_iter()
        .map(|entry: String| {
            if directory == "." || entry.contains('/') {
                entry
            } else {
                format!("{directory}/{entry}")
            }
        })
        .collect()
}

fn file_read_final_answer(mode: &FileReadMode, files: &[(String, String)]) -> String {
    match mode {
        FileReadMode::FirstLine => {
            let (path, content) = &files[0];
            let first = content.lines().next().unwrap_or_default();
            format!("First line of `{path}`:\n\n```text\n{first}\n```")
        }
        FileReadMode::ExtractValue(key) => {
            let (path, content) = &files[0];
            let value =
                extract_jsonish_value(content, key).unwrap_or_else(|| content.trim().to_owned());
            format!("Value of `{key}` in `{path}`: {value}")
        }
        FileReadMode::Summary => {
            let mut lines = vec![format!("Read {} file(s):", files.len())];
            for (path, content) in files {
                let summary = content.lines().next().unwrap_or_default().trim();
                lines.push(format!("- `{path}`: {summary}"));
            }
            lines.join("\n")
        }
        FileReadMode::Full => {
            let (path, content) = &files[0];
            format!(
                "Contents of `{path}`:\n\n```text\n{}\n```",
                content.trim_end()
            )
        }
    }
}

fn extract_jsonish_value(content: &str, key: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(found) = value.get(key) {
            return Some(match found {
                serde_json::Value::String(text) => text.clone(),
                other => other.to_string(),
            });
        }
    }
    let quoted_key = format!("\"{key}\"");
    let start = content.find(&quoted_key)?;
    let after_key = &content[start + quoted_key.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    if let Some(rest) = after_colon.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_owned());
    }
    Some(
        after_colon
            .split(|character: char| {
                character == ',' || character == '}' || character.is_whitespace()
            })
            .next()
            .unwrap_or_default()
            .to_owned(),
    )
}

fn tool_for<'a>(tool_names: &[&'a str], capability: Capability) -> Option<&'a str> {
    tool_names
        .iter()
        .copied()
        .find(|name| tool_capability(name) == Some(capability))
}

fn plan_one(tool: &str, arguments: String) -> AgenticPlan {
    AgenticPlan::ToolCalls(vec![PlannedToolCall {
        tool: tool.to_owned(),
        arguments,
    }])
}
