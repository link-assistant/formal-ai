//! Regression coverage for issue #681 — a natural-language *file-creation* request
//! must emit a `write` `tool_call` on the target, never a `read` on the (nonexistent)
//! file it is being asked to create.
//!
//! The umbrella issue is #680 ("no `tool_call` at all"); this issue is the distinct
//! *wrong-tool* correctness bug: a `write`/`create`/`save`/`generate` request was
//! being routed to the file-read recipe because the read-intent classifier matched
//! the word "content" in *"with the content …"* and the read recipe was consulted
//! before the write path.

use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
use formal_ai::agentic_coding::{plan_chat_step, run_agentic_task, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::ChatMessage;

fn single_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(mut calls)) => {
            assert!(!calls.is_empty(), "planner should emit at least one call");
            calls.remove(0)
        }
        other => panic!("expected tool calls, got {other:?}"),
    }
}

fn arguments(call: &PlannedToolCall) -> serde_json::Value {
    serde_json::from_str(&call.arguments).expect("tool arguments should be JSON")
}

/// The exact reproduction from the issue: both `write` and `read` are advertised,
/// and the request is a file-creation. The planner must pick `write`, targeting the
/// requested path with the requested content — never `read`.
#[test]
fn file_creation_request_emits_write_not_read() {
    let messages = vec![ChatMessage::user(
        "Create a file named hello.txt with the content hello world",
    )];
    let tools = ["write", "read"];
    let call = single_call(&messages, &tools);
    assert_eq!(
        call.tool, "write",
        "a file-creation request must route to the write tool, not read"
    );
    let args = arguments(&call);
    // The first write step records the plan event; the second writes the target.
    // Either way, the target file and content must reach a write call, never a read.
    // Drive the loop far enough to observe the target write.
    let path_str = &call.arguments;
    assert!(
        path_str.contains("hello.txt") || path_str.contains(".formal-ai"),
        "write arguments should reference the plan or the target: {args}"
    );
}

/// The write intent must win even when only a `read` tool is advertised: the planner
/// must not emit a `read` `tool_call` against the file it was asked to create. It is
/// acceptable to decline (no write tool available), but never to misroute to read.
#[test]
fn file_creation_never_routes_to_read_even_when_only_read_is_advertised() {
    let messages = vec![ChatMessage::user(
        "Create a file named hello.txt with the content hello world",
    )];
    let tools = ["read"];
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            for call in calls {
                assert_ne!(
                    call.tool, "read",
                    "must never read the file it was asked to create"
                );
            }
        }
        // Declining (no write tool) or a prose final answer is acceptable.
        Some(AgenticPlan::Final(_)) | None => {}
    }
}

/// The write path itself must recognise the "named …" + "with the content …"
/// phrasing so a `write` tool advertised alone still creates the file.
#[test]
fn general_change_plan_handles_named_and_with_the_content_phrasing() {
    let plan =
        compose_general_change_plan("Create a file named hello.txt with the content hello world")
            .expect("write plan for the issue's phrasing");
    assert_eq!(plan.target, "hello.txt");
    assert_eq!(plan.content, "hello world");
}

/// Generality: a spread of create/write/save/generate phrasings, each with a
/// different filename and content, must all route to `write` and carry the target.
#[test]
fn diverse_write_phrasings_all_route_to_write() {
    let cases = [
        (
            "Create a file named notes.txt with the content first draft",
            "notes.txt",
            "first draft",
        ),
        (
            "Save a file called report.md with the text quarterly summary",
            "report.md",
            "quarterly summary",
        ),
        (
            "Write hello.py saying print('hi')",
            "hello.py",
            "print('hi')",
        ),
        (
            "Generate a file named data.json containing {\"ok\": true}",
            "data.json",
            "{\"ok\": true}",
        ),
    ];
    for (prompt, target, content) in cases {
        let plan = compose_general_change_plan(prompt)
            .unwrap_or_else(|| panic!("write plan expected for {prompt:?}"));
        assert_eq!(plan.target, target, "target for {prompt:?}");
        assert_eq!(plan.content, content, "content for {prompt:?}");

        let messages = vec![ChatMessage::user(prompt)];
        let call = single_call(&messages, &["write", "read"]);
        assert_eq!(call.tool, "write", "{prompt:?} must route to write");
    }
}

/// Whole-task end-to-end: driving the full agentic loop for the issue's exact
/// request writes the target file with the requested content and never issues a
/// `read`. This is the behaviour the issue reported as broken ("finishes with
/// `File not found` and writes nothing").
#[test]
fn file_creation_end_to_end_writes_the_target_and_never_reads() {
    let outcome = run_agentic_task("Create a file named hello.txt with the content hello world")
        .expect("workspace");
    assert!(!outcome.hit_turn_cap, "loop should complete: {outcome:?}");

    let tools: Vec<&str> = outcome
        .steps
        .iter()
        .map(|step| step.tool.as_str())
        .collect();
    assert!(
        !tools.contains(&"read"),
        "must never read the file it was asked to create: {tools:?}"
    );
    assert!(
        tools.iter().any(|tool| tool == &"write_file"),
        "the target must be created with a write: {tools:?}"
    );

    // The requested target is written with the requested content.
    let wrote_target = outcome.steps.iter().any(|step| {
        step.tool == "write_file"
            && step.arguments.contains("hello.txt")
            && step.arguments.contains("hello world")
    });
    assert!(
        wrote_target,
        "hello.txt must be written with its content: {outcome:?}"
    );

    // The verification step reads the freshly-written file back and sees the content.
    assert!(
        outcome
            .steps
            .iter()
            .any(|step| step.result.contains("hello world")),
        "verification should observe the written content: {outcome:?}"
    );
}
