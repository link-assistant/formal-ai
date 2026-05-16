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
            if normalized.contains(unit) && !found.iter().any(|(_, d)| d == dim) {
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
