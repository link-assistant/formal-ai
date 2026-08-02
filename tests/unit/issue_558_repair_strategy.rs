//! The issue-#558 general repair-classification loop — deciding *which part* of the
//! system to repair for **every** class of failure (`R558-02`).
//!
//! Issue #558 asks that a failure trace can trigger a repair run that changes a
//! *solver method*, a *data record*, or a *test*. The self-healing slice executes the
//! solver-method path on a single canonical failure; this slice generalises the front
//! of the loop. These pins lock that: the deterministic classifier maps each canonical
//! trace onto the correct target, the strategy serialises to valid Links Notation with a
//! stable content id, it stays human-gated and proposal-only, the committed
//! `data/meta/repair-strategies.lino` is byte-for-byte what the recipe renders, and the
//! recipe is reachable through the agentic interface as the tenth recipe.

use formal_ai::agentic_coding::{
    is_repair_strategy_task, plan_chat_step, repair_strategy as recipe, run_agentic_task,
    AgenticPlan, PlannedToolCall, DRIVER_TOOLS, REPAIR_STRATEGY_PATH, REPAIR_STRATEGY_TASK,
};
use formal_ai::repair_strategy::{
    canonical_data_record_failure, canonical_solver_method_failure, canonical_strategies,
    canonical_test_failure, RepairStrategy, RepairTarget,
};
use formal_ai::{ChatMessage, ToolCall};
use lino_objects_codec::format::parse_indented;

fn expect_single_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(mut calls)) => {
            assert_eq!(calls.len(), 1, "the planner emits one call per step");
            calls.remove(0)
        }
        other => panic!("expected a single tool call, got {other:?}"),
    }
}

fn answer_tool_call(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

#[test]
fn the_classifier_maps_every_failure_class_onto_the_right_target() {
    // The core guarantee: the loop is *total* — an arbitrary failure trace is
    // deterministically classified into exactly one of the three targets issue #558
    // names, keyed on the trace's own signals.
    let solver = RepairStrategy::classify(&canonical_solver_method_failure());
    assert_eq!(solver.target, RepairTarget::SolverMethod);
    assert!(solver.rationale.contains("rule_synthesis"));
    assert!(solver.proposed_change.contains("solver method"));

    let data = RepairStrategy::classify(&canonical_data_record_failure());
    assert_eq!(data.target, RepairTarget::DataRecord);
    assert!(data.rationale.contains("missing surface"));
    assert!(data.proposed_change.contains("seed data record"));

    let test = RepairStrategy::classify(&canonical_test_failure());
    assert_eq!(test.target, RepairTarget::Test);
    assert!(test.rationale.contains("regression"));
    assert!(test.proposed_change.contains("guard test"));

    // Provenance and slugs are consistent across the three targets.
    assert_eq!(
        [solver.target.slug(), data.target.slug(), test.target.slug()],
        ["solver_method", "data_record", "test"]
    );
    for strategy in [&solver, &data, &test] {
        assert!(strategy.is_human_gated());
        assert!(!strategy.trace_id.is_empty());
    }
}

#[test]
fn classification_is_deterministic_and_content_addressed() {
    // Same trace → same strategy and same content id; different traces → different ids.
    let a = RepairStrategy::classify(&canonical_solver_method_failure());
    let b = RepairStrategy::classify(&canonical_solver_method_failure());
    assert_eq!(a, b);
    assert_eq!(a.content_id(), b.content_id());

    let other = RepairStrategy::classify(&canonical_test_failure());
    assert_ne!(a.id, other.id);
    assert_ne!(a.content_id(), other.content_id());
}

#[test]
fn strategy_renders_valid_links_notation() {
    let strategy = RepairStrategy::classify(&canonical_data_record_failure());
    let notation = strategy.links_notation();
    parse_indented(&notation).expect("the strategy should render valid Links Notation");
    assert!(notation.starts_with("repair_strategy"));
    assert!(notation.contains("target \"data_record\""));
    assert!(notation.contains("human_gated \"true\""));
    assert!(notation.contains("rationale"));
    assert!(notation.contains("proposed_change"));
    assert!(notation.contains("verification"));
}

#[test]
fn the_canonical_set_covers_all_three_classes() {
    let strategies = canonical_strategies();
    let targets: Vec<_> = strategies.iter().map(|s| s.target).collect();
    assert_eq!(
        targets,
        vec![
            RepairTarget::SolverMethod,
            RepairTarget::DataRecord,
            RepairTarget::Test
        ],
        "the canonical set must cover every repair class in a stable order"
    );
}

#[test]
fn committed_repair_strategies_document_is_generated_by_the_recipe() {
    // The committed artifact is *generated* from the classifier — never hand written —
    // so it can never drift, and it is byte-for-byte what the Agent CLI writes. Unlike
    // the source-links/change-request documents it depends only on self-contained
    // canonical traces, so it is safe to pin.
    let committed = include_str!("../../data/meta/repair-strategies.lino");
    assert_eq!(
        committed,
        recipe::render_document(),
        "the committed repair-strategies document is stale — regenerate it with `cargo run --example dump_repair_strategies`"
    );
    parse_indented(committed).expect("the committed document should parse as Links Notation");
    assert!(committed.contains("repair_strategies"));
    assert!(committed.contains("strategy_count \"3\""));
    assert!(committed.contains("target \"solver_method\""));
    assert!(committed.contains("target \"data_record\""));
    assert!(committed.contains("target \"test\""));
}

#[test]
fn recognises_the_repair_task_without_colliding_with_the_sibling_recipes() {
    assert!(is_repair_strategy_task(REPAIR_STRATEGY_TASK));
    assert!(is_repair_strategy_task(
        "Please classify a failure and produce a repair strategy."
    ));
    assert!(is_repair_strategy_task(
        "Decide which part to repair: solver method, a data record, or a test."
    ));
    // Unrelated and sibling requests do not route here.
    assert!(!is_repair_strategy_task("what files are in this folder?"));
    assert!(!is_repair_strategy_task("change this line to uppercase"));
    assert!(!is_repair_strategy_task(
        formal_ai::agentic_coding::self_heal::SELF_HEAL_TASK
    ));
    assert!(!is_repair_strategy_task(
        formal_ai::agentic_coding::CHANGE_TASK
    ));
    assert!(!is_repair_strategy_task(
        formal_ai::agentic_coding::LEDGER_TASK
    ));
    // The repair task itself must not trip the sibling routers (in particular the
    // self-healing recipe, whose keywords must stay disjoint).
    assert!(!formal_ai::agentic_coding::is_self_heal_task(
        REPAIR_STRATEGY_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_self_ast_task(
        REPAIR_STRATEGY_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_source_links_task(
        REPAIR_STRATEGY_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_ledger_task(
        REPAIR_STRATEGY_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_explain_task(
        REPAIR_STRATEGY_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_change_request_task(
        REPAIR_STRATEGY_TASK
    ));
}

#[test]
fn planner_walks_the_repair_strategy_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(REPAIR_STRATEGY_TASK)];

    // Step 1: write the generated repair-strategies document (no web step — it is a pure
    // function of the self-contained canonical traces).
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(REPAIR_STRATEGY_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    answer_tool_call(&mut messages, &call, "wrote repair-strategies.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(REPAIR_STRATEGY_PATH));
    answer_tool_call(&mut messages, &call, &recipe::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the strategies.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(REPAIR_STRATEGY_PATH));
            assert!(answer.contains("solver method, a data record, or a test"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn driver_drives_the_repair_strategy_recipe_to_a_write() {
    // End-to-end through the in-repo Agent CLI driver: the loop finishes and writes
    // exactly the generated repair-strategies document.
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(REPAIR_STRATEGY_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    assert_eq!(written["path"], REPAIR_STRATEGY_PATH);
    assert!(outcome.final_answer.contains(REPAIR_STRATEGY_PATH));
}
