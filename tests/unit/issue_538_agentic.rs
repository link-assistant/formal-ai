//! The issue-#538 agentic recipe — Formal AI driving its *own* agentic CLI to
//! make one of its meanings more detailed, reproducing the exact seed data change.
//!
//! Issue #468 proved the loop can *formalize a text*. Issue #538 asks the harder,
//! self-referential question the maintainer set as the project's direction: can
//! the **same** loop edit Formal AI's own seed knowledge to make a meaning more
//! detailed? These pins lock that: the deterministic planner walks
//! search → fetch → write → verify → final for the tomato meaning, and the block
//! the loop writes is **byte-for-byte** the enriched seed block in
//! `data/seed/meanings-translation.lino`. So the agentic CLI reproduces the exact
//! change the issue asked for — not a hand-authored approximation.

use formal_ai::agentic_coding::{
    corpus, diagram, is_meaning_detail_task, meaning_detail, plan_chat_step, run_agentic_task,
    self_ast, AgenticPlan, PlannedToolCall, DRIVER_TOOLS,
};
use formal_ai::{ChatMessage, ToolCall};

/// Derive the enriched tomato block from the given fetched text (or the embedded
/// real Wikidata cache when `None`) — the recipe under test, pinned to the tomato
/// concept for the tomato-specific assertions.
fn enrich_tomato_block(fetched: Option<&str>) -> String {
    meaning_detail::enrich_block(&meaning_detail::TOMATO, fetched)
}

/// Extract the enriched tomato meaning block from the seed file — everything from
/// the `  tomato` lemma head up to (but not including) the next lemma `  cucumber`.
/// This is the ground truth the agentic loop must reproduce.
fn seed_tomato_block() -> String {
    let seed = include_str!("../../data/seed/meanings-translation.lino");
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
    let seed = include_str!("../../data/seed/meanings-translation.lino");
    let start = seed.find("\n  potato\n").expect("seed has a potato lemma") + 1;
    let rest = &seed[start..];
    let end = rest.find("\n  carrot\n").expect("carrot follows potato") + 1;
    rest[..end].to_owned()
}

#[test]
fn recognises_the_meaning_detail_task() {
    // The canonical task string and its keywords route to the #538 recipe…
    assert!(is_meaning_detail_task(meaning_detail::MEANING_DETAIL_TASK));
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
    let produced = enrich_tomato_block(None);
    assert_eq!(produced, seed_tomato_block());
}

#[test]
fn enriched_block_is_grounded_and_adds_the_missing_plural() {
    let block = enrich_tomato_block(None);
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
fn fetch_and_canonical_fallback_yield_the_same_block() {
    // Whether the fetch "succeeds" (returns the corpus body) or errors (falls back
    // to the canonical facts), the loop derives an identical block — the
    // determinism invariant the planner relies on.
    let from_fetch = enrich_tomato_block(Some(
        corpus::web_fetch(meaning_detail::TOMATO.source_url).as_str(),
    ));
    let from_fallback = enrich_tomato_block(Some("web_fetch error: 404 not found"));
    let from_none = enrich_tomato_block(None);
    assert_eq!(from_fetch, from_none);
    assert_eq!(from_fallback, from_none);
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
fn planner_walks_the_full_recipe_for_the_meaning_detail_task() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(meaning_detail::MEANING_DETAIL_TASK)];

    // Step 1: search for the Wikidata lexeme data.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_search");
    assert!(call.arguments.contains(meaning_detail::TOMATO.search_query));
    answer_tool_call(
        &mut messages,
        &call,
        &corpus::web_search(meaning_detail::TOMATO.search_query),
    );

    // Step 2: fetch the tomato lexemes.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_fetch");
    assert!(call.arguments.contains(meaning_detail::TOMATO.source_url));
    answer_tool_call(
        &mut messages,
        &call,
        &corpus::web_fetch(meaning_detail::TOMATO.source_url),
    );

    // Step 3: write the enriched meaning block — byte-for-byte the seed block.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(meaning_detail::TOMATO.kb_path));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], seed_tomato_block());
    answer_tool_call(&mut messages, &call, "wrote meanings-tomato-detail.lino");

    // Step 4: verify by reading the file back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(meaning_detail::TOMATO.kb_path));
    answer_tool_call(&mut messages, &call, &seed_tomato_block());

    // Step 5: the recipe is exhausted — the final answer carries the block inline.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("more detailed"));
            assert!(answer.contains("томаты"));
            assert!(answer.contains(meaning_detail::TOMATO.kb_path));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn corpus_serves_the_tomato_lexemes_for_search_and_fetch() {
    // A tomato-lexeme query surfaces the Wikidata page and its url…
    let results = corpus::web_search("wikidata lexeme tomato помидор томат grammatical number");
    assert!(results.contains(meaning_detail::TOMATO.source_url));
    // …and fetching that url returns the canonical lexeme facts (with the plural).
    let body = corpus::web_fetch(meaning_detail::TOMATO.source_url);
    // The body is the real Wikidata lexeme JSON (the linguistic core): the plural
    // form `томаты` (L170542-F7, feature Q146786) is present to be derived from.
    assert!(body.contains("\"L170542\""));
    assert!(body.contains("L170542-F7"));
    assert!(body.contains("томаты"));
    assert!(body.contains("Q146786"));
    // It really is JSON we can parse back, not a bespoke compact fixture.
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("web_fetch body is JSON");
    assert!(parsed["entities"]["L170542"].is_object());
}

#[test]
fn driver_runs_the_meaning_detail_loop_and_writes_the_seed_block() {
    // End-to-end: the in-repo agentic CLI drives the server through the whole loop
    // and the file it writes into the sandbox is byte-for-byte the enriched seed
    // block — Formal AI making its own meaning more detailed, offline and
    // deterministically.
    let outcome = run_agentic_task(meaning_detail::MEANING_DETAIL_TASK).expect("sandbox workspace");

    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");

    // The recipe executes exactly the four tools, in canonical order.
    let executed: Vec<&str> = outcome.steps.iter().map(|s| s.tool.as_str()).collect();
    assert_eq!(executed, DRIVER_TOOLS.to_vec());

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
    assert!(outcome
        .final_answer
        .contains(meaning_detail::TOMATO.kb_path));
}

#[test]
fn driver_is_deterministic_for_the_meaning_detail_task() {
    let first = run_agentic_task(meaning_detail::MEANING_DETAIL_TASK).expect("workspace");
    let second = run_agentic_task(meaning_detail::MEANING_DETAIL_TASK).expect("workspace");
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
    let tomato = meaning_detail::concept_for_task(meaning_detail::MEANING_DETAIL_TASK)
        .expect("tomato request routes");
    let potato = meaning_detail::concept_for_task(meaning_detail::POTATO_DETAIL_TASK)
        .expect("potato request routes");
    assert_eq!(tomato.name, "tomato");
    assert_eq!(potato.name, "potato");
    assert_ne!(tomato.kb_path, potato.kb_path);
    // Both requests are recognised as the #538 meaning-detail task…
    assert!(is_meaning_detail_task(meaning_detail::POTATO_DETAIL_TASK));
    // …and the potato request uses *none* of the tomato task's wording.
    assert!(!meaning_detail::POTATO_DETAIL_TASK.contains("tomato"));
    assert!(!meaning_detail::POTATO_DETAIL_TASK.contains("томат"));
}

#[test]
fn enriched_potato_block_matches_the_seed_byte_for_byte() {
    // The same recipe, a different concept: the block re-derived for potato from
    // its Wikidata lexeme facts is exactly the enriched seed potato block. This is
    // the missing English plural `potatoes` (form L3784-F2) recovered, every
    // surface pinned to part of speech and grammatical number, byte-for-byte.
    let produced = meaning_detail::enrich_block(&meaning_detail::POTATO, None);
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
    let outcome = run_agentic_task(meaning_detail::POTATO_DETAIL_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");

    let executed: Vec<&str> = outcome.steps.iter().map(|s| s.tool.as_str()).collect();
    assert_eq!(executed, DRIVER_TOOLS.to_vec());

    let write = &outcome.steps[2];
    assert_eq!(write.tool, "write_file");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], seed_potato_block());
    // It writes to the potato KB path, not the tomato one.
    assert_eq!(written["path"], meaning_detail::POTATO.kb_path);

    // The verify step reads the enriched block back and observes the added plural.
    let run = &outcome.steps[3];
    assert!(run.result.contains("potatoes"));
    assert!(run.result.contains("grammatical_number plural"));

    assert!(outcome.final_answer.contains("more detailed"));
    assert!(outcome
        .final_answer
        .contains(meaning_detail::POTATO.kb_path));
}

#[test]
fn committed_potato_session_matches_a_fresh_run() {
    // The second committed Agent CLI session (a *different* natural-language
    // request solving a *different* concept with the same recipe). Regenerate with:
    //   formal-ai agent --task "<POTATO_DETAIL_TASK>" \
    //       --session-json docs/case-studies/issue-538/agent-cli-session-potato.json
    let committed = include_str!("../../docs/case-studies/issue-538/agent-cli-session-potato.json");
    let fresh = run_agentic_task(meaning_detail::POTATO_DETAIL_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed potato Agent CLI session is stale — regenerate it with \
         `formal-ai agent --session-json …`"
    );
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
    // The diagram request uses none of the tomato/potato task wording.
    assert!(!diagram::DIAGRAM_TASK.contains("tomato"));
    assert!(!diagram::DIAGRAM_TASK.contains("grammatical"));
}

#[test]
fn committed_diagram_is_generated_and_written_by_the_driver() {
    // The heart of this axis: the committed diagram document is *generated* from
    // the planner's own recipe table (not hand-drawn), so it can never drift from
    // the code — and it is byte-for-byte what the Agent CLI writes.
    let committed = include_str!("../../docs/diagrams/agentic-recipes.md");
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
fn committed_diagram_session_matches_a_fresh_run() {
    // The third committed Agent CLI session (a different request, a different axis,
    // the same recipe machinery). Regenerate with:
    //   formal-ai agent --task "<DIAGRAM_TASK>" \
    //       --session-json docs/case-studies/issue-538/agent-cli-session-diagram.json
    let committed =
        include_str!("../../docs/case-studies/issue-538/agent-cli-session-diagram.json");
    let fresh = run_agentic_task(diagram::DIAGRAM_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed diagram Agent CLI session is stale — regenerate it with \
         `formal-ai agent --session-json …`"
    );
}

#[test]
fn committed_agent_cli_session_matches_a_fresh_run() {
    // The committed artifact (docs/case-studies/issue-538/agent-cli-session.json)
    // is the reproducible record the maintainer asked for: "the json file with the
    // Agent CLI session that fully solved this exact task." Because the loop is
    // deterministic, a fresh run must reproduce it byte-for-byte — so the artifact
    // can never silently drift from what the code actually does. Regenerate with:
    //   formal-ai agent --task "<MEANING_DETAIL_TASK>" \
    //       --session-json docs/case-studies/issue-538/agent-cli-session.json
    let committed = include_str!("../../docs/case-studies/issue-538/agent-cli-session.json");
    let fresh = run_agentic_task(meaning_detail::MEANING_DETAIL_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed Agent CLI session is stale — regenerate it with `formal-ai agent \
         --session-json …`"
    );
}

// --- Fourth axis (issue #538): store the CST/AST of the meta algorithm itself ---

#[test]
fn recognises_the_self_ast_task() {
    // The self-AST recipe is reached from a *differently worded* request, and the
    // router recognises the intent from the words (AST/CST + self-reference), not a
    // hardcoded string.
    assert!(self_ast::is_self_ast_task(self_ast::AST_TASK));
    assert!(self_ast::is_self_ast_task(
        "record the abstract-syntax of our planner in our data"
    ));
    // It does not steal the other recipes' requests or unrelated turns.
    assert!(!self_ast::is_self_ast_task(
        "make the tomato meaning more detailed"
    ));
    assert!(!self_ast::is_self_ast_task(
        "generate the mermaid diagrams of our recipes"
    ));
    assert!(!self_ast::is_self_ast_task("What is the capital of France?"));
    // The task uses none of the other axes' distinctive wording.
    assert!(!self_ast::AST_TASK.contains("tomato"));
    assert!(!self_ast::AST_TASK.contains("mermaid"));
}

#[test]
fn self_ast_task_wins_over_the_formalization_keyword() {
    // Regression pin: the self-AST task legitimately says "in Links Notation" (its
    // output format), which the broad formalization router also matches. The more
    // specific self-AST router must win, so the loop parses the planner rather than
    // formalizing the fisherman tale.
    assert!(self_ast::AST_TASK.to_lowercase().contains("links notation"));
    let outcome = run_agentic_task(self_ast::AST_TASK).expect("workspace");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["path"], self_ast::AST_PATH);
    assert!(outcome.final_answer.contains("CST/AST"));
    assert!(!outcome.final_answer.contains("fisherman"));
}

#[test]
fn committed_self_ast_is_generated_and_written_by_the_driver() {
    // The committed CST/AST-in-data artifact is *generated* by parsing a real module
    // of our own meta algorithm through the meta-language links network — never hand
    // written — so it can never drift from the code, and it is byte-for-byte what the
    // Agent CLI writes.
    let committed = include_str!("../../data/meta/self-ast.lino");
    assert_eq!(
        committed,
        self_ast::render_document(),
        "the committed self-AST census is stale — regenerate it from the planner source"
    );
    // It really is a real abstract-syntax census: named nodes and node kinds.
    assert!(committed.contains("named_node_count "));
    assert!(committed.contains("distinct_node_kinds "));
    assert!(committed.contains("text_preserved true"));
    assert!(committed.contains("clean true"));
    assert!(committed.contains("function_item "));

    // End-to-end: the in-repo Agent CLI writes exactly this document.
    let outcome = run_agentic_task(self_ast::AST_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], self_ast::render_document());
    assert_eq!(written["path"], self_ast::AST_PATH);
    assert!(outcome.final_answer.contains(self_ast::AST_PATH));
}

#[test]
fn planner_walks_the_self_ast_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(self_ast::AST_TASK)];

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
            assert!(answer.contains("named_node_count "));
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
fn self_ast_node_kinds_are_sorted_for_determinism() {
    let census = self_ast::render_document();
    // Extract the node-kind lines (four-space indented) and check they are sorted.
    let kinds: Vec<&str> = census
        .lines()
        .filter_map(|line| line.strip_prefix("    "))
        .map(|line| line.split_whitespace().next().unwrap_or(""))
        .collect();
    let mut sorted = kinds.clone();
    sorted.sort_unstable();
    assert_eq!(kinds, sorted, "node kinds must be emitted in sorted order");
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
fn committed_self_ast_session_matches_a_fresh_run() {
    // The fourth committed Agent CLI session (a different request, a self-inspection
    // axis, the same recipe machinery). Regenerate with:
    //   formal-ai agent --task "<AST_TASK>" \
    //       --session-json docs/case-studies/issue-538/agent-cli-session-self-ast.json
    let committed =
        include_str!("../../docs/case-studies/issue-538/agent-cli-session-self-ast.json");
    let fresh = run_agentic_task(self_ast::AST_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed self-AST Agent CLI session is stale — regenerate it with \
         `formal-ai agent --session-json …`"
    );
}
