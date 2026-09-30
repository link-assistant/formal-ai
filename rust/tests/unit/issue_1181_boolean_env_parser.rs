//! Regression coverage for issue #1181: one boolean spelling table for every
//! `FORMAL_AI_*` environment switch.
//!
//! The defect: `FORMAL_AI_SILENT=1 formal-ai chat …` failed with
//! `invalid value '1' for '--silent' [possible values: true, false]`, because
//! the `--silent` argument only accepted clap's literal `true`/`false` while
//! every ad-hoc `std::env::var` read site invented its own spellings. The fix
//! routes all of them through `formal_ai::cli_env`.
//!
//! These tests are hermetic: they set variables through `temp-env` (edition
//! 2024 made `std::env::set_var` unsafe and this crate forbids unsafe code)
//! and use a synthetic variable name for the pure helper cases so they cannot
//! collide with any other test reading a real switch.

use formal_ai::cli_env::{bool_env_value_parser, flag_disabled, flag_enabled, parse_bool_env};
use formal_ai::dialog_log;
use formal_ai::memory_sync;

/// A variable no other test reads, so the flag helper cases are hermetic.
const SYNTHETIC_SWITCH: &str = "FORMAL_AI_TEST_1181_SWITCH";

#[test]
fn parse_bool_env_accepts_every_documented_spelling() {
    // Exact-answer documentation of the whole table, per the tests-as-docs
    // rule: a reader sees precisely what each spelling means.
    for (value, parsed) in [
        ("1", Some(true)),
        ("true", Some(true)),
        ("yes", Some(true)),
        ("on", Some(true)),
        ("0", Some(false)),
        ("false", Some(false)),
        ("no", Some(false)),
        ("off", Some(false)),
        // Case-insensitive…
        ("TRUE", Some(true)),
        ("Yes", Some(true)),
        ("ON", Some(true)),
        ("FALSE", Some(false)),
        ("No", Some(false)),
        ("OFF", Some(false)),
        // …and tolerant of surrounding whitespace.
        ("  1  ", Some(true)),
        ("\toff\t", Some(false)),
    ] {
        assert_eq!(parse_bool_env(value), parsed, "value {value:?}");
    }
}

#[test]
fn parse_bool_env_rejects_unrecognised_values_instead_of_guessing() {
    // Garbage is `None`, never silently coerced: the caller keeps the
    // documented default rather than guessing what "maybe" means.
    for value in ["", "  ", "maybe", "2", "y", "t", "enabled", "01", "1.0", "on-off"] {
        assert_eq!(parse_bool_env(value), None, "value {value:?}");
    }
}

#[test]
fn bool_env_value_parser_maps_spellings_to_clap_results() {
    assert_eq!(bool_env_value_parser("1"), Ok(true));
    assert_eq!(bool_env_value_parser("yes"), Ok(true));
    assert_eq!(bool_env_value_parser("0"), Ok(false));
    assert_eq!(bool_env_value_parser("OFF"), Ok(false));

    let error = bool_env_value_parser("maybe").expect_err("garbage must be an error");
    assert!(
        error.contains("expected one of"),
        "the error states what is accepted: {error}"
    );
}

#[test]
fn flag_enabled_is_true_only_for_true_spellings() {
    for (value, enabled) in [
        (None, false),
        (Some("1"), true),
        (Some("on"), true),
        (Some("YES"), true),
        (Some("0"), false),
        (Some("off"), false),
        // Unrecognised values never turn an opt-in switch on.
        (Some("maybe"), false),
    ] {
        temp_env::with_var(SYNTHETIC_SWITCH, value, || {
            assert_eq!(flag_enabled(SYNTHETIC_SWITCH), enabled, "value {value:?}");
        });
    }
}

#[test]
fn flag_disabled_is_true_only_for_false_spellings() {
    for (value, disabled) in [
        (None, false),
        (Some("0"), true),
        (Some("off"), true),
        (Some("NO"), true),
        (Some("1"), false),
        (Some("on"), false),
        // Unrecognised values never disable a default-on feature.
        (Some("maybe"), false),
    ] {
        temp_env::with_var(SYNTHETIC_SWITCH, value, || {
            assert_eq!(flag_disabled(SYNTHETIC_SWITCH), disabled, "value {value:?}");
        });
    }
}

#[test]
fn silent_switch_now_accepts_one_as_the_issue_reported() {
    // The exact evidence from issue #1181: `FORMAL_AI_SILENT=1` used to be
    // rejected by the CLI and ignored by `verbose_enabled`'s old
    // `!= Ok("1")`-style reads; through the shared parser it silences.
    dialog_log::configure_verbose(true);
    for (value, verbose) in [
        (None, true),
        (Some("1"), false),
        (Some("true"), false),
        (Some("yes"), false),
        (Some("ON"), false),
        (Some("0"), true),
        (Some("off"), true),
        // Garbage keeps the documented default (verbose stays on).
        (Some("maybe"), true),
    ] {
        temp_env::with_var("FORMAL_AI_SILENT", value, || {
            assert_eq!(dialog_log::verbose_enabled(), verbose, "value {value:?}");
        });
    }
}

#[test]
fn record_chat_opt_out_accepts_every_false_spelling() {
    for (value, recording) in [
        (None, true),
        (Some("0"), false),
        (Some("false"), false),
        (Some("off"), false),
        (Some("no"), false),
        (Some("NO"), false),
        (Some("1"), true),
        (Some("true"), true),
        // Unrecognised values keep recording on (the safe default).
        (Some("maybe"), true),
    ] {
        temp_env::with_var("FORMAL_AI_RECORD_CHAT", value, || {
            assert_eq!(
                memory_sync::chat_recording_enabled(),
                recording,
                "value {value:?}"
            );
        });
    }
}
