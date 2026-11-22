//! The meaning-detail recipe derives its output from explicit Wikidata captures.

use formal_ai::agentic_coding::{
    AgenticPlan, CORE_RECIPE_TOOLS, PlannedToolCall, corpus, diagram, is_meaning_detail_task,
    meaning_detail, plan_chat_step, run_agentic_task, run_agentic_task_with_source_fetch,
    run_agentic_task_with_tools, run_agentic_task_with_tools_and_source_fetch, self_ast,
};
use formal_ai::{ChatMessage, ToolCall};

const TOMATO_TASK: &str = "Make the tomato meaning more detailed: pin every surface's \
    part of speech and grammatical number, ground it in Wikidata, and add the missing plural to томат.";
const POTATO_TASK: &str = "Please make the potato word and meaning richer — record the \
    singular/plural of each surface, add the missing plural form potatoes, and keep it grounded in Wikidata.";
const APPLE_TASK: &str = "Find the grammatical forms for apple.";
const TOMATO_CAPTURE: &str = include_str!("../../fixtures/publisher-agentic/tomato.json");
const POTATO_CAPTURE: &str = include_str!("../../fixtures/publisher-agentic/potato.json");
const APPLE_CAPTURE: &str = include_str!("../../fixtures/publisher-agentic/apple.json");

fn tomato() -> &'static meaning_detail::Concept {
    meaning_detail::concept_for_task(TOMATO_TASK).expect("tomato concept")
}

fn potato() -> &'static meaning_detail::Concept {
    meaning_detail::concept_for_task(POTATO_TASK).expect("potato concept")
}

fn publisher_capture(url: &str) -> Result<String, String> {
    for (task, capture) in [
        (TOMATO_TASK, TOMATO_CAPTURE),
        (POTATO_TASK, POTATO_CAPTURE),
        (APPLE_TASK, APPLE_CAPTURE),
    ] {
        if meaning_detail::concept_for_task(task).is_some_and(|concept| concept.source_url == url) {
            return Ok(capture.to_owned());
        }
    }
    Err(format!("no fixture for {url}"))
}

/// Extract the enriched tomato meaning block from the seed file — everything from
/// the `  tomato` lemma head up to (but not including) the next lemma `  cucumber`.
/// This is the ground truth the agentic loop must reproduce.
fn seed_tomato_block() -> String {
    let seed = include_str!("../../../../data/seed/meanings-translation.lino");
    let start = seed.find("\n  tomato\n").expect("seed has a tomato lemma") + 1; // skip the leading newline so the block starts at `  tomato`
    let rest = &seed[start..];
    let end = rest
        .find("\n  cucumber\n")
        .expect("cucumber follows tomato")
        + 1;
    rest[..end].to_owned()
}

/// Extract the enriched potato meaning block from the seed file — everything from
/// the `  potato` lemma head up to (but not including) the next lemma `  carrot`.
fn seed_potato_block() -> String {
    let seed = include_str!("../../../../data/seed/meanings-translation.lino");
    let start = seed.find("\n  potato\n").expect("seed has a potato lemma") + 1;
    let rest = &seed[start..];
    let end = rest.find("\n  carrot\n").expect("carrot follows potato") + 1;
    rest[..end].to_owned()
}

include!("../../fixtures/native-protocol-observation.rs");
#[test]
fn recognises_the_meaning_detail_task() {
    // The canonical task string and its keywords route to the #538 recipe…
    assert!(is_meaning_detail_task(TOMATO_TASK));
    assert!(is_meaning_detail_task(
        "make the помидор meaning more detailed with grammatical number"
    ));
    // …while an unrelated request does not.
    assert!(!is_meaning_detail_task("What is the capital of France?"));
}

#[test]
fn enriched_block_matches_the_seed_byte_for_byte() {
    // The heart of the issue: the block re-derived from the fetched Wikidata lexeme
    // facts is exactly the enriched seed block. If someone hand-edits the seed, this
    // fails until the recipe is re-derived — the loop and the seed stay in lockstep.
    let produced = meaning_detail::enrich_block(tomato(), TOMATO_CAPTURE).unwrap();
    assert_eq!(produced, seed_tomato_block());
}

#[test]
fn enriched_block_is_grounded_and_adds_the_missing_plural() {
    let block = meaning_detail::enrich_block(tomato(), TOMATO_CAPTURE).unwrap();
    // Every surface pins part of speech (6 grounded + 3 extra: hi, zh×2) and every
    // grounded surface pins grammatical number.
    assert_eq!(block.matches("part_of_speech noun").count(), 9);
    assert_eq!(block.matches("grammatical_number singular").count(), 3);
    assert_eq!(block.matches("grammatical_number plural").count(), 3);
    // The previously missing Russian plural `томаты` (form L170542-F7) is present…
    assert!(block.contains("surface L170542-F7"));
    assert!(block.contains("text томаты"));
    // …so `томат` now matches its synonym `помидор`, both grounded in Wikidata.
    assert!(block.contains("grounded-in Q23501"));
    assert!(block.contains("source-lexeme L170542"));
    assert!(block.contains("feature Q146786 # wikidata grammatical feature plural"));
}

#[test]
fn missing_partial_and_wrong_publisher_evidence_leave_no_enrichment() {
    assert_eq!(meaning_detail::enrich_block(tomato(), ""), None);
    assert_eq!(meaning_detail::enrich_block(tomato(), POTATO_CAPTURE), None);
    let mut capture: serde_json::Value = serde_json::from_str(TOMATO_CAPTURE).unwrap();
    capture["entities"]["L170542"]["forms"] = serde_json::json!([]);
    assert_eq!(
        meaning_detail::enrich_block(tomato(), &capture.to_string()),
        None
    );
    let mut capture: serde_json::Value = serde_json::from_str(TOMATO_CAPTURE).unwrap();
    capture["entities"]["L7993"]["senses"] = serde_json::json!([]);
    assert_eq!(
        meaning_detail::enrich_block(tomato(), &capture.to_string()),
        None
    );
    let unavailable =
        run_agentic_task_with_source_fetch(TOMATO_TASK, |_| Err("offline".to_owned())).unwrap();
    assert_eq!(
        unavailable
            .steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>(),
        ["web_search", "web_fetch"]
    );
    assert_eq!(
        unavailable.final_answer,
        format!(
            "I could not verify the required grammatical forms at {}.",
            tomato().source_url
        )
    );
}

#[test]
fn enriches_another_registered_noun_and_binds_the_publisher_sense() {
    let apple = meaning_detail::concept_for_task(APPLE_TASK).expect("apple concept");
    let block = meaning_detail::enrich_block(apple, APPLE_CAPTURE).expect("apple forms");
    assert!(block.contains("surface L3257-F2"));
    assert!(block.contains("text apples"));
    let tomato_block =
        meaning_detail::enrich_block(tomato(), TOMATO_CAPTURE).expect("tomato forms");
    assert!(tomato_block.contains("sense L7993-S2"));
    assert!(!tomato_block.contains("sense L7993-S1"));
    assert!(!tomato_block.contains("sense L170542-S1"));
}

/// Plan one step and assert it is a single tool call, returning it.
fn expect_single_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(mut calls)) => {
            assert_eq!(calls.len(), 1, "the planner emits one call per step");
            calls.remove(0)
        }
        other => panic!("expected a single tool call, got {other:?}"),
    }
}

/// Append the assistant `tool_calls` turn plus the tool's `result`.
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
fn unregistered_word_uses_the_requested_subject_for_research() {
    let messages = [ChatMessage::user(
        "Find the grammatical number for the word garden.",
    )];
    let call = expect_single_call(&messages, &["web_search", "web_fetch", "write_file"]);
    assert_eq!(call.tool, "web_search");
    assert!(call.arguments.contains("garden"), "{}", call.arguments);
    assert!(!call.arguments.contains("tomato"), "{}", call.arguments);
}

#[test]
fn planner_walks_the_full_recipe_for_the_meaning_detail_task() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(TOMATO_TASK)];

    // Step 1: search for the Wikidata lexeme data.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_search");
    assert!(call.arguments.contains(&tomato().search_query));
    answer_tool_call(
        &mut messages,
        &call,
        &corpus::web_search(&tomato().search_query),
    );

    // Step 2: fetch the tomato lexemes.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_fetch");
    assert!(call.arguments.contains(&tomato().source_url));
    answer_tool_call(&mut messages, &call, TOMATO_CAPTURE);

    // Step 3: write the enriched meaning block — byte-for-byte the seed block.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(&tomato().kb_path));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], seed_tomato_block());
    answer_tool_call(&mut messages, &call, "wrote meanings-tomato-detail.lino");

    // Step 4: verify by reading the file back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(&tomato().kb_path));
    answer_tool_call(&mut messages, &call, &seed_tomato_block());

    // Step 5: the recipe is exhausted — the final answer carries the block inline.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("more detailed"));
            assert!(answer.contains("томаты"));
            assert!(answer.contains(&tomato().kb_path));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn corpus_search_lists_the_publisher_without_a_synthetic_response() {
    let results = corpus::web_search(&tomato().search_query);
    assert!(results.contains(&tomato().source_url));
    assert!(!results.contains("L170542-F7"));
}

#[test]
fn driver_runs_the_meaning_detail_loop_and_writes_the_seed_block() {
    // End-to-end: the in-repo agentic CLI drives the server through the whole loop
    // and the file it writes into the sandbox is byte-for-byte the enriched seed
    // block — Formal AI making its own meaning more detailed, offline and
    // deterministically.
    let outcome = run_agentic_task_with_source_fetch(TOMATO_TASK, publisher_capture)
        .expect("sandbox workspace");

    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");

    // The recipe executes exactly the four core tools, in canonical order —
    // the advertised surface additionally carries the translator, which this
    // recipe never calls.
    let executed: Vec<&str> = outcome.steps.iter().map(|s| s.tool.as_str()).collect();
    assert_eq!(executed, CORE_RECIPE_TOOLS.to_vec());

    // Step 3 writes the enriched block; its JSON content is the seed block.
    let write = &outcome.steps[2];
    assert_eq!(write.tool, "write_file");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], seed_tomato_block());

    // Step 4 reads the file back and observes the grounded plural — proof the write
    // landed in the workspace.
    let run = &outcome.steps[3];
    assert_eq!(run.tool, "run_command");
    assert!(run.result.contains("томаты"));
    assert!(run.result.contains("grammatical_number plural"));

    // The final answer summarises the change with the block inline.
    assert!(outcome.final_answer.contains("more detailed"));
    assert!(outcome.final_answer.contains(&tomato().kb_path));
}

#[test]
fn driver_is_deterministic_for_the_meaning_detail_task() {
    let first =
        run_agentic_task_with_source_fetch(TOMATO_TASK, publisher_capture).expect("workspace");
    let second =
        run_agentic_task_with_source_fetch(TOMATO_TASK, publisher_capture).expect("workspace");
    assert_eq!(first.steps, second.steps);
    assert_eq!(first.final_answer, second.final_answer);
    assert_eq!(first.turns, second.turns);
}

#[test]
fn routes_different_requests_to_different_concepts() {
    // The maintainer's generality check: *"each time you should use different
    // natural language requests, so we test that solutions are never hardcoded but
    // truly general."* Two differently worded requests route to two different
    // concepts — the recipe is parameterized, not tomato-hardcoded.
    let tomato = meaning_detail::concept_for_task(TOMATO_TASK).expect("tomato request routes");
    let potato = meaning_detail::concept_for_task(POTATO_TASK).expect("potato request routes");
    assert_eq!(tomato.name, "tomato");
    assert_eq!(potato.name, "potato");
    assert_ne!(tomato.kb_path, potato.kb_path);
    // Both requests are recognised as the #538 meaning-detail task…
    assert!(is_meaning_detail_task(POTATO_TASK));
    // …and the potato request uses *none* of the tomato task's wording.
    assert!(!POTATO_TASK.contains("tomato"));
    assert!(!POTATO_TASK.contains("томат"));
}

#[test]
fn enriched_potato_block_matches_the_seed_byte_for_byte() {
    // The same recipe, a different concept: the block re-derived for potato from
    // its Wikidata lexeme facts is exactly the enriched seed potato block. This is
    // the missing English plural `potatoes` (form L3784-F2) recovered, every
    // surface pinned to part of speech and grammatical number, byte-for-byte.
    let produced = meaning_detail::enrich_block(potato(), POTATO_CAPTURE).unwrap();
    assert_eq!(produced, seed_potato_block());
    // Proof it is a real enrichment: the plural surface and its number are present.
    assert!(produced.contains("surface L3784-F2 # wikidata english plural surface"));
    assert!(produced.contains("text potatoes"));
    assert!(produced.contains("grammatical_number plural"));
    assert!(produced.contains("grounded-in Q10998"));
}

#[test]
fn driver_runs_the_potato_recipe_with_a_different_request() {
    // End-to-end proof of generality: the *same* in-repo agentic CLI, driven by a
    // *differently worded* request, enriches the potato meaning and writes
    // byte-for-byte the enriched seed potato block into the sandbox.
    let outcome =
        run_agentic_task_with_source_fetch(POTATO_TASK, publisher_capture).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");

    let executed: Vec<&str> = outcome.steps.iter().map(|s| s.tool.as_str()).collect();
    assert_eq!(executed, CORE_RECIPE_TOOLS.to_vec());

    let write = &outcome.steps[2];
    assert_eq!(write.tool, "write_file");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], seed_potato_block());
    // It writes to the potato KB path, not the tomato one.
    assert_eq!(written["path"], potato().kb_path);

    // The verify step reads the enriched block back and observes the added plural.
    let run = &outcome.steps[3];
    assert!(run.result.contains("potatoes"));
    assert!(run.result.contains("grammatical_number plural"));

    assert!(outcome.final_answer.contains("more detailed"));
    assert!(outcome.final_answer.contains(&potato().kb_path));
}

#[test]
fn potato_request_accepts_the_tools_in_the_archived_session() {
    let recorded: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../docs/case-studies/issue-538/agent-cli-session-potato.json"
    ))
    .expect("immutable session JSON");
    let tools: Vec<_> = recorded["tools_advertised"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool.as_str().unwrap())
        .collect();
    assert_eq!(recorded["task"], POTATO_TASK);
    let fresh =
        run_agentic_task_with_tools_and_source_fetch(POTATO_TASK, &tools, publisher_capture)
            .unwrap();
    assert_eq!(
        fresh
            .steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>(),
        CORE_RECIPE_TOOLS
    );
    assert!(fresh.steps[1].arguments.contains(&potato().source_url));
    let write: serde_json::Value = serde_json::from_str(&fresh.steps[2].arguments).unwrap();
    assert_eq!(write["content"], seed_potato_block());
}

#[test]
fn recognises_the_diagram_task() {
    // A *third* recipe on a *non-lexeme* axis (issue #538's "generated mermaid
    // diagram split into parts"), driven by a differently worded request.
    assert!(diagram::is_diagram_task(diagram::DIAGRAM_TASK));
    assert!(diagram::is_diagram_task(
        "please draw a mermaid flowchart of the recipes"
    ));
    // It does not steal the meaning-detail request or unrelated turns.
    assert!(!diagram::is_diagram_task(
        "make the tomato meaning more detailed"
    ));
    assert!(!diagram::is_diagram_task("What is the capital of France?"));
    // A cue inside a quoted literal is payload, not the request.
    assert!(!diagram::is_diagram_task(
        "Insert the line 'js/mermaid.bundle.js' after the line 'js/app.js' in .gitignore."
    ));
    // The diagram request uses none of the tomato/potato task wording.
    assert!(!diagram::DIAGRAM_TASK.contains("tomato"));
    assert!(!diagram::DIAGRAM_TASK.contains("grammatical"));
}

#[test]
fn committed_diagram_is_generated_and_written_by_the_driver() {
    // The heart of this axis: the committed diagram document is *generated* from
    // the planner's own recipe table (not hand-drawn), so it can never drift from
    // the code — and it is byte-for-byte what the Agent CLI writes.
    let committed = include_str!("../../../../docs/diagrams/agentic-recipes.md");
    assert_eq!(
        committed,
        diagram::render_document(),
        "the committed diagram is stale — regenerate it from the recipe table"
    );
    // It really is split into parts, each a mermaid flowchart: one overview plus
    // one per recipe in the planner's recipe table.
    assert_eq!(committed.matches("```mermaid").count(), 5);
    assert!(committed.contains("## Part 1 — Overview"));
    assert!(committed.contains("## Part 5 — Recipe: Store the CST/AST of the meta algorithm"));

    // End-to-end: the in-repo Agent CLI writes exactly this document.
    let outcome = run_agentic_task(diagram::DIAGRAM_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], diagram::render_document());
    assert_eq!(written["path"], diagram::DIAGRAM_PATH);
    assert!(outcome.final_answer.contains(diagram::DIAGRAM_PATH));
}

#[test]
fn planner_walks_the_diagram_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(diagram::DIAGRAM_TASK)];

    // Step 1: no web step — the diagrams are a pure function of the recipe table,
    // so the planner goes straight to writing the generated document.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(diagram::DIAGRAM_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], diagram::render_document());
    answer_tool_call(&mut messages, &call, "wrote agentic-recipes.md");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(diagram::DIAGRAM_PATH));
    answer_tool_call(&mut messages, &call, &diagram::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the diagrams.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("```mermaid"));
            assert!(answer.contains("## Part 1 — Overview"));
            assert!(answer.contains(diagram::DIAGRAM_PATH));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn historical_diagram_session_preserves_the_recipe_tool_sequence() {
    // The third committed Agent CLI session (a different request, a different axis,
    // the same recipe machinery). Regenerate with:
    //   formal-ai agent --task "<DIAGRAM_TASK>" \
    //       --session-json docs/case-studies/issue-538/agent-cli-session-diagram.json
    let committed = include_str!(
        "../../../../docs/case-studies/pull-request-1188/native-protocol-captures/8abb066db/diagram.json"
    );
    assert_protocol_transition(
        include_str!("../../../../docs/case-studies/issue-538/agent-cli-session-diagram.json"),
        committed,
    );
    let recorded: serde_json::Value =
        serde_json::from_str(committed).expect("immutable session JSON");
    let tools = recorded["tools_advertised"]
        .as_array()
        .expect("archived advertised tool schema")
        .iter()
        .map(|tool| tool.as_str().expect("advertised tool name"))
        .collect::<Vec<_>>();
    let fresh = run_agentic_task_with_tools(diagram::DIAGRAM_TASK, &tools).expect("workspace");
    let fresh = fresh.session_json();
    assert_eq!(recorded["task"], fresh["task"]);
    for session in [&recorded, &fresh] {
        let tools = session["steps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|step| step["tool"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(tools, ["write_file", "run_command"]);
    }
    assert!(
        fresh["final_answer"]
            .as_str()
            .unwrap()
            .contains("report a source gap")
    );
}

#[test]
fn historical_tomato_request_uses_an_explicit_publisher_capture() {
    let committed = include_str!("../../../../docs/case-studies/issue-538/agent-cli-session.json");
    let recorded: serde_json::Value = serde_json::from_str(committed).expect("immutable session");
    let tools = recorded["tools_advertised"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(recorded["task"], TOMATO_TASK);
    let fresh =
        run_agentic_task_with_tools_and_source_fetch(TOMATO_TASK, &tools, publisher_capture)
            .expect("workspace");
    let executed: Vec<_> = fresh.steps.iter().map(|step| step.tool.as_str()).collect();
    assert_eq!(
        executed,
        ["web_search", "web_fetch", "write_file", "run_command"]
    );
    assert!(
        fresh.final_answer.contains("Wikidata"),
        "{}",
        fresh.final_answer
    );
    let written: serde_json::Value = serde_json::from_str(&fresh.steps[2].arguments).unwrap();
    assert_eq!(written["content"], seed_tomato_block());
}

// --- Fourth axis (issue #538): store the CST/AST of the meta algorithm itself ---

#[test]
fn recognises_the_self_ast_task() {
    // The self-AST recipe is reached from a *differently worded* request, and the
    // router recognises the intent from the words (AST/CST + self-reference), not a
    // hardcoded string.
    assert!(self_ast::is_self_ast_task(self_ast::ast_task()));
    assert!(self_ast::is_self_ast_task(
        "record the abstract-syntax of our planner in our data"
    ));
    assert!(self_ast::is_self_ast_task(
        "Store our planner CST with every comment and space."
    ));
    assert!(self_ast::is_self_ast_task(
        "Save the concrete syntax of our own Rust source."
    ));
    assert!(!self_ast::is_self_ast_task("What is a CST?"));
    assert!(!self_ast::is_self_ast_task(
        "Describe source CST for a remote component."
    ));
    assert!(self_ast::is_self_ast_task(
        "Store your CST with every comment."
    ));
    assert!(!self_ast::is_self_ast_task(
        "Describe a podcast in our notes."
    ));
    // It does not steal the other recipes' requests or unrelated turns.
    assert!(!self_ast::is_self_ast_task(
        "make the tomato meaning more detailed"
    ));
    assert!(!self_ast::is_self_ast_task(
        "generate the mermaid diagrams of our recipes"
    ));
    assert!(!self_ast::is_self_ast_task(
        "What is the capital of France?"
    ));
    // The task uses none of the other axes' distinctive wording.
    assert!(!self_ast::ast_task().contains("tomato"));
    assert!(!self_ast::ast_task().contains("mermaid"));
}

#[test]
fn self_ast_task_wins_over_the_formalization_keyword() {
    // Regression pin: the self-AST task legitimately says "in Links Notation" (its
    // output format), which the broad formalization router also matches. The more
    // specific self-AST router must win, so the loop parses the planner rather than
    // formalizing the fisherman tale.
    assert!(
        self_ast::ast_task()
            .to_lowercase()
            .contains("links notation")
    );
    let outcome = run_agentic_task(self_ast::ast_task()).expect("workspace");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["path"], self_ast::AST_PATH);
    assert!(outcome.final_answer.contains("complete source CST"));
    let document = formal_ai::readable_source::read_source_links(
        written["content"]
            .as_str()
            .expect("complete source CST content"),
    )
    .expect("readable recursive source model");
    assert_eq!(
        formal_ai::readable_source::source_owner(&document).unwrap(),
        Some(self_ast::TARGET_MODULE_PATH)
    );
    assert_eq!(
        formal_ai::readable_source::source_from_links(&document).unwrap(),
        self_ast::target_source()
    );
    formal_ai::readable_source::verify_source_links(&document)
        .expect("independent current grammar verification");
    assert!(!outcome.final_answer.contains("fisherman"));
}

#[test]
fn recursive_source_content_is_generated_and_written_by_the_driver() {
    let expected = self_ast::render_document();
    let document =
        formal_ai::readable_source::read_source_links(&expected).expect("recursive source model");
    formal_ai::readable_source::verify_source_links(&document)
        .expect("independent grammar reparse");
    assert_eq!(
        formal_ai::readable_source::source_from_links(&document).unwrap(),
        self_ast::target_source()
    );
    assert_eq!(
        formal_ai::readable_source::source_owner(&document).unwrap(),
        Some(self_ast::TARGET_MODULE_PATH)
    );
    let outcome = run_agentic_task(self_ast::ast_task()).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], expected);
    assert_eq!(written["path"], self_ast::AST_PATH);
    assert!(outcome.final_answer.contains(self_ast::AST_PATH));
}

#[test]
fn planner_walks_the_self_ast_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(self_ast::ast_task())];

    // Step 1: no web step — the census is a pure function of the source, so the
    // planner goes straight to writing the generated document.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(self_ast::AST_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], self_ast::render_document());
    answer_tool_call(&mut messages, &call, "wrote self-ast.lino");

    // Step 2: verify by reading the document back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(self_ast::AST_PATH));
    answer_tool_call(&mut messages, &call, &self_ast::render_document());

    // Step 3: the recipe is exhausted — the final answer carries the census.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("owned-source"));
            assert!(answer.contains(self_ast::AST_PATH));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn self_ast_census_is_real_and_deterministic() {
    let src = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
    let a = self_ast::ast_census(src);
    let b = self_ast::ast_census(src);
    assert_eq!(
        a, b,
        "the census must be a deterministic function of the source"
    );
    assert!(
        a.text_preserved,
        "the lossless network must reconstruct the source"
    );
    assert!(a.clean, "a well-formed function must parse without errors");
    assert!(a.named_node_count > 0, "expected named AST nodes");
    assert!(
        a.node_kinds.iter().any(|n| n.kind == "function_item"),
        "expected a function_item node, got: {:?}",
        a.node_kinds
    );
}

#[test]
fn self_ast_census_generalises_to_different_sources() {
    // A struct-only source yields a different census than a function-only one,
    // proving the census reflects the actual source, not a hardcoded answer.
    let func = self_ast::ast_census("fn f() {}\n");
    let strukt = self_ast::ast_census("struct S {\n    field: i32,\n}\n");
    assert_ne!(func.node_kinds, strukt.node_kinds);
    assert!(strukt.node_kinds.iter().any(|n| n.kind == "struct_item"));
    assert!(!func.node_kinds.iter().any(|n| n.kind == "struct_item"));
}

#[test]
fn recursive_source_model_is_deterministic_and_source_free() {
    let first = self_ast::render_document();
    assert_eq!(first, self_ast::render_document());
    let document = formal_ai::readable_source::read_source_links(&first).unwrap();
    assert_eq!(
        formal_ai::readable_source::write_source_links(&document).unwrap(),
        first
    );
    assert_eq!(
        formal_ai::readable_source::source_from_links(&document).unwrap(),
        self_ast::target_source()
    );
}

#[test]
fn self_ast_document_has_single_trailing_newline() {
    let document = self_ast::render_document();
    assert!(document.ends_with('\n'));
    assert!(!document.ends_with("\n\n"));
}

#[test]
fn self_ast_target_is_a_real_module_of_the_meta_algorithm() {
    // The pinned target parses cleanly and reconstructs — proving we stored the AST
    // of real, well-formed logic, not a toy snippet.
    let census = self_ast::target_census();
    assert!(census.text_preserved);
    assert!(census.clean);
    assert!(
        census.named_node_count > 100,
        "the planner is a substantial module"
    );
}

#[test]
fn self_inspection_session_retains_actual_source_model_write_and_read() {
    let fresh = run_agentic_task(self_ast::ast_task()).expect("workspace");
    assert!(!fresh.hit_turn_cap);
    let writes: Vec<_> = fresh
        .steps
        .iter()
        .filter(|step| step.tool == "write_file")
        .collect();
    assert_eq!(writes.len(), 1);
    let arguments: serde_json::Value = serde_json::from_str(&writes[0].arguments).unwrap();
    let text = arguments["content"]
        .as_str()
        .expect("actual written content");
    let document = formal_ai::readable_source::read_source_links(text).unwrap();
    assert_eq!(
        formal_ai::readable_source::source_from_links(&document).unwrap(),
        self_ast::target_source()
    );
    formal_ai::readable_source::verify_source_links(&document).unwrap();
    assert!(fresh.steps.iter().any(|step| step.tool == "run_command"));
    assert!(fresh.final_answer.contains(self_ast::AST_PATH));
}

#[test]
fn historical_self_inspection_request_produces_current_recursive_source_content() {
    let captured: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../docs/case-studies/issue-538/agent-cli-session-self-ast.json"
    ))
    .unwrap();
    let fresh = run_agentic_task(captured["task"].as_str().unwrap()).expect("workspace");
    assert!(!fresh.hit_turn_cap);
    let writes: Vec<_> = fresh
        .steps
        .iter()
        .filter(|step| step.tool == "write_file")
        .collect();
    assert_eq!(writes.len(), 1);
    let arguments: serde_json::Value = serde_json::from_str(&writes[0].arguments).unwrap();
    let document =
        formal_ai::readable_source::read_source_links(arguments["content"].as_str().unwrap())
            .unwrap();
    assert_eq!(
        formal_ai::readable_source::source_from_links(&document).unwrap(),
        self_ast::target_source()
    );
    formal_ai::readable_source::verify_source_links(&document).unwrap();
    assert!(fresh.steps.iter().any(|step| step.tool == "run_command"));
    assert!(fresh.final_answer.contains(self_ast::AST_PATH));
}
