//! Hidden-number interval riddles translated into linear constraints.

use std::fmt::Write as _;

use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::proof_engine::{
    attempt_proof_with_config, render_outcome_with_config, ProofRenderConfig,
};
use crate::solver_handlers::finalize_simple;

#[derive(Clone, Copy, Debug)]
struct Bound {
    value: i64,
    inclusive: bool,
}

impl Bound {
    const fn lower_operator(self) -> &'static str {
        if self.inclusive {
            ">="
        } else {
            ">"
        }
    }

    const fn upper_operator(self) -> &'static str {
        if self.inclusive {
            "<="
        } else {
            "<"
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct IntervalBounds {
    lower: Bound,
    upper: Bound,
}

pub fn try_number_riddle(
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
    let statement = formal_statement(bounds);
    let outcome = attempt_proof_with_config(
        prompt,
        &statement,
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
    let mentions_number = contains_any(cleaned, &["число", "number", "integer"]);
    let asks_identity = contains_any(
        cleaned,
        &[
            "что это за число",
            "какое это число",
            "какое число",
            "what is the number",
            "what number",
            "which number",
        ],
    );
    let hidden_number = contains_any(
        cleaned,
        &[
            "загадал",
            "загадала",
            "задумал",
            "задумала",
            "i guessed",
            "i picked",
            "i chose",
            "i thought of",
        ],
    );
    let has_bounds = contains_any(
        cleaned,
        &[
            "больше",
            "более",
            "меньше",
            "менее",
            "greater than",
            "more than",
            "less than",
            "at least",
            "at most",
        ],
    ) || contains_any(source, &[">", "<", "≥", "≤"]);

    mentions_number && has_bounds && (asks_identity || hidden_number)
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn extract_interval_bounds(word_text: &str, symbol_text: &str) -> Option<IntervalBounds> {
    let lower = find_bound(
        word_text,
        &[
            ("больше или равно", true),
            ("более или равно", true),
            ("не меньше", true),
            ("not less than", true),
            ("greater than or equal to", true),
            ("more than or equal to", true),
            ("at least", true),
            ("больше", false),
            ("более", false),
            ("greater than", false),
            ("more than", false),
        ],
    )
    .or_else(|| find_bound(symbol_text, &[(">=", true), (">", false)]))?;
    let upper = find_bound(
        word_text,
        &[
            ("меньше или равно", true),
            ("менее или равно", true),
            ("не больше", true),
            ("not more than", true),
            ("less than or equal to", true),
            ("at most", true),
            ("меньше", false),
            ("менее", false),
            ("less than", false),
        ],
    )
    .or_else(|| find_bound(symbol_text, &[("<=", true), ("<", false)]))?;

    Some(IntervalBounds { lower, upper })
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

fn phrase_has_boundary(text: &str, index: usize, phrase: &str) -> bool {
    let before_ok = text[..index]
        .chars()
        .next_back()
        .map_or(true, |character| !character.is_alphanumeric());
    let after_index = index + phrase.len();
    let after_ok = text[after_index..]
        .chars()
        .next()
        .map_or(true, |character| !character.is_alphanumeric());
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

fn formal_statement(bounds: IntervalBounds) -> String {
    format!(
        "x {} {} and x {} {} is satisfiable",
        bounds.lower.lower_operator(),
        bounds.lower.value,
        bounds.upper.upper_operator(),
        bounds.upper.value
    )
}

enum IntegerSolutions {
    None,
    Unique(i64),
    Multiple(Vec<i64>),
    Range { start: i64, end: i64 },
}

fn integer_solutions(bounds: IntervalBounds) -> IntegerSolutions {
    let start = if bounds.lower.inclusive {
        bounds.lower.value
    } else {
        bounds.lower.value.saturating_add(1)
    };
    let end = if bounds.upper.inclusive {
        bounds.upper.value
    } else {
        bounds.upper.value.saturating_sub(1)
    };
    if start > end {
        return IntegerSolutions::None;
    }
    if start == end {
        return IntegerSolutions::Unique(start);
    }
    if end.saturating_sub(start) > 20 {
        return IntegerSolutions::Range { start, end };
    }
    IntegerSolutions::Multiple((start..=end).collect())
}

fn render_interval_answer(
    language: &str,
    bounds: IntervalBounds,
    integer_solutions: &IntegerSolutions,
    statement: &str,
    formal_check: &str,
) -> String {
    match language {
        "ru" => render_interval_answer_ru(bounds, integer_solutions, statement, formal_check),
        _ => render_interval_answer_en(bounds, integer_solutions, statement, formal_check),
    }
}

fn render_interval_answer_ru(
    bounds: IntervalBounds,
    integer_solutions: &IntegerSolutions,
    statement: &str,
    formal_check: &str,
) -> String {
    let integer_line = match integer_solutions {
        IntegerSolutions::Unique(only) => {
            format!("Если это задача про целое число, единственный ответ: {only}.")
        }
        IntegerSolutions::None => String::from("Если это задача про целое число, решения нет."),
        IntegerSolutions::Range { start, end } => format!(
            "Если это задача про целые числа, ответ не единственный: подходит любое целое от {start} до {end}."
        ),
        IntegerSolutions::Multiple(candidates) => format!(
            "Если это задача про целые числа, ответ не единственный: подходят {}.",
            format_candidates(candidates)
        ),
    };
    let real_line = real_domain_line_ru(bounds);
    format!(
        "{integer_line}\n\n\
         Формализация над целыми: x in Z, x {} {}, x {} {}. \
         Проверяемая форма для решателя: `{statement}`.\n\n\
         {real_line}\n\n\
         Формальная проверка relative-meta-logic / SMT:\n{formal_check}",
        bounds.lower.lower_operator(),
        bounds.lower.value,
        bounds.upper.upper_operator(),
        bounds.upper.value
    )
}

fn render_interval_answer_en(
    bounds: IntervalBounds,
    integer_solutions: &IntegerSolutions,
    statement: &str,
    formal_check: &str,
) -> String {
    let integer_line = match integer_solutions {
        IntegerSolutions::Unique(only) => {
            format!("If this is an integer-number riddle, the unique answer is {only}.")
        }
        IntegerSolutions::None => {
            String::from("If this is an integer-number riddle, there is no solution.")
        }
        IntegerSolutions::Range { start, end } => format!(
            "If this is an integer-number riddle, the answer is not unique: every integer from {start} through {end} fits."
        ),
        IntegerSolutions::Multiple(candidates) => format!(
            "If this is an integer-number riddle, the answer is not unique: {} all fit.",
            format_candidates(candidates)
        ),
    };
    let real_line = real_domain_line_en(bounds);
    format!(
        "{integer_line}\n\n\
         Integer formalization: x in Z, x {} {}, x {} {}. \
         Solver form: `{statement}`.\n\n\
         {real_line}\n\n\
         Formal relative-meta-logic / SMT check:\n{formal_check}",
        bounds.lower.lower_operator(),
        bounds.lower.value,
        bounds.upper.upper_operator(),
        bounds.upper.value
    )
}

fn real_domain_line_ru(bounds: IntervalBounds) -> String {
    if has_multiple_real_solutions(bounds) {
        format!(
            "Если разрешены вещественные числа, ответ не единственный: например, x = {} тоже подходит.",
            real_example(bounds)
        )
    } else if has_single_real_solution(bounds) {
        format!(
            "На вещественных числах тоже есть единственное решение: x = {}.",
            bounds.lower.value
        )
    } else {
        String::from("На вещественных числах эти ограничения несовместимы.")
    }
}

fn real_domain_line_en(bounds: IntervalBounds) -> String {
    if has_multiple_real_solutions(bounds) {
        format!(
            "If real numbers are allowed, the answer is not unique; for example, x = {} also fits.",
            real_example(bounds)
        )
    } else if has_single_real_solution(bounds) {
        format!(
            "Over the real numbers there is also a single solution: x = {}.",
            bounds.lower.value
        )
    } else {
        String::from("Over the real numbers, these constraints are inconsistent.")
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
