//! Paired target-only creation and fresh independent readback for declared files.
use crate::agentic_coding::code_task::render_seeded_outcome;
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, record};
use crate::agentic_coding::general_planner::{
    GeneralChangePlan, GeneralPlanMode, owned_declared_create_frame,
};
use crate::agentic_coding::planner::{
    AgenticPlan, Capability, plan_one, tool_for, write_arguments,
};
use crate::agentic_coding::progress::{
    Progress, owned_creation_readback_command as readback_command,
};
use crate::agentic_coding::tool_result;
use serde_json::json;

fn outcome(
    plan: &GeneralChangePlan,
    result: &mut Option<FinalResult>,
    intent: &str,
    disposition: FinalDisposition,
) -> AgenticPlan {
    let origin = match disposition {
        FinalDisposition::Finding => "general_change_verified",
        FinalDisposition::Gap => "coding_workspace_written_unverified",
        _ => "general_change_failed",
    };
    record(
        AgenticPlan::Final(
            render_seeded_outcome(intent, &plan.goal, &plan.target)
                .unwrap_or_else(|| plan.target.clone()),
        ),
        disposition,
        origin,
        result,
    )
}

fn failure(
    plan: &GeneralChangePlan,
    result: &mut Option<FinalResult>,
    label: &str,
    detail: &str,
) -> AgenticPlan {
    record(
        AgenticPlan::Final(tool_result::render_failure(label, detail, &plan.goal)),
        FinalDisposition::Failure,
        "general_change_failed",
        result,
    )
}

fn mismatch(
    plan: &GeneralChangePlan,
    result: &mut Option<FinalResult>,
    observed: &str,
) -> AgenticPlan {
    record(
        AgenticPlan::Final(super::general_plan_mismatch(plan, observed)),
        FinalDisposition::Failure,
        "general_change_failed",
        result,
    )
}

pub(in crate::agentic_coding) struct OwnedCreationObservation {
    pub(in crate::agentic_coding) method: &'static str,
    pub(in crate::agentic_coding) target: String,
    pub(in crate::agentic_coding) source: String,
    pub(in crate::agentic_coding) command: Option<String>,
    pub(in crate::agentic_coding) exit_code: Option<i64>,
    pub(in crate::agentic_coding) tool: String,
    pub(in crate::agentic_coding) arguments: String,
}

/// A typed file observation never manufactures a process status for a Read.
pub(in crate::agentic_coding) fn owned_creation_observation(
    plan: &GeneralChangePlan,
    progress: &Progress,
) -> Option<OwnedCreationObservation> {
    if plan.mode != GeneralPlanMode::LiteralFile {
        return None;
    }
    let (target, content) = owned_declared_create_frame(&plan.goal)?;
    if target != plan.target
        || content != plan.content
        || progress
            .successful_write_content_for(&plan.target)
            .as_deref()
            != Some(plan.content.as_str())
    {
        return None;
    }
    if progress.attempted_read_after_latest_write_for(&plan.target) {
        let attempt = progress.latest_post_write_read_attempt_for(&plan.target)?;
        let observed = progress.source_read_after_latest_write_for(&plan.target)?;
        let arguments = attempt.arguments.as_deref()?;
        let value = serde_json::from_str(arguments).ok()?;
        let frame = progress.latest_attempt_for(Capability::Read, &value)?;
        let tool = attempt.tool.as_deref()?;
        if !attempt.succeeded
            || !frame.matches_receipt(tool, &attempt.detail)
            || observed.error.is_some()
            || !observed.complete
            || observed.source.as_deref() != Some(plan.content.as_str())
        {
            return None;
        }
        return Some(OwnedCreationObservation {
            method: "read-tool",
            target: plan.target.clone(),
            source: observed.source.as_ref()?.clone(),
            command: None,
            exit_code: None,
            tool: tool.to_owned(),
            arguments: arguments.to_owned(),
        });
    }
    let command = readback_command(&plan.target);
    let receipt = progress.post_write_command_receipt_for(&plan.target, &command)?;
    let expected = format!("({})", plan.content);
    let payload = tool_result::observed_payload(receipt.raw)?;
    if !receipt.qualified
        || !receipt.succeeded
        || tool_result::reported_exit_code(receipt.raw) != Some(0)
        || tool_result::incomplete_receipt(receipt.raw)
        || tool_result::harness_reported_failure(receipt.raw)
        || payload != expected
    {
        return None;
    }
    Some(OwnedCreationObservation {
        method: "framed-run",
        target: plan.target.clone(),
        source: payload[1..payload.len() - 1].to_owned(),
        command: Some(command),
        exit_code: Some(0),
        tool: String::new(),
        arguments: String::new(),
    })
}

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
    let Some(written) = progress.successful_write_content_for(&plan.target) else {
        return Some(if progress.attempted_write_for(&plan.target) {
            let unobserved = crate::seed::agentic_message("literal_create_write_unobserved", &[]);
            let detail = progress
                .latest_failure()
                .map(|attempt| {
                    attempt
                        .raw_receipt
                        .as_deref()
                        .unwrap_or(attempt.detail.as_str())
                })
                .unwrap_or(&unobserved);
            failure(plan, result, &plan.target, detail)
        } else if let Some(write) = tool_for(tool_names, Capability::Write) {
            plan_one(write, write_arguments(&plan.target, &plan.content))
        } else {
            outcome(
                plan,
                result,
                "coding_workspace_written_unverified",
                FinalDisposition::Gap,
            )
        });
    };
    if written != plan.content {
        return Some(mismatch(plan, result, &written));
    }
    if owned_creation_observation(plan, progress).is_some() {
        return Some(outcome(
            plan,
            result,
            "coding_workspace_effect_observed",
            FinalDisposition::Finding,
        ));
    }
    if progress.attempted_read_after_latest_write_for(&plan.target) {
        let observed = progress.source_read_after_latest_write_for(&plan.target);
        let incomplete = crate::seed::agentic_message("literal_create_read_incomplete", &[]);
        let unbound = crate::seed::agentic_message("literal_create_receipt_unbound", &[]);
        let latest_detail = progress
            .latest_failure()
            .map(|attempt| attempt.detail.as_str());
        return Some(match observed {
            Some(read) if read.error.is_some() || !read.complete => failure(
                plan,
                result,
                &plan.target,
                latest_detail
                    .or(read.error.as_deref())
                    .unwrap_or(&incomplete),
            ),
            Some(read) if read.source.as_deref() == Some(plan.content.as_str()) => {
                failure(plan, result, &plan.target, &unbound)
            }
            Some(read) => mismatch(plan, result, read.source.as_deref().unwrap_or_default()),
            None => failure(
                plan,
                result,
                &plan.target,
                latest_detail.unwrap_or(&incomplete),
            ),
        });
    }
    let command = readback_command(&plan.target);
    if let Some(receipt) = progress.post_write_command_receipt_for(&plan.target, &command) {
        if !receipt.qualified
            || !receipt.succeeded
            || tool_result::reported_exit_code(receipt.raw) != Some(0)
            || tool_result::incomplete_receipt(receipt.raw)
            || tool_result::harness_reported_failure(receipt.raw)
        {
            return Some(failure(plan, result, &command, receipt.raw));
        }
        return Some(mismatch(
            plan,
            result,
            tool_result::observed_payload(receipt.raw)
                .as_deref()
                .unwrap_or_default(),
        ));
    }
    Some(if let Some(read) = tool_for(tool_names, Capability::Read) {
        plan_one(read, super::read_arguments(&plan.target))
    } else if let Some(run) = tool_for(tool_names, Capability::Run) {
        plan_one(run, json!({"command": command}).to_string())
    } else {
        outcome(
            plan,
            result,
            "coding_workspace_written_unverified",
            FinalDisposition::Gap,
        )
    })
}
