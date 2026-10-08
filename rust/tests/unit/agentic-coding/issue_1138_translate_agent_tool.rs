//! Plan 16 L2g (issue #1138): the agent `translate` tool. The registry
//! record, the driver's advertised surface, the planner lowering from a
//! natural source-tree request, and the solver's no-tool fallback all have
//! to agree on one shape — `{"from", "to", "path", "write"}` — or the
//! dogfood loop of L3 turns against a tool nobody can call.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, DRIVER_TOOLS, plan_chat_step};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(root().join(path)).expect("the pinned repository file is readable")
}

/// A file no fixture seeds, so the solver's read of it is a missing source
/// on every machine the suite runs on.
const MISSING_SOURCE: &str = "js/no_such_probe_file.js";

#[test]
fn the_translate_tool_is_registered_in_the_seed() {
    let tools = read("data/seed/tools.lino");
    for needle in [
        "tool tool_translate",
        "inputs (\"from\" \"to\" \"path\" \"write\")",
        "outputs (\"target\" \"carried\" \"refused\" \"written\")",
        "rust:meta_translate",
    ] {
        assert!(
            tools.contains(needle),
            "the registry record must pin {needle:?}"
        );
    }
    // The embedded mirror ships the same record into the crate package.
    let embedded = read("rust/embedded/data/seed/tools.lino");
    assert_eq!(
        embedded, tools,
        "the embedded mirror must be refreshed (scripts/mirror-package-data.rs --write)"
    );
}

#[test]
fn the_driver_advertises_and_executes_the_translate_tool() {
    assert_eq!(DRIVER_TOOLS.len(), 5);
    assert!(
        DRIVER_TOOLS.contains(&"translate"),
        "DRIVER_TOOLS must advertise translate: {DRIVER_TOOLS:?}"
    );
    // The description is seed-grounded (R379): the driver looks up the
    // `translate_tool_schema` row instead of carrying a source literal, and
    // the row states the write mapping's own argument shape.
    let schema = formal_ai::render_response("translate_tool_schema", "en", &[])
        .expect("the seed renders the tool schema");
    for needle in ["whole tree", "from js or ts", "write boolean"] {
        assert!(
            schema.contains(needle),
            "the schema must state {needle:?}: {schema}"
        );
    }
    let driver = read("rust/src/agentic_coding/driver.rs");
    for needle in ["translate_tool_schema", "translate_without_write"] {
        assert!(
            driver.contains(needle),
            "the driver surface must pin {needle:?}"
        );
    }
}

#[test]
fn the_planner_lowers_a_source_tree_request_to_one_translate_call() {
    let messages = vec![ChatMessage::user(format!(
        "Translate {MISSING_SOURCE} to TypeScript and write it"
    ))];
    match plan_chat_step(&messages, &["translate", "write_file"]) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(
                calls.len(),
                1,
                "one tool call, not a plan of many: {calls:?}"
            );
            assert_eq!(calls[0].tool, "translate");
            assert_eq!(
                calls[0].arguments,
                format!(
                    "{{\"from\":\"js\",\"to\":\"ts\",\"path\":\"{MISSING_SOURCE}\",\"write\":true}}"
                ),
                "the lowered arguments are the write mapping's own shape"
            );
        }
        other => panic!(
            "a source-tree translation with the tool advertised must lower to a \
             translate call, got {other:?}"
        ),
    }
}

#[test]
fn without_the_tool_the_solver_answers_with_its_seed_gap() {
    let messages = vec![ChatMessage::user(format!(
        "Translate {MISSING_SOURCE} to TypeScript and write it"
    ))];
    // The client advertises writing but not translation: the read belongs
    // to the shared solver, whose honest gap — not a guessed target — is
    // the answer, exactly like `write_program` on a toolless client.
    match plan_chat_step(&messages, &["write_file"]) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(
                answer.contains(MISSING_SOURCE),
                "the gap names the source: {answer}"
            );
            assert!(
                answer.contains("no source file to translate at that path"),
                "the gap text is the seed response, not an invented literal: {answer}"
            );
        }
        other => panic!(
            "without the tool the solver's rendered gap is the final answer, \
             got {other:?}"
        ),
    }
}
