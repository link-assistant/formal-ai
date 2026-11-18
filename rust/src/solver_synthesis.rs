//! Link-native synthesis over solved sub-impulses.
//!
//! The universal solver records decomposition as links first, solves those
//! sub-impulses, then lets this module build answer candidates by composing
//! the solved sub-result links. The rules here are intentionally small and
//! structural: they consume extracted quantities, assignments, and list items
//! instead of matching whole benchmark prompts.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::calculation::evaluate_calculation;
use crate::engine::{SymbolicAnswer, answer_links_notation, stable_id};
use crate::event_log::{EventLog, build_evidence_links};
use crate::intent_formalization::IntentFormalizationCache;
use crate::least_action::{ActionCost, SolutionCandidate, rank_by_least_action};
use crate::meta_executor::{Draft, TaskOutcome, TaskTest, Verdict, run_task_test};
use crate::probability::ProbabilityStore;
use crate::solver::UniversalSolver;
use crate::solver_helpers::DecomposedSubImpulse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolvedSubImpulse {
    pub impulse_id: String,
    pub result_id: String,
    pub text: String,
    pub independent: bool,
    pub intent: String,
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ComposedCandidate {
    target: String,
    intent: String,
    answer: String,
    response_link: String,
    confidence: f32,
    /// Issue #1550 step 6: the relation of the decomposition the composer
    /// follows (`sequence`, `aggregation`), which names the parent test.
    relation: &'static str,
    /// The structural steps of the composition: its least-action cost.
    steps: u32,
    source_result_ids: Vec<String>,
    /// Only an answer assembled from child answers depends on their verdicts.
    consumes_sub_answers: bool,
}

pub fn record_solved_sub_impulse(
    log: &mut EventLog,
    sub_impulse: &DecomposedSubImpulse,
    answer: &SymbolicAnswer,
) -> SolvedSubImpulse {
    let payload = format!(
        "sub_impulse={} independent={} intent={} answer={}",
        sub_impulse.id,
        sub_impulse.independent,
        answer.intent,
        truncate_for_trace(&answer.answer, 240),
    );
    let result_id = log.append("sub_result", payload);
    SolvedSubImpulse {
        impulse_id: sub_impulse.id.clone(),
        result_id,
        text: sub_impulse.text.clone(),
        independent: sub_impulse.independent,
        intent: answer.intent.clone(),
        answer: answer.answer.clone(),
    }
}

impl UniversalSolver {
    pub(crate) fn solve_sub_impulses(
        &self,
        log: &mut EventLog,
        sub_impulses: &[DecomposedSubImpulse],
        probability_store: &ProbabilityStore,
        intent_cache: &mut IntentFormalizationCache,
    ) -> Vec<SolvedSubImpulse> {
        if self.config.max_decomposition_depth <= 1 {
            return Vec::new();
        }
        let mut sub_config = self.config;
        sub_config.max_decomposition_depth -= 1;
        let sub_solver = Self::new(sub_config);
        // Issue #1550 step 8: the breadth bound of the seed's
        // `task-decomposition` rule, the bound `meta_exec:bounds` traces.
        let breadth = crate::meta_executor::decomposition_bounds(
            crate::meta_executor::BoundOverrides::default(),
        )
        .breadth;
        sub_impulses
            .iter()
            .take(usize::try_from(breadth).unwrap_or(usize::MAX))
            .filter(|sub_impulse| !sub_impulse.text.trim().is_empty())
            .map(|sub_impulse| {
                // Issue #1550: a sub-impulse solve is a nested solve; it
                // starts no recursive procedure of its own.
                let answer = crate::meta_executor::nested(|| {
                    sub_solver.solve_with_history_probability_store_and_intent_cache(
                        &sub_impulse.text,
                        &[],
                        probability_store,
                        intent_cache,
                    )
                });
                record_solved_sub_impulse(log, sub_impulse, &answer)
            })
            .collect()
    }
}

/// Compose the solved sub-results by the structure of the decomposition.
///
/// Issue #1550 step 6: every composition runs the parent test (the leaf check
/// on the composed answer and the relation's check over the sub-results), and
/// the passing compositions are ranked by least action over their structural
/// cost. No composer carries a fixed prior.
pub fn try_synthesize_from_sub_results(
    prompt: &str,
    log: &mut EventLog,
    sub_results: &[SolvedSubImpulse],
) -> Option<SymbolicAnswer> {
    if sub_results.is_empty() {
        return None;
    }

    let mut candidates = Vec::new();
    if let Some(candidate) = compose_algebra_substitution(prompt, log, sub_results) {
        candidates.push(candidate);
    }
    if let Some(candidate) = compose_remainder_sale(prompt, log, sub_results) {
        candidates.push(candidate);
    }
    if candidates.is_empty()
        && let Some(candidate) = compose_compound_response(log, sub_results)
    {
        candidates.push(candidate);
    }
    if candidates.is_empty() {
        return None;
    }

    let children = child_outcomes(sub_results);
    let mut ranked = Vec::with_capacity(candidates.len());
    for candidate in &candidates {
        log.append(
            "candidate",
            format!(
                "composition:{} from={}",
                candidate.target,
                candidate.source_result_ids.join(",")
            ),
        );
        let verdict = parent_verdict(candidate, &children);
        log.append(
            "validation",
            format!(
                "{}({})={}",
                verdict.executed.join("+"),
                candidate.intent,
                verdict.passed
            ),
        );
        ranked.push(SolutionCandidate {
            id: candidate.target.clone(),
            solves_entire_range: verdict.passed,
            cost: ActionCost {
                steps: candidate.steps,
                code_units: u32::try_from(candidate.answer.chars().count()).unwrap_or(u32::MAX),
                compute_ms: 0,
                memory_kb: 0,
            },
        });
    }

    let order = rank_by_least_action(&ranked);
    let Some(best) = order.first() else {
        log.append(
            "composition:unresolved",
            format!("no_passing_composition=0/{}", candidates.len()),
        );
        return None;
    };
    let selected = candidates
        .iter()
        .find(|candidate| candidate.target == best.id)?;
    log.append("composition:selected", selected.target.clone());
    log.append(
        "composition:reason",
        [
            ("reason", "least_action".to_owned()),
            ("relation", selected.relation.to_owned()),
            ("steps", best.cost.steps.to_string()),
            ("code_units", best.cost.code_units.to_string()),
            ("passing", format!("{}/{}", order.len(), candidates.len())),
        ]
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(" "),
    );
    Some(finalize_composed_candidate(prompt, log, selected))
}

/// The outcome of each sub-result under its own leaf test.
fn child_outcomes(sub_results: &[SolvedSubImpulse]) -> Vec<TaskOutcome> {
    sub_results
        .iter()
        .map(|result| {
            let test = TaskTest {
                task: result.impulse_id.clone(),
                checks: crate::formalization::structural_rule_values(
                    "task-decomposition",
                    "leaf-check",
                ),
                requires: None,
            };
            let draft = Draft {
                intent: result.intent.clone(),
                content: result.answer.clone(),
                confidence: 0.0,
                evidence: Vec::new(),
            };
            let resolved = run_task_test(&test, Some(&draft), &[]).passed;
            TaskOutcome {
                task: result.impulse_id.clone(),
                status: if resolved { "resolved" } else { "unresolved" },
                reason: None,
                candidate: None,
                fallback: None,
            }
        })
        .collect()
}

/// The parent test of one composition: the leaf check on its answer and the
/// children check of its relation over the sub-results.
fn parent_verdict(candidate: &ComposedCandidate, children: &[TaskOutcome]) -> Verdict {
    let rule =
        |kind: &str| crate::formalization::structural_rule_values("task-decomposition", kind);
    let mut checks = rule("leaf-check");
    if candidate.consumes_sub_answers {
        checks.extend(
            rule("composite-check")
                .into_iter()
                .map(|check| format!("{check}:{}", candidate.relation)),
        );
    }
    let test = TaskTest {
        task: candidate.target.clone(),
        checks,
        requires: candidate
            .consumes_sub_answers
            .then(|| {
                rule(&format!("{}-requires", candidate.relation))
                    .into_iter()
                    .next()
            })
            .flatten(),
    };
    let draft = Draft {
        intent: candidate.intent.clone(),
        content: candidate.answer.clone(),
        confidence: candidate.confidence,
        evidence: Vec::new(),
    };
    run_task_test(&test, Some(&draft), children)
}

fn compose_algebra_substitution(
    prompt: &str,
    log: &mut EventLog,
    sub_results: &[SolvedSubImpulse],
) -> Option<ComposedCandidate> {
    let assignments = extract_variable_assignments(prompt);
    if assignments.is_empty() {
        return None;
    }
    let expression = extract_requested_expression(prompt)?;
    if !assignments
        .iter()
        .any(|(name, _)| expression_mentions_variable(&expression, name))
    {
        return None;
    }

    let substituted = substitute_variables(&expression, &assignments);
    let evaluation = evaluate_calculation(&substituted).ok()?;
    let assignment_trace = assignments
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(",");
    log.append("composition:substitution", assignment_trace);
    log.append(
        "composition:expression",
        format!("{} => {}", expression.trim(), substituted.trim()),
    );
    log.append(
        "composition:evaluation",
        format!("{} = {}", substituted.trim(), evaluation.formatted),
    );
    log.append("composition:engine", evaluation.engine.slug());
    if !evaluation.steps.is_empty() {
        log.append("composition:steps", evaluation.steps.len().to_string());
    }

    // A sequence: each assignment, the substitution, then each evaluation step.
    let steps = assignments.len() + 1 + evaluation.steps.len();
    Some(candidate(
        "algebra_substitution",
        evaluation.formatted,
        ("sequence", steps),
        sub_results,
        false,
    ))
}

fn compose_remainder_sale(
    prompt: &str,
    log: &mut EventLog,
    sub_results: &[SolvedSubImpulse],
) -> Option<ComposedCandidate> {
    let lower = prompt.to_ascii_lowercase();
    if !(lower.contains("remainder") && lower.contains("sell")) {
        return None;
    }
    let quantities = extract_quantities(prompt);
    if quantities.len() < 4 {
        return None;
    }
    let total = *quantities.first()?;
    let price = *quantities.last()?;
    let consumed = quantities[1..quantities.len() - 1].iter().sum::<i64>();
    let remainder = total.checked_sub(consumed)?;
    let expression = format!("({total} - {consumed}) * {price}");
    let evaluation = evaluate_calculation(&expression).ok()?;
    log.append(
        "composition:remainder",
        format!("total={total} consumed={consumed} remainder={remainder} price={price}"),
    );
    log.append(
        "composition:evaluation",
        format!("{expression} = {}", evaluation.formatted),
    );
    log.append("composition:engine", evaluation.engine.slug());
    if !evaluation.steps.is_empty() {
        log.append("composition:steps", evaluation.steps.len().to_string());
    }
    // A sequence: each consumed quantity is subtracted, then the price applies.
    let steps = quantities.len() - 1 + evaluation.steps.len();
    Some(candidate(
        "arithmetic_word_problem",
        evaluation.formatted,
        ("sequence", steps),
        sub_results,
        false,
    ))
}

fn compose_compound_response(
    log: &mut EventLog,
    sub_results: &[SolvedSubImpulse],
) -> Option<ComposedCandidate> {
    if sub_results.len() < 2 || !sub_results.iter().all(|result| result.independent) {
        return None;
    }
    let answers = sub_results
        .iter()
        .map(|result| result.answer.trim())
        .filter(|answer| !answer.is_empty())
        .collect::<Vec<_>>();
    if answers.len() < 2 {
        return None;
    }
    log.append(
        "composition:compound_response",
        format!(
            "parts={} from={}",
            answers.len(),
            sub_results
                .iter()
                .map(|result| result.result_id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ),
    );
    // An aggregation: one step per independent part.
    let steps = answers.len();
    Some(candidate(
        "compound_response",
        answers.join("\n\n"),
        ("aggregation", steps),
        sub_results,
        true,
    ))
}

fn candidate(
    intent: &str,
    answer: String,
    (relation, steps): (&'static str, usize),
    sub_results: &[SolvedSubImpulse],
    consumes_sub_answers: bool,
) -> ComposedCandidate {
    let source_result_ids = sub_results
        .iter()
        .map(|result| result.result_id.clone())
        .collect::<Vec<_>>();
    let target = stable_id(
        "composition_candidate",
        &format!("{intent}:{answer}:{}", source_result_ids.join(",")),
    );
    ComposedCandidate {
        response_link: format!("response:synthesis:{target}"),
        target,
        intent: intent.to_owned(),
        answer,
        confidence: 1.0,
        relation,
        steps: u32::try_from(steps).unwrap_or(u32::MAX),
        source_result_ids,
        consumes_sub_answers,
    }
}

fn finalize_composed_candidate(
    prompt: &str,
    log: &mut EventLog,
    candidate: &ComposedCandidate,
) -> SymbolicAnswer {
    let answer = crate::question_necessity::enforce_questions(&candidate.answer, log);
    log.append("intent", candidate.intent.clone());
    crate::solver_helpers::record_answer_test(log, &candidate.intent, &answer);
    log.append("response", candidate.response_link.clone());
    crate::solver_helpers::record_selection(log, &candidate.intent);
    let trace_id = log.append("trace", candidate.intent.clone());
    let evidence_links = build_evidence_links(prompt, log, &candidate.response_link);
    let links_notation = answer_links_notation(prompt, &candidate.intent, &answer, log, &trace_id);
    let thinking_steps = log.thinking_steps_for_answer(&answer);
    SymbolicAnswer {
        intent: candidate.intent.clone(),
        answer,
        execution_recipe: None,
        confidence: candidate.confidence,
        evidence_links,
        thinking_steps,
        links_notation,
    }
}

fn extract_variable_assignments(prompt: &str) -> Vec<(String, String)> {
    let mut assignments = Vec::new();
    for (index, character) in prompt.char_indices() {
        if character != '=' {
            continue;
        }
        let Some(variable) = trailing_identifier(&prompt[..index]) else {
            continue;
        };
        let Some(value) = leading_number(&prompt[index + character.len_utf8()..]) else {
            continue;
        };
        if variable.chars().count() <= 2 && !assignments.iter().any(|(name, _)| name == &variable) {
            assignments.push((variable, value));
        }
    }
    assignments
}

fn trailing_identifier(value: &str) -> Option<String> {
    let trimmed = value.trim_end_matches(|ch: char| {
        ch.is_whitespace() || matches!(ch, ',' | ';' | ':' | '(' | '[' | '{')
    });
    let mut reversed = String::new();
    for character in trimmed.chars().rev() {
        if character.is_ascii_alphabetic() || character == '_' {
            reversed.push(character.to_ascii_lowercase());
        } else {
            break;
        }
    }
    if reversed.is_empty() {
        return None;
    }
    Some(reversed.chars().rev().collect())
}

fn leading_number(value: &str) -> Option<String> {
    let trimmed = value.trim_start();
    let mut end = 0usize;
    let mut saw_digit = false;
    for (index, character) in trimmed.char_indices() {
        if index == 0 && matches!(character, '-' | '+') {
            end = character.len_utf8();
            continue;
        }
        if character.is_ascii_digit() {
            saw_digit = true;
            end = index + character.len_utf8();
            continue;
        }
        if character == '.' {
            end = index + character.len_utf8();
            continue;
        }
        break;
    }
    saw_digit.then(|| trimmed[..end].to_owned())
}

fn extract_requested_expression(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    for marker in ["value of", "evaluate", "calculate"] {
        if let Some(index) = lower.find(marker) {
            let raw = &prompt[index + marker.len()..];
            let expression = clean_expression(raw);
            if expression.contains(|ch: char| ch.is_ascii_digit() || ch.is_ascii_alphabetic())
                && expression.contains(['+', '-', '*', '/', '^', '(', ')'])
            {
                return Some(expression);
            }
        }
    }
    None
}

fn clean_expression(value: &str) -> String {
    let mut expression = value.trim();
    if let Some(stripped) = expression.strip_prefix("of ") {
        expression = stripped.trim_start();
    }
    let end = expression.find(['?', '\n']).unwrap_or(expression.len());
    expression[..end]
        .trim()
        .trim_end_matches('.')
        .trim()
        .to_owned()
}

fn expression_mentions_variable(expression: &str, variable: &str) -> bool {
    expression
        .split(|ch: char| !(ch.is_ascii_alphabetic() || ch == '_'))
        .any(|token| token.eq_ignore_ascii_case(variable))
}

fn substitute_variables(expression: &str, assignments: &[(String, String)]) -> String {
    let values = assignments
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut out = String::with_capacity(expression.len());
    let mut index = 0usize;
    while index < expression.len() {
        let Some(character) = expression[index..].chars().next() else {
            break;
        };
        if character.is_ascii_alphabetic() || character == '_' {
            let start = index;
            index += character.len_utf8();
            while index < expression.len() {
                let Some(next) = expression[index..].chars().next() else {
                    break;
                };
                if next.is_ascii_alphabetic() || next == '_' {
                    index += next.len_utf8();
                } else {
                    break;
                }
            }
            let token = expression[start..index].to_ascii_lowercase();
            if let Some(value) = values.get(token.as_str()) {
                if last_non_whitespace(&out)
                    .is_some_and(|previous| previous.is_ascii_digit() || previous == ')')
                {
                    out.push('*');
                }
                out.push_str(value);
                if expression[index..].starts_with('(') {
                    out.push('*');
                }
            } else {
                out.push_str(&expression[start..index]);
            }
            continue;
        }
        if character == '('
            && last_non_whitespace(&out).is_some_and(|previous| previous.is_ascii_digit())
        {
            out.push('*');
        }
        out.push(character);
        index += character.len_utf8();
    }
    out
}

fn last_non_whitespace(value: &str) -> Option<char> {
    value
        .chars()
        .rev()
        .find(|character| !character.is_whitespace())
}

fn extract_quantities(prompt: &str) -> Vec<i64> {
    prompt
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '$' || ch == '-'))
        .filter_map(|raw| {
            let token = raw.trim_matches('$').trim_matches('-').to_ascii_lowercase();
            if token.is_empty() {
                return None;
            }
            token
                .parse::<i64>()
                .ok()
                .or_else(|| number_word_value(&token))
        })
        .collect()
}

fn number_word_value(token: &str) -> Option<i64> {
    match token {
        "zero" => Some(0),
        "one" | "a" | "an" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        _ => None,
    }
}

fn truncate_for_trace(value: &str, limit: usize) -> String {
    let mut out = String::new();
    for character in value.chars().take(limit) {
        if character == '\n' {
            out.push_str("\\n");
        } else {
            out.push(character);
        }
    }
    if value.chars().count() > limit {
        let _ = write!(out, "...");
    }
    out
}

#[cfg(test)]
#[path = "../tests/fixtures/synthesis-parent-tests.rs"]
mod parent_tests;
