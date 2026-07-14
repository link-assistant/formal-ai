//! The issue-#558 promotion ledger — the terminal, human-gated step that records a
//! self-healing lesson only after *both* the tests and the user accept it, and
//! recalls it on a repeated failure ("auto learning").
//!
//! Issue #558 asks for a self-healing algorithm that *"promotes improvements when
//! tests and the user accept them"* and writes the accepted result *"to mainline
//! history as an approved learning record"*. These pins lock that promotion
//! protocol: a green, faithful, human-approved `RepairCase` becomes a durable
//! `LedgerEntry`; every other case is refused with a specific reason; a repeated
//! failure is answered from the ledger instead of re-derived; and the ledger
//! serialises to valid Links Notation pinned byte-for-byte to a committed artifact.

use std::fs;
use std::path::Path;

use formal_ai::agentic_coding::{
    is_ledger_task, ledger, plan_chat_step, run_agentic_task, AgenticPlan, PlannedToolCall,
    DRIVER_TOOLS, LEDGER_PATH, LEDGER_TASK,
};
use formal_ai::{
    canonical_case, canonical_ledger, ChatMessage, HumanApproval, LearningLedger,
    PromotionRejected, RepairCase, RepairOutcome, ToolCall,
};
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

/// A green, faithful, `AwaitingReview` case — the only shape that may be promoted.
fn green_case() -> RepairCase {
    let case = canonical_case();
    assert_eq!(case.outcome, RepairOutcome::AwaitingReview);
    assert!(case.source_round_trip.faithful);
    case
}

#[test]
fn promotes_a_green_and_approved_case_into_an_approved_learning_record() {
    let mut ledger = LearningLedger::new();
    assert!(ledger.is_empty());

    let case = green_case();
    let entry = ledger
        .promote(&case, &HumanApproval::granted("maintainer"))
        .expect("a green, approved case must promote");

    // The approved learning record carries everything a future lookup needs.
    assert_eq!(entry.case_id, case.id);
    assert_eq!(entry.failure_prompt, case.failure_prompt);
    assert_eq!(entry.module_path, case.source_round_trip.module_path);
    assert_eq!(entry.rule_id, "reverse_sort_list_files");
    assert_eq!(entry.resolved_task, "list_files_reverse_sort");
    assert_eq!(entry.reviewer, "maintainer");
    assert!(entry.benchmark_passed >= 1);
    assert!(!entry.lesson_id.is_empty());
    assert_eq!(ledger.len(), 1);
}

#[test]
fn recalls_an_approved_lesson_on_a_repeated_failure() {
    // The payoff of auto-learning: a failure seen once and approved is answered from
    // the ledger the next time, without re-deriving it.
    let mut ledger = LearningLedger::new();
    let case = green_case();
    assert!(
        ledger.lesson_for(&case.failure_prompt).is_none(),
        "unknown before promotion"
    );

    ledger
        .promote(&case, &HumanApproval::granted("maintainer"))
        .unwrap();

    assert!(ledger.knows(&case.failure_prompt));
    let recalled = ledger
        .lesson_for(&case.failure_prompt)
        .expect("the lesson is recalled");
    assert_eq!(recalled.rule_id, "reverse_sort_list_files");
    // Matching tolerates whitespace/case rephrasings of the same failure.
    let rephrased = format!("  {}  ", case.failure_prompt.to_uppercase());
    assert!(
        ledger.knows(&rephrased),
        "normalised match on {rephrased:?}"
    );
    // An unrelated prompt is not falsely recalled.
    assert!(ledger.lesson_for("compute a factorial").is_none());
}

#[test]
fn refuses_promotion_when_the_user_declines() {
    let mut ledger = LearningLedger::new();
    let result = ledger.promote(&green_case(), &HumanApproval::declined("maintainer"));
    assert_eq!(result.unwrap_err(), PromotionRejected::HumanDeclined);
    assert!(ledger.is_empty(), "nothing is recorded without approval");
}

#[test]
fn refuses_promotion_when_the_benchmark_gate_is_not_green() {
    // A real case whose benchmark gate did not pass: a lesson exists but "tests" did
    // not accept, so it is blocked.
    let blocked = RepairCase::from_trace(
        formal_ai::canonical_failure_trace(),
        formal_ai::SourceRoundTrip::for_pinned_target(),
        formal_ai::BenchmarkGateReport::issue_362_from_counts(0, 5),
    );
    assert_eq!(blocked.outcome, RepairOutcome::BlockedByBenchmark);

    let mut ledger = LearningLedger::new();
    let result = ledger.promote(&blocked, &HumanApproval::granted("maintainer"));
    assert_eq!(result.unwrap_err(), PromotionRejected::TestsNotGreen);
    assert!(ledger.is_empty());
}

#[test]
fn refuses_promotion_when_no_lesson_was_synthesised() {
    let mut case = green_case();
    case.outcome = RepairOutcome::NoCandidate;

    let mut ledger = LearningLedger::new();
    let result = ledger.promote(&case, &HumanApproval::granted("maintainer"));
    assert_eq!(result.unwrap_err(), PromotionRejected::NoReviewableProposal);
    assert!(ledger.is_empty());
}

#[test]
fn refuses_promotion_when_the_source_does_not_round_trip() {
    // Recompile guardrail: a lesson whose source cannot be reconstructed
    // byte-for-byte must never be recorded, even if approved and green.
    let mut case = green_case();
    case.source_round_trip.faithful = false;

    let mut ledger = LearningLedger::new();
    let result = ledger.promote(&case, &HumanApproval::granted("maintainer"));
    assert_eq!(result.unwrap_err(), PromotionRejected::SourceNotFaithful);
    assert!(ledger.is_empty());
}

#[test]
fn promotion_is_idempotent_per_failure() {
    let mut ledger = LearningLedger::new();
    let case = green_case();
    ledger
        .promote(&case, &HumanApproval::granted("maintainer"))
        .unwrap();

    // A second promotion of the same failure is refused, and the ledger is unchanged.
    let result = ledger.promote(&case, &HumanApproval::granted("someone-else"));
    assert_eq!(result.unwrap_err(), PromotionRejected::AlreadyPromoted);
    assert_eq!(ledger.len(), 1);
}

#[test]
fn ledger_renders_valid_links_notation_with_a_stable_content_id() {
    let ledger = canonical_ledger();
    let notation = ledger.links_notation();
    parse_indented(&notation).expect("ledger should render valid Links Notation");
    assert!(notation.contains("learning_ledger"));
    assert!(notation.contains("human_gated \"true\""));
    assert!(notation.contains("lesson_count \"1\""));
    assert!(notation.contains("rule_id \"reverse_sort_list_files\""));
    assert!(notation.contains("reviewer \"maintainer\""));

    // Deterministic content id, and it changes with the ledger's contents.
    assert_eq!(ledger.content_id(), canonical_ledger().content_id());
    assert_ne!(LearningLedger::new().content_id(), ledger.content_id());
}

#[test]
fn committed_ledger_artifact_matches_the_generated_ledger() {
    // The committed artifact is *generated* (never hand-written) and pinned
    // byte-for-byte to the canonical ledger, so it can never silently drift.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let committed = fs::read_to_string(root.join("data/meta/learning-ledger.lino"))
        .expect("data/meta/learning-ledger.lino should be committed");
    let expected = format!("{}\n", canonical_ledger().links_notation());
    assert_eq!(
        committed, expected,
        "regenerate with `cargo run --example dump_learning_ledger > data/meta/learning-ledger.lino`"
    );
}

#[test]
fn rejection_slugs_and_approval_accessors_are_stable() {
    assert_eq!(PromotionRejected::HumanDeclined.slug(), "human_declined");
    assert_eq!(PromotionRejected::TestsNotGreen.slug(), "tests_not_green");
    assert_eq!(
        PromotionRejected::SourceNotFaithful.slug(),
        "source_not_faithful"
    );
    assert_eq!(
        PromotionRejected::NoReviewableProposal.slug(),
        "no_reviewable_proposal"
    );
    assert_eq!(
        PromotionRejected::AlreadyPromoted.slug(),
        "already_promoted"
    );

    let granted = HumanApproval::granted("reviewer-a");
    assert!(granted.is_granted());
    assert_eq!(granted.reviewer(), "reviewer-a");
    assert!(!HumanApproval::declined("reviewer-b").is_granted());
}

#[test]
fn recognises_the_ledger_task_without_colliding_with_the_sibling_recipes() {
    assert!(is_ledger_task(LEDGER_TASK));
    assert!(is_ledger_task("record the learning ledger"));
    assert!(is_ledger_task(
        "promote the lesson into the promotion ledger"
    ));
    // Unrelated and sibling self-inspection requests do not route here.
    assert!(!is_ledger_task("what files are in this folder?"));
    assert!(!is_ledger_task(
        formal_ai::agentic_coding::self_heal::SELF_HEAL_TASK
    ));
    assert!(!is_ledger_task(
        formal_ai::agentic_coding::source_links::SOURCE_LINKS_TASK
    ));
    // The ledger task itself must not trip the sibling routers.
    assert!(!formal_ai::agentic_coding::is_self_heal_task(LEDGER_TASK));
    assert!(!formal_ai::agentic_coding::is_self_ast_task(LEDGER_TASK));
    assert!(!formal_ai::agentic_coding::is_source_links_task(
        LEDGER_TASK
    ));
}

#[test]
fn planner_walks_the_ledger_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(LEDGER_TASK)];

    // Step 1: write the generated ledger document (no web step — it is a pure
    // function of the canonical approved ledger).
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(LEDGER_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], ledger::render_document());
    answer_tool_call(&mut messages, &call, "wrote learning-ledger.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(LEDGER_PATH));
    answer_tool_call(&mut messages, &call, &ledger::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the ledger.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(LEDGER_PATH));
            assert!(answer.contains("learning ledger"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn driver_drives_the_ledger_recipe_to_a_write() {
    // End-to-end through the in-repo Agent CLI driver: the loop finishes and writes
    // exactly the generated ledger document.
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(LEDGER_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], ledger::render_document());
    assert_eq!(written["path"], LEDGER_PATH);
    assert!(outcome.final_answer.contains(LEDGER_PATH));
}
