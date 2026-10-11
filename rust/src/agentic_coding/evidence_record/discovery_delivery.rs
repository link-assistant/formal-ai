//! Delivery of qualified discovery observations without task-model promotion.
//!
//! The investigation remains Gap; the parent owns pinned headers and readback.

use super::super::final_result::{FinalDisposition, FinalResult, record};
use super::super::planner::{AgenticPlan, plan_one, write_arguments};
use super::super::progress::Progress;
use super::{DeliveryBinding, render_obligation, written_observation};

pub(super) fn recorded_discovery_gap(
    obligation: &DeliveryBinding,
    content: &Option<String>,
    progress: &Progress,
    tool_names: &[&str],
    task: &str,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if super::super::workspace_inspection::workspace_inspection_search_for_task(
        &obligation.residual,
    )
    .is_none()
        && let Some(super::super::workspace_discovery::DiscoveryStep::Observation { .. }) =
            super::super::workspace_discovery::workspace_discovery_step(
                &obligation.residual,
                &progress,
                tool_names,
                task,
            )
    {
        let answer = content
            .as_deref()
            .map(|bytes| written_observation(&obligation, bytes))
            .unwrap_or_default();
        return Some(record(
            AgenticPlan::Final(answer),
            FinalDisposition::Gap,
            "workspace_discovery_observed_gap",
            result,
        ));
    }
    None
}

pub(super) fn plan_discovery_delivery(
    obligation: &DeliveryBinding,
    progress: &Progress,
    tool_names: &[&str],
    task: &str,
    write_tool: &str,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if super::super::workspace_inspection::workspace_inspection_search_for_task(
        &obligation.residual,
    )
    .is_none()
        && let Some(discovery) = super::super::workspace_discovery::workspace_discovery_step(
            &obligation.residual,
            &progress,
            tool_names,
            task,
        )
    {
        return Some(match discovery {
            super::super::workspace_discovery::DiscoveryStep::Calls(plan) => plan,
            super::super::workspace_discovery::DiscoveryStep::Observation { kind, answer } => {
                if matches!(kind, "no_candidate" | "ambiguous" | "candidate_read") {
                    plan_one(
                        write_tool,
                        write_arguments(
                            &obligation.target,
                            &render_obligation(&obligation, &answer),
                        ),
                    )
                } else {
                    record(
                        AgenticPlan::Final(answer),
                        FinalDisposition::Gap,
                        "workspace_discovery_unqualified",
                        result,
                    )
                }
            }
        });
    }
    None
}
