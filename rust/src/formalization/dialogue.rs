//! Seed-driven question obligations and exact typed answer evidence.
use super::text_statements::{
    Statement, Word, WordKind, numeric_identity_prefix, seed_words, statement_terms,
    structural_rule_values,
};
use regex::Regex;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

const QUESTIONS: [(&str, &str); 6] = [
    ("where", "location"),
    ("how-many", "quantity"),
    ("why", "text"),
    ("who", "entity"),
    ("when", "time"),
    ("which", "text"),
];
const TOPICS: [&str; 6] = [
    "location",
    "experience",
    "compensation",
    "contract",
    "education",
    "availability",
];
const CURRENCIES: [&str; 3] = ["RUB", "USD", "EUR"];
const UNITS: [&str; 3] = ["year", "month", "hour"];

/// A question describes required evidence, never the addressee's personal facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Interrogative kind.
    pub kind: String,
    /// First grounded property, if any.
    pub topic: Option<String>,
    /// All grounded properties.
    pub topics: Vec<String>,
    /// Addressee or unknown.
    pub subject: String,
    /// Required answer type.
    pub answer_type: String,
    /// Explicitly requested currencies.
    pub currencies: Vec<String>,
    /// Explicit unit, if known.
    pub unit: Option<String>,
}

/// Exact UTF-8 source-token evidence, before logical numeric grouping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericSourceComponent {
    /// Original token surface.
    pub surface: String,
    /// Start byte in the clause.
    pub start: usize,
    /// Exclusive end byte in the clause.
    pub end: usize,
}

/// Exact decimal spelling avoids binary floating-point corruption or conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantity {
    /// Original numeric surface.
    pub surface: String,
    /// Decimal text with valid group separators removed.
    pub value: String,
    /// A lossless literal with no inferred numeric value when parsing is unsupported.
    pub uncertainty: Option<String>,
    /// Original numeric source words and exact byte spans.
    pub source_components: Vec<NumericSourceComponent>,
    /// Equal, at-least or at-most.
    pub comparison: String,
    /// Currency from owned vocabulary, if unambiguous.
    pub currency: Option<String>,
    /// Unit from owned vocabulary.
    pub unit: Option<String>,
    /// Recurrence period, if stated.
    pub period: Option<String>,
    /// Explicit net qualifier.
    pub net: bool,
}

/// Typed answer evidence without inventing missing knowledge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// Boolean, quantity or location.
    pub kind: String,
    /// Affirmed or denied for polar answers.
    pub polarity: Option<String>,
    /// Original typed quantities.
    pub quantities: Vec<Quantity>,
    /// Literal proper-name surfaces; no inferred geographic identity.
    pub locations: Vec<String>,
}

/// Shared metadata carried by each statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dialogue {
    /// Question obligations, if present.
    pub question: Option<Question>,
    /// Answer evidence, if present.
    pub answer: Option<Answer>,
    /// Every safely read numeric quantity.
    pub quantities: Vec<Quantity>,
}

fn normalize(text: &str) -> String {
    text.nfkc()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn word(character: Option<char>) -> bool {
    character.is_some_and(|value| value.is_alphanumeric() || value == '_')
}
fn contains(text: &str, surface: &str) -> bool {
    text.match_indices(surface).any(|(start, _)| {
        let end = start + surface.len();
        (!word(surface.chars().next()) || !word(text[..start].chars().last()))
            && (!word(surface.chars().last()) || !word(text[end..].chars().next()))
    })
}
fn mentions(text: &str, language: &str, kind: &str) -> bool {
    seed_words(language, kind)
        .iter()
        .any(|surface| contains(text, surface))
}
fn listed(text: &str, language: &str, kind: &str) -> bool {
    seed_words(language, kind)
        .iter()
        .any(|surface| text == surface)
}
fn ending(text: &str, language: &str, kind: &str) -> bool {
    seed_words(language, kind)
        .iter()
        .any(|surface| text == surface || text.ends_with(&format!(" {surface}")))
}
fn numeric_expression() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r"[+-]?[0-9]+(?:[.,][0-9]+|[ \x{00a0}\x{202f}][0-9]{3})*")
            .expect("owned numeric grammar")
    })
}

/// Canonical identity is independent of surrounding prose/Han boundaries.
fn numeric_value(surface: &str, language: &str) -> Option<String> {
    let decimal = seed_words(language, "numeric-decimal").first()?.clone();
    let grouping = seed_words(language, "numeric-group").first()?.clone();
    let primary = seed_words(language, "numeric-primary-width")
        .first()?
        .parse::<usize>()
        .ok()?;
    let secondary = seed_words(language, "numeric-secondary-width")
        .first()?
        .parse::<usize>()
        .ok()?;
    let sign = if surface.starts_with('-') { "-" } else { "" };
    let unsigned = surface.trim_start_matches(['+', '-']);
    let pieces = unsigned.split(decimal.as_str()).collect::<Vec<_>>();
    if pieces.len() > 2
        || (pieces.len() == 2
            && (pieces[1].is_empty() || !pieces[1].bytes().all(|ch| ch.is_ascii_digit())))
    {
        return None;
    }
    let integer = pieces[0]
        .replace(grouping.as_str(), " ")
        .replace(['\u{00a0}', '\u{202f}'], " ");
    let groups = integer.split(' ').collect::<Vec<_>>();
    if groups
        .iter()
        .any(|group| group.is_empty() || !group.bytes().all(|ch| ch.is_ascii_digit()))
    {
        return None;
    }
    if groups.len() > 1
        && (groups.last()?.len() != primary
            || groups[0].len() > secondary
            || groups[1..groups.len() - 1]
                .iter()
                .any(|group| group.len() != secondary))
    {
        return None;
    }
    Some(format!(
        "{sign}{}{}",
        groups.join(""),
        if pieces.len() == 2 {
            format!(".{}", pieces[1])
        } else {
            String::new()
        }
    ))
}

fn numeric_components(surface: &str, start: usize) -> Vec<NumericSourceComponent> {
    static COMPONENT: OnceLock<Regex> = OnceLock::new();
    COMPONENT
        .get_or_init(|| Regex::new(r"[0-9]+(?:[.,][0-9]+)*").expect("numeric source token grammar"))
        .find_iter(surface)
        .map(|component| NumericSourceComponent {
            surface: component.as_str().to_owned(),
            start: start + component.start(),
            end: start + component.end(),
        })
        .collect()
}

/// Read finite numeric syntax without conversion between currencies or units.
#[must_use]
pub fn dialogue_quantities(text: &str, language: &str) -> Vec<Quantity> {
    let mut result = Vec::new();
    let mut previous = 0;
    let all = normalize(text);
    for matched in numeric_expression().find_iter(text) {
        let start = matched.start();
        let end = matched.end();
        if word(text[..start].chars().last()) || word(text[end..].chars().next()) {
            continue;
        }
        let tail = &text[end..];
        if tail.starts_with(['.', ',']) && tail.chars().nth(1).is_some_and(|ch| ch.is_ascii_digit())
        {
            continue;
        }
        let surface = matched.as_str();
        let canonical = numeric_value(surface, language);
        let uncertainty = canonical
            .is_none()
            .then(|| "unsupported-locale-number-format".to_owned());
        let value = canonical.unwrap_or_else(|| surface.to_owned());
        let prefix = normalize(&text[previous..start]);
        let found = CURRENCIES
            .iter()
            .filter(|code| {
                let kind = format!("currency-{}", code.to_lowercase());
                ending(&prefix, language, &kind)
                    || seed_words(language, &kind)
                        .iter()
                        .any(|alias| normalize(tail).starts_with(alias))
            })
            .copied()
            .collect::<Vec<_>>();
        let context = CURRENCIES
            .iter()
            .filter(|code| mentions(&all, language, &format!("currency-{}", code.to_lowercase())))
            .copied()
            .collect::<Vec<_>>();
        let currency = if found.len() == 1 {
            Some(found[0].to_owned())
        } else if context.len() == 1 {
            Some(context[0].to_owned())
        } else {
            None
        };
        let mut constraint_prefix = prefix;
        for code in CURRENCIES {
            for alias in seed_words(language, &format!("currency-{}", code.to_lowercase())) {
                if constraint_prefix.ends_with(&alias) {
                    constraint_prefix.truncate(constraint_prefix.len() - alias.len());
                    constraint_prefix = constraint_prefix.trim().to_owned();
                }
            }
        }
        let comparison = if ending(&constraint_prefix, language, "amount-minimum") {
            "at-least"
        } else if ending(&constraint_prefix, language, "amount-maximum") {
            "at-most"
        } else {
            "equal"
        };
        let unit = UNITS
            .iter()
            .find(|unit| mentions(&all, language, &format!("unit-{unit}")))
            .map(|unit| (*unit).to_owned());
        let period = UNITS
            .iter()
            .find(|unit| mentions(&all, language, &format!("period-{unit}")))
            .map(|unit| (*unit).to_owned());
        result.push(Quantity {
            surface: surface.to_owned(),
            value,
            uncertainty,
            source_components: numeric_components(surface, start),
            comparison: comparison.to_owned(),
            currency,
            unit,
            period,
            net: mentions(&all, language, "net-qualifier"),
        });
        previous = end;
    }
    result
}

/// Formalize dialogue obligations using linguistic vocabulary, never task answers.
#[must_use]
pub fn formalize_dialogue(text: &str, language: &str) -> Dialogue {
    formalize_dialogue_in_context(text, language, false)
}

/// A relative continuation is declarative unless explicitly marked as a question.
#[must_use]
pub fn formalize_dialogue_in_context(
    text: &str,
    language: &str,
    declarative_continuation: bool,
) -> Dialogue {
    let normalized = normalize(text);
    let clean = normalized.trim_end_matches(['.', '!', '?']).trim();
    let found_topics = TOPICS
        .iter()
        .filter(|topic| mentions(&normalized, language, &format!("topic-{topic}")))
        .map(|topic| (*topic).to_owned())
        .collect::<Vec<_>>();
    let quantities = dialogue_quantities(text, language);
    let interrogative = QUESTIONS
        .iter()
        .find(|(kind, _)| mentions(&normalized, language, &format!("question-{kind}")));
    let starts_question = QUESTIONS.iter().any(|(kind, _)| {
        seed_words(language, &format!("question-{kind}"))
            .iter()
            .any(|surface| normalized == *surface || normalized.starts_with(&format!("{surface} ")))
    });
    let auxiliary = seed_words(language, "question-auxiliary")
        .iter()
        .any(|surface| normalized.starts_with(&format!("{surface} ")));
    let marked = text.contains(['?', '？'])
        || starts_question
        || auxiliary
        || mentions(&normalized, language, "question-particle");
    let bare_topic = found_topics
        .iter()
        .any(|topic| listed(clean, language, &format!("topic-{topic}")));
    let question = if !declarative_continuation && (marked || bare_topic) {
        let mut kind = interrogative.map_or("yes-no", |(kind, _)| *kind);
        if found_topics.iter().any(|topic| topic == "compensation")
            && (kind == "which" || bare_topic)
        {
            kind = "how-many";
        }
        let answer_type = if kind == "yes-no" {
            "boolean"
        } else {
            QUESTIONS
                .iter()
                .find(|(value, _)| *value == kind)
                .map_or("text", |(_, value)| *value)
        };
        Some(Question {
            kind: kind.to_owned(),
            topic: found_topics.first().cloned(),
            topics: found_topics,
            subject: if mentions(&normalized, language, "addressee") {
                "addressee"
            } else {
                "unknown"
            }
            .to_owned(),
            answer_type: answer_type.to_owned(),
            currencies: CURRENCIES
                .iter()
                .filter(|code| {
                    mentions(
                        &normalized,
                        language,
                        &format!("currency-{}", code.to_lowercase()),
                    )
                })
                .map(|code| (*code).to_owned())
                .collect(),
            unit: UNITS
                .iter()
                .find(|unit| mentions(&normalized, language, &format!("unit-{unit}")))
                .map(|unit| (*unit).to_owned()),
        })
    } else {
        None
    };
    let mut answer = None;
    if question.is_none() {
        let polarity = if listed(clean, language, "answer-yes") {
            Some("affirmed")
        } else if listed(clean, language, "answer-no") {
            Some("denied")
        } else {
            None
        };
        if let Some(polarity) = polarity {
            answer = Some(Answer {
                kind: "boolean".to_owned(),
                polarity: Some(polarity.to_owned()),
                quantities: Vec::new(),
                locations: Vec::new(),
            });
        } else if !quantities.is_empty() {
            answer = Some(Answer {
                kind: "quantity".to_owned(),
                polarity: None,
                quantities: quantities.clone(),
                locations: Vec::new(),
            });
        } else if seed_words(language, "location-prefix")
            .iter()
            .any(|prefix| normalized.starts_with(&format!("{prefix} ")))
        {
            let locations = text
                .split(|ch: char| !ch.is_alphabetic())
                .filter(|part| {
                    part.chars().next().is_some_and(char::is_uppercase)
                        && part.chars().count() > 1
                        && !seed_words(language, "location-prefix").contains(&normalize(part))
                })
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if !locations.is_empty() {
                answer = Some(Answer {
                    kind: "location".to_owned(),
                    polarity: None,
                    quantities: Vec::new(),
                    locations,
                });
            }
        }
    }
    Dialogue {
        question,
        answer,
        quantities,
    }
}

/// Concrete property obligations must not be claimed by generic self-description.
#[must_use]
pub fn blocks_self_description(text: &str, language: &str) -> bool {
    formalize_dialogue(text, language)
        .question
        .is_some_and(|question| question.topic.is_some())
}

/// Preserve validated grouped numeric surfaces as a single numeric word.
#[must_use]
pub fn merge_numeric_words(words: &[Word], quantities: &[Quantity], language: &str) -> Vec<Word> {
    let words = words
        .iter()
        .cloned()
        .map(|mut word| {
            if word.kind == WordKind::Number {
                word.id = numeric_value(&word.surface, language).map_or_else(
                    || format!("{}{}", numeric_identity_prefix(true), word.surface),
                    |value| format!("{}{value}", numeric_identity_prefix(false)),
                );
            }
            word
        })
        .collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    for quantity in quantities {
        let unsigned = quantity.surface.trim_start_matches(['+', '-']);
        let pieces = unsigned
            .split([' ', '\u{00a0}', '\u{202f}'])
            .collect::<Vec<_>>();
        while index < words.len() && words[index].surface != pieces[0] {
            output.push(words[index].clone());
            index += 1;
        }
        if index >= words.len() {
            break;
        }
        if pieces.iter().enumerate().all(|(offset, piece)| {
            words
                .get(index + offset)
                .is_some_and(|word| word.surface == *piece)
        }) {
            output.push(Word {
                surface: quantity.surface.clone(),
                kind: WordKind::Number,
                id: format!(
                    "{}{}",
                    numeric_identity_prefix(quantity.uncertainty.is_some()),
                    quantity.value
                ),
            });
            index += pieces.len();
        }
    }
    output.extend_from_slice(&words[index..]);
    output
}

/// Typed compatibility is evidence, never authority to substitute unknown answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compatibility {
    /// Compatible, incompatible or unknown.
    pub status: String,
    /// Missing or contradictory evidence.
    pub reason: String,
}

/// Reject type/currency contradictions and retain unknown units as unknown.
#[must_use]
pub fn answer_compatibility(question: Option<&Question>, answer: Option<&Answer>) -> Compatibility {
    let outcome = |status: &str, reason: &str| Compatibility {
        status: status.to_owned(),
        reason: reason.to_owned(),
    };
    let (Some(question), Some(answer)) = (question, answer) else {
        return outcome(
            "unknown",
            &structural_rule_values("answer-compatibility", "missing-reason")[0],
        );
    };
    if question.answer_type != answer.kind {
        return outcome("incompatible", "answer kind");
    }
    if answer.kind == "quantity" {
        if answer.quantities.len() != 1 {
            return outcome("unknown", "multiple amounts");
        }
        let amount = &answer.quantities[0];
        if let Some(reason) = &amount.uncertainty {
            return Compatibility {
                status: "unknown".to_owned(),
                reason: reason.clone(),
            };
        }
        if !question.currencies.is_empty() {
            let Some(currency) = &amount.currency else {
                return outcome("unknown", "missing currency");
            };
            if !question.currencies.contains(currency) {
                return outcome("incompatible", "currency");
            }
        }
        if question.unit.is_some() && question.unit != amount.unit {
            return outcome(
                if amount.unit.is_some() {
                    "incompatible"
                } else {
                    "unknown"
                },
                "unit",
            );
        }
    }
    outcome("compatible", "typed evidence")
}

#[cfg(test)]
#[path = "../../tests/source/source_tests/formalization/dialogue.rs"]
mod tests;

/// Page term-share historically counts source words. Keep that basis when
/// multiple original numeric words become one logical amount.
#[must_use]
pub fn source_term_count(statement: &Statement) -> usize {
    let mut remaining = statement.dialogue.quantities.iter().collect::<Vec<_>>();
    statement_terms(statement)
        .iter()
        .map(|term| {
            if term.kind != WordKind::Number {
                return 1;
            }
            if let Some(index) = remaining
                .iter()
                .position(|quantity| quantity.surface == term.surface)
            {
                remaining.remove(index).source_components.len().max(1)
            } else {
                1
            }
        })
        .sum()
}

/// One locale-aware entry point shared by plain/Han clause tokenization.
#[must_use]
pub fn read_numeric_words(words: &[Word], text: &str, language: &str) -> Vec<Word> {
    merge_numeric_words(words, &dialogue_quantities(text, language), language)
}
