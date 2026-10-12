//! One evidence-bearing transaction shared by early literal and settled routes.
use super::AgenticPlan;
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, record};
use crate::agentic_coding::general_execution::plan_general_change_step;
use crate::agentic_coding::general_planner::compose_general_change_plan;
use crate::agentic_coding::task_obligations::{self, Obligation};
use crate::protocol::ChatMessage;

/// Every declared artifact is observed before the transaction returns success.
pub(super) fn plan_obligations_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    obligations: &[Obligation],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    match task_obligations::next_step(task, messages) {
        Some(
            task_obligations::ObligationStep::Observe(node)
            | task_obligations::ObligationStep::Decompose(node),
        ) => {
            // Observable artifact nodes re-enter the ordinary composer;
            // underivable nodes have already been recursively split by
            // `ObligationNode::build`. If no executable plan can be
            // derived, decline instead of turning an unobserved node into
            // completion prose.
            compose_general_change_plan(&node.clause)
                .map(|plan| plan_general_change_step(messages, tool_names, &plan, result))
        }
        Some(task_obligations::ObligationStep::ReportGap {
            node_id,
            clause,
            span,
            reason,
        }) => Some(record(
            AgenticPlan::Final(task_obligations::gap_answer(
                &node_id, &clause, span, &reason,
            )),
            FinalDisposition::Gap,
            "task_obligation_gap",
            result,
        )),
        None if task_obligations::successfully_discharged(task, messages) => {
            // Re-enter the final executable obligation's ordinary state
            // machine so the completion wording and verification report
            // stay identical to a single-target request.
            obligations
                .iter()
                .rev()
                .find_map(|obligation| compose_general_change_plan(&obligation.clause))
                .map(|plan| plan_general_change_step(messages, tool_names, &plan, result))
        }
        None => None,
    }
}
