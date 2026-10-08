//! Issue #1173 R1173-3, the fourth routing-probe pass: requests that need no
//! outside knowledge reach their class handler in the native solver, each
//! read from seed data. The browser twin is
//! `rust/tests/web/issue-1173-route2-probes.test.mjs`.

use formal_ai::{SolverConfig, UniversalSolver};

fn solved(prompt: &str) -> (String, String) {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve(prompt);
    (answer.intent, answer.answer)
}

fn failure(name: &str) -> String {
    [
        "Execution status: failed in isolated sandbox.\n```python\n",
        name,
        "()\n```\nTraceback (most recent call last):\n  File 'main.py', line 1, in <module>\nNameError: name '",
        name,
        "' is not defined.\nThe failure trace is appended to the action log; see the trace link.",
    ]
    .concat()
}

#[test]
fn a_script_calling_a_function_nothing_defines_fails_whatever_the_function_is_named() {
    for name in [
        "missing_helper",
        "not_defined_anywhere",
        "undefined_helper",
        "undefined_function",
    ] {
        let (intent, answer) = solved(&format!("Write a Python script that calls {name}()"));
        assert_eq!(intent, "execution_failure", "{name}: {answer}");
        assert_eq!(answer, failure(name));
    }
    assert_eq!(
        formal_ai::capability_routing::claim_operands(
            "undefined_call",
            "Write a Python script that calls missing_helper()"
        ),
        Some(vec!["missing_helper".to_owned()])
    );
}

#[test]
fn a_builtin_a_method_a_defined_name_and_a_call_without_a_program_are_no_undefined_call() {
    for prompt in [
        "Write a Python script that calls print()",
        "Write a Python script that calls len()",
        "Write a script that calls items.sort()",
        "Write a Python script that defines helper and calls helper()",
        "How do I call foo() from bar?",
    ] {
        assert_eq!(
            formal_ai::capability_routing::claim_operands("undefined_call", prompt),
            Some(Vec::new()),
            "{prompt}"
        );
    }
    let (intent, _) = solved("Write a Python script that calls print()");
    assert_ne!(intent, "execution_failure");
}

#[test]
fn convert_to_json_over_a_json_payload_reads_the_source_format_from_the_payload() {
    let (intent, answer) = solved("Convert to JSON: {\"word_counts\": {\"lines\": 4}}");
    assert_eq!(intent, "format_conversion");
    assert_eq!(
        answer,
        "The text is already JSON, so there is no other format to convert from; here it is parsed and re-emitted with two-space indentation.\n\n```json\n{\n  \"word_counts\": {\n    \"lines\": 4\n  }\n}\n```\nRound-trip check: parsing the emitted JSON back reproduced the same values and nesting as the input."
    );
}

#[test]
fn the_request_line_of_a_yaml_conversion_is_not_read_as_the_first_key() {
    for prompt in [
        "Convert this YAML to JSON:\ncounts: 3\nlines: 4",
        "Convert to JSON:\ncounts: 3\nlines: 4",
    ] {
        let (intent, answer) = solved(prompt);
        assert_eq!(intent, "format_conversion", "{prompt}");
        assert!(
            answer.contains("```json\n{\n  \"counts\": 3,\n  \"lines\": 4\n}"),
            "{answer}"
        );
        assert!(!answer.contains("\"Convert"), "{answer}");
    }
}

#[test]
fn a_product_request_with_no_marketplace_names_the_catalogued_ones_instead_of_guessing() {
    let (intent, answer) = solved("Find me a laptop under 1000 dollars");
    assert_eq!(intent, "product_search");
    assert_eq!(
        answer,
        "I compose a product search only on a marketplace the request names, and this one names none, so I will not guess a store for laptop. Name one of the marketplaces I know (amazon in, app store ios, google play) and I will compose the exact search link.\nConstraints you asked for: (none stated)"
    );
    let (intent, answer) = solved("Найди мне ноутбук до 1000 долларов");
    assert_eq!(intent, "product_search");
    assert!(answer.starts_with("Я составляю поиск товара"), "{answer}");
}

#[test]
fn a_quantity_between_two_units_is_a_conversion_not_a_translation() {
    let (intent, answer) = solved("Переведи 10 миль в километры");
    assert_eq!(intent, "unit_conversion", "{answer}");
}

#[test]
fn a_hyphenated_trip_plan_reaches_the_planner() {
    for prompt in ["Plan a 2-day trip to Berlin", "Plan a 5-day trip to Tokyo"] {
        let (intent, answer) = solved(prompt);
        assert_eq!(intent, "planning", "{prompt}: {answer}");
    }
}

#[test]
fn a_colon_after_the_summarize_verb_still_summarizes_the_supplied_text() {
    let (intent, answer) = solved(
        "Summarize: Cats sleep a lot. Cats like fish. Cats are independent animals that hunt mice.",
    );
    assert_eq!(intent, "summarization_free_text", "{answer}");
}

#[test]
fn a_concept_alias_inside_a_longer_word_is_no_mention_of_the_concept() {
    let (intent, answer) = solved("colourless green ideas sleep furiously");
    assert_eq!(intent, "unknown", "{answer}");
}
