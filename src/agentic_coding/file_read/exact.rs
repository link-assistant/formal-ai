//! Exact-field and multi-file branches of the local file-read recipe.

use serde_json::json;

use super::{
    AgenticPlan, FileReadMode, PlannedToolCall, ToolResultRecord, failed_step_answer,
    file_read_final_answer, read_arguments, read_command_for, read_result_for_path,
    run_record_for_command,
};
use crate::seed;

/// Read every explicitly named input before reporting a composed observation.
///
/// A successful first read does not satisfy a request that names more than one
/// file. Plan every unobserved input together (or the remaining inputs on a
/// client that serializes tool calls), and only produce an answer once the
/// current turn holds all of them.
pub(super) fn plan_direct_file_reads(
    paths: &[String],
    mode: &FileReadMode,
    read_tool: Option<&str>,
    run_tool: Option<&str>,
    records: &[ToolResultRecord],
    request: &str,
) -> AgenticPlan {
    // Agent's display-oriented `read` tool abbreviates a physical line after
    // 1,000 columns. A request that explicitly names an exact machine-field
    // line cannot accept that rendered view as file contents, so use the
    // shell's byte-preserving field extractor whenever the client provides one.
    let exact_run = exact_line_key(request).is_some() && run_tool.is_some();
    let mut contents = Vec::with_capacity(paths.len());
    for path in paths {
        let command = read_command_for(path, mode);
        if exact_run
            && let Some(raw) = run_record_for_command(records, &command)
        {
            if let Some(failure) = failed_step_answer(&command, raw, request) {
                return AgenticPlan::Final(failure);
            }
            contents.push((
                path.clone(),
                super::super::tool_result::strip_transport_envelope(raw),
            ));
            continue;
        }
        if !exact_run
            && let Some(raw) = read_result_for_path(records, path)
        {
            if let Some(failure) = failed_step_answer(path, raw, request) {
                return AgenticPlan::Final(failure);
            }
            contents.push((
                path.clone(),
                super::super::code_artifact::source_from_read_result(raw),
            ));
            continue;
        }
        if !exact_run
            && let Some(raw) = run_record_for_command(records, &command)
        {
            if let Some(failure) = failed_step_answer(&command, raw, request) {
                return AgenticPlan::Final(failure);
            }
            contents.push((
                path.clone(),
                super::super::tool_result::strip_transport_envelope(raw),
            ));
        }
    }
    if contents.len() == paths.len() {
        return AgenticPlan::Final(file_read_final_answer(mode, &contents));
    }

    if exact_run
        && let Some(tool) = run_tool
    {
        let calls = paths
            .iter()
            .filter(|path| {
                run_record_for_command(records, &read_command_for(path, mode)).is_none()
            })
            .map(|path| PlannedToolCall {
                tool: tool.to_owned(),
                arguments: json!({ "command": read_command_for(path, mode) }).to_string(),
            })
            .collect();
        return AgenticPlan::ToolCalls(calls);
    }
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
    if let Some(tool) = run_tool {
        let calls = paths
            .iter()
            .filter(|path| {
                run_record_for_command(records, &read_command_for(path, mode)).is_none()
            })
            .map(|path| PlannedToolCall {
                tool: tool.to_owned(),
                arguments: json!({ "command": read_command_for(path, mode) }).to_string(),
            })
            .collect();
        return AgenticPlan::ToolCalls(calls);
    }

    let language = crate::language::detect(request);
    AgenticPlan::Final(
        seed::localized_response("file_read_many_unavailable", language.slug())
            .unwrap_or_default(),
    )
}

/// A field prefix the request says to take from one exact line.
///
/// This recognizes the contract rather than a particular field name: callers
/// may ask for `status=`, `digest=`, or any other machine-readable value.
pub(super) fn exact_line_key(prompt: &str) -> Option<String> {
    super::sentences(prompt).into_iter().find_map(|sentence| {
        let lower = sentence.text.to_ascii_lowercase();
        let identifies_line = lower.contains("line beginning exactly")
            || lower.contains("line that begins exactly")
            || lower.contains("line starting exactly")
            || lower.contains("line that starts exactly");
        identifies_line.then_some(())?;
        sentence
            .text
            .split('`')
            .enumerate()
            .filter(|(index, _)| index % 2 == 1)
            .map(|(_, quoted)| quoted.trim())
            .find_map(|quoted| {
                let key = quoted.strip_suffix('=')?;
                (!key.is_empty()
                    && key
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_'))
                .then(|| key.to_owned())
            })
    })
}
