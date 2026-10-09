//! Source-owned mixed actions reuse obligation nodes and request-local replay.
use super::literal_request::{
    LiteralWriteContract, instruction_view, owns_instruction_span, parse_write_contract,
};
use super::{GeneralPlanMode, compose_general_change_plan};
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, ResolvedPlan, record};
use crate::agentic_coding::planner::AgenticPlan;
use crate::agentic_coding::request_sequence::plan_bound_request_steps;
use crate::agentic_coding::shell_command_policy::sentences;
use crate::agentic_coding::write_request::{
    compose_edit_clauses, first_action_cue_end, first_action_cue_start, preferred_binding, tokens,
};
use crate::normal_markov::{quote_fault, quoted_segment_spans};
use crate::obligation_ledger::{ObligationExpectation, ObligationNode};
use crate::protocol::ChatMessage;
use std::ops::Range;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoalKind {
    LiteralFile,
    SourceEdit,
    Unsupported,
}
struct Goal {
    node: ObligationNode,
    kind: GoalKind,
}
fn grammar_tail(text: &str) -> bool {
    text.chars()
        .all(|character| character.is_whitespace() || ".!?。！？।;；".contains(character))
}
fn closed_payload(request: &str, contract: &LiteralWriteContract) -> Option<Range<usize>> {
    quoted_segment_spans(request)
        .into_iter()
        .find(|span| {
            span.start >= contract.payload.start
                && request
                    .get(contract.payload.start..span.start)
                    .is_some_and(|prefix| {
                        prefix
                            .chars()
                            .all(|character| character.is_whitespace() || character == ':')
                    })
        })
        .map(|span| span.start..span.end)
}
fn literal_tail(request: &str, contract: &LiteralWriteContract) -> bool {
    let end = closed_payload(request, contract).map_or(contract.payload.end, |span| span.end);
    if request.get(end..).is_some_and(grammar_tail) {
        return true;
    }
    let words = tokens(request);
    let Some(binding) = preferred_binding(&words) else {
        return false;
    };
    let Some(target) = words.get(binding.index) else {
        return false;
    };
    binding.path == contract.target
        && binding.cue_precedes
        && target.start == contract.target_span.start
        && target.end == contract.target_span.end
        && binding.cue_start >= end
        && binding.cue_end <= target.start
        && request
            .get(end..binding.cue_start)
            .is_some_and(grammar_tail)
        && request
            .get(binding.cue_end..target.start)
            .is_some_and(grammar_tail)
        && request.get(target.end..).is_some_and(grammar_tail)
}
fn literal_write_ownership(request: &str) -> Option<LiteralWriteContract> {
    let contract = parse_write_contract(request)?;
    if !owns_instruction_span(&contract, &contract.target_span)
        || compose_general_change_plan(request)?.mode != GeneralPlanMode::LiteralFile
    {
        return None;
    }
    let header = instruction_view(request, &contract)?;
    let words = tokens(&header);
    let start = first_action_cue_start(&words)?;
    let end = first_action_cue_end(&words)?;
    let lexicon = crate::seed::lexicon();
    (lexicon.mentions_role(
        "file_whole_write_action",
        &crate::engine::normalize_prompt(header.get(start..end)?),
    ) || lexicon.mentions_role(
        "file_overwrite_consent",
        &crate::engine::normalize_prompt(&header),
    ))
    .then_some(contract)
}
pub(in crate::agentic_coding) fn instruction_view_for_request(request: &str) -> String {
    literal_write_ownership(request)
        .and_then(|contract| instruction_view(request, &contract))
        .unwrap_or_else(|| request.to_owned())
}
fn goal_ledger(request: &str) -> Option<Vec<Goal>> {
    let contract = literal_write_ownership(request);
    let mut view = if contract
        .as_ref()
        .and_then(|contract| closed_payload(request, contract))
        .is_none()
    {
        instruction_view_for_request(request)
    } else {
        request.to_owned()
    };
    for span in quoted_segment_spans(request) {
        view.get(span.start..span.end)?;
        view.replace_range(span.start..span.end, &" ".repeat(span.end - span.start));
    }
    let mut goals = Vec::new();
    for sentence in sentences(&view) {
        let raw = request.get(sentence.span.clone())?;
        let clause = raw.trim();
        let start = sentence.span.start + raw.find(clause)?;
        let literal = literal_write_ownership(clause);
        let edit = literal
            .as_ref()
            .map_or_else(|| compose_edit_clauses(clause), |_| None);
        let complete_literal = literal
            .as_ref()
            .is_some_and(|contract| literal_tail(clause, contract));
        let kind = if complete_literal {
            GoalKind::LiteralFile
        } else if edit.as_ref().is_some_and(|edit| edit.spans.is_some()) {
            GoalKind::SourceEdit
        } else {
            GoalKind::Unsupported
        };
        let mut node = ObligationNode::build(clause, 0);
        node.span = (start, start + clause.len());
        node.expectation =
            if let Some(contract) = literal.as_ref().filter(|_| kind == GoalKind::LiteralFile) {
                ObligationExpectation::FileBytes {
                    path: contract.target.clone(),
                    sha256: None,
                }
            } else {
                ObligationExpectation::Underivable {
                    reason: if kind == GoalKind::SourceEdit {
                        "source-edit-preimage-required"
                    } else {
                        "no_artifact_in_clause"
                    }
                    .to_owned(),
                }
            };
        goals.push(Goal { node, kind });
    }
    if goals.is_empty() || goals.len() == 1 && goals[0].kind != GoalKind::Unsupported {
        return None;
    }
    (contract
        .as_ref()
        .is_some_and(|contract| contract.target_span.start >= contract.payload.end)
        || goals
            .iter()
            .any(|goal| literal_write_ownership(&goal.node.clause).is_some()))
    .then_some(goals)
}
fn goal_gap(goal: &Goal, result: &mut Option<FinalResult>) -> AgenticPlan {
    record(
        AgenticPlan::Final(crate::agentic_coding::task_obligations::gap_answer(
            &goal.node.node_id,
            &goal.node.clause,
            goal.node.span,
            "no_artifact_in_clause",
        )),
        FinalDisposition::Gap,
        "owned-goal-missing-contract",
        result,
    )
}
/// Literal-only scheduling stays authoritative; supported mixed actions retain all goals.
pub(in crate::agentic_coding) fn plan_owned_goal_step(
    request: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan_for: fn(&[ChatMessage], &[&str]) -> Option<ResolvedPlan>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let goals = goal_ledger(request)?;
    if quote_fault(request).is_some()
        || crate::agentic_coding::quote_nesting::nested_quote_fault(request).is_some()
    {
        return None;
    }
    if let Some(index) = goals
        .iter()
        .position(|goal| goal.kind == GoalKind::Unsupported)
    {
        let missing = &goals[index];
        if literal_write_ownership(&missing.node.clause).is_some() {
            return Some(goal_gap(missing, result));
        }
        if index == 0 {
            return literal_write_ownership(request)
                .filter(|contract| {
                    contract.target_span.start >= contract.payload.end
                        && first_action_cue_start(&tokens(&goals[index].node.clause)).is_some()
                })
                .map(|_| goal_gap(&goals[index], result));
        }
        let parts: Vec<_> = goals[..index]
            .iter()
            .map(|goal| goal.node.clause.clone())
            .collect();
        let plan = plan_bound_request_steps(&parts, messages, tool_names, plan_for, result)?;
        return Some(
            if matches!(&plan, AgenticPlan::Final(_))
                && ResolvedPlan::new(plan.clone(), result.clone()).can_deliver()
            {
                goal_gap(missing, result)
            } else {
                plan
            },
        );
    }
    if !goals.iter().any(|goal| goal.kind == GoalKind::SourceEdit) {
        return None;
    }
    let parts: Vec<_> = goals.iter().map(|goal| goal.node.clause.clone()).collect();
    plan_bound_request_steps(&parts, messages, tool_names, plan_for, result)
}
