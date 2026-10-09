//! Exact-field and multi-file branches of the local file-read recipe.

use super::super::final_result::{
    FinalDisposition, FinalPayloadRole, FinalResult, record, record_with_role,
};
use serde_json::json;

use super::{
    AgenticPlan, FileReadMode, FileReadTools, PlannedToolCall, ToolResultRecord,
    failed_step_answer, file_analysis_pattern, file_read_final_answer, grep_arguments,
    grep_result_for_path, read_arguments, read_command_for, read_result_for_path,
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
    tools: FileReadTools<'_>,
    records: &[ToolResultRecord],
    request: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let FileReadTools {
        read: read_tool,
        run: run_tool,
        grep: grep_tool,
        progress,
    } = tools;
    if mode == &FileReadMode::Audit
        && let Some(tool) = grep_tool
    {
        let pattern = file_analysis_pattern();
        let mut contents = Vec::with_capacity(paths.len());
        for path in paths {
            if let Some(raw) = grep_result_for_path(records, path, &pattern) {
                if let Some(failure) = failed_step_answer(path, raw, request) {
                    return AgenticPlan::Final(failure);
                }
                contents.push((path.clone(), raw.to_owned()));
            }
        }
        if contents.len() == paths.len() {
            return record_with_role(
                AgenticPlan::Final(file_read_final_answer(mode, &contents, request)),
                FinalDisposition::Finding,
                "file_read_observed",
                FinalPayloadRole::AuditReport,
                result,
            );
        }
        let calls = paths
            .iter()
            .filter(|path| grep_result_for_path(records, path, &pattern).is_none())
            .map(|path| PlannedToolCall {
                tool: tool.to_owned(),
                arguments: grep_arguments(path, &pattern),
            })
            .collect();
        return AgenticPlan::ToolCalls(calls);
    }

    // Agent's display-oriented `read` tool abbreviates a physical line after
    // 1,000 columns. A request that explicitly names an exact machine-field
    // line cannot accept that rendered view as file contents, so use the
    // shell's byte-preserving field extractor whenever the client provides one.
    let exact_run = exact_line_key(request).is_some() && run_tool.is_some();
    if !exact_run
        && let Some(answer) =
            super::source::source_read_answer(paths, mode, records, progress, request, result)
    {
        return answer;
    }
    let mut complete = true;
    let mut contents = Vec::with_capacity(paths.len());
    for path in paths {
        let command = read_command_for(path, mode);
        if exact_run && let Some(raw) = run_record_for_command(records, &command) {
            if let Some(failure) = failed_step_answer(&command, raw, request) {
                return AgenticPlan::Final(failure);
            }
            contents.push((
                path.clone(),
                super::super::tool_result::strip_transport_envelope(raw),
            ));
            continue;
        }
        if !exact_run && let Some(read) = super::source::read_observation(records, progress, path) {
            complete &= read.complete;
            contents.push((path.clone(), read.source.unwrap_or_default()));
            continue;
        }
        if !exact_run && let Some(raw) = run_record_for_command(records, &command) {
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
        return record(
            AgenticPlan::Final(file_read_final_answer(mode, &contents, request)),
            if complete {
                FinalDisposition::Finding
            } else {
                FinalDisposition::Unknown
            },
            "file_read_observed",
            result,
        );
    }

    if exact_run && let Some(tool) = run_tool {
        let calls = paths
            .iter()
            .filter(|path| run_record_for_command(records, &read_command_for(path, mode)).is_none())
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
                arguments: read_arguments(path, mode),
            })
            .collect();
        return AgenticPlan::ToolCalls(calls);
    }
    if let Some(tool) = run_tool {
        let calls = paths
            .iter()
            .filter(|path| run_record_for_command(records, &read_command_for(path, mode)).is_none())
            .map(|path| PlannedToolCall {
                tool: tool.to_owned(),
                arguments: json!({ "command": read_command_for(path, mode) }).to_string(),
            })
            .collect();
        return AgenticPlan::ToolCalls(calls);
    }

    let language = crate::language::detect(request);
    AgenticPlan::Final(
        seed::localized_response("file_read_many_unavailable", language.slug()).unwrap_or_default(),
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
