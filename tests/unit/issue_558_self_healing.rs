//! The issue-#558 self-healing / auto-learning loop — Formal AI driving its *own*
//! agentic CLI to run a closed repair pass on an input it could not answer.
//!
//! Issue #558 ("Auto learning") asks the system, when it cannot answer an input, to
//! reason about the failure, map it onto the source that would change (with a
//! source↔links round-trip), learn a benchmark-gated lesson, and — only with human
//! approval — promote it. These pins lock that closed loop: the composed
//! [`RepairCase`] reaches a human-gated `AwaitingReview` outcome, the mapped source
//! round-trips byte-for-byte through the meta-language, the deterministic planner
//! walks write → verify → final so the loop is reachable through the agentic
//! interface, and the committed `data/meta/self-healing-case.lino` is byte-for-byte
//! what the Agent CLI writes — never hand-authored.

use formal_ai::agentic_coding::{
    plan_chat_step, run_agentic_task, self_ast, self_heal, AgenticPlan, PlannedToolCall,
    DRIVER_TOOLS,
};
use formal_ai::{ChatMessage, RepairOutcome, SourceRoundTrip, ToolCall};
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
fn self_healing_case_closes_the_loop_and_stays_human_gated() {
    let case = self_heal::case();

    // The loop reasoned about a real failure the system could not answer directly.
    assert_eq!(
        case.failure_prompt,
        "List the files but sort the results in reverse order"
    );
    assert!(case
        .trace
        .events
        .iter()
        .any(|event| event.kind == "rule_synthesis_candidate"));

    // It mapped the failure onto real source that round-trips through the meta
    // language byte-for-byte (source → links → source).
    assert_eq!(
        case.source_round_trip.module_path,
        self_ast::TARGET_MODULE_PATH
    );
    assert!(
        case.source_round_trip.faithful,
        "the mapped source must round-trip losslessly through the links/meta language"
    );
    assert!(case.source_round_trip.named_node_count > 0);

    // It learned a benchmark-gated lesson that is *ready for review* — never applied.
    assert_eq!(case.outcome, RepairOutcome::AwaitingReview);
    assert!(case.outcome.has_reviewable_proposal());
    assert!(case.is_human_gated());
    assert_eq!(case.learning.adoptable_rules().len(), 1);
}

#[test]
fn a_failing_benchmark_gate_blocks_promotion_but_still_records_the_case() {
    // Same failure, but the benchmark ratchet would regress: the lesson is learned
    // yet adoption is blocked — the human gate holds.
    let case = formal_ai::RepairCase::from_trace(
        formal_ai::canonical_failure_trace(),
        SourceRoundTrip::for_pinned_target(),
        formal_ai::BenchmarkGateReport::issue_362_from_counts(3, 1),
    );
    assert_eq!(case.outcome, RepairOutcome::BlockedByBenchmark);
    assert!(case.learning.adoptable_rules().is_empty());
    assert!(!case.learning.proposals.is_empty());
    assert!(case.links_notation().contains("blocked_by_benchmark"));
}

#[test]
fn source_round_trip_is_lossless_for_a_real_module() {
    // The concrete form of #558's "translate the source to the meta language and
    // back": parse to links, reconstruct to source, byte-for-byte equal.
    let source = self_ast::target_source();
    assert!(self_ast::round_trips(source));
    assert_eq!(self_ast::reconstruct_source(source), source);
}

#[test]
fn repair_case_renders_valid_links_notation() {
    let lino = self_heal::case().links_notation();
    parse_indented(&lino).expect("repair case should render valid Links Notation");
    assert!(lino.contains("repair_case"));
    assert!(lino.contains("outcome \"awaiting_review\""));
    assert!(lino.contains("human_gated \"true\""));
    assert!(lino.contains("faithful \"true\""));
    // The composed sub-artifacts are folded in for a single auditable document.
    assert!(lino.contains("failure_trace"));
    assert!(lino.contains("learning_run"));
    assert!(lino.contains("adoption \"adoptable\""));
}

#[test]
fn recognises_the_self_heal_task() {
    assert!(self_heal::is_self_heal_task(self_heal::SELF_HEAL_TASK));
    assert!(self_heal::is_self_heal_task(
        "Please run your self-healing loop on that failure."
    ));
    assert!(self_heal::is_self_heal_task(
        "record a repair case for the input you couldn't answer"
    ));
    // Unrelated requests, and the sibling self-AST request, do not route here.
    assert!(!self_heal::is_self_heal_task(
        "what files are in this folder?"
    ));
    assert!(!self_heal::is_self_heal_task(
        "store the cst/ast of our meta algorithm so it can reason about itself"
    ));
    // The self-heal task itself must not trip the self-AST router.
    assert!(!self_ast::is_self_ast_task(self_heal::SELF_HEAL_TASK));
}

#[test]
fn planner_walks_the_self_heal_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(self_heal::SELF_HEAL_TASK)];

    // Step 1: no web step — the repair case is a pure function of the canonical
    // failure, so the planner goes straight to writing the generated document.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(self_heal::SELF_HEAL_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], self_heal::render_document());
    answer_tool_call(&mut messages, &call, "wrote self-healing-case.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(self_heal::SELF_HEAL_PATH));
    answer_tool_call(&mut messages, &call, &self_heal::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the repair case.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(self_heal::SELF_HEAL_PATH));
            assert!(answer.contains("human-approved"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn committed_self_healing_case_is_generated_and_written_by_the_driver() {
    // The committed repair-case artifact is *generated* by running the real
    // self-healing loop — never hand written — so it can never drift, and it is
    // byte-for-byte what the Agent CLI writes.
    let committed = include_str!("../../data/meta/self-healing-case.lino");
    assert_eq!(
        committed,
        self_heal::render_document(),
        "the committed self-healing case is stale — regenerate it with `cargo run --example dump_self_healing_case`"
    );
    assert!(committed.contains("outcome \"awaiting_review\""));
    assert!(committed.contains("faithful \"true\""));

    // End-to-end: the in-repo Agent CLI drives the loop to completion and writes
    // exactly this document. DRIVER_TOOLS advertises write_file + run_command.
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(self_heal::SELF_HEAL_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], self_heal::render_document());
    assert_eq!(written["path"], self_heal::SELF_HEAL_PATH);
    assert!(outcome.final_answer.contains(self_heal::SELF_HEAL_PATH));
}
