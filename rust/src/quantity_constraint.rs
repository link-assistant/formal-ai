//! Finite seeded aggregate/comparison grammar; labels do not imply scheduling.
//! Captures retain the original source and UTF-8 offsets. Calculation stays in
//! the existing arithmetic executor, not in the language recognizer.

use std::collections::{BTreeMap, BTreeSet};

use regex::{Regex, RegexBuilder};
use serde::Serialize;

use crate::calculation::{CalculationEvaluation, evaluate_calculation};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

#[derive(Clone, Debug, Serialize)]
struct SourceSpan {
    start: usize,
    end: usize,
}

#[derive(Clone, Debug, Serialize)]
struct AggregateProof {
    kind: &'static str,
    surface: String,
    span: SourceSpan,
}

#[derive(Debug, Serialize)]
struct ComparisonProof {
    role: &'static str,
    surface: String,
    span: SourceSpan,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Quantity {
    literal: String,
    unit: String,
    unit_surface: String,
    span: SourceSpan,
    unit_span: SourceSpan,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SupportedFrame {
    aggregation: &'static str,
    relation: &'static str,
    terms: Vec<Quantity>,
    bound: Quantity,
    aggregation_proof: AggregateProof,
    comparison_proof: ComparisonProof,
    source: String,
    source_utf8_bytes: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UnknownFrame {
    reason: &'static str,
    aggregation_proof: AggregateProof,
    source: String,
    source_utf8_bytes: usize,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum QuantityConstraint {
    Supported(SupportedFrame),
    Unknown(UnknownFrame),
}

fn words(role: &str) -> Vec<String> {
    crate::seed::lexicon().words_for_role_in_languages(role, &["en"])
}

fn escaped_surface(text: &str) -> String {
    let mut result = String::new();
    let mut whitespace = false;
    for character in text.chars() {
        if character.is_whitespace() {
            if !whitespace {
                result.push_str(r"\s+");
            }
            whitespace = true;
        } else {
            result.push_str(&regex::escape(&character.to_string()));
            whitespace = false;
        }
    }
    result
}

fn role_pattern(role: &str) -> Option<String> {
    let mut surfaces = words(role);
    surfaces.sort_by_key(|surface| std::cmp::Reverse(surface.len()));
    (!surfaces.is_empty()).then(|| {
        surfaces
            .iter()
            .map(|surface| escaped_surface(surface))
            .collect::<Vec<_>>()
            .join("|")
    })
}

fn expression(pattern: &str) -> Option<Regex> {
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .build()
        .ok()
}

fn word_character(character: char) -> bool {
    static WORD: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let regex =
        WORD.get_or_init(|| Regex::new(r"[\p{L}\p{N}_]").expect("constant character class"));
    regex.is_match(&character.to_string())
}

fn token_edges(text: &str, start: usize, end: usize) -> bool {
    text[..start]
        .chars()
        .next_back()
        .is_none_or(|character| !word_character(character))
        && text[end..]
            .chars()
            .next()
            .is_none_or(|character| !word_character(character))
}

struct FrameMatch {
    fields: BTreeMap<String, SourceSpan>,
}

impl FrameMatch {
    fn span(&self, name: &str, origin: usize) -> Result<SourceSpan, &'static str> {
        let span = self.fields.get(name).ok_or("missing-seeded-capture")?;
        Ok(SourceSpan {
            start: origin + span.start,
            end: origin + span.end,
        })
    }

    fn text<'a>(&self, text: &'a str, name: &str) -> Result<&'a str, &'static str> {
        let span = self.fields.get(name).ok_or("missing-seeded-capture")?;
        text.get(span.start..span.end)
            .ok_or("invalid-seeded-capture")
    }
}

fn match_frame(
    text: &str,
    role: &str,
    patterns: &[(&str, &str)],
    whole: bool,
) -> Option<FrameMatch> {
    let slots = Regex::new(r"\{([a-z_]+)\}").ok()?;
    for template in words(role) {
        let mut pattern = String::new();
        let mut cursor = 0;
        let mut valid = true;
        for slot in slots.captures_iter(&template) {
            let (Some(full), Some(name)) = (slot.get(0), slot.get(1)) else {
                valid = false;
                break;
            };
            pattern.push_str(&escaped_surface(&template[cursor..full.start()]));
            let Some((_, capture)) = patterns.iter().find(|(key, _)| *key == name.as_str()) else {
                valid = false;
                break;
            };
            pattern.push_str(&format!("(?P<{}>{capture})", name.as_str()));
            cursor = full.end();
        }
        if !valid {
            continue;
        }
        pattern.push_str(&escaped_surface(&template[cursor..]));
        if whole {
            pattern = format!(r"^\s*{pattern}\s*[.?!]?\s*$");
        }
        let Some(regex) = expression(&pattern) else {
            continue;
        };
        for captures in regex.captures_iter(text) {
            let Some(full) = captures.get(0) else {
                continue;
            };
            if !whole && !token_edges(text, full.start(), full.end()) {
                continue;
            }
            let fields = regex
                .capture_names()
                .flatten()
                .filter_map(|name| {
                    captures.name(name).map(|capture| {
                        (
                            name.to_owned(),
                            SourceSpan {
                                start: capture.start(),
                                end: capture.end(),
                            },
                        )
                    })
                })
                .collect();
            return Some(FrameMatch { fields });
        }
    }
    None
}

fn canonical_unit(surface: &str) -> Option<String> {
    if surface != surface.to_lowercase() {
        return None;
    }
    let canonical = crate::si_units::unit_named_by(surface)?;
    let mut candidates = BTreeSet::new();
    for meaning in crate::seed::lexicon().meanings_with_role(crate::seed::ROLE_MEASUREMENT_UNIT) {
        if meaning.words().any(|word| word == surface) {
            candidates.insert(
                crate::si_units::unit_named_by(&meaning.slug)
                    .unwrap_or_else(|| meaning.slug.clone()),
            );
        }
    }
    for unit in crate::si_units::si_units() {
        if unit.unit == surface || unit.surfaces.iter().any(|word| word == surface) {
            candidates.insert(unit.unit);
        }
    }
    (candidates.len() == 1 && candidates.contains(&canonical)).then_some(canonical)
}

fn operand(text: &str) -> Option<i128> {
    let value = text.parse::<i128>().ok()?;
    (0..=9_007_199_254_740_991)
        .contains(&value)
        .then_some(value)
}

fn quantity(
    text: &str,
    captures: &FrameMatch,
    amount: &str,
    unit_field: &str,
    origin: usize,
    canonical: &str,
) -> Result<Quantity, &'static str> {
    let literal = captures.text(text, amount)?;
    let unit_surface = captures.text(text, unit_field)?;
    if operand(literal).is_none() || canonical_unit(unit_surface).as_deref() != Some(canonical) {
        return Err("unresolved-quantity-unit");
    }
    Ok(Quantity {
        literal: literal.to_owned(),
        unit: canonical.to_owned(),
        unit_surface: unit_surface.to_owned(),
        span: captures.span(amount, origin)?,
        unit_span: captures.span(unit_field, origin)?,
    })
}

fn supported_frame(
    raw: &str,
    sentences: &[(usize, &str)],
    bound_names: &str,
    relation: &str,
) -> Result<SupportedFrame, &'static str> {
    if sentences.len() != 3 || sentences.iter().map(|(_, text)| *text).collect::<String>() != raw {
        return Err("incomplete-raw-frame");
    }
    let members =
        role_pattern("quantity-constraint-member-reference").ok_or("missing-seeded-grammar")?;
    let aggregate =
        role_pattern("quantity-constraint-aggregate").ok_or("missing-seeded-grammar")?;
    let conjunction =
        role_pattern("quantity-constraint-list-conjunction").ok_or("missing-seeded-grammar")?;
    let patterns = [
        ("bound_ref", bound_names),
        ("query_ref", bound_names),
        ("bound_amount", r"[+-]?[0-9]+"),
        ("bound_unit", r"[\p{L}]+"),
        ("members_ref", &members),
        ("quantities", r"[\s\S]+?"),
        ("aggregation", &aggregate),
        ("relation", relation),
    ];
    let bound = match_frame(
        sentences[0].1,
        "quantity-constraint-bound-frame",
        &patterns,
        true,
    )
    .ok_or("unsupported-constraint-frame")?;
    let collection = match_frame(
        sentences[1].1,
        "quantity-constraint-members-frame",
        &patterns,
        true,
    )
    .ok_or("unsupported-constraint-frame")?;
    let comparison = match_frame(
        sentences[2].1,
        "quantity-constraint-comparison-frame",
        &patterns,
        true,
    )
    .ok_or("unsupported-constraint-frame")?;
    if bound.text(sentences[0].1, "bound_ref")?.to_lowercase()
        != comparison.text(sentences[2].1, "query_ref")?.to_lowercase()
    {
        return Err("unbound-capacity-reference");
    }
    let unit =
        canonical_unit(bound.text(sentences[0].1, "bound_unit")?).ok_or("unresolved-bound")?;
    if operand(bound.text(sentences[0].1, "bound_amount")?).is_none() {
        return Err("unresolved-bound");
    }
    let bound_quantity = quantity(
        sentences[0].1,
        &bound,
        "bound_amount",
        "bound_unit",
        sentences[0].0,
        &unit,
    )?;
    let values = collection.text(sentences[1].1, "quantities")?;
    let value_start = collection.span("quantities", sentences[1].0)?.start;
    let token = expression(r"^\s*(?P<amount>[+-]?[0-9]+)\s+(?P<unit>[\p{L}]+)\s*")
        .ok_or("missing-seeded-grammar")?;
    let join =
        expression(&format!(r"^(?:{conjunction})(?:\s+|$)")).ok_or("missing-seeded-grammar")?;
    let mut terms = Vec::new();
    let mut cursor = 0;
    let mut final_term = false;
    while cursor < values.len() && terms.len() < 32 {
        let captures = token
            .captures(&values[cursor..])
            .ok_or("unresolved-quantity")?;
        let mut fields = BTreeMap::new();
        for name in ["amount", "unit"] {
            let capture = captures.name(name).ok_or("unresolved-quantity")?;
            fields.insert(
                name.to_owned(),
                SourceSpan {
                    start: capture.start(),
                    end: capture.end(),
                },
            );
        }
        terms.push(quantity(
            &values[cursor..],
            &FrameMatch { fields },
            "amount",
            "unit",
            value_start + cursor,
            &unit,
        )?);
        cursor += captures.get(0).ok_or("unresolved-quantity")?.end();
        if cursor == values.len() {
            break;
        }
        if final_term {
            return Err("ambiguous-collection-tail");
        }
        let comma = values[cursor..].starts_with(',');
        if comma {
            cursor += 1;
        }
        while let Some(character) = values[cursor..].chars().next() {
            if !character.is_whitespace() {
                break;
            }
            cursor += character.len_utf8();
        }
        if let Some(joined) = join.find(&values[cursor..]) {
            cursor += joined.end();
            final_term = true;
        } else if !comma {
            return Err("unowned-collection-separator");
        }
        if cursor >= values.len() {
            return Err("missing-quantity");
        }
    }
    if terms.is_empty() || cursor != values.len() {
        return Err("unbounded-collection");
    }
    Ok(SupportedFrame {
        aggregation: "sum",
        relation: "le",
        terms,
        bound: bound_quantity,
        aggregation_proof: AggregateProof {
            kind: "explicit-sum-cue",
            surface: comparison.text(sentences[2].1, "aggregation")?.to_owned(),
            span: comparison.span("aggregation", sentences[2].0)?,
        },
        comparison_proof: ComparisonProof {
            role: "quantity-constraint-relation-le",
            surface: comparison.text(sentences[2].1, "relation")?.to_owned(),
            span: comparison.span("relation", sentences[2].0)?,
        },
        source: raw.to_owned(),
        source_utf8_bytes: raw.len(),
    })
}

/// Recognition includes unsupported constraints only after an explicit
/// aggregate, inclusive comparison relation and bound-reference phrase.
fn for_prompt(raw: &str) -> Option<QuantityConstraint> {
    if raw.encode_utf16().count() > 8192 {
        return None;
    }
    let prefix = role_pattern("quantity-constraint-query-prefix")?;
    let aggregate = role_pattern("quantity-constraint-aggregate")?;
    let sentence_regex = Regex::new(r"[^.?!]+(?:[.?!]|$)").ok()?;
    let sentences = sentence_regex
        .find_iter(raw)
        .map(|found| (found.start(), found.as_str()))
        .collect::<Vec<_>>();
    let &(query_start, query) = sentences.last()?;
    if !expression(&format!(r"^\s*(?:{prefix})(?:\s|$)"))?.is_match(query) {
        return None;
    }
    let aggregate_regex = expression(&format!("(?:{aggregate})"))?;
    let cue = aggregate_regex
        .find_iter(query)
        .find(|found| token_edges(query, found.start(), found.end()))?;
    let bound_names = role_pattern("quantity-constraint-bound-reference")?;
    let relation = role_pattern("quantity-constraint-relation-le")?;
    match_frame(
        query,
        "quantity-constraint-ownership-frame",
        &[("relation", &relation), ("query_ref", &bound_names)],
        false,
    )?;
    Some(
        match supported_frame(raw, &sentences, &bound_names, &relation) {
            Ok(frame) => QuantityConstraint::Supported(frame),
            Err(reason) => QuantityConstraint::Unknown(UnknownFrame {
                reason,
                aggregation_proof: AggregateProof {
                    kind: "explicit-sum-cue",
                    surface: cue.as_str().to_owned(),
                    span: SourceSpan {
                        start: query_start + cue.start(),
                        end: query_start + cue.end(),
                    },
                },
                source: raw.to_owned(),
                source_utf8_bytes: raw.len(),
            }),
        },
    )
}

pub(crate) fn owns_prompt(prompt: &str) -> bool {
    for_prompt(prompt).is_some()
}

fn decline(prompt: &str, log: &mut EventLog, reason: &str) -> SymbolicAnswer {
    log.append("quantity_constraint:unknown", reason.to_owned());
    let body = crate::seed::localized_response(
        "quantity-constraint-unknown",
        crate::language::detect(prompt).slug(),
    )
    .unwrap_or_default();
    crate::solver_handlers::finalize_simple(
        prompt,
        log,
        "quantity_constraint_unknown",
        "response:quantity-constraint-unknown",
        &body,
        0.4,
    )
}

fn exact_evaluation(expression: &str) -> Option<(i128, CalculationEvaluation)> {
    let evaluated = evaluate_calculation(expression).ok()?;
    let number = evaluated.formatted.parse::<i128>().ok()?;
    Some((number, evaluated))
}

fn record_calculation(log: &mut EventLog, evaluated: &CalculationEvaluation) {
    log.append("calculation:engine", evaluated.engine.slug());
    if let Some(lino) = &evaluated.lino {
        log.append("calculation:lino", lino.clone());
    }
    if !evaluated.steps.is_empty() {
        log.append("calculation:steps", evaluated.steps.len().to_string());
    }
}

pub(crate) fn try_answer(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let frame = for_prompt(prompt)?;
    let Ok(serialized) = serde_json::to_string(&frame) else {
        return Some(decline(prompt, log, "frame-serialization"));
    };
    log.append("quantity_constraint:frame", serialized);
    let supported = match &frame {
        QuantityConstraint::Supported(supported) => supported,
        QuantityConstraint::Unknown(unknown) => return Some(decline(prompt, log, unknown.reason)),
    };
    let expression = supported
        .terms
        .iter()
        .map(|term| term.literal.as_str())
        .collect::<Vec<_>>()
        .join(" + ");
    let difference_expression = format!("({expression}) - ({})", supported.bound.literal);
    let Some((total, total_evaluation)) = exact_evaluation(&expression) else {
        return Some(decline(prompt, log, "arithmetic-unresolved"));
    };
    let Some((difference, difference_evaluation)) = exact_evaluation(&difference_expression) else {
        return Some(decline(prompt, log, "arithmetic-unresolved"));
    };
    let holds = difference <= 0;
    let intent = if holds {
        "quantity_constraint_within"
    } else {
        "quantity_constraint_exceeds"
    };
    let total_text = total.to_string();
    let values = [
        ("total", total_text.as_str()),
        ("bound", supported.bound.literal.as_str()),
        ("unit", supported.bound.unit_surface.as_str()),
    ];
    let language = crate::language::detect(prompt).slug();
    let response = if holds {
        "quantity-constraint-within"
    } else {
        "quantity-constraint-exceeds"
    };
    let Some(body) = crate::seed::render_response(response, language, &values)
        .or_else(|| crate::seed::render_response(response, "en", &values))
    else {
        return Some(decline(prompt, log, "missing-response-template"));
    };
    log.append("quantity_constraint:verdict", holds.to_string());
    log.append("quantity_constraint:expression", expression.clone());
    log.append("quantity_constraint:difference", difference.to_string());
    log.append("calculation:request", expression);
    record_calculation(log, &total_evaluation);
    record_calculation(log, &difference_evaluation);
    Some(crate::solver_handlers::finalize_simple(
        prompt,
        log,
        intent,
        &format!("response:{response}"),
        &body,
        1.0,
    ))
}
