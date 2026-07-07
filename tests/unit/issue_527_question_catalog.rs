//! The issue-#527 question-catalog loop — *"generate all possible questions and
//! answer them"*, reachable through the agentic interface as the eleventh recipe.
//!
//! Issue #527 asks Formal AI to enumerate questions from smallest to largest out of a
//! frequency-tiered vocabulary, classify each candidate grammatically and logically,
//! and answer the meaningful ones with the best possible answer. The generator itself
//! is exercised by `issue_527.rs`; these pins lock the *agentic* capability:
//!
//! 1. the catalog records the four-way classification smallest-first and answers the
//!    grammatical-and-meaningful questions with the deterministic engine;
//! 2. the answered questions form a recall table (a repeated question is answered from
//!    the catalog, not re-derived) that never changes solver behaviour on its own;
//! 3. the committed `data/meta/question-catalog.lino` is byte-for-byte what the recipe
//!    renders and parses as Links Notation;
//! 4. the recipe routes without colliding with the sibling recipes and the planner and
//!    the in-repo Agent CLI driver walk it write → verify → final.

use formal_ai::agentic_coding::{
    is_question_catalog_task, plan_chat_step, question_catalog as recipe, run_agentic_task,
    AgenticPlan, PlannedToolCall, DRIVER_TOOLS, QUESTION_CATALOG_PATH, QUESTION_CATALOG_TASK,
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
fn catalog_classifies_smallest_first_and_answers_meaningful_questions() {
    let catalog = recipe::catalog();

    // Candidates are enumerated smallest-first (word count is non-decreasing).
    assert!(
        catalog
            .candidates
            .windows(2)
            .all(|pair| pair[0].word_count <= pair[1].word_count),
        "candidates must be recorded smallest-first: {:?}",
        catalog.candidates,
    );
    assert_eq!(
        catalog.candidates.first().map(|c| c.text.as_str()),
        Some("what?"),
        "the smallest candidate is the single most-frequent interrogative opener",
    );

    // The catalog demonstrates the four-way distinction: a fragment (one-word opener),
    // an ungrammatical candidate, and — among the answered — grammatical-and-meaningful.
    assert!(
        catalog
            .candidates
            .iter()
            .any(|c| c.class == "fragment" && c.grammar == "fragment"),
        "the catalog should record at least one fragment candidate",
    );
    assert!(
        catalog
            .candidates
            .iter()
            .any(|c| c.class == "ungrammatical"),
        "the catalog should record at least one ungrammatical candidate",
    );

    // Every answered entry is a grammatical, meaningful question with a real answer.
    assert!(
        !catalog.answered.is_empty(),
        "the catalog should answer at least one meaningful question",
    );
    for answered in &catalog.answered {
        assert!(answered.question.ends_with('?'));
        assert!(
            !answered.answer.trim().is_empty(),
            "every meaningful question gets a best-possible answer: {answered:?}",
        );
        assert!(
            (0.0..=1.0).contains(&answered.confidence),
            "confidence stays a probability: {answered:?}",
        );
    }

    // The engine answers what it can and stays honest about what it cannot: the identity
    // question resolves with full confidence, an unknown concept stays low-confidence.
    let identity = catalog
        .answer_for("what is you?")
        .expect("the identity question should be answered");
    assert_eq!(identity.intent, "identity");
    assert!(identity.confidence > 0.99);
    assert!(identity.answer.to_lowercase().contains("formal-ai"));
}

#[test]
fn answered_questions_form_a_recall_table() {
    // The catalog is a *recall table*: a question it already answered is recognised and
    // answered from the catalog (case/whitespace-insensitive), not re-derived. This is
    // the issue-#527 auto-learning link — without touching the human-gated ledger.
    let catalog = recipe::catalog();
    let first = catalog
        .answered
        .first()
        .expect("at least one answered question");

    let recalled = catalog
        .answer_for(&format!("  {}  ", first.question.to_uppercase()))
        .expect("a trivially rephrased question should be recalled");
    assert_eq!(recalled, first);

    // A question the catalog never generated is not recalled (no hallucinated answers).
    assert!(catalog.answer_for("what colour is the sky?").is_none());
}

#[test]
fn committed_catalog_document_is_generated_by_the_recipe() {
    // The committed artifact is *generated* from the recipe — never hand written — so it
    // can never drift, and it is byte-for-byte what the Agent CLI writes. It depends only
    // on the seed lexicon and the deterministic engine, so it is safe to pin.
    let committed = include_str!("../../data/meta/question-catalog.lino");
    assert_eq!(
        committed,
        recipe::render_document(),
        "the committed catalog is stale — regenerate it with `cargo run --example dump_question_catalog`",
    );
    parse_indented(committed).expect("the committed document should parse as Links Notation");
    assert!(committed.contains("question_catalog"));
    assert!(committed.contains("intent \"generate_all_possible_questions\""));
    assert!(committed.contains("class \"fragment\""));
    assert!(committed.contains("class \"ungrammatical\""));
    assert!(committed.contains("answered"));
}

#[test]
fn recognises_the_catalog_task_without_colliding_with_the_sibling_recipes() {
    assert!(is_question_catalog_task(QUESTION_CATALOG_TASK));
    assert!(is_question_catalog_task(
        "Please generate every possible question and answer each one."
    ));
    assert!(is_question_catalog_task(
        "Build the question catalog and record it in Links Notation."
    ));
    assert!(is_question_catalog_task(
        "Enumerate questions from smallest to largest and answer them."
    ));

    // Unrelated and sibling requests do not route here.
    assert!(!is_question_catalog_task("what files are in this folder?"));
    assert!(!is_question_catalog_task("what is formal ai?"));
    assert!(!is_question_catalog_task(
        formal_ai::agentic_coding::REPAIR_STRATEGY_TASK
    ));
    assert!(!is_question_catalog_task(
        formal_ai::agentic_coding::LEDGER_TASK
    ));
    assert!(!is_question_catalog_task(
        formal_ai::agentic_coding::self_heal::SELF_HEAL_TASK
    ));

    // The catalog task itself must not trip the sibling routers.
    assert!(!formal_ai::agentic_coding::is_repair_strategy_task(
        QUESTION_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_ledger_task(
        QUESTION_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_self_heal_task(
        QUESTION_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_source_graph_task(
        QUESTION_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_change_request_task(
        QUESTION_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_explain_task(
        QUESTION_CATALOG_TASK
    ));
}

#[test]
fn planner_walks_the_question_catalog_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(QUESTION_CATALOG_TASK)];

    // Step 1: write the generated catalog document (no web step — it is a pure function
    // of the seed lexicon and the deterministic engine).
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(QUESTION_CATALOG_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    answer_tool_call(&mut messages, &call, "wrote question-catalog.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(QUESTION_CATALOG_PATH));
    answer_tool_call(&mut messages, &call, &recipe::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the catalog.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(QUESTION_CATALOG_PATH));
            assert!(answer.contains("question catalog"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn committed_agent_cli_session_matches_a_fresh_run() {
    // The committed Agent CLI session (docs/case-studies/issue-527) is byte-for-byte what
    // a fresh driven run produces from the recipe's *own* task wording — the issue-#527
    // reproducibility pin, mirroring the issue-#538 sessions. It depends only on the seed
    // lexicon and the deterministic engine, so it is safe to pin.
    let committed =
        include_str!("../../docs/case-studies/issue-527/agent-cli-session-question-catalog.json");
    let fresh = run_agentic_task(QUESTION_CATALOG_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed question-catalog Agent CLI session is stale — regenerate it with \
         `formal-ai agent --task \"<QUESTION_CATALOG_TASK>\" --session-json \
         docs/case-studies/issue-527/agent-cli-session-question-catalog.json`",
    );
}

#[test]
fn driver_drives_the_question_catalog_recipe_to_a_write() {
    // End-to-end through the in-repo Agent CLI driver: the loop finishes and writes
    // exactly the generated catalog document.
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(QUESTION_CATALOG_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    assert_eq!(written["path"], QUESTION_CATALOG_PATH);
    assert!(outcome.final_answer.contains(QUESTION_CATALOG_PATH));
}
