//! Sentence-local request recognition and exact numeral rendering.
//! JavaScript twin: formal_ai_worker_units.js (unitConversionSpan and labels).
use super::{Decimal, ROLE_UNIT_CONVERSION_QUESTION, UnitMention, contains_term};
use crate::seed::parser;

pub(super) fn surfaces(field: &str) -> Vec<String> {
    let text = crate::seed::seed_files()
        .into_iter()
        .find(|(path, _)| *path == "data/seed/meanings-units.lino")
        .map_or("", |(_, text)| text);
    let root = parser::parse_lino(text);
    root.children
        .iter()
        .flat_map(|section| &section.children)
        .find(|row| row.name == ROLE_UNIT_CONVERSION_QUESTION)
        .map(|row| {
            row.children
                .iter()
                .filter(|child| child.name == field)
                .map(|child| child.id.trim_matches('"').to_owned())
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn explicit_conversion(text: &str) -> bool {
    let numeric = text.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-');
    surfaces("request-start")
        .iter()
        .any(|word| contains_term(text, word) && (text.starts_with(word.as_str()) || numeric))
}

pub(crate) fn conversion_span(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut clauses = Vec::new();
    let mut start = 0;
    for (at, ch) in text.char_indices() {
        let decimal = ch == '.'
            && at > 0
            && bytes[at - 1].is_ascii_digit()
            && bytes.get(at + 1).is_some_and(u8::is_ascii_digit);
        if matches!(ch, '\n' | ';' | '!' | '?' | '？') || (ch == '.' && !decimal) {
            clauses.push(text[start..at].trim());
            start = at + ch.len_utf8();
        }
    }
    clauses.push(text[start..].trim());
    let candidates: Vec<_> = clauses
        .into_iter()
        .map(|clause| clause.trim_start_matches(['¿', '¡']).trim())
        .filter(|clause| {
            explicit_conversion(clause)
                || clause.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-')
        })
        .collect();
    let explicit: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|text| explicit_conversion(text))
        .collect();
    match explicit.as_slice() {
        [span] => Some(span),
        [] if candidates.len() == 1 => Some(candidates[0]),
        _ => None,
    }
}

pub(super) fn rational_text(numerator: i128, denominator: i128) -> String {
    let decimal =
        Decimal::parse(&numerator.to_string()).and_then(|value| value.mul_div(1, denominator, 3));
    match decimal {
        Some((value, true)) => value.render(),
        _ => format!("{numerator}/{denominator}"),
    }
}

pub(super) fn local_number(text: &str, language: &str) -> String {
    if language == "ru" {
        text.replace('.', ",")
    } else {
        text.to_owned()
    }
}

pub(super) fn numeral_label(
    mention: &UnitMention,
    numerator: i128,
    denominator: i128,
    language: &str,
) -> String {
    let count = numerator.unsigned_abs() / denominator.unsigned_abs();
    let form = if numerator % denominator != 0 {
        "fraction"
    } else if language == "ru" {
        let teen = (11..=14).contains(&(count % 100));
        match (teen, count % 10) {
            (false, 1) => "one",
            (false, 2..=4) => "few",
            _ => "many",
        }
    } else if count == 1 {
        "one"
    } else {
        "many"
    };
    let text = crate::seed::seed_files()
        .into_iter()
        .find(|(path, _)| *path == "data/seed/si-unit-dimensions.lino")
        .map_or("", |(_, text)| text);
    let root = parser::parse_lino(text);
    let key = format!("{language}.{form} ");
    for section in &root.children {
        for row in &section.children {
            if row.name != "si_unit" || row.find_child_value("unit") != mention.slug {
                continue;
            }
            if let Some(label) = row
                .children
                .iter()
                .find(|child| child.name == "numeral-form" && child.id.starts_with(&key))
            {
                return label.id[key.len()..].trim_matches('"').to_owned();
            }
        }
    }
    mention.surface.clone()
}
