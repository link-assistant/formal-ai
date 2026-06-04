use super::*;

#[test]
fn bare_dot_is_detected_only_for_non_decimal_periods() {
    // Issue #334 OOM triggers: a `.` that is not flanked by digits.
    for unsafe_expr in ["2. 3", "2+2. 3+3", "5. 5", ".5", "5.", "2 .3", "a.b"] {
        assert!(
            has_bare_dot(unsafe_expr),
            "{unsafe_expr:?} contains a bare dot and must be rejected"
        );
    }
    // Genuine decimals / dot-grouped digits stay on the link-calculator path.
    for safe_expr in ["3.14", "3.14 + 2.5", "1.000.000", "2024.01.01", "8% of 500"] {
        assert!(
            !has_bare_dot(safe_expr),
            "{safe_expr:?} is a valid decimal expression and must stay safe"
        );
    }
}

#[test]
fn link_calculator_path_is_skipped_for_bare_dot_expressions() {
    // Before the guard this aborted the process with a multi-GB allocation.
    assert!(evaluate_with_link_calculator("2. 3").is_err());
    assert!(evaluate_with_link_calculator("2+2. 3+3").is_err());
    // Real decimals still reach the upstream calculator and evaluate.
    let evaluation = evaluate_with_link_calculator("3.14 + 2.5").expect("decimal evaluates");
    assert_eq!(evaluation.engine, CalculationEngine::LinkCalculator);
}

#[test]
fn arithmetic_word_tables_match_seed() {
    // src/arithmetic.rs is compiled into the wasm worker (no_std, no build.rs),
    // so its spelled-word→value tables are materialized at author time into
    // src/arithmetic_word_tables.rs by the issue_386_gen_arith_table example.
    // This guard fails CI whenever that static drifts from the live seed.
    let own = |table: &[(&'static str, &'static str)]| {
        table
            .iter()
            .map(|(surface, value)| ((*surface).to_string(), (*value).to_string()))
            .collect::<Vec<_>>()
    };
    let (seed_tokens, seed_phrases) = crate::seed::lexicon().arithmetic_normalization_tables();
    assert_eq!(
        own(crate::arithmetic::WORD_VALUE_TOKENS),
        seed_tokens,
        "src/arithmetic_word_tables.rs WORD_VALUE_TOKENS is stale; regenerate with \
             `cargo run -p formal-ai --example issue_386_gen_arith_table`"
    );
    assert_eq!(
        own(crate::arithmetic::WORD_VALUE_PHRASES),
        seed_phrases,
        "src/arithmetic_word_tables.rs WORD_VALUE_PHRASES is stale; regenerate with \
             `cargo run -p formal-ai --example issue_386_gen_arith_table`"
    );
}
