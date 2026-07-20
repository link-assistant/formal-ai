//! The issue-#558 grounded self-explanation — answering "how does Formal AI work?"
//! from the system's *own* source, data, and tests rather than prose docs (`R558-08`).
//!
//! Issue #558 asks that a user be able to *"ask how Formal AI itself works"* and
//! receive an answer *"grounded in its source and data"*. These pins lock that
//! grounding: every source citation resolves against the compile-time owned manifest
//! with a matching content id (a fabricated citation cannot even be constructed),
//! every data/test citation points at a file that exists on disk, the explanation
//! serialises to valid Links Notation with a stable content id, and it is reachable
//! through the agentic interface as the eighth recipe.

use std::path::Path;

use formal_ai::agentic_coding::{
    explain, is_explain_task, plan_chat_step, run_agentic_task, AgenticPlan, PlannedToolCall,
    DRIVER_TOOLS, EXPLAIN_PATH, EXPLAIN_TASK,
};
use formal_ai::{
    canonical_explanation, owned_manifest, ChatMessage, Citation, CitationKind, SystemExplanation,
    ToolCall,
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

#[test]
fn every_source_citation_is_grounded_in_the_owned_manifest() {
    // The core guarantee: a source citation resolves to a real owned file, and its
    // content id matches what the manifest content-addresses — so the explanation
    // cannot cite source the repository does not actually ship.
    let manifest = owned_manifest();
    let explanation = canonical_explanation();
    let source_citations = explanation.citations_of(CitationKind::Source);
    assert!(
        !source_citations.is_empty(),
        "the explanation must cite real source"
    );
    for citation in source_citations {
        let digest = manifest
            .iter()
            .find(|digest| digest.path == citation.path)
            .unwrap_or_else(|| panic!("cited source not in manifest: {}", citation.path));
        assert_eq!(
            citation.content_id.as_deref(),
            Some(digest.content_id.as_str()),
            "content id for {} must match the manifest",
            citation.path
        );
    }
}

#[test]
fn every_data_and_test_citation_exists_on_disk() {
    // Data/test citations are not embedded in the binary, so their existence is
    // checked against the checked-in tree here.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let explanation = canonical_explanation();
    for citation in explanation.citations() {
        if citation.kind == CitationKind::Source {
            continue;
        }
        assert!(
            root.join(&citation.path).exists(),
            "cited {} artifact does not exist: {}",
            citation.kind.slug(),
            citation.path
        );
        assert!(
            citation.content_id.is_none(),
            "data/test citations carry a path reference, not an embedded content id"
        );
    }
}

#[test]
fn the_explanation_covers_all_three_artifact_kinds() {
    let explanation = canonical_explanation();
    assert!(explanation.section_count() >= 3, "several grounded topics");
    // Every section must be backed by at least one real citation.
    for section in &explanation.sections {
        assert!(
            !section.citations.is_empty(),
            "topic {} has no citation",
            section.topic
        );
        assert!(!section.statement.trim().is_empty());
    }
    // Source, data, and test are all represented — the issue asks for all three.
    assert!(!explanation.citations_of(CitationKind::Source).is_empty());
    assert!(!explanation.citations_of(CitationKind::Data).is_empty());
    assert!(!explanation.citations_of(CitationKind::Test).is_empty());
    assert_eq!(
        explanation.citation_count(),
        explanation.citations().len(),
        "the citation count is the flattened total"
    );
}

#[test]
#[should_panic(expected = "not in the owned manifest")]
fn a_fabricated_source_citation_cannot_be_constructed() {
    // The grounding is enforced at construction: citing a path the repository does
    // not ship panics rather than lying at runtime.
    let _ = Citation::source("src/this_module_does_not_exist.rs");
}

#[test]
fn explanation_renders_valid_links_notation_with_a_stable_content_id() {
    let explanation = canonical_explanation();
    let notation = explanation.links_notation();
    parse_indented(&notation).expect("the explanation should render valid Links Notation");
    assert!(notation.contains("system_explanation"));
    assert!(notation.contains("question \"how does Formal AI work?\""));
    assert!(notation.contains("kind source"));
    assert!(notation.contains("kind data"));
    assert!(notation.contains("kind test"));
    // The header anchors the answer to the source-to-links graph.
    assert!(notation.contains("source_manifest_content_id"));

    // Deterministic content id; an empty explanation has a different one.
    assert_eq!(
        explanation.content_id(),
        canonical_explanation().content_id()
    );
    assert_ne!(
        SystemExplanation { sections: vec![] }.content_id(),
        explanation.content_id()
    );
}

#[test]
fn recognises_the_explain_task_without_colliding_with_the_sibling_recipes() {
    assert!(is_explain_task(EXPLAIN_TASK));
    assert!(is_explain_task("How does Formal AI work?"));
    assert!(is_explain_task(
        "Explain how Formal AI itself works, grounded in its source, data, and tests."
    ));
    // Unrelated and sibling self-inspection requests do not route here.
    assert!(!is_explain_task("what files are in this folder?"));
    assert!(!is_explain_task("how do I sort a list in reverse?"));
    assert!(!is_explain_task(
        formal_ai::agentic_coding::self_heal::SELF_HEAL_TASK
    ));
    assert!(!is_explain_task(
        formal_ai::agentic_coding::source_links::SOURCE_LINKS_TASK
    ));
    assert!(!is_explain_task(formal_ai::agentic_coding::LEDGER_TASK));
    // The explain task itself must not trip the sibling routers.
    assert!(!formal_ai::agentic_coding::is_self_heal_task(EXPLAIN_TASK));
    assert!(!formal_ai::agentic_coding::is_self_ast_task(EXPLAIN_TASK));
    assert!(!formal_ai::agentic_coding::is_source_links_task(
        EXPLAIN_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_ledger_task(EXPLAIN_TASK));
}

#[test]
fn planner_walks_the_explain_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(EXPLAIN_TASK)];

    // Step 1: write the generated grounded-explanation document (no web step — it is a
    // pure function of the system's own embedded source).
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(EXPLAIN_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], explain::render_document());
    answer_tool_call(&mut messages, &call, "wrote how-formal-ai-works.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(EXPLAIN_PATH));
    answer_tool_call(&mut messages, &call, &explain::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the explanation.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(EXPLAIN_PATH));
            assert!(answer.contains("how Formal AI works"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn driver_drives_the_explain_recipe_to_a_write() {
    // End-to-end through the in-repo Agent CLI driver: the loop finishes and writes
    // exactly the generated grounded-explanation document.
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(EXPLAIN_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], explain::render_document());
    assert_eq!(written["path"], EXPLAIN_PATH);
    assert!(outcome.final_answer.contains(EXPLAIN_PATH));
}
