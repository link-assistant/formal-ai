//! Issue #1185 (E149): the error-driven repair loop — formalize compiler and
//! runtime diagnostics, search the error text, retain a fetched fix as a
//! meta-language record, and retry on a bounded ladder instead of stopping
//! at the first failed step.
//!
//! Wiring note (main session): these tests read
//! `formal_ai::agentic_coding::repair_loop`, which needs `pub mod repair_loop;`
//! in `rust/src/agentic_coding/modules.rs`, and the two seeds
//! `data/seed/diagnostic-code-shapes.lino` (bundle) and
//! `data/seed/multilingual-responses-repair.lino` (bundle, lexicon response)
//! registered in `data/meta/seed-registry.lino` with the embedded mirrors
//! refreshed. Until then the loop is inert by design: `formalize_diagnostic`
//! returns no diagnostics and `plan_repair` declines every step, exactly as
//! a missing cue table declines a handler.
//!
//! Residuals recorded by design (not covered offline here):
//! - the live-fetch variant (a real E0502 repaired from a fetched Stack
//!   Exchange or doc page) is gated on `FORMAL_AI_LIVE_FETCH=1` upstream;
//!   the offline fixtures pin the same decision chain,
//! - the three-roots parity translation (R8, `formal-ai translate --to js|ts`)
//!   needs the translator built, which the no-build drafting constraint
//!   forbids; the Rust root is the reference implementation.
//!
//! R3's apply half (rendering the retained fix back into source) is pinned
//! by `issue_1185_repair_apply.rs`.

use formal_ai::agentic_coding::repair_loop::{
    self, FailedStep, MAX_REPAIR_RUNGS, RepairOutcome, RepairStop,
};
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall};
use formal_ai::{ChatMessage, FunctionCall, ToolCall};

const TOOLS: [&str; 4] = ["web_search", "web_fetch", "write_file", "bash"];

/// A user turn opening the evidence window every scan reads from.
fn user_turn() -> ChatMessage {
    ChatMessage::new("user", "run the generated program and verify it")
}

/// An assistant turn requesting one tool call.
fn assistant_call(id: &str, name: &str, arguments: &str) -> ChatMessage {
    let mut message = ChatMessage::new("assistant", "");
    message.tool_calls = vec![ToolCall {
        id: id.to_owned(),
        kind: "function".to_owned(),
        function: FunctionCall {
            name: name.to_owned(),
            arguments: arguments.to_owned(),
        },
    }];
    message
}

/// The tool result answering `assistant_call`'s id.
fn tool_result(id: &str, name: &str, output: &str) -> ChatMessage {
    let mut message = ChatMessage::new("tool", output);
    message.tool_call_id = Some(id.to_owned());
    message.name = Some(name.to_owned());
    message
}

fn rustc_failure() -> FailedStep {
    FailedStep::new(
        "rust",
        "error[E0308]: mismatched types\n --> src/main.rs:6:33\n  |\n6 |     let total = left + right;\n  |                          ^ expected `f64`, found `i32`\n",
    )
    .with_exit_code(Some(1))
    .with_failed_command(Some("rustc --edition 2021 src/main.rs".to_owned()))
    .with_artifact_path("src/main.rs")
}

// ---------------------------------------------------------------------------
// R1 — diagnostics are formalized from the raw output, per seed shapes
// ---------------------------------------------------------------------------

#[test]
fn formalize_diagnostic_extracts_file_line_code_and_message_from_rustc_output() {
    let diagnostics = repair_loop::formalize_diagnostic("rust", &rustc_failure().reported);
    assert_eq!(diagnostics.len(), 1, "one error line, one diagnostic");
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code.as_deref(), Some("E0308"));
    assert_eq!(diagnostic.message, "mismatched types");
    assert_eq!(diagnostic.file.as_deref(), Some("src/main.rs"));
    assert_eq!(diagnostic.line, Some(6));
    assert!(diagnostic.raw.contains("error[E0308]"), "raw line kept");
}

#[test]
fn formalize_diagnostic_extracts_from_kotlinc_output_with_no_error_code() {
    let diagnostics = repair_loop::formalize_diagnostic(
        "kotlin",
        "main.kt:3:5 error: unresolved reference: greeting",
    );
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert!(
        diagnostic.code.is_none(),
        "Kotlin diagnostics carry no machine code; absent, never fabricated"
    );
    assert_eq!(diagnostic.message, "unresolved reference: greeting");
    assert_eq!(diagnostic.file.as_deref(), Some("main.kt"));
    assert_eq!(diagnostic.line, Some(3));
}

#[test]
fn formalize_diagnostic_reads_python_traceback_head_and_location() {
    let output = "Traceback (most recent call last):\n  File \"main.py\", line 4, in <module>\n    greet()\n  File \"main.py\", line 2, in greet\n    return gretting(\"hi\")\nNameError: name 'gretting' is not defined\n";
    let diagnostics = repair_loop::formalize_diagnostic("python", output);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.message, "name 'gretting' is not defined");
    // The traceback's innermost frame binds, not the outermost one.
    assert_eq!(diagnostic.file.as_deref(), Some("main.py"));
    assert_eq!(diagnostic.line, Some(2));
    assert!(diagnostic.code.is_none());
}

#[test]
fn formalize_diagnostic_yields_nothing_for_output_without_a_shape() {
    let diagnostics = repair_loop::formalize_diagnostic("rust", "finished in 0.3s\nall good");
    assert!(
        diagnostics.is_empty(),
        "no recognizable diagnostic means no fabricated one"
    );
}

// ---------------------------------------------------------------------------
// R2/R4/R6 — the loop's plan, its bound, and its honest stops
// ---------------------------------------------------------------------------

#[test]
fn repair_loop_plans_the_diagnostic_search_before_any_final() {
    let messages = vec![user_turn()];
    let outcome =
        repair_loop::repair_step(&messages, &TOOLS, &rustc_failure(), 0, MAX_REPAIR_RUNGS);
    let AgenticPlan::ToolCalls(calls) = outcome.plan().expect("a search rung opens the ladder")
    else {
        panic!("the first rung is a tool call, not a final answer");
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "web_search");
    assert!(
        calls[0].arguments.contains("E0308"),
        "the query carries the exact error code: {}",
        calls[0].arguments
    );
    assert!(
        calls[0].arguments.contains("rust"),
        "the query names the language: {}",
        calls[0].arguments
    );
}

#[test]
fn repair_loop_stops_when_no_fetched_source_matches_the_diagnostic() {
    let messages = vec![
        user_turn(),
        assistant_call(
            "c1",
            "web_search",
            r#"{ "query": "rust E0308 mismatched types" }"#,
        ),
        tool_result("c1", "web_search", "https://example.org/gardening"),
        assistant_call(
            "c2",
            "web_fetch",
            r#"{ "url": "https://example.org/gardening" }"#,
        ),
        tool_result(
            "c2",
            "web_fetch",
            "A page about raised beds and compost. Nothing about types.",
        ),
    ];
    let outcome =
        repair_loop::repair_step(&messages, &TOOLS, &rustc_failure(), 0, MAX_REPAIR_RUNGS);
    assert_eq!(
        outcome,
        RepairOutcome::Stop(RepairStop::NoMatch),
        "an unmatched diagnostic is an unresolved need, never a fabricated fix"
    );
    assert!(
        repair_loop::plan_repair(&messages, &TOOLS, &rustc_failure(), 0, MAX_REPAIR_RUNGS)
            .is_none(),
        "the caller falls back to today's honest failure report"
    );
    let note = repair_loop::unresolved_note("en", "rust E0308 mismatched types");
    assert!(
        note.contains("unresolved") && note.contains("E0308"),
        "the unresolved note names the need: {note}"
    );
}

#[test]
fn repair_loop_is_bounded_and_reports_reaching_the_top_rung() {
    let messages = vec![user_turn()];
    let outcome = repair_loop::repair_step(
        &messages,
        &TOOLS,
        &rustc_failure(),
        MAX_REPAIR_RUNGS,
        MAX_REPAIR_RUNGS,
    );
    assert_eq!(
        outcome,
        RepairOutcome::Stop(RepairStop::Exhausted),
        "the ladder terminates honestly instead of looping"
    );
    let note = repair_loop::ladder_note("en", MAX_REPAIR_RUNGS, MAX_REPAIR_RUNGS);
    assert!(
        note.contains("rung 3") && note.contains("at most 3"),
        "the bound is reported, not silently truncated: {note}"
    );
}

#[test]
fn repair_loop_declines_output_without_a_diagnostic() {
    let failure = FailedStep::new("rust", "finished in 0.3s").with_exit_code(Some(1));
    let outcome = repair_loop::repair_step(&[user_turn()], &TOOLS, &failure, 0, MAX_REPAIR_RUNGS);
    assert_eq!(
        outcome,
        RepairOutcome::Stop(RepairStop::NoDiagnostic),
        "nothing to search means today's failure report stands"
    );
}

#[test]
fn repair_loop_records_the_fix_then_retries_the_failed_command() {
    let matched_page = "The borrow error E0308 means the value is moved while still borrowed.\n```rust\nlet total = left.clone() + right;\n```\nClone before the move.";
    // A fetched page is the harness's own success report: the envelope's zero
    // exit status is the primary signal, so a page that legitimately discusses
    // an error is source text and not a failed fetch (issues #905/#908).
    let fetched = fetched_page_success(matched_page);
    let base = vec![
        user_turn(),
        assistant_call(
            "c1",
            "web_search",
            r#"{ "query": "rust E0308 mismatched types" }"#,
        ),
        tool_result("c1", "web_search", "https://example.org/e0308"),
        assistant_call(
            "c2",
            "web_fetch",
            r#"{ "url": "https://example.org/e0308" }"#,
        ),
        tool_result("c2", "web_fetch", &fetched),
    ];
    // Rung phase 3: the matched fix becomes a meta-language record first.
    let outcome = repair_loop::repair_step(&base, &TOOLS, &rustc_failure(), 0, MAX_REPAIR_RUNGS);
    let AgenticPlan::ToolCalls(calls) = outcome.plan().expect("the fix is recorded") else {
        panic!("recording the fix is a tool call");
    };
    assert_eq!(calls[0].tool, "write_file");
    assert!(
        calls[0].arguments.contains(".repair.lino"),
        "the record is a sidecar next to the artifact: {}",
        calls[0].arguments
    );
    assert!(
        calls[0].arguments.contains("repair_edit"),
        "the record is a Links Notation repair_edit document"
    );
    // Rung phase 4: with the record written, the failed command retries.
    let arguments = calls[0].arguments.clone();
    let with_write = [
        base,
        vec![
            assistant_call("c3", "write_file", &arguments),
            tool_result("c3", "write_file", "recorded"),
        ],
    ]
    .concat();
    let outcome =
        repair_loop::repair_step(&with_write, &TOOLS, &rustc_failure(), 0, MAX_REPAIR_RUNGS);
    let AgenticPlan::ToolCalls(calls) = outcome.plan().expect("the command retries") else {
        panic!("the retry is a tool call");
    };
    assert_eq!(calls[0].tool, "bash");
    assert!(
        calls[0].arguments.contains("rustc"),
        "the retry re-runs the failed command: {}",
        calls[0].arguments
    );
}

// ---------------------------------------------------------------------------
// R3/R5 — the fix as a meta-language record; the chain as evidence
// ---------------------------------------------------------------------------

#[test]
fn repair_edit_document_is_links_notation_not_a_patch() {
    let failure = rustc_failure();
    let diagnostic =
        repair_loop::formalize_diagnostic(&failure.language, &failure.reported).remove(0);
    let document = repair_loop::repair_edit_document(
        "rust",
        &diagnostic,
        Some("let total = left.clone() + right;"),
        "https://example.org/e0308",
    );
    assert!(document.starts_with("repair_edit\n"));
    assert!(document.contains("language rust"));
    assert!(document.contains("file \"src/main.rs\""));
    assert!(document.contains("line 6"));
    assert!(document.contains("error_code \"E0308\""));
    assert!(document.contains("source \"https://example.org/e0308\""));
    assert!(document.contains("retained \"let total = left.clone() + right;\""));
    assert!(
        document.contains("rendering meta_language"),
        "the fix is expressed for the meta-language renderer, not pasted as text"
    );
}

#[test]
fn repair_attempt_chain_is_recorded_for_the_answer_derivation() {
    let matched_page = "The borrow error E0308 means the value is moved while still borrowed.\n```rust\nlet total = left.clone() + right;\n```\nClone before the move.";
    let failure = rustc_failure();
    let messages = vec![
        user_turn(),
        assistant_call(
            "c1",
            "web_search",
            r#"{ "query": "rust E0308 mismatched types" }"#,
        ),
        tool_result("c1", "web_search", "https://example.org/e0308"),
        assistant_call(
            "c2",
            "web_fetch",
            r#"{ "url": "https://example.org/e0308" }"#,
        ),
        tool_result("c2", "web_fetch", &fetched_page_success(matched_page)),
        assistant_call(
            "c3",
            "write_file",
            r#"{ "path": "src/main.repair.lino", "content": "repair_edit\n  language rust\n" }"#,
        ),
        tool_result("c3", "write_file", "recorded"),
        assistant_call(
            "c4",
            "bash",
            r#"{ "command": "rustc --edition 2021 src/main.rs" }"#,
        ),
        tool_result("c4", "bash", "compilation finished without errors"),
    ];
    let attempts = repair_loop::attempts_from(&messages, &failure);
    assert_eq!(attempts.len(), 1);
    let attempt = &attempts[0];
    assert!(
        attempt.candidate_fix.is_some(),
        "the matched page's fragment is retained"
    );
    assert!(
        attempt.applied,
        "the sidecar write marks the attempt applied"
    );
    assert!(
        attempt.resolved,
        "the retried command's success marks it resolved"
    );
    let evidence = repair_loop::evidence_document(&attempts);
    assert!(evidence.starts_with("repair_attempts\n"));
    assert!(evidence.contains("attempt_count 1"));
    assert!(evidence.contains("repair_attempt"));
    assert!(evidence.contains("resolved true"));
}

#[test]
fn search_query_bounds_the_message_fragment() {
    let failure = rustc_failure();
    let diagnostic =
        repair_loop::formalize_diagnostic(&failure.language, &failure.reported).remove(0);
    let query = repair_loop::search_query("rust", &diagnostic);
    assert!(query.starts_with("rust"));
    assert!(query.contains("E0308") || query.contains("mismatched"));
    assert!(
        query.split(' ').count() <= 14,
        "the query stays what a source can answer: {query}"
    );
}

#[test]
fn planned_call_shapes_match_the_planner_contract() {
    let messages = vec![user_turn()];
    let outcome =
        repair_loop::repair_step(&messages, &TOOLS, &rustc_failure(), 0, MAX_REPAIR_RUNGS);
    let AgenticPlan::ToolCalls(calls) = outcome.plan().unwrap() else {
        panic!("first rung plans a call");
    };
    let call: &PlannedToolCall = &calls[0];
    assert!(
        serde_json_call_is_object(&call.arguments),
        "arguments are a JSON object the harness can execute"
    );
}

/// The tool result for a fetch that succeeded, as the harness reports one:
/// the payload plus the zero exit status that makes it source text.
fn fetched_page_success(page: &str) -> String {
    serde_json::json!({ "content": page, "exit_code": 0 }).to_string()
}

fn serde_json_call_is_object(arguments: &str) -> bool {
    let value: serde_json::Value =
        serde_json::from_str(arguments).unwrap_or(serde_json::Value::Null);
    value.is_object()
}

// ---------------------------------------------------------------------------
// R4/R5/R6 — the stop is reported in the answer, with the attempt chain
// ---------------------------------------------------------------------------

#[test]
fn stop_note_names_the_unresolved_need_and_carries_the_attempt_chain() {
    let messages = vec![
        user_turn(),
        assistant_call(
            "c1",
            "web_search",
            r#"{ "query": "rust E0308 mismatched types" }"#,
        ),
        tool_result("c1", "web_search", "https://example.org/gardening"),
        assistant_call(
            "c2",
            "web_fetch",
            r#"{ "url": "https://example.org/gardening" }"#,
        ),
        tool_result(
            "c2",
            "web_fetch",
            "A page about raised beds and compost. Nothing about types.",
        ),
    ];
    let note = repair_loop::stop_note(
        &messages,
        &rustc_failure(),
        RepairStop::NoMatch,
        0,
        MAX_REPAIR_RUNGS,
    );
    assert!(
        note.contains("unresolved") && note.contains("E0308"),
        "the stop names the unresolved need: {note}"
    );
    assert!(
        note.contains("repair_attempts")
            && note.contains("attempt_count 1")
            && note.contains("candidate_fix false"),
        "the attempt chain follows the note as evidence: {note}"
    );
}

#[test]
fn stop_note_reports_the_spent_ladder_and_is_silent_without_a_rung() {
    let exhausted = repair_loop::stop_note(
        &[user_turn()],
        &rustc_failure(),
        RepairStop::Exhausted,
        MAX_REPAIR_RUNGS,
        MAX_REPAIR_RUNGS,
    );
    assert!(
        exhausted.contains("rung 3") && exhausted.contains("repair_attempts"),
        "the bound and the attempts are reported: {exhausted}"
    );
    for stop in [RepairStop::NoDiagnostic, RepairStop::NoTools] {
        assert_eq!(
            repair_loop::stop_note(&[user_turn()], &rustc_failure(), stop, 0, MAX_REPAIR_RUNGS),
            "",
            "{stop:?} ran no rung, so today's failure report stands alone"
        );
    }
}

// ---------------------------------------------------------------------------
// R1/R7 — one shape table, all fourteen emitted languages
// ---------------------------------------------------------------------------

/// One captured diagnostic per emitted language, with the structure the
/// generic matcher reads from it: (language, raw output, file, line, code,
/// message). The JavaScript root replays the same table in
/// `rust/tests/web/issue-1185-fourteen-languages.test.mjs`.
const FOURTEEN_LANGUAGES: [(&str, &str, &str, u32, Option<&str>, &str); 14] = [
    (
        "rust",
        "error[E0308]: mismatched types\n --> src/main.rs:6:33",
        "src/main.rs",
        6,
        Some("E0308"),
        "mismatched types",
    ),
    (
        "typescript",
        "src/app.ts(4,7): error TS2322: Type 'string' is not assignable to type 'number'.",
        "src/app.ts",
        4,
        Some("TS2322"),
        "Type 'string' is not assignable to type 'number'.",
    ),
    (
        "javascript",
        "/work/index.js:3\nconsole.log(total);\n            ^\nerror: total is not defined",
        "/work/index.js",
        3,
        None,
        "total is not defined",
    ),
    (
        "kotlin",
        "Main.kt:3:5: error: unresolved reference: printn",
        "Main.kt",
        3,
        None,
        "unresolved reference: printn",
    ),
    (
        "scala",
        "Main.scala:4: error: not found: value printn",
        "Main.scala",
        4,
        None,
        "not found: value printn",
    ),
    (
        "java",
        "Main.java:5: error: cannot find symbol",
        "Main.java",
        5,
        None,
        "cannot find symbol",
    ),
    (
        "go",
        "./main.go:7:2: undefined: fmt.Printn",
        "./main.go",
        7,
        None,
        "undefined: fmt.Printn",
    ),
    (
        "python",
        "Traceback (most recent call last):\n  File \"main.py\", line 2, in <module>\n    print(1/0)\nZeroDivisionError: division by zero",
        "main.py",
        2,
        None,
        "division by zero",
    ),
    (
        "c",
        "main.c:4:5: error: implicit declaration of function 'printff'",
        "main.c",
        4,
        None,
        "implicit declaration of function 'printff'",
    ),
    (
        "cpp",
        "main.cpp:6:3: error: 'cout' was not declared in this scope",
        "main.cpp",
        6,
        None,
        "'cout' was not declared in this scope",
    ),
    (
        "csharp",
        "Program.cs(9,13): error CS0103: The name 'Consle' does not exist in the current context",
        "Program.cs",
        9,
        Some("CS0103"),
        "The name 'Consle' does not exist in the current context",
    ),
    (
        "ruby",
        "main.rb:2: syntax error, unexpected end-of-input",
        "main.rb",
        2,
        None,
        "syntax error, unexpected end-of-input",
    ),
    (
        "php",
        "PHP Parse error: syntax error, unexpected end of file in /work/index.php on line 5",
        "/work/index.php",
        5,
        None,
        "syntax error, unexpected end of file",
    ),
    (
        "swift",
        "main.swift:3:1: error: cannot find 'prin' in scope",
        "main.swift",
        3,
        None,
        "cannot find 'prin' in scope",
    ),
];

#[test]
fn every_emitted_language_formalizes_through_the_one_shape_table() {
    for (language, raw, file, line, code, message) in FOURTEEN_LANGUAGES {
        let diagnostics = repair_loop::formalize_diagnostic(language, raw);
        let read: Vec<(Option<&str>, Option<u32>, Option<&str>, &str)> = diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.file.as_deref(),
                    diagnostic.line,
                    diagnostic.code.as_deref(),
                    diagnostic.message.as_str(),
                )
            })
            .collect();
        assert_eq!(
            read,
            vec![(Some(file), Some(line), code, message)],
            "{language}: one diagnostic with its file, line, code and message"
        );
    }
}

// R7: a PHP diagnostic states its location inline; before the shape table
// carried it inside the pattern, a `{file} on line {line}` location row read
// the whole line as a path and the diagnostic was dropped.
#[test]
fn an_inline_php_location_does_not_swallow_the_diagnostic() {
    let diagnostics = repair_loop::formalize_diagnostic(
        "php",
        "PHP Fatal error: Uncaught Error: Call to undefined function greet() in /work/index.php on line 3",
    );
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "Uncaught Error: Call to undefined function greet()"
    );
    assert_eq!(diagnostics[0].file.as_deref(), Some("/work/index.php"));
    assert_eq!(diagnostics[0].line, Some(3));
}

// R2: the query names the language and carries the exact code, and a
// fetched page is retained only when it addresses that exact code.
#[test]
fn the_search_query_names_the_language_and_the_exact_code() {
    let diagnostic = repair_loop::formalize_diagnostic(
        "csharp",
        "Program.cs(9,13): error CS0103: The name 'Consle' does not exist in the current context",
    )
    .remove(0);
    assert_eq!(
        repair_loop::search_query("csharp", &diagnostic),
        "csharp CS0103 The name Consle does not exist in the current context"
    );
}
