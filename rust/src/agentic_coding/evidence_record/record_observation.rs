//! Request-local certification of independently persisted record bytes.
use crate::agentic_coding::capability_router::tool_for;
use crate::agentic_coding::code_artifact::source_from_read_result;
use crate::agentic_coding::code_task::render_seeded_outcome;
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, record};
use crate::agentic_coding::general_planner::shell_quote;
use crate::agentic_coding::planner::{AgenticPlan, Capability, plan_one};
use crate::agentic_coding::progress::{Progress, ToolAttempt};
use crate::agentic_coding::tool_result::{
    command_argument, harness_reported_failure, observed_bytes_match, render_failure,
    reported_exit_code,
};

pub(super) enum RecordObservation {
    Verified,
    Pending,
    Mismatch,
    Failed { label: String, detail: String },
}

fn record_readback_command(target: &str) -> String {
    if !target.starts_with('-')
        && target
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
    {
        format!("cat {target}")
    } else {
        format!("cat -- {}", shell_quote(target))
    }
}

fn attempt_targets(attempt: &ToolAttempt, target: &str) -> bool {
    attempt
        .arguments
        .as_deref()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .and_then(|value| {
            ["path", "filePath", "file_path"].iter().find_map(|key| {
                value
                    .get(*key)
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
            })
        })
        .is_some_and(|path| path == target)
}

fn record_observation_outcome(
    progress: &Progress,
    target: &str,
    content: Option<&str>,
) -> RecordObservation {
    let Some(content) = content else {
        return RecordObservation::Pending;
    };
    let Some(attempts) = progress.attempts_after_latest_write(target) else {
        return RecordObservation::Pending;
    };
    let command = record_readback_command(target);
    for attempt in attempts.iter().rev() {
        let reads = attempt.capability == Capability::Read && attempt_targets(attempt, target);
        let runs = attempt.capability == Capability::Run
            && attempt
                .arguments
                .as_deref()
                .and_then(command_argument)
                .as_deref()
                == Some(command.as_str());
        let writes = attempt.capability == Capability::Write && attempt_targets(attempt, target);
        if !reads && !runs && !writes {
            continue;
        }
        if !attempt.succeeded
            || writes
            || harness_reported_failure(&attempt.detail)
            || reported_exit_code(&attempt.detail).is_some_and(|code| code != 0)
        {
            return RecordObservation::Failed {
                label: if runs {
                    command
                } else {
                    attempt.tool.clone().unwrap_or_else(|| target.to_owned())
                },
                detail: attempt.detail.clone(),
            };
        }
        let exact = if runs {
            observed_bytes_match(&attempt.detail, content)
        } else {
            source_from_read_result(&attempt.detail) == content
                || observed_bytes_match(&attempt.detail, content)
        };
        return if exact {
            RecordObservation::Verified
        } else {
            RecordObservation::Mismatch
        };
    }
    RecordObservation::Pending
}

pub(super) fn plan_record_readback_step(
    task: &str,
    target: &str,
    content: Option<&str>,
    progress: &Progress,
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let outcome = record_observation_outcome(progress, target, content);
    let (answer, disposition, origin) = match outcome {
        RecordObservation::Verified => return None,
        RecordObservation::Failed { label, detail } => (
            render_failure(&label, &detail, task),
            FinalDisposition::Failure,
            "record_readback_failed",
        ),
        RecordObservation::Mismatch => (
            render_seeded_outcome("coding_workspace_verification_failed", task, target)
                .unwrap_or_default(),
            FinalDisposition::Failure,
            "record_readback_mismatch",
        ),
        RecordObservation::Pending => {
            if content.is_some() {
                if let Some(tool) = tool_for(tool_names, Capability::Run) {
                    return Some(plan_one(
                        tool,
                        serde_json::json!({"command": record_readback_command(target)}).to_string(),
                    ));
                }
                if let Some(tool) = tool_for(tool_names, Capability::Read) {
                    return Some(plan_one(tool, serde_json::json!({"path": target, "filePath": target, "file_path": target}).to_string()));
                }
            }
            (
                render_seeded_outcome("coding_workspace_written_unverified", task, target)
                    .unwrap_or_default(),
                FinalDisposition::Gap,
                "record_readback_unavailable",
            )
        }
    };
    Some(record(
        AgenticPlan::Final(answer),
        disposition,
        origin,
        result,
    ))
}
