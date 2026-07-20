//! The issue-#558 user-requested self-change — turning a natural-language "change
//! Formal AI itself" request into a reviewable pull request through the *same*
//! human-gated repair loop (`R558-07`).
//!
//! Issue #558 asks that *"users must be able to ask for changes in the AI system
//! through this mechanism"*, producing *"requirements, tests, patches, and a reviewable
//! PR through the same repair loop"*. These pins lock that: the request derives a
//! requirement, a test, and a patch plan whose target module is grounded against the
//! compile-time owned manifest (a fabricated target cannot be constructed), the whole
//! thing serialises to valid Links Notation with a stable content id, it is accepted
//! only when a benchmark gate is green *and* a human approves, and it is reachable
//! through the agentic interface as the ninth recipe.

use formal_ai::agentic_coding::{
    change_request as recipe, is_change_request_task, plan_chat_step, run_agentic_task,
    AgenticPlan, PlannedToolCall, CHANGE_PATH, CHANGE_TASK, DRIVER_TOOLS,
};
use formal_ai::self_improvement::BenchmarkGateReport;
use formal_ai::{
    canonical_change_request, owned_manifest, ChangeRejected, ChangeRequest, ChatMessage,
    HumanApproval, ToolCall,
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
fn the_target_module_is_grounded_in_the_owned_manifest() {
    // The core guarantee: the change targets a real owned module, and its content id
    // matches what the manifest content-addresses — so a request cannot target source
    // the repository does not actually ship.
    let manifest = owned_manifest();
    let request = canonical_change_request();
    let digest = manifest
        .iter()
        .find(|digest| digest.path == request.target_module)
        .unwrap_or_else(|| panic!("target not in manifest: {}", request.target_module));
    assert_eq!(
        request.target_content_id, digest.content_id,
        "the target content id must match the manifest"
    );
}

#[test]
#[should_panic(expected = "not in the owned manifest")]
fn a_change_cannot_target_a_module_that_is_not_owned() {
    // Grounding is enforced at construction: targeting a path the repository does not
    // ship panics rather than lying at runtime.
    let _ = ChangeRequest::for_module("please add a feature", "src/this_does_not_exist.rs");
}

#[test]
#[should_panic(expected = "non-empty request")]
fn an_empty_request_cannot_be_constructed() {
    let _ = ChangeRequest::for_module("   ", "src/agentic_coding/planner.rs");
}

#[test]
fn a_request_derives_a_requirement_a_test_and_a_patch_plan() {
    // A natural-language request becomes the four artifacts the issue names:
    // requirement, test, patch plan, and (via links_notation) a reviewable PR.
    let request = ChangeRequest::for_module(
        "Please add a reverse-sort capability to Formal AI.",
        "src/agentic_coding/planner.rs",
    );
    // Politeness is stripped and the requirement reads as an imperative statement.
    assert_eq!(
        request.derived_requirement,
        "The system must add a reverse-sort capability to Formal AI."
    );
    // The test name is a deterministic snake_case slug of the request.
    assert!(request.proposed_test.starts_with("user_requested_change_"));
    assert!(request.proposed_test.contains("add"));
    assert!(!request.proposed_test.contains(' '));
    // The patch plan is an ordered, non-empty set of steps ending in a reviewed merge.
    assert!(request.patch_plan.len() >= 4, "a multi-step patch plan");
    assert!(request
        .patch_plan
        .iter()
        .any(|step| step.contains(&request.proposed_test)));
    assert!(request
        .patch_plan
        .iter()
        .any(|step| step.contains("src/agentic_coding/planner.rs")));
    assert!(request
        .patch_plan
        .last()
        .is_some_and(|step| step.contains("pull request")));
    assert!(request.is_human_gated());
}

#[test]
fn a_change_merges_only_when_tests_are_green_and_a_human_approves() {
    // The same two acceptance conditions as the learning ledger: a green benchmark
    // gate AND an explicit human approval. Every other combination is refused.
    let request = canonical_change_request();
    let green = BenchmarkGateReport::issue_362_from_counts(4, 0);
    let red = BenchmarkGateReport::issue_362_from_counts(0, 4);

    // Tests red → refused, regardless of approval.
    assert_eq!(
        request.review(&red, &HumanApproval::granted("maintainer")),
        Err(ChangeRejected::TestsNotGreen)
    );
    // Tests green but human declines → refused.
    assert_eq!(
        request.review(&green, &HumanApproval::declined("maintainer")),
        Err(ChangeRejected::HumanDeclined)
    );
    // Green AND approved → merged, with provenance back to the request and the gate.
    let accepted = request
        .review(&green, &HumanApproval::granted("maintainer"))
        .expect("a green, approved change merges");
    assert_eq!(accepted.change_id, request.id);
    assert_eq!(accepted.target_module, request.target_module);
    assert_eq!(accepted.requirement, request.derived_requirement);
    assert_eq!(accepted.test, request.proposed_test);
    assert_eq!(accepted.reviewer, "maintainer");
    assert_eq!(accepted.benchmark_passed, 4);
}

#[test]
fn request_renders_valid_links_notation_with_a_stable_content_id() {
    let request = canonical_change_request();
    let notation = request.links_notation();
    parse_indented(&notation).expect("the change request should render valid Links Notation");
    assert!(notation.contains("change_request"));
    assert!(notation.contains("human_gated \"true\""));
    assert!(notation.contains("target_module \"src/agentic_coding/planner.rs\""));
    assert!(notation.contains("target_content_id"));
    assert!(notation.contains("derived_requirement"));
    assert!(notation.contains("proposed_test"));
    assert!(notation.contains("reviewable_pull_request"));
    assert!(notation.contains("  step "));

    // Deterministic content id; a differently targeted request has a different one.
    assert_eq!(
        request.content_id(),
        canonical_change_request().content_id()
    );
    let other = ChangeRequest::for_module(request.request.as_str(), "src/change_request.rs");
    assert_ne!(other.content_id(), request.content_id());
}

#[test]
fn recognises_the_change_task_without_colliding_with_the_sibling_recipes() {
    assert!(is_change_request_task(CHANGE_TASK));
    assert!(is_change_request_task("Please change Formal AI itself."));
    assert!(is_change_request_task(
        "I want to add a new feature to the AI system."
    ));
    // Unrelated and sibling requests do not route here.
    assert!(!is_change_request_task("what files are in this folder?"));
    assert!(!is_change_request_task("change this line to uppercase"));
    assert!(!is_change_request_task(
        formal_ai::agentic_coding::self_heal::SELF_HEAL_TASK
    ));
    assert!(!is_change_request_task(
        formal_ai::agentic_coding::source_links::SOURCE_LINKS_TASK
    ));
    assert!(!is_change_request_task(
        formal_ai::agentic_coding::LEDGER_TASK
    ));
    assert!(!is_change_request_task(
        formal_ai::agentic_coding::EXPLAIN_TASK
    ));
    // The change task itself must not trip the sibling routers.
    assert!(!formal_ai::agentic_coding::is_self_heal_task(CHANGE_TASK));
    assert!(!formal_ai::agentic_coding::is_self_ast_task(CHANGE_TASK));
    assert!(!formal_ai::agentic_coding::is_source_links_task(
        CHANGE_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_ledger_task(CHANGE_TASK));
    assert!(!formal_ai::agentic_coding::is_explain_task(CHANGE_TASK));
}

#[test]
fn planner_walks_the_change_request_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(CHANGE_TASK)];

    // Step 1: write the generated reviewable pull-request document (no web step — it is
    // a pure function of the request and its grounded target).
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(CHANGE_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    answer_tool_call(&mut messages, &call, "wrote requested-change.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(CHANGE_PATH));
    answer_tool_call(&mut messages, &call, &recipe::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the reviewable PR.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(CHANGE_PATH));
            assert!(answer.contains("reviewable pull request"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn driver_drives_the_change_request_recipe_to_a_write() {
    // End-to-end through the in-repo Agent CLI driver: the loop finishes and writes
    // exactly the generated reviewable pull-request document.
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(CHANGE_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    assert_eq!(written["path"], CHANGE_PATH);
    assert!(outcome.final_answer.contains(CHANGE_PATH));
}
