//! Scratch design: intended sibling of general_planner::literal_request; UTF-8 source spans.
use super::literal_request::{LiteralWriteContract, instruction_view, parse_write_contract};
use super::{GeneralPlanMode, compose_general_change_plan};
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, ResolvedPlan, record};
use crate::agentic_coding::planner::AgenticPlan;
use crate::agentic_coding::shell_command_policy::sentences;
use crate::agentic_coding::write_request::{
    compose_edit_clauses, first_action_cue_end, first_action_cue_start, tokens,
};
use crate::normal_markov::{quote_fault, quoted_segment_spans};
use crate::protocol::{ChatMessage, MessageContent};
use std::ops::Range;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoalKind {
    LiteralFile,
    SourceEdit,
    Unsupported,
}
struct Goal {
    clause: String,
    span: Range<usize>,
    kind: GoalKind,
    target: Option<String>,
    expected: Option<String>,
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
fn literal_write_ownership(request: &str) -> Option<LiteralWriteContract> {
    let contract = parse_write_contract(request)?;
    if compose_general_change_plan(request)?.mode != GeneralPlanMode::LiteralFile {
        return None;
    }
    let header = instruction_view(request, &contract)?;
    let words = tokens(&header);
    let start = first_action_cue_start(&words)?;
    let end = first_action_cue_end(&words)?;
    let action = crate::engine::normalize_prompt(header.get(start..end)?);
    let lexicon = crate::seed::lexicon();
    (lexicon.mentions_role("file_whole_write_action", &action)
        || lexicon.mentions_role(
            "file_overwrite_consent",
            &crate::engine::normalize_prompt(&header),
        ))
    .then_some(contract)
}
fn goal_ledger(request: &str) -> Option<Vec<Goal>> {
    let contract = literal_write_ownership(request);
    let first_literal = contract
        .as_ref()
        .and_then(|contract| closed_payload(request, contract));
    let mut view = if first_literal.is_none() {
        contract
            .as_ref()
            .and_then(|contract| instruction_view(request, contract))
            .unwrap_or_else(|| request.to_owned())
    } else {
        request.to_owned()
    };
    for span in quoted_segment_spans(request) {
        view.get(span.start..span.end)?;
        view.replace_range(span.start..span.end, &" ".repeat(span.end - span.start));
    }
    let mut goals = Vec::new();
    for sentence in sentences(&view) {
        let clause = request.get(sentence.span.clone())?.trim().to_owned();
        let literal = literal_write_ownership(&clause);
        let edit = if literal.is_none() {
            compose_edit_clauses(&clause)
        } else {
            None
        };
        let complete_literal = literal.as_ref().is_some_and(|contract| {
            let end =
                closed_payload(&clause, contract).map_or(contract.payload.end, |span| span.end);
            clause.get(end..).is_some_and(grammar_tail)
        });
        let kind = if complete_literal {
            GoalKind::LiteralFile
        } else if edit.as_ref().is_some_and(|edit| edit.spans.is_some()) {
            GoalKind::SourceEdit
        } else {
            GoalKind::Unsupported
        };
        let target = literal
            .as_ref()
            .map(|contract| contract.target.clone())
            .or_else(|| edit.as_ref().map(|edit| edit.edit.0.clone()));
        let expected = literal.as_ref().map(|contract| contract.content.clone());
        goals.push(Goal {
            clause,
            span: sentence.span,
            kind,
            target,
            expected,
        });
    }
    if goals.len() == 1 && goals[0].kind != GoalKind::Unsupported {
        return None;
    }
    goals
        .iter()
        .any(|goal| literal_write_ownership(&goal.clause).is_some())
        .then_some(goals)
}
fn goal_gap(goal: &Goal, result: &mut Option<FinalResult>) -> AgenticPlan {
    let answer = crate::agentic_coding::task_obligations::gap_answer(
        &crate::engine::stable_id("obligation", &goal.clause),
        &goal.clause,
        (goal.span.start, goal.span.end),
        "no_artifact_in_clause",
    );
    record(
        AgenticPlan::Final(answer),
        FinalDisposition::Gap,
        "owned-goal-missing-contract",
        result,
    )
}
// Preserve the existing literal-only/underivable ledger; this bridge sequences supported mixed actions.
pub(super) fn plan_owned_goal_step(
    request: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan_for: fn(&[ChatMessage], &[&str]) -> Option<ResolvedPlan>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let goals = goal_ledger(request)?;
    if quote_fault(request).is_some() {
        return None;
    }
    if let Some(missing) = goals.iter().find(|goal| goal.kind == GoalKind::Unsupported) {
        if literal_write_ownership(&missing.clause).is_some() {
            return Some(goal_gap(missing, result));
        }
        if goals.first()?.kind == GoalKind::Unsupported {
            return None;
        }
    } else if !goals.iter().any(|goal| goal.kind == GoalKind::SourceEdit) {
        return None;
    }
    replay_owned_goals(&goals, messages, tool_names, plan_for, result)
}
/// The conversation with its latest user turn asking `part` alone.
fn with_request(messages: &[ChatMessage], part: &str) -> Vec<ChatMessage> {
    let latest = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"));
    messages
        .iter()
        .enumerate()
        .map(|(index, message)| {
            let mut message = message.clone();
            if Some(index) == latest {
                message.content = MessageContent::Text(part.to_owned());
            }
            message
        })
        .collect()
}

fn replay_owned_goals(
    goals: &[Goal],
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan_for: fn(&[ChatMessage], &[&str]) -> Option<ResolvedPlan>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let (base, exchanges) = turn_exchanges(messages);
    let mut taken = 0;
    let mut answers = Vec::new();
    for goal in goals {
        if goal.kind == GoalKind::Unsupported {
            return Some(goal_gap(goal, result));
        }
        let part = &goal.clause;
        // A step is planned over the tool calls made for it alone: replayed
        // one exchange at a time until it answers or asks for its next call.
        let mut own: Vec<ChatMessage> = base.to_vec();
        loop {
            let resolved = plan_for(&with_request(&own, part), tool_names)?;
            if matches!(&resolved.plan, AgenticPlan::Final(_)) && !resolved.can_deliver() {
                return Some(resolved.into_plan(result));
            }
            match resolved.plan {
                AgenticPlan::Final(answer) => {
                    answers.push(answer);
                    break;
                }
                plan @ AgenticPlan::ToolCalls(_) => {
                    let Some(exchange) = exchanges.get(taken) else {
                        return Some(plan);
                    };
                    own.extend_from_slice(exchange);
                    taken += 1;
                }
            }
        }
    }
    Some(record(
        AgenticPlan::Final(answers.join("\n\n")),
        FinalDisposition::Finding,
        "request_sequence_verified",
        result,
    ))
}

/// The conversation up to its latest user turn, and the tool exchanges after
/// it, each an assistant message with the messages that answer it (mirrors
/// `turnExchanges`).
fn turn_exchanges(messages: &[ChatMessage]) -> (&[ChatMessage], Vec<&[ChatMessage]>) {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |latest| latest + 1);
    let mut exchanges: Vec<&[ChatMessage]> = Vec::new();
    let mut from = start;
    for (index, message) in messages.iter().enumerate().skip(start) {
        let opens = message.role.eq_ignore_ascii_case("assistant");
        if opens && index > from {
            exchanges.push(&messages[from..index]);
            from = index;
        }
    }
    if from < messages.len() {
        exchanges.push(&messages[from..]);
    }
    (&messages[..start], exchanges)
}
