//! Client-observed preimages authorize generated-source replacement.
//! Twin: js/agentic/code_task/target_guard.mjs.
use super::super::code_artifact::{source_from_agent_read_result, source_from_read_result};
use super::super::final_result::{FinalDisposition, FinalResult, record};
use super::super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::super::{tool_result, workspace_change};
use super::GeneratedSource;
use crate::protocol::ChatMessage;
use crate::seed;
pub(super) fn guarded_source_step(
    task: &str,
    artifact: &GeneratedSource,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let read = tool_for(tools, Capability::Read);
    let response = |intent: &str, values: &[(&str, &str)]| {
        seed::render_response(intent, crate::language::detect(task).slug(), values)
            .or_else(|| seed::render_response(intent, "en", values))
            .unwrap_or_default()
    };
    let Some(raw) =
        workspace_change::result_for_path(messages, Capability::Read, &artifact.path, None)
    else {
        return Some(match read {
            Some(read) => plan_one(read, workspace_change::read_arguments(&artifact.path)),
            None => record(
                AgenticPlan::Final(response(
                    "general_plan_unverified",
                    &[
                        ("target", &artifact.path),
                        ("command", &format!("read {}", artifact.path)),
                    ],
                )),
                FinalDisposition::Gap,
                "generated_source_preimage_unobserved",
                result,
            ),
        });
    };
    let envelope = source_from_agent_read_result(&raw);
    let failure = if envelope.is_none() {
        tool_result::failure_message(&raw, false, true)
    } else {
        None
    };
    if let Some(failure) = failure {
        if seed::lexicon().mentions_role_raw(
            "filesystem-absent-result",
            &crate::engine::normalize_prompt(&failure),
        ) {
            return None;
        }
        return Some(record(
            AgenticPlan::Final(tool_result::render_failure(
                read.unwrap_or("read"),
                &raw,
                task,
            )),
            FinalDisposition::Failure,
            "generated_source_preimage_failed",
            result,
        ));
    }
    let source = envelope.unwrap_or_else(|| source_from_read_result(&raw));
    if source.is_empty()
        || source == artifact.content
        || seed::lexicon().mentions_role(
            "file_overwrite_consent",
            &crate::engine::normalize_prompt(task),
        )
    {
        return None;
    }
    Some(record(
        AgenticPlan::Final(response(
            "general_change_existing_file_kept",
            &[("path", &artifact.path)],
        )),
        FinalDisposition::Gap,
        "generated_source_existing_file_kept",
        result,
    ))
}
