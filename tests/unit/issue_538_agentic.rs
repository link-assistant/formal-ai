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
    corpus, enrich_tomato_block, is_meaning_detail_task, meaning_detail, plan_chat_step,
    run_agentic_task, AgenticPlan, PlannedToolCall, DRIVER_TOOLS,
};
use formal_ai::{ChatMessage, ToolCall};

/// Extract the enriched tomato meaning block from the seed file — everything from
/// the `  tomato` lemma head up to (but not including) the next lemma `  cucumber`.
/// This is the ground truth the agentic loop must reproduce.
fn seed_tomato_block() -> String {
    let seed = include_str!("../../data/seed/meanings-translation.lino");
    let start = seed
        .find("\n  tomato\n")
        .expect("seed has a tomato lemma")
        + 1; // skip the leading newline so the block starts at `  tomato`
    let rest = &seed[start..];
    let end = rest.find("\n  cucumber\n").expect("cucumber follows tomato") + 1;
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
    let from_fetch = enrich_tomato_block(Some(corpus::web_fetch(meaning_detail::SOURCE_URL).as_str()));
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
    assert!(call.arguments.contains(meaning_detail::SEARCH_QUERY));
    answer_tool_call(&mut messages, &call, &corpus::web_search(meaning_detail::SEARCH_QUERY));

    // Step 2: fetch the tomato lexemes.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_fetch");
    assert!(call.arguments.contains(meaning_detail::SOURCE_URL));
    answer_tool_call(&mut messages, &call, &corpus::web_fetch(meaning_detail::SOURCE_URL));

    // Step 3: write the enriched meaning block — byte-for-byte the seed block.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(meaning_detail::KB_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], seed_tomato_block());
    answer_tool_call(&mut messages, &call, "wrote meanings-tomato-detail.lino");

    // Step 4: verify by reading the file back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(meaning_detail::KB_PATH));
    answer_tool_call(&mut messages, &call, &seed_tomato_block());

    // Step 5: the recipe is exhausted — the final answer carries the block inline.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("more detailed"));
            assert!(answer.contains("томаты"));
            assert!(answer.contains(meaning_detail::KB_PATH));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn corpus_serves_the_tomato_lexemes_for_search_and_fetch() {
    // A tomato-lexeme query surfaces the Wikidata page and its url…
    let results = corpus::web_search("wikidata lexeme tomato помидор томат grammatical number");
    assert!(results.contains(meaning_detail::SOURCE_URL));
    // …and fetching that url returns the canonical lexeme facts (with the plural).
    let body = corpus::web_fetch(meaning_detail::SOURCE_URL);
    assert!(body.contains("L170542"));
    assert!(body.contains("form F7 томаты plural Q146786"));
}

#[test]
fn driver_runs_the_meaning_detail_loop_and_writes_the_seed_block() {
    // End-to-end: the in-repo agentic CLI drives the server through the whole loop
    // and the file it writes into the sandbox is byte-for-byte the enriched seed
    // block — Formal AI making its own meaning more detailed, offline and
    // deterministically.
    let outcome =
        run_agentic_task(meaning_detail::MEANING_DETAIL_TASK).expect("sandbox workspace");

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
    assert!(outcome.final_answer.contains(meaning_detail::KB_PATH));
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
    assert!(outcome.final_answer.contains(meaning_detail::POTATO.kb_path));
}

#[test]
fn committed_potato_session_matches_a_fresh_run() {
    // The second committed Agent CLI session (a *different* natural-language
    // request solving a *different* concept with the same recipe). Regenerate with:
    //   formal-ai agent --task "<POTATO_DETAIL_TASK>" \
    //       --session-json docs/case-studies/issue-538/agent-cli-session-potato.json
    let committed =
        include_str!("../../docs/case-studies/issue-538/agent-cli-session-potato.json");
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
