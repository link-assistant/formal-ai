//! Unit-incompatibility handler extracted from `solver_handlers` to keep each
//! source file under the 1000-line cap enforced by `scripts/check-file-size.rs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::{lexicon, Lexicon, Meaning, ROLE_MEASUREMENT_UNIT, ROLE_PHYSICAL_DIMENSION};
use crate::solver_handlers::finalize_simple;

/// Detect queries that ask to convert between dimensionally incompatible units.
///
/// Meters measure length; kilobytes measure data storage. These quantities
/// live in different physical dimensions and have no conversion factor, so
/// the symbolic answer must say so explicitly rather than falling through to
/// `intent:unknown`.
///
/// The units, the physical dimensions they measure, and their surface words in
/// every supported language are **not** hardcoded here — they are read from the
/// meaning lexicon (`data/seed/meanings-units.lino`), where each unit meaning is
/// `defined_by` the dimension it measures (issue #386). This code knows only the
/// *concepts* "measurement unit" and "physical dimension"; the words live once,
/// in the data, and translate to any supported language.
pub fn try_incompatible_units(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let (unit_a, dim_a, unit_b, dim_b) = detect_incompatible_unit_pair(normalized)?;
    log.append(
        "unit_incompatibility",
        format!("{unit_a}:{dim_a} vs {unit_b}:{dim_b}"),
    );
    let body = format!(
        "{unit_a} measures {dim_a}; {unit_b} measures {dim_b}. \
         These are different physical dimensions and cannot be converted into each other. \
         The incompatibility is recorded as a `unit_incompatibility` link in the network."
    );
    Some(finalize_simple(
        prompt,
        log,
        "unit_incompatibility",
        "response:unit_incompatibility",
        &body,
        1.0,
    ))
}

/// The English label of the physical dimension a `unit` measures.
///
/// Resolved through the unit meaning's `defined_by` graph: the dimension is the
/// `defined_by` target that itself plays the [`ROLE_PHYSICAL_DIMENSION`] role
/// (e.g. `meter` is `defined_by "length"` and `defined_by "unit"`, and `length`
/// carries the dimension role). The label is the dimension's English lexeme so
/// the rendered explanation reads naturally — it lives in the data, not here.
fn dimension_label<'a>(lex: &'a Lexicon, unit: &'a Meaning) -> Option<&'a str> {
    unit.defined_by
        .iter()
        .filter_map(|slug| lex.meaning(slug))
        .find(|m| m.has_role(ROLE_PHYSICAL_DIMENSION))
        .and_then(|dim| dim.word_in("en"))
}

/// Whether `unit` appears in `normalized` as a standalone word rather than as a
/// fragment of a larger word.
///
/// Issue #334: a plain `normalized.contains(unit)` matched "mb" inside
/// "nu**mb**er" and "gram" inside "pro**gram**", so the coding prompt "Write a
/// program that computes the 10th Fibonacci number" was misread as a
/// length/mass conversion and answered with a unit-incompatibility refusal. A
/// unit token only counts when both of its neighbouring characters are
/// non-alphabetic (string edge, whitespace, punctuation, or a digit such as the
/// "500" in "500mb"), so genuine units still match while embedded fragments do
/// not.
fn contains_unit_word(normalized: &str, unit: &str) -> bool {
    // Inflected languages (Russian "килобайт" -> "килобайте") and space-less
    // scripts (CJK) attach suffixes directly to the unit, so a strict word
    // boundary would reject legitimate forms. The substring false positives
    // that motivated this guard ("mb" in "number", "gram" in "program") are all
    // ASCII, so the boundary check only applies to ASCII units; non-ASCII units
    // keep the original permissive substring match.
    if !unit.is_ascii() {
        return normalized.contains(unit);
    }
    let boundary_ok = |ch: Option<char>| ch.map_or(true, |c| !c.is_alphabetic());
    let mut search_from = 0;
    while let Some(offset) = normalized[search_from..].find(unit) {
        let start = search_from + offset;
        let end = start + unit.len();
        let before = normalized[..start].chars().next_back();
        let after = normalized[end..].chars().next();
        if boundary_ok(before) && boundary_ok(after) {
            return true;
        }
        // Advance past this occurrence. `end` is always a char boundary (the
        // unit matched there), whereas `start + 1` could land inside a
        // multi-byte UTF-8 character and panic when sliced.
        search_from = end;
    }
    false
}

/// Return the first matched unit token for each of two distinct physical
/// dimensions, together with their dimension labels, or `None` if `normalized`
/// does not mention units from at least two different dimensions.
///
/// Walks every meaning that plays the [`ROLE_MEASUREMENT_UNIT`] role in lexicon
/// declaration order, keeping the first matched unit per dimension. Two units
/// that measure different dimensions cannot be converted into one another —
/// that is the incompatibility the caller reports. The result tuple is
/// `(unit_a, dim_a, unit_b, dim_b)`.
#[allow(clippy::type_complexity)]
fn detect_incompatible_unit_pair(
    normalized: &str,
) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
    let lex = lexicon();
    // (matched surface word, dimension label) — one entry per distinct
    // dimension, in lexicon order, so the rendered message is deterministic.
    let mut found: Vec<(&'static str, &'static str)> = Vec::new();
    for unit in lex.meanings_with_role(ROLE_MEASUREMENT_UNIT) {
        let Some(dim) = dimension_label(lex, unit) else {
            continue;
        };
        if found.iter().any(|(_, seen)| *seen == dim) {
            continue; // already have a unit witnessing this dimension
        }
        let mut matched: Option<&'static str> = None;
        for word in unit.words() {
            if contains_unit_word(normalized, word) {
                matched = Some(word);
                break;
            }
        }
        if let Some(word) = matched {
            found.push((word, dim));
        }
    }

    if found.len() < 2 {
        return None;
    }
    let (unit_a, dim_a) = found[0];
    let (unit_b, dim_b) = found[1];
    Some((unit_a, dim_a, unit_b, dim_b))
}
