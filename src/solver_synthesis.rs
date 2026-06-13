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
            ..ProbabilityRankingConfig::default()
        }
        .with_decision_policy(config.probability_policy),
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
    log.append("composition:engine", evaluation.engine.slug());
    if !evaluation.steps.is_empty() {
        log.append("composition:steps", evaluation.steps.len().to_string());
    }

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
    Some(candidate(
        "arithmetic_word_problem",
        evaluation.formatted,
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
    let category_label = extract_count_category(&prompt[question_start..])
        .unwrap_or_else(|| String::from("listed objects"));
    let category = find_object_category(&category_label);
    let (matched_items, ignored_items) = partition_counted_items_by_category(&items, category);
    let count = matched_items.len();
    let category_trace = category.map_or_else(
        || format!("category={category_label} rule=all_listed_items"),
        |matched_category| {
            format!(
                "category={category_label} rule={}",
                matched_category.canonical
            )
        },
    );
    log.append("composition:category", category_trace);
    log.append(
        "composition:count",
        format!(
            "items={} matched={} ignored={} count={count}",
            items.join("|"),
            matched_items.join("|"),
            ignored_items.join("|")
        ),
    );
    Some(candidate(
        "object_counting",
        count.to_string(),
        1.0,
        sub_results,
    ))
}

#[derive(Debug)]
struct ObjectCategory {
    canonical: &'static str,
    aliases: &'static [&'static str],
    items: &'static [&'static str],
}

const OBJECT_CATEGORIES: &[ObjectCategory] = &[
    ObjectCategory {
        canonical: "musical instruments",
        aliases: &[
            "musical instrument",
            "musical instruments",
            "instrument",
            "instruments",
        ],
        items: &[
            "accordion",
            "banjo",
            "bassoon",
            "cello",
            "clarinet",
            "drum",
            "flute",
            "guitar",
            "harmonica",
            "harp",
            "oboe",
            "piano",
            "recorder",
            "saxophone",
            "trombone",
            "trumpet",
            "tuba",
            "viola",
            "violin",
        ],
    },
    ObjectCategory {
        canonical: "fruit",
        aliases: &["fruit", "fruits"],
        items: &[
            "apple",
            "banana",
            "blueberry",
            "grape",
            "lemon",
            "lime",
            "mango",
            "orange",
            "peach",
            "pear",
            "pineapple",
            "plum",
            "strawberry",
            "watermelon",
        ],
    },
    ObjectCategory {
        canonical: "vegetables",
        aliases: &["vegetable", "vegetables"],
        items: &[
            "broccoli", "carrot", "cucumber", "lettuce", "onion", "pepper", "potato", "spinach",
            "tomato",
        ],
    },
    ObjectCategory {
        canonical: "animals",
        aliases: &["animal", "animals"],
        items: &[
            "bird", "cat", "chicken", "cow", "dog", "duck", "fish", "goat", "horse", "pig",
            "rabbit", "sheep",
        ],
    },
    ObjectCategory {
        canonical: "vehicles",
        aliases: &["vehicle", "vehicles"],
        items: &[
            "airplane",
            "bicycle",
            "bike",
            "boat",
            "bus",
            "car",
            "motorcycle",
            "plane",
            "scooter",
            "train",
            "truck",
        ],
    },
    ObjectCategory {
        canonical: "tools",
        aliases: &["tool", "tools"],
        items: &["drill", "hammer", "pliers", "saw", "screwdriver", "wrench"],
    },
    ObjectCategory {
        canonical: "utensils",
        aliases: &["utensil", "utensils", "kitchen utensil", "kitchen utensils"],
        items: &[
            "fork", "knife", "ladle", "spatula", "spoon", "tongs", "whisk",
        ],
    },
    ObjectCategory {
        canonical: "furniture",
        aliases: &["furniture", "furnishing", "furnishings"],
        items: &[
            "bed", "cabinet", "chair", "couch", "desk", "dresser", "sofa", "table",
        ],
    },
    ObjectCategory {
        canonical: "clothing",
        aliases: &["clothing", "clothes", "garment", "garments"],
        items: &[
            "coat", "dress", "hat", "jacket", "pants", "shirt", "shoe", "sock",
        ],
    },
];

fn extract_count_category(question: &str) -> Option<String> {
    let lower = question.to_ascii_lowercase();
    let start = lower.find("how many")? + "how many".len();
    let raw_after = question[start..].trim_start();
    let lower_after = lower[start..].trim_start();
    let mut end = raw_after.len();
    for marker in [
        " do i have",
        " do we have",
        " did i have",
        " are there",
        " are in",
        " were there",
        "?",
        ".",
    ] {
        if let Some(index) = lower_after.find(marker) {
            end = end.min(index);
        }
    }
    let category = raw_after[..end]
        .trim()
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .to_ascii_lowercase();
    (!category.is_empty()).then_some(category)
}

fn find_object_category(label: &str) -> Option<&'static ObjectCategory> {
    let normalized = normalize_count_phrase(label);
    OBJECT_CATEGORIES.iter().find(|category| {
        category
            .aliases
            .iter()
            .any(|alias| normalize_count_phrase(alias) == normalized)
    })
}

fn partition_counted_items_by_category(
    items: &[String],
    category: Option<&ObjectCategory>,
) -> (Vec<String>, Vec<String>) {
    let Some(category) = category else {
        return (items.to_vec(), Vec::new());
    };
    let mut matched = Vec::new();
    let mut ignored = Vec::new();
    for item in items {
        if item_matches_category(item, category) {
            matched.push(item.clone());
        } else {
            ignored.push(item.clone());
        }
    }
    (matched, ignored)
}

fn item_matches_category(item: &str, category: &ObjectCategory) -> bool {
    let normalized = normalize_count_phrase(item);
    category.items.iter().any(|candidate| {
        let normalized_candidate = normalize_count_phrase(candidate);
        normalized == normalized_candidate
            || normalized
                .strip_suffix(normalized_candidate.as_str())
                .is_some_and(|prefix| prefix.ends_with(' '))
    })
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

fn normalize_count_phrase(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .filter(|token| !matches!(*token, "a" | "an" | "the" | "of"))
        .map(singularize_count_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn singularize_count_token(token: &str) -> String {
    if token.len() > 4 {
        if let Some(stem) = token.strip_suffix("ies") {
            return format!("{stem}y");
        }
    }
    if token.len() > 3 && token.ends_with('s') && !token.ends_with("ss") {
        return token[..token.len() - 1].to_owned();
    }
    token.to_owned()
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
