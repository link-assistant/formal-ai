//! Unit-incompatibility handler extracted from `solver_handlers` to keep each
//! source file under the 1000-line cap enforced by `scripts/check-file-size.rs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver_handlers::finalize_simple;

/// Detect queries that ask to convert between dimensionally incompatible units.
///
/// Meters measure length; kilobytes measure data storage. These quantities
/// live in different physical dimensions and have no conversion factor, so
/// the symbolic answer must say so explicitly rather than falling through to
/// `intent:unknown`.
///
/// Supports English and Russian surface forms (the reported prompt was Russian).
pub fn try_incompatible_units(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let (unit_a, unit_b) = detect_incompatible_unit_pair(normalized)?;
    let (dim_a, dim_b) = (dimension_of(unit_a), dimension_of(unit_b));
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

/// Length units (metre family).
const LENGTH_UNITS: &[&str] = &[
    "meter",
    "metre",
    "meters",
    "metres",
    "km",
    "kilometer",
    "kilometre",
    "cm",
    "centimeter",
    "centimetre",
    "mm",
    "millimeter",
    "millimetre",
    "метр",
    "метра",
    "метров",
    "километр",
    "сантиметр",
    "миллиметр",
];

/// Data-storage units (byte family).
const DATA_UNITS: &[&str] = &[
    "byte",
    "bytes",
    "kilobyte",
    "kilobytes",
    "kb",
    "megabyte",
    "megabytes",
    "mb",
    "gigabyte",
    "gigabytes",
    "gb",
    "terabyte",
    "terabytes",
    "tb",
    "bit",
    "bits",
    "байт",
    "байта",
    "байтов",
    "килобайт",
    "мегабайт",
    "гигабайт",
    "терабайт",
];

/// Mass units.
const MASS_UNITS: &[&str] = &[
    "kilogram",
    "kilograms",
    "kg",
    "gram",
    "grams",
    "pound",
    "pounds",
    "килограмм",
    "грамм",
];

/// Time units.
const TIME_UNITS: &[&str] = &[
    "second",
    "seconds",
    "minute",
    "minutes",
    "hour",
    "hours",
    "секунда",
    "секунды",
    "секунд",
    "минута",
    "минуты",
    "минут",
    "час",
    "часа",
    "часов",
];

/// Temperature units.
const TEMPERATURE_UNITS: &[&str] = &[
    "celsius",
    "fahrenheit",
    "kelvin",
    "цельсий",
    "фаренгейт",
    "кельвин",
];

fn dimension_of(unit: &str) -> &'static str {
    if LENGTH_UNITS.contains(&unit) {
        return "length";
    }
    if DATA_UNITS.contains(&unit) {
        return "data storage";
    }
    if MASS_UNITS.contains(&unit) {
        return "mass";
    }
    if TIME_UNITS.contains(&unit) {
        return "time";
    }
    if TEMPERATURE_UNITS.contains(&unit) {
        return "temperature";
    }
    "unknown"
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

/// Return two unit tokens from `normalized` that belong to different dimensions,
/// or `None` if no incompatible pair is found.
fn detect_incompatible_unit_pair(normalized: &str) -> Option<(&'static str, &'static str)> {
    let all_units: &[(&[&str], &'static str)] = &[
        (LENGTH_UNITS, "length"),
        (DATA_UNITS, "data storage"),
        (MASS_UNITS, "mass"),
        (TIME_UNITS, "time"),
        (TEMPERATURE_UNITS, "temperature"),
    ];

    let mut found: Vec<(&'static str, &'static str)> = Vec::new();
    for (units, dim) in all_units {
        for unit in *units {
            if contains_unit_word(normalized, unit) && !found.iter().any(|(_, d)| d == dim) {
                found.push((unit, dim));
            }
        }
    }

    if found.len() < 2 {
        return None;
    }
    let (unit_a, dim_a) = found[0];
    let (unit_b, dim_b) = found[1];
    if dim_a == dim_b {
        return None;
    }
    Some((unit_a, unit_b))
}
