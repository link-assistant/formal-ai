//! The issue-#558 whole-repository source ↔ links projection — Formal AI translating
//! its *entire* source code to the links / meta language and back, reachable through
//! the agentic interface.
//!
//! Issue #558 asks for a meta-algorithm advanced enough to *"recompile itself"*:
//! *"translate the entire source code of our system to links / meta language (that
//! must be present in our data), and back to the source code."* These pins lock that
//! whole-repository translation: every owned source file is enumerated and
//! content-addressed as data (the cheap manifest), a representative slice of real
//! modules round-trips byte-for-byte through the sole CST/AST engine (the lossless
//! proof), the projection renders as valid Links Notation, and the deterministic
//! planner walks write → verify → final so the loop is reachable through the agentic
//! interface. The exhaustive lossless proof over *all* owned files is the library
//! invariant, verified by an ignored-by-default test (it parses every file, which is
//! deliberately slow).

use formal_ai::agentic_coding::{
    plan_chat_step, run_agentic_task, source_graph, AgenticPlan, PlannedToolCall, DRIVER_TOOLS,
};
use formal_ai::{
    owned_file_count, owned_manifest, owned_manifest_content_id, owned_manifest_notation,
    owned_source_files, owned_total_bytes, ChatMessage, SourceGraph, SourceModuleProjection,
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

/// Build a synthetic projection with a chosen faithfulness so coverage math can be
/// exercised without paying for a real parse.
fn fake_module(path: &str, faithful: bool) -> SourceModuleProjection {
    SourceModuleProjection {
        path: path.to_owned(),
        byte_len: 10,
        content_id: format!("id_{path}"),
        total_link_count: 5,
        named_node_count: 3,
        faithful,
    }
}

#[test]
fn owned_manifest_content_addresses_every_source_file() {
    let manifest = owned_manifest();
    let files = owned_source_files();

    // The entire owned source tree is present in our data as content-addressed links.
    assert_eq!(manifest.len(), owned_file_count());
    assert_eq!(manifest.len(), files.len());
    assert!(manifest.len() > 150, "the whole repository, not a corner");

    let mut total_bytes = 0usize;
    let mut previous: Option<&str> = None;
    for (digest, (path, source)) in manifest.iter().zip(files.iter()) {
        assert_eq!(digest.path, *path);
        assert_eq!(
            std::path::Path::new(&digest.path).extension(),
            Some(std::ffi::OsStr::new("rs")),
            "{}",
            digest.path
        );
        assert!(digest.path.starts_with("src/"), "{}", digest.path);
        assert_eq!(digest.byte_len, source.len());
        assert!(!digest.content_id.is_empty());
        // Deterministic, path-sorted, no duplicates.
        if let Some(prev) = previous {
            assert!(
                prev < digest.path.as_str(),
                "manifest must be sorted & unique: {prev} !< {}",
                digest.path
            );
        }
        previous = Some(digest.path.as_str());
        total_bytes += source.len();
    }
    assert_eq!(total_bytes, owned_total_bytes());

    // The whole tree collapses to one stable id, deterministic across calls.
    assert_eq!(owned_manifest_content_id(), owned_manifest_content_id());
    assert!(!owned_manifest_content_id().is_empty());
}

#[test]
fn owned_manifest_renders_valid_links_notation() {
    let notation = owned_manifest_notation();
    parse_indented(&notation).expect("manifest should render valid Links Notation");
    assert!(notation.contains("source_manifest"));
    assert!(notation.contains("engine meta_language"));
    assert!(notation.contains(&format!("file_count {}", owned_file_count())));
    assert!(notation.contains(&format!("total_bytes {}", owned_total_bytes())));
    // This module's own source is part of the tree it describes.
    assert!(notation.contains("src/self_source_graph.rs"));
}

#[test]
fn representative_slice_round_trips_losslessly() {
    // The concrete "and back" proof over real, varied modules: parse to links,
    // reconstruct to source, byte-for-byte equal — for every module in the slice.
    let slice = source_graph::slice();
    assert!(slice.module_count() >= 2, "a real, spread slice");
    assert!(
        slice.is_fully_faithful(),
        "every module in the representative slice must round-trip losslessly"
    );
    assert_eq!(slice.coverage_permille(), 1000);
    for module in &slice.modules {
        assert!(module.faithful, "{} did not round-trip", module.path);
        assert!(module.named_node_count > 0, "{}", module.path);
        assert!(module.byte_len > 0, "{}", module.path);
    }
}

#[test]
fn compile_projects_a_known_module_losslessly() {
    // `compile` is the general primitive — it projects any file list. A single known
    // module round-trips and yields 100% coverage.
    let planner = owned_source_files()
        .iter()
        .copied()
        .find(|(path, _)| *path == "src/agentic_coding/planner.rs")
        .expect("the planner module is embedded");
    let graph = SourceGraph::compile(&[planner]);
    assert_eq!(graph.module_count(), 1);
    assert_eq!(graph.faithful_count(), 1);
    assert!(graph.is_fully_faithful());
    assert_eq!(graph.coverage_permille(), 1000);
    assert!(graph.total_named_node_count() > 0);
    assert_eq!(graph.total_byte_len(), planner.1.len());
}

#[test]
fn coverage_is_integer_permille_and_flags_unfaithful_modules() {
    // Deterministic coverage math without a real parse: 3 of 4 faithful → 750‰.
    let graph = SourceGraph {
        modules: vec![
            fake_module("a.rs", true),
            fake_module("b.rs", true),
            fake_module("c.rs", false),
            fake_module("d.rs", true),
        ],
    };
    assert_eq!(graph.module_count(), 4);
    assert_eq!(graph.faithful_count(), 3);
    assert_eq!(graph.coverage_permille(), 750);
    assert!(!graph.is_fully_faithful());
    let unfaithful = graph.unfaithful_modules();
    assert_eq!(unfaithful.len(), 1);
    assert_eq!(unfaithful[0].path, "c.rs");

    // An empty graph is 0‰ and never "fully faithful".
    let empty = SourceGraph { modules: vec![] };
    assert_eq!(empty.coverage_permille(), 0);
    assert!(!empty.is_fully_faithful());
}

#[test]
fn render_document_is_valid_links_and_reports_the_whole_repo() {
    let document = source_graph::render_document();
    parse_indented(&document).expect("projection should render valid Links Notation");
    assert!(document.contains("self_source_graph"));
    assert!(document.contains("engine meta_language"));
    assert!(document.contains("task translate_entire_source_to_links_and_back"));
    // The cheap "entire source in our data" view accounts for every file.
    assert!(document.contains("entire_source"));
    assert!(document.contains(&format!("file_count {}", owned_file_count())));
    assert!(document.contains("manifest_content_id"));
    // The lossless "and back" proof over the representative slice.
    assert!(document.contains("round_trip_proof"));
    assert!(document.contains("slice_fully_faithful true"));
    assert!(document.contains("faithful true"));
    // Exactly one trailing newline.
    assert!(document.ends_with('\n') && !document.ends_with("\n\n"));
}

#[test]
fn recognises_the_source_graph_task() {
    assert!(source_graph::is_source_graph_task(
        source_graph::SOURCE_GRAPH_TASK
    ));
    assert!(source_graph::is_source_graph_task(
        "recompile yourself: project the whole source graph to links"
    ));
    assert!(source_graph::is_source_graph_task(
        "translate the entire source of the system to links and back"
    ));
    // Unrelated requests, and the sibling self-inspection requests, do not route here.
    assert!(!source_graph::is_source_graph_task(
        "what files are in this folder?"
    ));
    assert!(!source_graph::is_source_graph_task(
        "store the cst/ast of our meta algorithm so it can reason about itself"
    ));
    assert!(!source_graph::is_source_graph_task(
        formal_ai::agentic_coding::self_heal::SELF_HEAL_TASK
    ));
    // The source-graph task itself must not trip the sibling routers.
    assert!(!formal_ai::agentic_coding::is_self_ast_task(
        source_graph::SOURCE_GRAPH_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_self_heal_task(
        source_graph::SOURCE_GRAPH_TASK
    ));
}

#[test]
fn planner_walks_the_source_graph_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(source_graph::SOURCE_GRAPH_TASK)];

    // Step 1: no web step — the projection is a pure function of the embedded source,
    // so the planner goes straight to writing the generated document.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(source_graph::SOURCE_GRAPH_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], source_graph::render_document());
    answer_tool_call(&mut messages, &call, "wrote self-source-graph.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(source_graph::SOURCE_GRAPH_PATH));
    answer_tool_call(&mut messages, &call, &source_graph::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the projection.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(source_graph::SOURCE_GRAPH_PATH));
            assert!(answer.contains("entire source"));
            assert!(answer.contains("recompil"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn driver_drives_the_source_graph_projection_to_a_write() {
    // End-to-end through the in-repo Agent CLI driver: the loop finishes and writes
    // exactly the generated projection document. DRIVER_TOOLS advertises write_file.
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(source_graph::SOURCE_GRAPH_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], source_graph::render_document());
    assert_eq!(written["path"], source_graph::SOURCE_GRAPH_PATH);
    assert!(outcome
        .final_answer
        .contains(source_graph::SOURCE_GRAPH_PATH));
}

#[test]
#[ignore = "exhaustive: parses every owned source file through the CST/AST engine (minutes in debug); run with `cargo test -- --ignored`"]
fn exhaustive_whole_repo_round_trip_is_lossless() {
    // The full "and back" invariant issue #558 requires: EVERY owned source file
    // round-trips byte-for-byte through the meta-language links network.
    let graph = SourceGraph::owned();
    assert_eq!(graph.module_count(), owned_file_count());
    let unfaithful: Vec<&str> = graph
        .unfaithful_modules()
        .iter()
        .map(|module| module.path.as_str())
        .collect();
    assert!(
        graph.is_fully_faithful(),
        "these modules did not round-trip losslessly: {unfaithful:?}"
    );
    assert_eq!(graph.coverage_permille(), 1000);
}
