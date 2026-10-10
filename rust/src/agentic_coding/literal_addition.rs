//! Mirrors the ordinary Read/full-content Write/independent Read additive protocol.
use crate::agentic_coding::code_task::{render_seeded_change, render_seeded_outcome};
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, record};
use crate::agentic_coding::general_planner::owned_additive_literal_frame;
use crate::agentic_coding::planner::{
    AgenticPlan, Capability, plan_one, tool_for, write_arguments,
};
use crate::agentic_coding::progress::Progress;
use crate::agentic_coding::workspace_change::read_arguments;
use crate::protocol::ChatMessage;

fn refused(
    task: &str,
    target: &str,
    disposition: FinalDisposition,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    record(
        AgenticPlan::Final(
            render_seeded_outcome("file-addition-unverified", task, target)
                .unwrap_or_else(|| target.to_owned()),
        ),
        disposition,
        "literal-addition-unverified",
        result,
    )
}

/// Mirrors planLiteralAdditionStep; no undeclared append field or provider capability is emitted.
pub(super) fn plan_literal_addition_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let (target, content, position) = owned_additive_literal_frame(task)?;
    let Some(at_end) = position else {
        return Some(record(
            AgenticPlan::Final(
                render_seeded_outcome("file-addition-position-unknown", task, &target)
                    .unwrap_or_else(|| target.clone()),
            ),
            FinalDisposition::Gap,
            "literal-addition-position-unknown",
            result,
        ));
    };
    let (Some(read), Some(write)) = (
        tool_for(tool_names, Capability::Read),
        tool_for(tool_names, Capability::Write),
    ) else {
        return Some(refused(task, &target, FinalDisposition::Gap, result));
    };
    let progress = Progress::scan(messages);
    let written = progress.successful_write_content_for(&target);
    let before = if written.is_none() {
        progress.source_read_for(&target)
    } else {
        progress.source_read_before_latest_write_for(&target)
    };
    let Some(before) = before else {
        return Some(if written.is_none() {
            plan_one(read, read_arguments(&target))
        } else {
            refused(task, &target, FinalDisposition::Gap, result)
        });
    };
    let absent = before.absent;
    if !absent && (before.error.is_some() || !before.complete || before.source.is_none()) {
        return Some(refused(task, &target, FinalDisposition::Gap, result));
    }
    let source = if absent {
        ""
    } else {
        before.source.as_deref().unwrap_or_default()
    };
    let expected = if source.is_empty() {
        format!("{content}\n")
    } else if !at_end {
        format!("{content}\n{source}")
    } else {
        format!(
            "{source}{}{content}\n",
            if source.ends_with('\n') { "" } else { "\n" }
        )
    };
    let Some(written) = written else {
        return Some(if progress.attempted_write_for(&target) {
            refused(task, &target, FinalDisposition::Failure, result)
        } else {
            plan_one(write, write_arguments(&target, &expected))
        });
    };
    if written != expected {
        return Some(refused(task, &target, FinalDisposition::Gap, result));
    }
    if !progress
        .attempts_after_latest_write(&target)
        .is_some_and(|attempts| {
            attempts
                .iter()
                .any(|attempt| attempt.capability == Capability::Read)
        })
    {
        return Some(plan_one(read, read_arguments(&target)));
    }
    let after = progress.source_read_for(&target);
    if !after.is_some_and(|after| {
        after.error.is_none()
            && after.complete
            && after.source.as_deref() == Some(expected.as_str())
    }) {
        return Some(refused(task, &target, FinalDisposition::Gap, result));
    }
    let intent = if at_end {
        "file_edit_position_end"
    } else {
        "file_edit_position_start"
    };
    Some(record(
        AgenticPlan::Final(
            render_seeded_change(intent, task, &target, &[("{new}", &content)])
                .unwrap_or_else(|| target.to_owned()),
        ),
        FinalDisposition::Finding,
        "literal-addition-observed",
        result,
    ))
}
