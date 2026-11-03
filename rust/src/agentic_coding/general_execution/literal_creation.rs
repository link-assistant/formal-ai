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
    let mutation = progress.creation_mutation_for(&plan.target)?;
    if target != plan.target
        || content != plan.content
        || !mutation.qualified
        || !mutation.succeeded
        || mutation.content != plan.content
    {
        return None;
    }
    let command = readback_command(&plan.target);
    let state = progress.creation_readback_for(&mutation, &command);
    if !state.current || state.pending_read {
        return None;
    }
    if let Some(attempt) = state.read_attempt {
        let observed = attempt.source_observation()?;
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
    let receipt = state.command_receipt?;
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

fn creation_gap(
    plan: &GeneralChangePlan,
    result: &mut Option<FinalResult>,
    reason: &str,
) -> AgenticPlan {
    record(
        AgenticPlan::Final(
            json!({
                "schema": "workspace-creation-gap/v1", "reason": reason, "target": plan.target,
                "completed": false, "verified": false
            })
            .to_string(),
        ),
        FinalDisposition::Gap,
        "workspace-creation-gap",
        result,
    )
}
pub(super) fn plan_owned_creation_step(
    plan: &GeneralChangePlan,
    progress: &Progress,
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    use crate::agentic_coding::capability_router::{
        is_workspace_creation_tool, workspace_creation_tool,
    };
    use crate::agentic_coding::qualified_tool_observation::canonical_creation;
    if plan.mode != GeneralPlanMode::LiteralFile {
        return None;
    }
    let (target, content) = owned_declared_create_frame(&plan.goal)?;
    if target != plan.target || content != plan.content {
        return None;
    }
    let Some(mutation) = progress.creation_mutation_for(&plan.target) else {
        if progress.attempted_write_for(&plan.target) {
            let unobserved = crate::seed::agentic_message("literal-create-write-unobserved", &[]);
            let detail = progress
                .latest_failure()
                .map(|attempt| attempt.raw_receipt.as_deref().unwrap_or(&attempt.detail))
                .unwrap_or(&unobserved);
            return Some(failure(plan, result, &plan.target, detail));
        }
        if progress.research_attempts().iter().any(|attempt| {
            attempt.capability == Capability::Edit
                && attempt
                    .tool
                    .as_deref()
                    .is_some_and(is_workspace_creation_tool)
        }) {
            return Some(creation_gap(plan, result, "CreationEvidenceUnqualified"));
        }
        let Some(write) = workspace_creation_tool(tool_names) else {
            return Some(creation_gap(plan, result, "WriteToolUnavailable"));
        };
        let arguments = write_arguments(&plan.target, &plan.content);
        if tool_for(&[write], Capability::Write).is_some() {
            return Some(plan_one(write, arguments));
        }
        let representable =
            crate::protocol_responses::apply_patch_input(&arguments).and_then(|encoded| {
                canonical_creation(&encoded)
                    .map(|decoded| decoded.path == plan.target && decoded.content == plan.content)
            }) == Some(true);
        if !representable {
            return Some(creation_gap(plan, result, "CreationContentUnrepresentable"));
        }
        let preflight = progress.creation_preflight_for(&plan.target);
        if !preflight.attempted {
            return Some(if let Some(read) = tool_for(tool_names, Capability::Read) {
                plan_one(read, super::read_arguments(&plan.target))
            } else {
                creation_gap(plan, result, "ReadToolUnavailable")
            });
        }
        if !preflight
            .observation
            .is_some_and(|observation| observation.absent)
        {
            return Some(creation_gap(
                plan,
                result,
                if preflight
                    .observation
                    .is_some_and(|observation| observation.complete && observation.error.is_none())
                {
                    "CreationTargetAlreadyExists"
                } else {
                    "CreationPreflightUnverified"
                },
            ));
        }
        return Some(plan_one(write, arguments));
    };
    if !mutation.receipt_present {
        return Some(creation_gap(plan, result, "CreationReceiptPending"));
    }
    if !mutation.qualified {
        return Some(creation_gap(plan, result, "CreationEvidenceUnqualified"));
    }
    if !mutation.succeeded {
        let unobserved = crate::seed::agentic_message("literal-create-write-unobserved", &[]);
        let detail = mutation
            .attempt
            .map(|attempt| attempt.raw_receipt.as_deref().unwrap_or(&attempt.detail))
            .unwrap_or(&unobserved);
        return Some(failure(plan, result, &plan.target, detail));
    }
    if mutation.content != plan.content {
        return Some(mismatch(plan, result, &mutation.content));
    }
    let command = readback_command(&plan.target);
    let state = progress.creation_readback_for(&mutation, &command);
    if !state.current {
        if state.needs_fresh_read {
            if let Some(read) = tool_for(tool_names, Capability::Read) {
                return Some(plan_one(read, super::read_arguments(&plan.target)));
            }
            if let Some(run) = tool_for(tool_names, Capability::Run) {
                return Some(plan_one(run, json!({"command": command}).to_string()));
            }
        }
        return Some(creation_gap(plan, result, "CreationEvidenceNotCurrent"));
    }
    if owned_creation_observation(plan, progress).is_some() {
        return Some(outcome(
            plan,
            result,
            "coding_workspace_effect_observed",
            FinalDisposition::Finding,
        ));
    }
    if state.pending_read {
        return Some(creation_gap(plan, result, "CreationReadPending"));
    }
    if let Some(attempt) = state.read_attempt {
        let observed = attempt.source_observation();
        if observed.is_none_or(|read| read.error.is_some() || !read.complete) {
            return Some(failure(plan, result, &plan.target, &attempt.detail));
        }
        let observed = observed?;
        return Some(
            if observed.source.as_deref() == Some(plan.content.as_str()) {
                failure(
                    plan,
                    result,
                    &plan.target,
                    &crate::seed::agentic_message("literal-create-receipt-unbound", &[]),
                )
            } else {
                mismatch(plan, result, observed.source.as_deref().unwrap_or_default())
            },
        );
    }
    if let Some(receipt) = state.command_receipt {
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
