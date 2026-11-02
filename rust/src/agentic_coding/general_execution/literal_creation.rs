//! Paired target-only creation and fresh-read verification for declared literal files.
use crate::agentic_coding::code_task::render_seeded_outcome;
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, record};
use crate::agentic_coding::general_planner::{
    GeneralChangePlan, GeneralPlanMode, owned_declared_create_frame,
};
use crate::agentic_coding::planner::{
    AgenticPlan, Capability, plan_one, tool_for, write_arguments,
};
use crate::agentic_coding::progress::Progress;

pub(super) fn plan_owned_creation_step(
    plan: &GeneralChangePlan,
    progress: &Progress,
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if plan.mode != GeneralPlanMode::LiteralFile {
        return None;
    }
    let (target, content) = owned_declared_create_frame(&plan.goal)?;
    if target != plan.target || content != plan.content {
        return None;
    }
    let mut finish = |intent: &str, disposition| {
        record(
            AgenticPlan::Final(
                render_seeded_outcome(intent, &plan.goal, &plan.target)
                    .unwrap_or_else(|| plan.target.clone()),
            ),
            disposition,
            intent,
            result,
        )
    };
    let Some(written) = progress.successful_write_content_for(&plan.target) else {
        return Some(if progress.attempted_write_for(&plan.target) {
            finish(
                "coding_workspace_verification_failed",
                FinalDisposition::Failure,
            )
        } else if let Some(write) = tool_for(tool_names, Capability::Write) {
            plan_one(write, write_arguments(&plan.target, &plan.content))
        } else {
            finish("coding_workspace_written_unverified", FinalDisposition::Gap)
        });
    };
    if written != plan.content {
        return Some(finish(
            "coding_workspace_verification_failed",
            FinalDisposition::Failure,
        ));
    }
    if !progress.attempted_read_after_latest_write_for(&plan.target) {
        return Some(if let Some(read) = tool_for(tool_names, Capability::Read) {
            plan_one(read, super::read_arguments(&plan.target))
        } else {
            finish("coding_workspace_written_unverified", FinalDisposition::Gap)
        });
    }
    let observed = progress.source_read_after_latest_write_for(&plan.target);
    if observed.as_ref().is_none_or(|read| {
        read.error.is_some()
            || !read.complete
            || read.source.as_deref() != Some(plan.content.as_str())
    }) {
        return Some(finish(
            "coding_workspace_verification_failed",
            FinalDisposition::Failure,
        ));
    }
    Some(finish(
        "coding_workspace_effect_observed",
        FinalDisposition::Finding,
    ))
}
