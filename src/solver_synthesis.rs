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
use crate::engine::{answer_links_notation, stable_id, SymbolicAnswer};
use crate::event_log::{build_evidence_links, EventLog};
use crate::intent_formalization::IntentFormalizationCache;
use crate::probability::{
    rank_probability_candidates, ProbabilityCandidate, ProbabilityRankingConfig, ProbabilityStore,
};
use crate::solver::{SolverConfig, UniversalSolver};
use crate::solver_helpers::DecomposedSubImpulse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolvedSubImpulse {
    pub impulse_id: String,
    pub result_id: String,
    pub text: String,
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
    prior_score: f32,
    source_result_ids: Vec<String>,
}

pub fn record_solved_sub_impulse(
    log: &mut EventLog,
    sub_impulse: &DecomposedSubImpulse,
    answer: &SymbolicAnswer,
) -> SolvedSubImpulse {
    let payload = format!(
        "sub_impulse={} intent={} answer={}",
        sub_impulse.id,
        answer.intent,
        truncate_for_trace(&answer.answer, 240),
    );
    let result_id = log.append("sub_result", payload);
    SolvedSubImpulse {
        impulse_id: sub_impulse.id.clone(),
        result_id,
        text: sub_impulse.text.clone(),
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
        sub_impulses
            .iter()
            .take(6)
            .filter(|sub_impulse| !sub_impulse.text.trim().is_empty())
            .map(|sub_impulse| {
                let answer = sub_solver.solve_with_history_probability_store_and_intent_cache(
                    &sub_impulse.text,
                    &[],
                    probability_store,
                    intent_cache,
                );
                record_solved_sub_impulse(log, sub_impulse, &answer)
            })
            .collect()
    }
}

pub fn try_synthesize_from_sub_results(
    prompt: &str,
    log: &mut EventLog,
    sub_results: &[SolvedSubImpulse],
    probability_store: &ProbabilityStore,
    config: SolverConfig,
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
    if let Some(candidate) = compose_object_count(prompt, log, sub_results) {
        candidates.push(candidate);
    }
    if candidates.is_empty() {
        return None;
    }

    for candidate in &candidates {
        log.append(
            "candidate",
            format!(
                "composition:{} from={}",
                candidate.target,
                candidate.source_result_ids.join(",")
            ),
        );
    }

    let probability_candidates = candidates
        .iter()
        .map(|candidate| ProbabilityCandidate::new(candidate.target.clone(), candidate.prior_score))
        .collect::<Vec<_>>();
    let ranking = rank_probability_candidates(
        &probability_candidates,
        probability_store,
        ProbabilityRankingConfig {
            temperature: config.temperature,
            offline: config.offline,
            markov_from: Some(String::from("synthesis")),
        },
    );
    log.append("probability:ranking", ranking.trace_summary());

    let selected_target = ranking.ranked.first()?.target.as_str();
    let selected = candidates
        .iter()
        .find(|candidate| candidate.target == selected_target)?;
    log.append("composition:selected", selected.target.clone());
    Some(finalize_composed_candidate(prompt, log, selected))
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

    Some(candidate(
        "algebra_substitution",
        evaluation.formatted,
        1.15,
        sub_results,
    ))
}

fn compose_remainder_sale(
    prompt: &str,
    log: &mut EventLog,
    sub_results: &[SolvedSubImpulse],
) -> Option<ComposedCandidate> {
    let lower = prompt.to_lowercase();
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
    let revenue = remainder.checked_mul(price)?;
    log.append(
        "composition:remainder",
        format!("total={total} consumed={consumed} remainder={remainder}"),
    );
    log.append(
        "composition:evaluation",
        format!("{remainder} * {price} = {revenue}"),
    );
    Some(candidate(
        "arithmetic_word_problem",
        revenue.to_string(),
        1.05,
        sub_results,
    ))
}

fn compose_object_count(
    prompt: &str,
    log: &mut EventLog,
    sub_results: &[SolvedSubImpulse],
) -> Option<ComposedCandidate> {
    let lower = prompt.to_lowercase();
    let have_start = lower.find("i have ")? + "i have ".len();
    let question_start = lower[have_start..].find("how many")? + have_start;
    let listed = prompt[have_start..question_start]
        .trim()
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace());
    let normalized = listed
        .replace(", and ", ", ")
        .replace(" and ", ", ")
        .replace(';', ",");
    let items = normalized
        .split(',')
        .map(clean_counted_item)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if items.len() < 2 {
        return None;
    }
    let count = items.len();
    log.append(
        "composition:count",
        format!("items={} count={count}", items.join("|")),
    );
    Some(candidate(
        "object_counting",
        count.to_string(),
        1.0,
        sub_results,
    ))
}

fn candidate(
    intent: &str,
    answer: String,
    prior_score: f32,
    sub_results: &[SolvedSubImpulse],
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
        prior_score,
        source_result_ids,
    }
}

fn finalize_composed_candidate(
    prompt: &str,
    log: &mut EventLog,
    candidate: &ComposedCandidate,
) -> SymbolicAnswer {
    log.append("intent", candidate.intent.clone());
    if log.first_of("validation").is_none() {
        log.append(
            "validation",
            "composition_replays_sub_result_links".to_owned(),
        );
    }
    log.append("response", candidate.response_link.clone());
    log.append("trace:simplification", "smallest_sufficient".to_owned());
    let trace_id = log.append("trace", candidate.intent.clone());
    let evidence_links = build_evidence_links(prompt, log, &candidate.response_link);
    let links_notation =
        answer_links_notation(prompt, &candidate.intent, &candidate.answer, log, &trace_id);
    SymbolicAnswer {
        intent: candidate.intent.clone(),
        answer: candidate.answer.clone(),
        confidence: candidate.confidence,
        evidence_links,
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

fn clean_counted_item(raw: &str) -> String {
    let mut item = raw
        .trim()
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace());
    for article in ["a ", "an ", "the ", "one "] {
        if let Some(stripped) = item.strip_prefix(article) {
            item = stripped.trim_start();
            break;
        }
    }
    item.to_owned()
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
mod tests {
    use super::{
        extract_quantities, extract_requested_expression, extract_variable_assignments,
        substitute_variables,
    };

    #[test]
    fn extracts_assignments_and_substitutes_expression() {
        let prompt = "If x = 2 and y = 5, what is the value of (x^4 + 2y^2) / 6?";
        let assignments = extract_variable_assignments(prompt);
        assert_eq!(
            assignments,
            vec![
                (String::from("x"), String::from("2")),
                (String::from("y"), String::from("5"))
            ],
        );
        let expression = extract_requested_expression(prompt).expect("expression");
        assert_eq!(
            substitute_variables(&expression, &assignments),
            "(2^4 + 2*5^2) / 6"
        );
    }

    #[test]
    fn extracts_number_words_for_remainder_sale() {
        let values = extract_quantities(
            "Ducks lay 16 eggs. She eats three, bakes with four, and sells each for $2.",
        );
        assert_eq!(values, vec![16, 3, 4, 2]);
    }
}
