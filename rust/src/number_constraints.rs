//! Native interval extraction, arithmetic, and proof projection.
//!
//! Issue #699 moved this primitive out of the specialized-handler directory.
//! Natural-language recognition is supplied by semantic roles in
//! `data/seed/meanings-number-constraints.lino`; this module keeps only the
//! language-neutral constraint/proof operation and answer projection.

use std::fmt::Write as _;

use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::proof_engine::decision::satisfiability_claim;
use crate::proof_engine::{
    ProofRenderConfig, attempt_proof_with_config, render_outcome_with_config,
};
use crate::proof_program::FormalProof;
use crate::seed;
use crate::solver_handlers::finalize_simple;

#[derive(Clone, Copy, Debug)]
struct Bound {
    value: i64,
    inclusive: bool,
}

impl Bound {
    const fn lower_operator(self) -> &'static str {
        if self.inclusive { ">=" } else { ">" }
    }

    const fn upper_operator(self) -> &'static str {
        if self.inclusive { "<=" } else { "<" }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IntervalBounds {
    lower: Bound,
    upper: Bound,
}

pub fn solve_number_constraints(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lowercased = prompt
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>();
    let cleaned = normalize_prompt(normalized);
    if !looks_like_number_riddle(&cleaned, &lowercased) {
        return None;
    }

    let bounds = extract_interval_bounds(&cleaned, &lowercased)?;
    let language = detect_language(prompt).slug();
    let proof = FormalProof::integer_interval(
        "x",
        bounds.lower.value,
        bounds.lower.inclusive,
        bounds.upper.value,
        bounds.upper.inclusive,
    )?;
    let statement = proof.statement();
    // The existing decision procedure proves the real-valued interval. Keep
    // that check in its established grammar while the canonical statement
    // above records the integer domain carried by `FormalProof`.
    let decision_statement = formal_statement(bounds);
    let outcome = attempt_proof_with_config(
        prompt,
        &decision_statement,
        language,
        false,
        false,
        ProofRenderConfig::default(),
    );
    let formal_check = render_outcome_with_config(&outcome, language, ProofRenderConfig::default());
    let integer_solutions = integer_solutions(bounds);
    let body = render_interval_answer(
        language,
        bounds,
        &integer_solutions,
        &statement,
        &formal_check,
    );

    log.append(
        "reasoning:number_constraint",
        "hidden_number_interval".to_owned(),
    );
    log.append("formalization:linear_constraint", statement);
    Some(finalize_simple(
        prompt,
        log,
        "number_constraint_reasoning",
        "response:number_constraint_reasoning",
        &body,
        0.86,
    ))
}

fn looks_like_number_riddle(cleaned: &str, source: &str) -> bool {
    let lexicon = seed::lexicon();
    let mentions_number = lexicon.mentions_role(seed::ROLE_NUMBER_CONSTRAINT_ENTITY, cleaned);
    let asks_identity = lexicon.mentions_role(seed::ROLE_NUMBER_CONSTRAINT_QUERY, cleaned);
    let hidden_number = lexicon.mentions_role(seed::ROLE_NUMBER_CONSTRAINT_HIDDEN, cleaned);
    let has_lower = lexicon.mentions_role(seed::ROLE_NUMBER_CONSTRAINT_LOWER, cleaned)
        || contains_any(source, &[">", "≥"]);
    let has_upper = lexicon.mentions_role(seed::ROLE_NUMBER_CONSTRAINT_UPPER, cleaned)
        || contains_any(source, &["<", "≤"]);

    mentions_number && has_lower && has_upper && (asks_identity || hidden_number)
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

pub fn extract_interval_bounds(word_text: &str, symbol_text: &str) -> Option<IntervalBounds> {
    let lower = find_role_bound(
        word_text,
        seed::ROLE_NUMBER_CONSTRAINT_LOWER_INCLUSIVE,
        true,
    )
    .or_else(|| find_role_bound(word_text, seed::ROLE_NUMBER_CONSTRAINT_LOWER_STRICT, false))
    .or_else(|| find_bound(symbol_text, &[(">=", true), (">", false)]))?;
    let upper = find_role_bound(
        word_text,
        seed::ROLE_NUMBER_CONSTRAINT_UPPER_INCLUSIVE,
        true,
    )
    .or_else(|| find_role_bound(word_text, seed::ROLE_NUMBER_CONSTRAINT_UPPER_STRICT, false))
    .or_else(|| find_bound(symbol_text, &[("<=", true), ("<", false)]))?;

    Some(IntervalBounds { lower, upper })
}

fn find_role_bound(text: &str, role: &str, inclusive: bool) -> Option<Bound> {
    seed::lexicon()
        .role_word_forms(role)
        .iter()
        .find_map(|form| {
            find_number_after_phrase(text, &form.text)
                .or_else(|| find_number_before_phrase(text, &form.text))
                .map(|value| Bound { value, inclusive })
        })
}

fn find_bound(text: &str, phrases: &[(&str, bool)]) -> Option<Bound> {
    phrases.iter().find_map(|(phrase, inclusive)| {
        find_number_after_phrase(text, phrase).map(|value| Bound {
            value,
            inclusive: *inclusive,
        })
    })
}

fn find_number_after_phrase(text: &str, phrase: &str) -> Option<i64> {
    for (index, _) in text.match_indices(phrase) {
        if !phrase_has_boundary(text, index, phrase) {
            continue;
        }
        let tail = &text[index + phrase.len()..];
        if let Some(value) = parse_leading_integer(tail) {
            return Some(value);
        }
    }
    None
}

fn find_number_before_phrase(text: &str, phrase: &str) -> Option<i64> {
    text.match_indices(phrase).find_map(|(index, _)| {
        if !phrase_has_boundary(text, index, phrase) {
            return None;
        }
        let head = text[..index].trim_end();
        let start = head
            .char_indices()
            .rev()
            .take_while(|(_, character)| character.is_ascii_digit() || *character == '-')
            .last()
            .map_or(head.len(), |(position, _)| position);
        let candidate = &head[start..];
        (!candidate.is_empty() && candidate != "-")
            .then(|| candidate.parse().ok())
            .flatten()
    })
}

fn phrase_has_boundary(text: &str, index: usize, phrase: &str) -> bool {
    if crate::coding::contains_cjk(phrase) {
        return true;
    }
    let before_ok = text[..index]
        .chars()
        .next_back()
        .is_none_or(|character| !character.is_alphanumeric());
    let after_index = index + phrase.len();
    let after_ok = text[after_index..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_alphanumeric());
    before_ok && after_ok && !is_negated_strict_bound(text, index, phrase)
}

fn is_negated_strict_bound(text: &str, index: usize, phrase: &str) -> bool {
    if !matches!(phrase, "больше" | "меньше" | "more than" | "less than") {
        return false;
    }
    text[..index]
        .split_whitespace()
        .next_back()
        .is_some_and(|word| matches!(word, "не" | "not"))
}

fn parse_leading_integer(text: &str) -> Option<i64> {
    let trimmed = text.trim_start_matches(|character: char| {
        character.is_whitespace() || matches!(character, ':' | ',' | '=')
    });
    let mut end = 0usize;
    for (index, character) in trimmed.char_indices() {
        if index == 0 && character == '-' {
            end = character.len_utf8();
            continue;
        }
        if character.is_ascii_digit() {
            end = index + character.len_utf8();
            continue;
        }
        break;
    }
    if end == 0 || trimmed[..end].ends_with('-') {
        return None;
    }
    trimmed[..end].parse().ok()
}

/// The interval as a claim in the linear decision procedure's grammar.
fn formal_statement(bounds: IntervalBounds) -> String {
    satisfiability_claim(&[
        format!("x {} {}", bounds.lower.lower_operator(), bounds.lower.value),
        format!("x {} {}", bounds.upper.upper_operator(), bounds.upper.value),
    ])
}

enum IntegerSolutions {
    None,
    Unique(i64),
    Multiple(Vec<i64>),
    Range { start: i64, end: i64 },
}

fn integer_solutions(bounds: IntervalBounds) -> IntegerSolutions {
    let start = if bounds.lower.inclusive {
        i128::from(bounds.lower.value)
    } else {
        i128::from(bounds.lower.value) + 1
    };
    let end = if bounds.upper.inclusive {
        i128::from(bounds.upper.value)
    } else {
        i128::from(bounds.upper.value) - 1
    };
    if start > end {
        return IntegerSolutions::None;
    }
    let (Ok(start), Ok(end)) = (i64::try_from(start), i64::try_from(end)) else {
        return IntegerSolutions::None;
    };
    if start == end {
        return IntegerSolutions::Unique(start);
    }
    if end.saturating_sub(start) > 20 {
        return IntegerSolutions::Range { start, end };
    }
    IntegerSolutions::Multiple((start..=end).collect())
}

/// The interval answer in `language`: the integer reading, the formalization,
/// the real-number reading and the rendered formal check, every sentence a
/// `number_constraint_*` phrase of `data/seed/proof-library.lino`.
fn render_interval_answer(
    language: &str,
    bounds: IntervalBounds,
    integer_solutions: &IntegerSolutions,
    statement: &str,
    formal_check: &str,
) -> String {
    let library = seed::proof_library();
    let integer_line = match integer_solutions {
        IntegerSolutions::Unique(only) => library.text(
            "number_constraint_integer_unique",
            language,
            &[("only", &only.to_string())],
        ),
        IntegerSolutions::None => library.text("number_constraint_integer_none", language, &[]),
        IntegerSolutions::Range { start, end } => library.text(
            "number_constraint_integer_range",
            language,
            &[("start", &start.to_string()), ("end", &end.to_string())],
        ),
        IntegerSolutions::Multiple(candidates) => library.text(
            "number_constraint_integer_multiple",
            language,
            &[("candidates", &format_candidates(candidates))],
        ),
    };
    let real_line = real_domain_line(bounds, language);
    let formalization = format!(
        "x {} {}, x {} {}",
        bounds.lower.lower_operator(),
        bounds.lower.value,
        bounds.upper.upper_operator(),
        bounds.upper.value
    );
    library.text(
        "number_constraint_answer",
        language,
        &[
            ("integer_line", &integer_line),
            ("formalization", &formalization),
            ("statement", statement),
            ("real_line", &real_line),
            ("formal_check", formal_check),
        ],
    )
}

fn real_domain_line(bounds: IntervalBounds, language: &str) -> String {
    let library = seed::proof_library();
    if has_multiple_real_solutions(bounds) {
        library.text(
            "number_constraint_real_multiple",
            language,
            &[("example", &real_example(bounds))],
        )
    } else if has_single_real_solution(bounds) {
        library.text(
            "number_constraint_real_single",
            language,
            &[("value", &bounds.lower.value.to_string())],
        )
    } else {
        library.text("number_constraint_real_inconsistent", language, &[])
    }
}

const fn has_multiple_real_solutions(bounds: IntervalBounds) -> bool {
    bounds.lower.value < bounds.upper.value
}

const fn has_single_real_solution(bounds: IntervalBounds) -> bool {
    bounds.lower.value == bounds.upper.value && bounds.lower.inclusive && bounds.upper.inclusive
}

fn real_example(bounds: IntervalBounds) -> String {
    format_half(i128::from(bounds.lower.value) * 2 + 1)
}

fn format_half(half_steps: i128) -> String {
    let sign = if half_steps < 0 { "-" } else { "" };
    let magnitude = half_steps.abs();
    let whole = magnitude / 2;
    if magnitude % 2 == 0 {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.5")
    }
}

fn format_candidates(candidates: &[i64]) -> String {
    let mut rendered = String::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if index > 0 {
            let _ = write!(rendered, ", ");
        }
        let _ = write!(rendered, "{candidate}");
    }
    rendered
}
