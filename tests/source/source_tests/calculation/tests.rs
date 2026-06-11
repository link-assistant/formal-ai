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
fn placeholder_unknown_equations_are_solved_by_local_fallback() {
    let equation_templates = [
        ("u + 2 = 4", "2"),
        ("2 + u = 4", "2"),
        ("4 = u + 2", "2"),
        ("u - 2 = 4", "6"),
        ("10 - u = 4", "6"),
        ("u * 2 = 8", "4"),
        ("2 * u = 8", "4"),
        ("u / 2 = 4", "8"),
        ("(u + 2) * 3 = 12", "2"),
        ("3 * (u + 2) = 12", "2"),
        ("u + u = 10", "5"),
        ("2 * u + 3 = 11", "4"),
        ("3 + 2 * u = 11", "4"),
        ("10 = u / 3 + 1", "27"),
        ("u / 3 + 1 = 10", "27"),
        ("u + 2.5 = 4", "1.5"),
        ("u - 0.5 = 2", "2.5"),
        ("2 * (u - 1) = 6", "4"),
        ("(u - 1) / 3 = 2", "7"),
        ("-u + 10 = 4", "6"),
        ("u + (-2) = 4", "6"),
        ("2 * u - 4 = 0", "2"),
        ("0 = 2 * u - 4", "2"),
        ("u + 0 = 7", "7"),
        ("1 * u = 9", "9"),
    ];

    for marker in ["?", "*"] {
        for (template, expected_value) in equation_templates {
            let expression = template.replace('u', marker);
            let evaluation = evaluate_calculation(&expression)
                .unwrap_or_else(|error| panic!("{expression:?} should solve: {error:?}"));
            assert_eq!(
                evaluation.engine,
                CalculationEngine::FormalAiEquationFallback,
                "{expression:?} should use the local placeholder fallback"
            );
            assert_eq!(
                evaluation.formatted,
                format!("{marker} = {expected_value}"),
                "{expression:?} should solve the placeholder unknown"
            );
        }
    }
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
