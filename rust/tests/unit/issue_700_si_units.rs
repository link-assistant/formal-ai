//! Issue #700 (E58): conversions between units the pairwise seed does
//! not state, via SI dimension algebra.
//!
//! The reported gap: "5 km in miles", "how many watts is 3 hp", psi to
//! pascals — every pair without an explicit `conversion` record in
//! data/seed/meanings-units.lino fell through the #1176 engine. The
//! dimension table (`data/seed/si-unit-dimensions.lino`) plus
//! `rust/src/si_units.rs` derive those conversions through the coherent
//! SI unit of each dimension, in exact rational arithmetic, and fail
//! honestly (named unknown unit, or incompatible dimensions) where no
//! honest number exists.
//!
//! These tests pin: the seed loads completely; the algebra (multiply,
//! invert, round-trip) holds; 40 conversions across every dimension in
//! the table compute the right values; surfaces in English, Russian,
//! Hindi, and Chinese resolve to canonical units so multilingual prompts
//! convert; unknown units and dimension mismatches fail by name.

use formal_ai::event_log::EventLog;
use formal_ai::si_units::{
    BASE_SYMBOLS, Dimension, SiConversion, base_dimensions, convert_through_si, note_unknown_unit,
    si_units, unit_named_by,
};

/// Assert a conversion equals an expected value within 1e-9 (the seed's
/// factors are exact decimals; only the comparison is floating-point).
fn assert_converts(value: i128, from: &str, to: &str, expected: f64) {
    let outcome = convert_through_si(value, 1, from, to);
    let got = outcome
        .as_f64()
        .unwrap_or_else(|| panic!("{from} to {to} did not convert: {outcome:?}"));
    assert!(
        (got - expected).abs() < 1e-9,
        "{value} {from} in {to}: got {got}, expected {expected}"
    );
}

#[test]
fn the_seed_loads_with_base_dimensions_and_units() {
    let bases = base_dimensions();
    assert_eq!(bases.len(), 7, "the seven SI base dimensions");
    for symbol in BASE_SYMBOLS {
        assert!(
            bases.iter().any(|base| base.symbol == symbol),
            "base dimension {symbol} missing"
        );
    }
    let units = si_units();
    assert!(
        units.len() >= 60,
        "the dimension table carries at least 60 units, got {}",
        units.len()
    );
    for entry in &units {
        assert!(
            entry.factor_num > 0 && entry.factor_den > 0,
            "{} has a non-positive factor",
            entry.unit
        );
    }
}

#[test]
fn every_dimension_in_the_table_parses_and_round_trips() {
    for entry in si_units() {
        let rendered = entry.dimension.as_string();
        assert_eq!(
            Dimension::parse(&rendered),
            Some(entry.dimension),
            "{} dimension {} must round-trip",
            entry.unit,
            rendered
        );
    }
}

#[test]
fn dimension_algebra_holds_for_products_and_inverses() {
    let speed = Dimension::parse("LT-1").unwrap();
    let time = Dimension::parse("T").unwrap();
    assert_eq!(
        speed.multiply(&time).unwrap(),
        Dimension::parse("L").unwrap()
    );
    assert_eq!(
        time.inverse().unwrap(),
        Dimension::parse("T-1").unwrap(),
        "frequency is inverted time"
    );
    let power = Dimension::parse("L2MT-3").unwrap();
    let energy = power.multiply(&time).unwrap();
    assert_eq!(energy, Dimension::parse("L2MT-2").unwrap());
}

/// 40 conversions, every dimension family in the table. Expected values
/// are computed from the defining constants (international yard and
/// pound agreements, standard atmosphere and torr, IT calorie, mechanical
/// horsepower), not from the code under test.
#[test]
fn forty_conversions_through_si() {
    // Length.
    assert_converts(5, "kilometer", "mile", 5.0 * 1000.0 / 1609.344);
    assert_converts(6, "foot", "meter", 6.0 * 0.3048);
    assert_converts(1, "inch", "millimeter", 25.4);
    assert_converts(1, "nautical_mile", "kilometer", 1.852);
    assert_converts(1, "astronomical_unit", "kilometer", 149597870.700);
    assert_converts(1, "light_year", "kilometer", 9460730472580.8);
    assert_converts(3, "yard", "meter", 3.0 * 0.9144);
    // Mass.
    assert_converts(150, "pound", "kilogram", 150.0 * 0.45359237);
    assert_converts(1, "ounce", "gram", 28.349523125);
    assert_converts(1, "stone", "kilogram", 6.35029318);
    assert_converts(1, "us_ton", "tonne", 0.90718474);
    assert_converts(1, "long_ton", "tonne", 1.0160469088);
    // Time.
    assert_converts(5, "hour", "minute", 300.0);
    assert_converts(1, "week", "day", 7.0);
    assert_converts(1, "julian_year", "day", 365.25);
    // Speed.
    assert_converts(90, "kilometer_per_hour", "meter_per_second", 25.0);
    assert_converts(60, "mile_per_hour", "meter_per_second", 60.0 * 0.44704);
    assert_converts(
        100,
        "mile_per_hour",
        "kilometer_per_hour",
        100.0 * 0.44704 / (1000.0 / 3600.0),
    );
    assert_converts(10, "knot", "kilometer_per_hour", 18.52);
    // Power.
    assert_converts(3, "horsepower", "watt", 3.0 * 745.699872);
    assert_converts(1, "horsepower", "kilowatt", 0.745699872);
    assert_converts(1, "horsepower_metric", "watt", 735.49875);
    // Energy.
    assert_converts(2000, "calorie", "kilojoule", 8.368);
    assert_converts(1, "kilocalorie", "kilojoule", 4.184);
    assert_converts(1, "watt_hour", "joule", 3600.0);
    assert_converts(1, "kilowatt_hour", "kilojoule", 3600.0);
    assert_converts(1, "btu", "joule", 1055.05585262);
    // Force.
    assert_converts(70, "kilogram_force", "newton", 70.0 * 9.80665);
    assert_converts(1000, "dyne", "newton", 0.01);
    assert_converts(1, "pound_force", "newton", 4.4482216152605);
    // Pressure.
    assert_converts(1, "atmosphere", "pascal", 101325.0);
    assert_converts(1, "atmosphere", "bar", 1.01325);
    assert_converts(32, "psi", "kilopascal", 32.0 * 6894.757293168 / 1000.0);
    assert_converts(1, "psi", "torr", 6894.757293168 / (101325.0 / 760.0));
    // Area.
    assert_converts(1, "acre", "hectare", 0.40468564224);
    assert_converts(1, "square_mile", "square_kilometer", 2.589988110336);
    assert_converts(1, "hectare", "square_meter", 10000.0);
    assert_converts(1, "square_foot", "square_meter", 0.09290304);
    // Volume.
    assert_converts(2, "liter", "us_gallon", 2.0 / 3.785411784);
    assert_converts(1, "imperial_gallon", "liter", 4.54609);
    assert_converts(2, "us_cup", "milliliter", 2.0 * 236.5882365);
    // Data and angle (dimensionless families).
    assert_converts(1, "gibibyte", "mebibyte", 1024.0);
    assert_converts(1, "kibibyte", "byte", 1024.0);
    let pi = std::f64::consts::PI;
    assert_converts(180, "degree", "radian", pi);
}

#[test]
fn exact_conversions_stay_rational() {
    let outcome = convert_through_si(3, 1, "horsepower", "watt");
    assert_eq!(
        outcome,
        SiConversion::Converted {
            // 2237.099616 reduced: 2237099616/1000000 → 69909363/31250.
            value_num: 69909363,
            value_den: 31250
        }
    );
    let outcome = convert_through_si(90, 1, "kilometer_per_hour", "meter_per_second");
    assert_eq!(
        outcome,
        SiConversion::Converted {
            value_num: 25,
            value_den: 1
        }
    );
}

#[test]
fn incompatible_dimensions_fail_instead_of_fabricating_a_number() {
    let outcome = convert_through_si(5, 1, "meter", "second");
    assert_eq!(
        outcome,
        SiConversion::Incompatible {
            from: Dimension::parse("L").unwrap(),
            to: Dimension::parse("T").unwrap(),
        }
    );
    // Joules to watts is energy to power: also incompatible, also named.
    assert!(matches!(
        convert_through_si(1, 1, "joule", "watt"),
        SiConversion::Incompatible { .. }
    ));
    assert!(
        convert_through_si(5, 1, "meter", "second")
            .as_f64()
            .is_none()
    );
}

#[test]
fn unknown_units_are_named_gaps_and_log_events() {
    let outcome = convert_through_si(1, 1, "furlong", "meter");
    assert_eq!(outcome, SiConversion::UnknownUnit("furlong".to_owned()));
    let mut log = EventLog::new();
    note_unknown_unit(&mut log, "furlong");
    assert_eq!(
        log.first_of("si_units:unknown_unit")
            .map(|event| event.payload.as_str()),
        Some("furlong"),
        "the gap must be a named event, not silence"
    );
}

/// Multilingual recognition: surfaces in four languages resolve to the
/// canonical units, so a prompt written in any of them converts. The
/// dispatch entry that routes prompts here is the maintainer's lift;
/// these tests pin the recognition and arithmetic beneath it.
#[test]
fn surfaces_in_four_languages_resolve_to_canonical_units() {
    let expected = [
        // English
        ("miles", "mile"),
        ("inches", "inch"),
        ("gallons", "us_gallon"),
        ("horsepower", "horsepower_mechanical"),
        // Russian
        ("километров", "kilometer"),
        ("метров", "meter"),
        ("миль", "mile"),
        ("ватт", "watt"),
        ("часов", "hour"),
        ("фунтов", "pound"),
        ("калорий", "calorie"),
        ("гектара", "hectare"),
        ("лошадиных сил", "horsepower_mechanical"),
        // Hindi
        ("मील", "mile"),
        ("किलोमीटर", "kilometer"),
        ("घंटे", "hour"),
        ("कैलोरी", "calorie"),
        ("अश्वशक्ति", "horsepower_mechanical"),
        ("पाउंड", "pound"),
        // Chinese
        ("公里", "kilometer"),
        ("米", "meter"),
        ("小时", "hour"),
        ("马力", "horsepower_mechanical"),
        ("升", "liter"),
        ("卡路里", "calorie"),
        ("英里", "mile"),
    ];
    for (surface, unit) in expected {
        assert_eq!(
            unit_named_by(surface),
            Some(unit.to_owned()),
            "surface {surface} must resolve to {unit}"
        );
    }
}

/// Four full prompts — Russian, English, Hindi, Chinese — resolve both
/// unit surfaces they contain and convert the stated value through SI.
#[test]
fn multilingual_prompts_resolve_and_convert() {
    let cases: [(&str, i128, &str, &str, f64); 4] = [
        (
            "Сколько метров в 5 километрах?",
            5,
            "километрах",
            "метров",
            5000.0,
        ),
        (
            "How many watts is 3 horsepower?",
            3,
            "horsepower",
            "watts",
            3.0 * 745.699872,
        ),
        (
            "5 मील कितने किलोमीटर हैं?",
            5,
            "मील",
            "किलोमीटर",
            5.0 * 1609.344 / 1000.0,
        ),
        ("3 马力是多少瓦?", 3, "马力", "瓦", 3.0 * 745.699872),
    ];
    for (prompt, value, from_surface, to_surface, expected) in cases {
        assert!(
            prompt.contains(from_surface),
            "case surface must occur in its prompt"
        );
        let from = unit_named_by(from_surface).expect("from surface resolves");
        let to = unit_named_by(to_surface).expect("to surface resolves");
        let outcome = convert_through_si(value, 1, &from, &to);
        let got = outcome
            .as_f64()
            .unwrap_or_else(|| panic!("{prompt} did not convert: {outcome:?}"));
        assert!(
            (got - expected).abs() < 1e-9,
            "{prompt}: got {got}, expected {expected}"
        );
    }
}

#[test]
fn the_engine_name_is_stable_for_evidence_links() {
    assert_eq!(formal_ai::si_units::ENGINE_NAME, "si-dimension-algebra");
}
