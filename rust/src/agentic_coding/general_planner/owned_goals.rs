//! Source-owned mixed actions reuse obligation nodes and request-local replay.
use super::literal_request::{
    LiteralWriteContract, instruction_view, owns_instruction_span, parse_write_contract,
};
use super::{GeneralPlanMode, compose_general_change_plan};
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, ResolvedPlan, record};
use crate::agentic_coding::planner::AgenticPlan;
use crate::agentic_coding::positional_edit::unquoted_path_tokens;
use crate::agentic_coding::request_sequence::plan_bound_request_steps;
use crate::agentic_coding::shell_command_policy::sentences;
use crate::agentic_coding::write_request::{
    bare_surfaces, clean_path_token, compose_edit_clauses, first_action_cue_end,
    first_action_cue_start, first_raw_prefix_lead_end, looks_like_file_path, pinned_first_line,
    preferred_binding, tokens,
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
pub(in crate::agentic_coding) fn pinned_line_need(
    request: &str,
    content: &str,
    payload_end: usize,
) -> Option<serde_json::Value> {
    let need = source_pinned_line_need(request, payload_end, false)?;
    (content.split('\n').next() == need["expected"].as_str()).then_some(need)
}
fn source_pinned_line_need(
    request: &str,
    payload_end: usize,
    allow_trailing: bool,
) -> Option<serde_json::Value> {
    let tail = request.get(payload_end..)?;
    let (lead_start, lead_end) =
        first_raw_prefix_lead_end(tail, "file_leading_line_constraint_lead")?;
    let grammar = |text: &str| {
        text.chars().all(|character| {
            matches!(
                character,
                ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}'
            ) || ".!?。！？।;；".contains(character)
        })
    };
    if !grammar(tail.get(..lead_start)?) {
        return None;
    }
    let remainder = tail.get(lead_end..)?;
    let raw = remainder.trim_start_matches(|character: char| {
        matches!(
            character,
            ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}' | ':' | '-' | '—' | '–'
        )
    });
    let delimiter = raw.chars().next()?;
    if !matches!(delimiter, '`' | '"' | '\'') {
        return None;
    }
    let close = raw.get(1..)?.find(delimiter)? + 1;
    if !allow_trailing && !grammar(raw.get(close + 1..)?) {
        return None;
    }
    let line = raw.get(1..close)?;
    if line.is_empty()
        || line.contains(['\n', '\r'])
        || pinned_first_line(tail).as_deref() != Some(line)
    {
        return None;
    }
    let start = payload_end + lead_end + remainder.len() - raw.len() + 1;
    let trailing = raw
        .get(close + 1..)?
        .chars()
        .take_while(|character| {
            matches!(
                character,
                ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}'
            ) || ".!?。！？।;；".contains(*character)
        })
        .map(char::len_utf8)
        .sum::<usize>();
    let end = start + line.len() + 1 + trailing;
    Some(
        serde_json::json!({"kind":"file_first_line","unit":"utf8", "span":[payload_end,end],
        "literalSpan":[start,start+line.len()],"expected":line,"condition":"composed-first-line-equals"}),
    )
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
    if compose_general_change_plan(request)
        .is_some_and(|plan| pinned_line_need(request, &plan.content, end).is_some())
    {
        return true;
    }
    let words = tokens(request);
    let Some(binding) = preferred_binding(&words) else {
        return false;
    };
    let Some(target) = words.get(binding.index) else {
        return false;
    };
    let suffix_owned = preferred_binding(&words[binding.index..]).is_some_and(|suffix| {
        !suffix.cue_precedes
            && suffix.path == contract.target
            && suffix.cue_start >= target.end
            && request
                .get(target.end..suffix.cue_start)
                .is_some_and(grammar_tail)
            && request
                .get(suffix.cue_start..suffix.cue_end)
                .is_some_and(|text| {
                    bare_surfaces("file_declared_noun").contains(
                        &text
                            .trim_end_matches(|character: char| {
                                character.is_whitespace() || ".!?。！？।;；".contains(character)
                            })
                            .to_lowercase(),
                    )
                })
            && request.get(suffix.cue_end..).is_some_and(grammar_tail)
    });
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
        && (request.get(target.end..).is_some_and(grammar_tail) || suffix_owned)
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
    if let Some(owner) = contract.as_ref()
        && let Some(closed) = closed_payload(request, owner)
        && let Some(need) = source_pinned_line_need(request, closed.end, true)
    {
        if owner.content.split('\n').next() != need["expected"].as_str() {
            let mut node = ObligationNode::build(request, 0);
            node.span = (0, request.len());
            node.expectation = ObligationExpectation::Underivable {
                reason: "first-line-conflicts-authoritative-payload".to_owned(),
            };
            return Some(vec![Goal {
                node,
                kind: GoalKind::Unsupported,
            }]);
        }
        if need["span"][1].as_u64() == Some(request.len() as u64)
            && !attributed_action_prefix(request)
            && contract_action_prologue(request, owner)
        {
            return None;
        }
    }
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
        let complete_literal = literal.as_ref().is_some_and(|contract| {
            literal_tail(clause, contract) && contract_action_prologue(clause, contract)
        }) && !attributed_action_prefix(clause);
        let kind = if complete_literal {
            GoalKind::LiteralFile
        } else if edit.as_ref().is_some_and(|edit| edit.spans.is_some()) {
            GoalKind::SourceEdit
        } else {
            GoalKind::Unsupported
        };
        let mut node = ObligationNode::build(clause, 0);
        node.span = (start, start + clause.len());
        node.expectation = literal
            .as_ref()
            .filter(|_| kind == GoalKind::LiteralFile)
            .map_or_else(
                || ObligationExpectation::Underivable {
                    reason: if kind == GoalKind::SourceEdit {
                        "source-edit-preimage-required"
                    } else {
                        "no_artifact_in_clause"
                    }
                    .to_owned(),
                },
                |contract| ObligationExpectation::FileBytes {
                    path: contract.target.clone(),
                    sha256: None,
                },
            );
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
    let declared_goals = goal_ledger(request)?;
    let has_context = declared_goals
        .iter()
        .any(|goal| source_context_declaration(&goal.node.clause));
    let goals = declared_goals
        .into_iter()
        .filter(|goal| !source_context_declaration(&goal.node.clause))
        .collect::<Vec<_>>();
    if goals.is_empty() {
        return None;
    }
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
        if has_context {
            return Some(goal_gap(missing, result));
        }
        if literal_write_ownership(&missing.node.clause).is_some() {
            return Some(goal_gap(missing, result));
        }
        if index == 0 {
            if has_context {
                return Some(goal_gap(missing, result));
            }
            return literal_write_ownership(request)
                .filter(|contract| {
                    contract.target_span.start >= contract.payload.end
                        && (first_action_cue_start(&tokens(&goals[index].node.clause)).is_some()
                            || contract.payload.start >= goals[index].node.span.1
                                && !source_context_declaration(&goals[index].node.clause)
                                && unquoted_path_tokens(&goals[index].node.clause).iter().any(
                                    |token| {
                                        let path = clean_path_token(token.text);
                                        looks_like_file_path(path)
                                    },
                                ))
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

fn context_grammar_patterns() -> &'static Vec<(regex::Regex, regex::Regex)> {
    static PATTERNS: std::sync::OnceLock<Vec<(regex::Regex, regex::Regex)>> =
        std::sync::OnceLock::new();
    PATTERNS.get_or_init(|| {
        let parsed = crate::seed::parser::parse_lino(include_str!(
            "../../../embedded/data/seed/source-context-grammar.lino"
        ));
        let Some(root) = parsed.children.first() else {
            return Vec::new();
        };
        let placeholders = regex::Regex::new(r"\{([a-z]+(?:-[a-z]+)*)\}")
            .expect("literal role placeholder grammar");
        let mut patterns = Vec::new();
        for language in root.children.iter().filter(|node| node.name == "language") {
            let roles = language
                .children
                .iter()
                .filter(|node| node.name == "role")
                .map(|role| {
                    (
                        role.id.as_str(),
                        role.children
                            .iter()
                            .filter(|node| node.name == "form")
                            .map(|form| form.id.as_str())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            for pattern in language
                .children
                .iter()
                .filter(|node| node.name == "pattern")
            {
                let mut expressions = Vec::new();
                let mut missing = false;
                for side in ["prefix", "suffix"] {
                    let Some(template) = pattern
                        .children
                        .iter()
                        .find(|node| node.name == side)
                        .map(|node| node.id.as_str())
                    else {
                        missing = true;
                        break;
                    };
                    let expanded =
                        placeholders.replace_all(template, |captures: &regex::Captures<'_>| {
                            let Some(forms) = roles
                                .get(captures.get(1).expect("role name captured").as_str())
                                .filter(|forms| !forms.is_empty())
                            else {
                                missing = true;
                                return "(?!)".to_owned();
                            };
                            format!(
                                "(?:{})",
                                forms
                                    .iter()
                                    .map(|form| regex::escape(form))
                                    .collect::<Vec<_>>()
                                    .join("|")
                            )
                        });
                    if let Ok(expression) = regex::RegexBuilder::new(&expanded)
                        .case_insensitive(true)
                        .build()
                    {
                        expressions.push(expression);
                    } else {
                        missing = true;
                        break;
                    }
                }
                if !missing && expressions.len() == 2 {
                    let suffix = expressions.pop().expect("suffix exists");
                    let prefix = expressions.pop().expect("prefix exists");
                    patterns.push((prefix, suffix));
                }
            }
        }
        patterns
    })
}

fn source_context_whitespace_supported(text: &str) -> bool {
    text.chars().all(|character| {
        !(character.is_whitespace() || character == '\u{feff}')
            || matches!(
                character,
                ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}'
            )
    })
}

fn source_context_atom(clause: &str) -> bool {
    if !source_context_whitespace_supported(clause) {
        return false;
    }
    let paths = unquoted_path_tokens(clause)
        .into_iter()
        .filter(|token| looks_like_file_path(clean_path_token(token.text)))
        .collect::<Vec<_>>();
    if paths.len() != 1 {
        return false;
    }
    let token = &paths[0];
    let path = clean_path_token(token.text);
    if !token.text.starts_with(path) {
        return false;
    }
    let prefix = &clause[..token.start];
    let suffix = &clause[token.start + path.len()..];
    context_grammar_patterns()
        .iter()
        .any(|(prefix_pattern, suffix_pattern)| {
            prefix_pattern.is_match(prefix) && suffix_pattern.is_match(suffix)
        })
}

fn source_context_declaration(clause: &str) -> bool {
    if source_context_atom(clause) {
        return true;
    }
    let joiners = bare_surfaces("file_edit_joiner_cue");
    let boundaries = tokens(clause)
        .into_iter()
        .filter(|token| joiners.contains(&token.text.to_lowercase()))
        .collect::<Vec<_>>();
    if boundaries.is_empty() {
        return false;
    }
    let mut start = 0;
    for boundary in boundaries {
        if !source_context_atom(&clause[start..boundary.start]) {
            return false;
        }
        start = boundary.end;
    }
    source_context_atom(&clause[start..])
}

fn attributed_action_prefix(clause: &str) -> bool {
    let mut view = clause.to_owned();
    for span in quoted_segment_spans(clause) {
        if view.get(span.start..span.end).is_none() {
            return true;
        }
        view.replace_range(span.start..span.end, &" ".repeat(span.end - span.start));
    }
    first_action_cue_start(&tokens(&view)).is_some_and(|action| {
        crate::seed::lexicon().mentions_role(
            "source_attribution_marker",
            &crate::engine::normalize_prompt(&view[..action]),
        )
    })
}

fn contract_action_prologue(clause: &str, contract: &LiteralWriteContract) -> bool {
    let mut view = clause.to_owned();
    for span in quoted_segment_spans(clause) {
        if view.get(span.start..span.end).is_none() {
            return false;
        }
        view.replace_range(span.start..span.end, &" ".repeat(span.end - span.start));
    }
    let Some(action) = first_action_cue_start(&tokens(&view)) else {
        return false;
    };
    let mut start = action
        .min(contract.payload.start)
        .min(contract.target_span.start);
    let words = tokens(clause);
    if let Some(binding) = preferred_binding(&words)
        && let Some(target) = words.get(binding.index)
        && binding.path == contract.target
        && target.start == contract.target_span.start
        && target.end == contract.target_span.end
    {
        start = start.min(binding.cue_start);
    }
    let prologue = crate::engine::normalize_prompt(&view[..start]);
    prologue.is_empty()
        || [
            "politeness_cue",
            "enumeration_cue",
            "file-edit-sequence-cue",
        ]
        .iter()
        .any(|role| {
            bare_surfaces(role)
                .iter()
                .any(|surface| crate::engine::normalize_prompt(surface) == prologue)
        })
}
