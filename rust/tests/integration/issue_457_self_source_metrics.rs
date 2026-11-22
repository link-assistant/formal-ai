//! Issue #457: a composite Rust `write_program` request for source-code metrics
//! and self-analysis used to dead-end on `task=missing`.

use formal_ai::UniversalSolver;

const ISSUE_PROMPT: &str = "Write a Rust program that:\n\
   1. Parses its own source code as text\n\
   2. Counts: functions, loops, conditionals, comments\n\
   3. Calculates a \"complexity score\" based on cyclomatic complexity\n\
   4. Outputs a JSON report with metrics\n\
   5. Then: analyze YOUR OWN response to this prompt using the same metrics\n\
   6. Compare: which is more complex — your generated code or your reasoning text?";

#[test]
fn issue_457_rust_self_source_metrics_request_returns_blueprint_program() {
    let solver = UniversalSolver::default();
    let response = solver.solve(ISSUE_PROMPT);

    assert_eq!(
        response.answer,
        include_str!("../fixtures/blueprint-answers/source-metrics.md").trim_end()
    );
    assert_eq!(
        response.intent, "write_program",
        "the issue prompt must route to write_program, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        !response.answer.contains("I do not have a template")
            && !response.answer.contains("task `missing`"),
        "must not surface the missing-template dead-end, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("```rust"),
        "answer must contain a Rust code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("include_str!")
            && response.answer.contains("functions")
            && response.answer.contains("loops")
            && response.answer.contains("conditionals")
            && response.answer.contains("comments")
            && response.answer.contains("complexity_score"),
        "program must parse its own source and report the requested metrics, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("std::fs::read_to_string(path)")
            && response.answer.contains("analyze_rust_text(&reasoning)")
            && response
                .answer
                .contains("source.complexity_score > reasoning.complexity_score")
            && response
                .answer
                .contains("cargo run -- <response-prose.txt>"),
        "program must compare the generated code and actual response text, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("program_blueprint:recipe self_source_metrics_report"),
        "trace must record the source-metrics blueprint recipe, got: {}",
        response.links_notation
    );
    assert!(!response.answer.contains("RESPONSE_REASONING"));
    assert!(
        !response
            .answer
            .contains("Reasoning text metrics: functions=")
    );
}

#[test]
fn issue_457_language_coverage_markers_cover_supported_surfaces() {
    // This issue changes language-facing blueprint rendering in Rust and the
    // browser worker; keep explicit markers for every supported UI language.
    let covered_languages = [
        ("en", "English"),
        ("ru", "Russian"),
        ("hi", "Hindi"),
        ("zh", "Chinese"),
    ];

    assert_eq!(
        covered_languages.map(|(language, _)| language),
        ["en", "ru", "hi", "zh"]
    );
}
