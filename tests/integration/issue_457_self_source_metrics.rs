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
        response.answer.contains("Response self-analysis")
            && response
                .answer
                .contains("generated Rust code is more complex"),
        "answer must compare generated code and reasoning text, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("program_blueprint:recipe self_source_metrics_report"),
        "trace must record the source-metrics blueprint recipe, got: {}",
        response.links_notation
    );
}
