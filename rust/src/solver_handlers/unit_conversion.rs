//! Conversion between the measurement units the seed knows.
//!
//! Issue #1176: "how many kilometers are 26.2 miles?" is arithmetic over a
//! stated factor, not a lookup question — the answer shows the
//! multiplication (26.2 × 1.609344 = 42.1648128, because 1 mile =
//! 1.609344 kilometres). Every factor and formula lives in
//! `data/seed/meanings-units.lino` as `conversion` records; this handler
//! parses them at run time and computes exactly, so adding a unit is a data
//! edit and never a code change. Unknown units (furlongs) are declined.
//!
//! The gate is a conjunction: conversion question vocabulary ("how many",
//! "convert", "in", "to", …), at least one number, exactly two distinct known
//! units, and a conversion between them. Single common words like "in" or
//! "a" appear in the question vocabulary, so no conjunct alone can claim a
//! prompt.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{
    MEANINGS_UNITS_LINO, ROLE_MEASUREMENT_UNIT, lexicon, localized_response, parser,
};
use crate::solver_handlers::finalize_simple;
use std::sync::OnceLock;
use super::calendar::{contains_term, term_position};
use super::numeric_list::parse_numbers;
use super::statistics::Decimal;

/// Semantic role of the question vocabulary that opens a conversion
/// ("how many", "convert", "in", "to", and translations), stated in
/// `data/seed/meanings-units.lino`.
const ROLE_UNIT_CONVERSION_QUESTION: &str = "unit_conversion_question";

/// How one seed unit turns into another, parsed from a `conversion` record.
#[derive(Debug, Clone)]
enum Conversion {
    /// `1 source = factor target` — multiply (mile → kilometre, 1.609344).
    Linear(Decimal),
    /// A fixed sequence of exact rational steps, e.g. °C → °F: ×9/5, +32.
    Formula(Vec<FormulaStep>),
}

#[derive(Debug, Clone, Copy)]
enum FormulaStep {
    /// Multiply by the exact rational `numerator / denominator` (9/5, 5/9).
    Scale { numerator: i128, denominator: i128 },
    /// Add an exact decimal constant (32, 273.15; subtraction is a negative).
    Shift(Decimal),
}

impl FormulaStep {
    /// Parse one `operand` of a formula step: a fraction ("9/5") scales, a
    /// plain decimal ("32", "273.15") shifts.
    fn parse(word: &str, operand: &str) -> Option<Self> {
        match word {
            "multiply" => {
                let (numerator, denominator) = parse_rational(operand)?;
                Some(Self::Scale {
                    numerator,
                    denominator,
                })
            }
            "divide" => {
                let (numerator, denominator) = parse_rational(operand)?;
                Some(Self::Scale {
                    numerator: denominator,
                    denominator: numerator,
                })
            }
            "add" => Some(Self::Shift(Decimal::parse(operand)?)),
            "subtract" => Some(Self::Shift(Decimal::parse(operand)?.negate())),
            _ => None,
        }
    }
}

/// Parse "9/5" (or a plain "2") as an exact rational.
fn parse_rational(text: &str) -> Option<(i128, i128)> {
    if let Some((numerator, denominator)) = text.split_once('/') {
        return Some((numerator.parse().ok()?, denominator.parse().ok()?));
    }
    Some((text.parse().ok()?, 1))
}

/// Every conversion the seed states, as `(source, target)` pairs. Parsed once
/// from `data/seed/meanings-units.lino`; factors and formulas are data, never
/// code constants (issue #1176).
fn conversions() -> &'static Vec<(String, String, Conversion)> {
    static TABLE: OnceLock<Vec<(String, String, Conversion)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = Vec::new();
        for record in &parser::parse_lino(MEANINGS_UNITS_LINO).children {
            // Each unit is one record under the file's `meanings` root, and
            // its `conversion` rows are that unit's children.
            for unit in &record.children {
                for child in &unit.children {
                    if child.name != "conversion" {
                        continue;
                    }
                    let mut tokens = child.id.split_whitespace();
                    let Some(target) = tokens.next() else {
                        continue;
                    };
                    let rest = tokens.collect::<Vec<_>>();
                    // `conversion kilometer 1.609344` is linear;
                    // `conversion fahrenheit formula multiply 9/5 add 32` is a
                    // formula. Anything unparsable is skipped, not guessed.
                    let conversion = match rest.first() {
                        Some(&"formula") => match parse_formula_steps(&rest[1..]) {
                            Some(steps) if !steps.is_empty() => Conversion::Formula(steps),
                            _ => continue,
                        },
                        Some(factor) => match Decimal::parse(factor) {
                            Some(factor) => Conversion::Linear(factor),
                            None => continue,
                        },
                        None => continue,
                    };
                    table.push((unit.name.clone(), target.to_owned(), conversion));
                }
            }
        }
        table
    })
}

/// Parse the `multiply 9/5 add 32` tail of a formula conversion record.
fn parse_formula_steps(tokens: &[&str]) -> Option<Vec<FormulaStep>> {
    tokens
        .chunks_exact(2)
        .map(|pair| FormulaStep::parse(pair[0], pair[1]))
        .collect()
}

/// A unit the prompt names: its meaning slug, the surface the prompt itself
/// used (echoed back in the answer), and where it appeared.
struct UnitMention {
    slug: String,
    surface: String,
    position: usize,
}

/// Every measurement unit the lowercased prompt names, at its earliest
/// surface occurrence. Surfaces come from the seed lexicon, so unit names in
/// a new language are a data edit.
fn unit_mentions(lowered: &str) -> Vec<UnitMention> {
    let mut mentions: Vec<UnitMention> = Vec::new();
    for meaning in lexicon().meanings_with_role(ROLE_MEASUREMENT_UNIT) {
        let Some((surface, position)) = meaning
            .words()
            .filter_map(|word| term_position(lowered, word).map(|position| (word, position)))
            .min_by_key(|(_, position)| *position)
        else {
            continue;
        };
        if !mentions.iter().any(|mention| mention.slug == meaning.slug) {
            mentions.push(UnitMention {
                slug: meaning.slug.clone(),
                surface: surface.to_owned(),
                position,
            });
        }
    }
    mentions
}

/// How to get from the source unit to the target unit.
enum Direction {
    /// The seed states source → target directly.
    Forward(Conversion),
    /// The seed states target → source; a linear factor inverts into a
    /// division (1 mile = 1.609344 km, so km → miles divides by it).
    ReverseLinear(Decimal),
}

/// The seed's conversion from `source` to `target`, directly or by inverting
/// a linear factor. Formula conversions are stated in both directions in the
/// seed (°C↔°F, °C↔K), so only linear factors invert.
fn find_conversion(source: &str, target: &str) -> Option<Direction> {
    let table = conversions();
    if let Some((_, _, conversion)) = table
        .iter()
        .find(|(from, to, _)| from == source && to == target)
    {
        return Some(Direction::Forward(conversion.clone()));
    }
    match table
        .iter()
        .find(|(from, to, _)| from == target && to == source)?
        .2
    {
        Conversion::Linear(factor) => Some(Direction::ReverseLinear(factor)),
        Conversion::Formula(_) => None,
    }
}

/// Apply the seed's formula steps, tracking exactness. The derivation echoes
/// each step — "25 × 9/5 + 32 = 77" — wrapping the accumulated expression so
/// a following scale cannot read with wrong precedence ("(77 - 32) × 5/9").
fn apply_formula(start: Decimal, steps: &[FormulaStep]) -> Option<(Decimal, bool, String)> {
    let mut value = start;
    let mut exact = true;
    let mut derivation = start.render();
    for step in steps {
        match *step {
            FormulaStep::Scale {
                numerator,
                denominator,
            } => {
                let (scaled, step_exact) = value.mul_div(numerator, denominator, 7)?;
                if derivation.contains(' ') {
                    derivation = format!("({derivation})");
                }
                derivation = format!("{} × {}/{}", derivation, numerator, denominator);
                value = scaled;
                exact &= step_exact;
            }
            FormulaStep::Shift(constant) => {
                value = value.add(constant)?;
                if constant.is_negative() {
                    derivation = format!("{derivation} - {}", constant.negate().render());
                } else {
                    derivation = format!("{derivation} + {}", constant.render());
                }
            }
        }
    }
    let joiner = if exact { "=" } else { "≈" };
    Some((
        value,
        exact,
        format!("{} {} {}", derivation, joiner, value.render()),
    ))
}

/// Unit-conversion entry point (issue #1176): answer "how many X are N Y",
/// "N Y in X", and "convert N Y to X" exactly, showing the arithmetic.
pub fn handle_unit_conversion(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // Question vocabulary gate — a conjunct, never sufficient alone.
    let asks_conversion = lexicon()
        .words_for_role(ROLE_UNIT_CONVERSION_QUESTION)
        .iter()
        .any(|word| contains_term(normalized, word));
    let lowered = prompt.to_lowercase();
    if !asks_conversion && !lowered.contains(['?', '？']) {
        return None;
    }
    let items = parse_numbers(&lowered);
    if items.is_empty() {
        return None;
    }
    let mentions = unit_mentions(&lowered);
    if mentions.len() != 2 {
        // Fewer than two known units is not a conversion; more is ambiguous
        // ("5 feet 9 inches in cm") and is declined rather than guessed.
        return None;
    }
    let value = Decimal::parse(&items[0].text)?;
    // The source unit is the one stated next to the number; the target is
    // the other one.
    let number_position = lowered.find(&items[0].text)?;
    let source = mentions
        .iter()
        .min_by_key(|mention| mention.position.abs_diff(number_position))?;
    let target = mentions.iter().find(|mention| mention.slug != source.slug)?;
    let direction = find_conversion(&source.slug, &target.slug)?;

    let language = detect_language(prompt).slug();
    log.append("unit_conversion:source_unit", source.slug.clone());
    log.append("unit_conversion:target_unit", target.slug.clone());
    log.append("unit_conversion:value", value.render());

    let (result, exact, derivation, intent, factor_text) = match direction {
        Direction::Forward(Conversion::Linear(factor)) => {
            let result = value.mul(factor)?;
            let derivation =
                format!("{} × {} = {}", value.render(), factor.render(), result.render());
            (result, true, derivation, "unit_conversion_multiply", factor.render())
        }
        Direction::ReverseLinear(factor) => {
            let (result, exact) = value.div(factor, 7)?;
            let equals = if exact { "=" } else { "≈" };
            let derivation = format!(
                "{} ÷ {} {} {}",
                value.render(),
                factor.render(),
                equals,
                result.render()
            );
            (result, exact, derivation, "unit_conversion_divide", factor.render())
        }
        Direction::Forward(Conversion::Formula(steps)) => {
            let (result, exact, derivation) = apply_formula(value, &steps)?;
            (result, exact, derivation, "unit_conversion_formula", String::new())
        }
    };
    let result_text = if exact {
        result.render()
    } else {
        format!("≈{}", result.render())
    };
    log.append("unit_conversion:result", result_text.clone());
    log.append("unit_conversion:derivation", derivation.clone());

    // The prose lives in `data/seed/multilingual-responses-quantities.lino`;
    // the placeholder set is the union of the three direction templates.
    let body = localized_response(intent, language)
        .map(|template| {
            template
                .replace("{value}", &value.render())
                .replace("{source_unit}", &source.surface)
                .replace("{target_unit}", &target.surface)
                .replace("{result}", &result_text)
                .replace("{factor}", &factor_text)
                .replace("{derivation}", &derivation)
                .replace("{equals}", if exact { "=" } else { "≈" })
        })
        .unwrap_or(derivation);
    Some(finalize_simple(
        prompt,
        log,
        "unit_conversion",
        "response:unit_conversion",
        &body,
        1.0,
    ))
}
