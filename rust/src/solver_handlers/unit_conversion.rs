//! Conversion between the measurement units the seed knows.
//!
//! Issue #1176 R1: a linear factor between two units the Wikidata capture
//! covers (`data/seed/wikidata-conversion-to-si.lino`, P2370 conversion to SI
//! unit) is derived at answer time from the two units' stated amounts — each
//! unit resolved to its item — and the answer cites both items and the
//! property; the seed factor is the fallback for uncaptured units.
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

use super::calendar::{contains_term, term_position};
use super::numeric_list::parse_numbers;
use super::statistics::Decimal;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{
    MEANINGS_UNITS_LINO, ROLE_MEASUREMENT_UNIT, lexicon, localized_response, parser,
};
use crate::solver_handlers::finalize_simple;
use std::sync::OnceLock;

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
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| FormulaStep::parse(pair[0], pair[1]))
        .collect()
}

/// The captured Wikidata P2370 (conversion to SI unit) statements, read at
/// answer time from `data/seed/wikidata-conversion-to-si.lino`.
const WIKIDATA_CONVERSION_PATH: &str = "data/seed/wikidata-conversion-to-si.lino";

/// One captured P2370 statement: the unit's item, the stated amount and the
/// SI unit item it converts into, with that unit's seed slug.
struct SiStatement {
    unit: String,
    item: String,
    amount: Decimal,
    target: String,
    target_unit: String,
}

/// The capture date and every statement of the capture, parsed once.
fn si_statements() -> &'static (String, Vec<SiStatement>) {
    static TABLE: OnceLock<(String, Vec<SiStatement>)> = OnceLock::new();
    TABLE.get_or_init(|| {
        let text = crate::seed::seed_files()
            .into_iter()
            .find(|(path, _)| *path == WIKIDATA_CONVERSION_PATH)
            .map_or("", |(_, text)| text);
        let mut fetched = String::new();
        let mut rows = Vec::new();
        for root in &parser::parse_lino(text).children {
            for child in &root.children {
                if child.name == "fetched" {
                    fetched.clone_from(&child.id);
                }
                if child.name != "conversion" {
                    continue;
                }
                let Some(amount) = Decimal::parse(child.find_child_value("amount")) else {
                    continue;
                };
                rows.push(SiStatement {
                    unit: child.find_child_value("unit").to_owned(),
                    item: child.find_child_value("item").to_owned(),
                    amount,
                    target: child.find_child_value("target").to_owned(),
                    target_unit: child.find_child_value("target_unit").to_owned(),
                });
            }
        }
        (fetched, rows)
    })
}

/// A unit resolved to its Wikidata item: the item, its amount of the SI
/// unit, and the SI unit's item. The SI unit itself resolves to its own item
/// with amount 1.
fn resolve_item(slug: &str) -> Option<(String, Decimal, String)> {
    let rows = &si_statements().1;
    if let Some(row) = rows.iter().find(|row| row.unit == slug) {
        return Some((row.item.clone(), row.amount, row.target.clone()));
    }
    let row = rows.iter().find(|row| row.target_unit == slug)?;
    Some((row.target.clone(), Decimal::parse("1")?, row.target.clone()))
}

/// The factor `1 numerator = factor denominator`, derived from the two
/// units' captured P2370 amounts when both convert into the same SI unit
/// and the quotient terminates, with the grounding sentence's values.
struct WikidataFactor {
    factor: Decimal,
    values: Vec<(&'static str, String)>,
}

fn wikidata_factor(numerator: &str, denominator: &str) -> Option<WikidataFactor> {
    let (numerator_item, numerator_amount, numerator_si) = resolve_item(numerator)?;
    let (denominator_item, denominator_amount, denominator_si) = resolve_item(denominator)?;
    if numerator_si != denominator_si || numerator_item == denominator_item {
        return None;
    }
    let (factor, exact) = numerator_amount.div(denominator_amount, 12)?;
    if !exact {
        return None;
    }
    let ratio = [
        numerator_amount.render(),
        " ÷ ".to_owned(),
        denominator_amount.render(),
        " = ".to_owned(),
        factor.render(),
    ]
    .concat();
    Some(WikidataFactor {
        factor,
        values: vec![
            ("numerator_item", numerator_item),
            ("numerator_amount", numerator_amount.render()),
            ("denominator_item", denominator_item),
            ("denominator_amount", denominator_amount.render()),
            ("si_item", numerator_si),
            ("fetched", si_statements().0.clone()),
            ("ratio", ratio),
        ],
    })
}

/// A unit the prompt names: its meaning slug, the surface the prompt itself
/// used (echoed back in the answer), and where it appeared.
pub struct UnitMention {
    slug: String,
    surface: String,
    position: usize,
}

/// Every measurement unit the lowercased prompt names, at its earliest
/// surface occurrence. Surfaces come from the seed lexicon, so unit names in
/// a new language are a data edit.
pub fn unit_mentions(lowered: &str) -> Vec<UnitMention> {
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
    // Issue #700: the SI catalogue extends the older direct-pair lexicon.
    // Recognition still requires whole seeded surfaces, never a substring.
    for unit in crate::si_units::si_units() {
        let Some((surface, position)) = unit
            .surfaces
            .iter()
            .chain(std::iter::once(&unit.unit))
            .filter(|word| word.as_str() != "in")
            .filter_map(|word| term_position(lowered, word).map(|position| (word, position)))
            .min_by_key(|(_, position)| *position)
        else {
            continue;
        };
        if !mentions.iter().any(|mention| mention.slug == unit.unit) {
            mentions.push(UnitMention {
                slug: unit.unit.clone(),
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
                derivation = format!("{derivation} × {numerator}/{denominator}");
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
    let target = mentions
        .iter()
        .find(|mention| mention.slug != source.slug)?;
    let direction = find_conversion(&source.slug, &target.slug);

    if direction.is_none() {
        let (numerator, denominator) = value.ratio()?;
        let outcome =
            crate::si_units::convert_through_si(numerator, denominator, &source.slug, &target.slug);
        // A pair the lexicon places in two physical dimensions has no
        // conversion path: `incompatible_units`, the next row, explains it.
        if matches!(outcome, crate::si_units::SiConversion::Incompatible { .. })
            && crate::solver_handler_units::names_incompatible_unit_pair(normalized)
        {
            return None;
        }
        // The refusals read the same here as everywhere else the SI path
        // is reported: `SiConversion::describe` renders them from the seed.
        let body = match &outcome {
            crate::si_units::SiConversion::Converted {
                value_num,
                value_den,
            } => {
                log.append(
                    "unit_conversion:si_path",
                    format!("{} -> {}", source.slug, target.slug),
                );
                format!(
                    "{} {} = {}/{} {}",
                    value.render(),
                    source.surface,
                    value_num,
                    value_den,
                    target.surface
                )
            }
            crate::si_units::SiConversion::UnknownUnit(unit) => {
                crate::si_units::note_unknown_unit(log, unit);
                outcome.describe()
            }
            _ => outcome.describe(),
        };
        return Some(finalize_simple(
            prompt,
            log,
            "unit_conversion",
            "response:unit_conversion_si",
            &body,
            1.0,
        ));
    }
    let direction = direction?;

    let language = detect_language(prompt).slug();
    log.append("unit_conversion:source_unit", source.slug.clone());
    log.append("unit_conversion:target_unit", target.slug.clone());
    log.append("unit_conversion:value", value.render());

    // A linear factor is taken from the two units' captured Wikidata P2370
    // statements when both resolve to an item; the seed factor is the
    // fallback for units the capture does not cover.
    let grounding = match &direction {
        Direction::Forward(Conversion::Linear(_)) => wikidata_factor(&source.slug, &target.slug),
        Direction::ReverseLinear(_) => wikidata_factor(&target.slug, &source.slug),
        Direction::Forward(Conversion::Formula(_)) => None,
    };
    let direction = match (direction, &grounding) {
        (Direction::Forward(Conversion::Linear(seed)), Some(wikidata)) => {
            note_seed_factor(log, seed, wikidata.factor);
            Direction::Forward(Conversion::Linear(wikidata.factor))
        }
        (Direction::ReverseLinear(seed), Some(wikidata)) => {
            note_seed_factor(log, seed, wikidata.factor);
            Direction::ReverseLinear(wikidata.factor)
        }
        (direction, _) => direction,
    };
    let (result, exact, derivation, intent, factor_text) = match direction {
        Direction::Forward(Conversion::Linear(factor)) => {
            let result = value.mul(factor)?;
            let derivation = format!(
                "{} × {} = {}",
                value.render(),
                factor.render(),
                result.render()
            );
            (
                result,
                true,
                derivation,
                "unit_conversion_multiply",
                factor.render(),
            )
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
            (
                result,
                exact,
                derivation,
                "unit_conversion_divide",
                factor.render(),
            )
        }
        Direction::Forward(Conversion::Formula(steps)) => {
            let (result, exact, derivation) = apply_formula(value, &steps)?;
            (
                result,
                exact,
                derivation,
                "unit_conversion_formula",
                String::new(),
            )
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
                .replace(concat!("{", "value}"), &value.render())
                .replace("{source_unit}", &source.surface)
                .replace("{target_unit}", &target.surface)
                .replace("{result}", &result_text)
                .replace("{factor}", &factor_text)
                .replace(concat!("{", "derivation}"), &derivation)
                .replace("{equals}", if exact { "=" } else { "≈" })
        })
        .unwrap_or(derivation);
    let body = match &grounding {
        Some(wikidata) => {
            let (numerator, denominator) = if intent == "unit_conversion_divide" {
                (&target.surface, &source.surface)
            } else {
                (&source.surface, &target.surface)
            };
            for (key, value) in &wikidata.values {
                if key.ends_with("_item") {
                    log.append("unit_conversion:wikidata_item", value.clone());
                }
            }
            let sentence =
                localized_response("unit_conversion_wikidata", language).map(|template| {
                    wikidata
                        .values
                        .iter()
                        .fold(template, |text, (key, value)| {
                            text.replace(&["{", *key, "}"].concat(), value)
                        })
                        .replace("{numerator_unit}", numerator)
                        .replace("{denominator_unit}", denominator)
                });
            match sentence {
                Some(sentence) => [body.as_str(), &sentence].concat(),
                None => body,
            }
        }
        None => body,
    };
    Some(finalize_simple(
        prompt,
        log,
        "unit_conversion",
        "response:unit_conversion",
        &body,
        1.0,
    ))
}

/// Record that the seed's stated factor was superseded by the captured
/// Wikidata one, when the two disagree (they agree for every captured unit
/// today; a capture refresh that moves a factor shows up here).
fn note_seed_factor(log: &mut EventLog, seed: Decimal, wikidata: Decimal) {
    if seed.render() != wikidata.render() {
        log.append(
            "unit_conversion:seed_factor_superseded",
            [seed.render(), " -> ".to_owned(), wikidata.render()].concat(),
        );
    }
}
