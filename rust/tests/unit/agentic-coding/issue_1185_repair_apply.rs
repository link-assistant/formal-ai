//! Issue #1185 R3, the apply half: the `repair_edit` record the loop writes
//! beside a failing artifact is read back and rendered into the artifact's
//! source at the diagnostic's line, accepted only when the meta-language CST
//! engine (the issue #1167 renderer seam) parses the rendering as a valid
//! program, and written before the failed command is re-run. Mirrored by
//! `rust/tests/web/issue-1185-repair-apply.test.mjs`.

use formal_ai::agentic_coding::AgenticPlan;
use formal_ai::agentic_coding::repair_apply::{
    RepairApplyGap, RepairEdit, parse_repair_edit, render_repair_edit, splice_repair_edit,
};
use formal_ai::agentic_coding::repair_loop::{self, FailedStep, MAX_REPAIR_RUNGS, RepairOutcome};
use formal_ai::{ChatMessage, FunctionCall, ToolCall};

const SOURCE: &str = "fn main() {\n    let left: f64 = 1.5;\n    let right: i32 = 2;\n    println!(\"{left}\");\n    println!(\"{right}\");\n    let total = left + right;\n    println!(\"{total}\");\n}\n";
const REPORTED: &str = "error[E0308]: mismatched types\n --> src/main.rs:6:33\n  |\n6 |     let total = left + right;\n  |                          ^ expected `f64`, found `i32`\n";
const FIX: &str = "let total = left + f64::from(right);";
const RENDERED: &str = "fn main() {\n    let left: f64 = 1.5;\n    let right: i32 = 2;\n    println!(\"{left}\");\n    println!(\"{right}\");\n    let total = left + f64::from(right);\n    println!(\"{total}\");\n}\n";
const TOOLS: [&str; 4] = ["web_search", "web_fetch", "write_file", "bash"];

fn record(fix: Option<&str>) -> String {
    let diagnostic = repair_loop::formalize_diagnostic("rust", REPORTED).remove(0);
    repair_loop::repair_edit_document("rust", &diagnostic, fix, "https://example.org/e0308")
}

fn failure() -> FailedStep {
    FailedStep::new("rust", REPORTED)
        .with_exit_code(Some(1))
        .with_failed_command(Some("rustc --edition 2021 src/main.rs".to_owned()))
        .with_artifact_path("src/main.rs")
}

fn call(id: &str, name: &str, arguments: &str) -> ChatMessage {
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

fn result(id: &str, name: &str, output: &str) -> ChatMessage {
    let mut message = ChatMessage::new("tool", output);
    message.tool_call_id = Some(id.to_owned());
    message.name = Some(name.to_owned());
    message
}

fn only_call(outcome: RepairOutcome) -> (String, String) {
    let Some(AgenticPlan::ToolCalls(calls)) = outcome.plan() else {
        panic!("the outcome is one tool call");
    };
    (calls[0].tool.clone(), calls[0].arguments.clone())
}

#[test]
fn repair_edit_record_reads_back_into_its_fields() {
    assert_eq!(
        parse_repair_edit(&record(Some(FIX))),
        Some(RepairEdit {
            language: "rust".to_owned(),
            file: Some("src/main.rs".to_owned()),
            line: Some(6),
            error_code: Some("E0308".to_owned()),
            retained: Some(FIX.to_owned()),
        })
    );
    assert_eq!(
        parse_repair_edit(&record(None)).map(|edit| edit.retained),
        Some(None)
    );
    assert_eq!(
        parse_repair_edit("formal_clause\n  quantifier forall\n"),
        None
    );
}

#[test]
fn repair_edit_record_renders_back_into_the_source() {
    let edit = parse_repair_edit(&record(Some(FIX))).expect("a repair_edit record");
    assert_eq!(splice_repair_edit(&edit, SOURCE), Ok(RENDERED.to_owned()));
    assert_eq!(
        render_repair_edit(&record(Some(FIX)), SOURCE),
        Ok(RENDERED.to_owned())
    );
    let multi_line = "let right = f64::from(right);\nlet total = left + right;";
    let spliced = splice_repair_edit(
        &parse_repair_edit(&record(Some(multi_line))).expect("a repair_edit record"),
        SOURCE,
    )
    .expect("the multi-line fix splices");
    assert_eq!(
        spliced.lines().skip(5).take(2).collect::<Vec<_>>(),
        vec![
            "    let right = f64::from(right);",
            "    let total = left + right;"
        ]
    );
}

#[test]
fn every_refusal_is_a_named_gap() {
    assert_eq!(
        render_repair_edit("not a record", SOURCE),
        Err(RepairApplyGap::NotARecord)
    );
    assert_eq!(
        render_repair_edit(&record(None), SOURCE),
        Err(RepairApplyGap::NoRetainedFix)
    );
    assert_eq!(
        render_repair_edit(&record(Some(FIX)), "fn main() {}\n"),
        Err(RepairApplyGap::LineOutOfRange { line: 6, lines: 1 })
    );
    assert_eq!(
        render_repair_edit(&record(Some("let total = (left + right;")), SOURCE),
        Err(RepairApplyGap::SyntaxInvalid {
            language: "rust".to_owned()
        })
    );
}

#[test]
fn repair_loop_applies_the_recorded_fix_before_the_retry() {
    let page = format!(
        "The error E0308 mismatched types: convert the integer before adding it.\n```rust\n{FIX}\n```\n"
    );
    let fetched = serde_json::json!({ "content": page, "exit_code": 0 }).to_string();
    let artifact = serde_json::json!({ "path": "src/main.rs", "content": SOURCE }).to_string();
    let base = vec![
        ChatMessage::new("user", "run the generated program and verify it"),
        call("c0", "write_file", &artifact),
        result("c0", "write_file", "written"),
        call(
            "c1",
            "web_search",
            r#"{ "query": "rust E0308 mismatched types" }"#,
        ),
        result("c1", "web_search", "https://example.org/e0308"),
        call(
            "c2",
            "web_fetch",
            r#"{ "url": "https://example.org/e0308" }"#,
        ),
        result("c2", "web_fetch", &fetched),
    ];
    let (tool, record_arguments) = only_call(repair_loop::repair_step(
        &base,
        &TOOLS,
        &failure(),
        0,
        MAX_REPAIR_RUNGS,
    ));
    assert_eq!(tool, "write_file");
    let with_record = [
        base,
        vec![
            call("c3", "write_file", &record_arguments),
            result("c3", "write_file", "recorded"),
        ],
    ]
    .concat();
    let outcome = repair_loop::repair_step(&with_record, &TOOLS, &failure(), 0, MAX_REPAIR_RUNGS);
    assert!(
        matches!(outcome, RepairOutcome::ApplyFix(_)),
        "the recorded fix is rendered into the artifact: {outcome:?}"
    );
    let (tool, arguments) = only_call(outcome);
    assert_eq!(tool, "write_file");
    let written: serde_json::Value = serde_json::from_str(&arguments).expect("JSON arguments");
    assert_eq!(written["path"], "src/main.rs");
    assert_eq!(written["content"], RENDERED);
    let rendered = serde_json::json!({ "path": "src/main.rs", "content": RENDERED }).to_string();
    let applied = [
        with_record,
        vec![
            call("c4", "write_file", &rendered),
            result("c4", "write_file", "written"),
        ],
    ]
    .concat();
    let outcome = repair_loop::repair_step(&applied, &TOOLS, &failure(), 0, MAX_REPAIR_RUNGS);
    assert!(
        matches!(outcome, RepairOutcome::Retry(_)),
        "an applied fix is not rendered twice; the command re-runs: {outcome:?}"
    );
}
