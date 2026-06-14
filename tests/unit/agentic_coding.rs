//! Behavioural pins for the Links Notation formalizer (issue #468).
//!
//! These lock the meta-language formalization: text in, Links Notation out, with
//! all nine protocol primitives realised as links — and the honest, grounded
//! behaviour on text the closed lexicon does not recognise.

use formal_ai::agentic_coding::{
    coverage_line, formalize_text_to_links, plan_chat_step, AgenticPlan, PlannedToolCall,
    CANONICAL_FISHERMAN_SYNOPSIS, CANONICAL_SOURCE_URL, FISHERMAN_DOC_ID, KB_PATH, PRIMITIVE_KINDS,
    SEARCH_QUERY,
};
use formal_ai::{
    create_chat_completion_with_solver, ChatCompletionRequest, ChatMessage, SolverConfig, ToolCall,
    UniversalSolver,
};

#[test]
fn canonical_synopsis_covers_all_nine_primitives() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let summary = &formalized.summary;

    assert!(
        summary.covers_all_nine(),
        "expected all nine primitives, got: {}",
        coverage_line(summary)
    );
    assert_eq!(summary.covered.len(), PRIMITIVE_KINDS.len());
    // The covered list is reported in canonical primitive order.
    assert_eq!(summary.covered, PRIMITIVE_KINDS.to_vec());

    // Pinned counts for the co-designed synopsis + lexicon.
    assert_eq!(summary.doc_id, FISHERMAN_DOC_ID);
    assert_eq!(summary.concepts, 3, "greed (lexicon) + ransom + wish");
    assert_eq!(
        summary.entities, 4,
        "old_man, old_woman, golden_fish, trough"
    );
    assert_eq!(summary.predicates, 6);
    assert_eq!(summary.assertions, 7);
    assert_eq!(summary.procedures, 1);
    assert_eq!(summary.contexts, 2);
    assert_eq!(summary.temporals, 3);
    assert_eq!(summary.modals, 3);
    assert_eq!(summary.annotations, 7);
    assert_eq!(summary.total_records(), 37);
}

#[test]
fn every_output_record_is_links_notation_not_a_rust_struct() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;

    // The header and each primitive kind appears as a Links Notation record head.
    assert!(document.starts_with("knowledge_base\n  id \"tale:fisherman-and-fish\""));
    for kind in PRIMITIVE_KINDS {
        assert!(
            document.contains(&format!("{kind}\n  id ")),
            "missing `{kind}` record in document"
        );
    }
    // Indentation is two spaces (the meta-language convention), never tabs.
    assert!(!document.contains('\t'));
    assert!(document.contains("\n  id \"a:0\""));
}

#[test]
fn grounded_svo_extraction_is_faithful() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;

    // "Старик поймал золотую рыбку." — subject, predicate, object all grounded,
    // with the predicate's temporal and the scene's context attached as links.
    assert!(document.contains("subject \"ent:old_man\""));
    assert!(document.contains("predicate \"pred:catch\""));
    assert!(document.contains("object \"ent:golden_fish\""));
    assert!(document.contains("time \"temporal:в-начале-сказки\""));
    assert!(document.contains("context \"ctx:seaside\""));

    // Modality is carried as a link to a modal record.
    assert!(document.contains("modal \"modal:commitment\""));
    assert!(document.contains("modal:commitment"));

    // Provenance ties each assertion back to a character span of the source.
    assert!(document.contains("provenance \"tale:fisherman-and-fish@0:28\""));
}

#[test]
fn unmatched_object_falls_back_to_an_honest_literal() {
    // "Старуха потребовала стать владычицей морской." — the demand's object is
    // not in the closed lexicon, so it is recorded as a literal rather than an
    // invented entity. The recogniser never hallucinates a relation it cannot
    // ground.
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;
    assert!(document.contains("object \"стать владычицей морской\""));
    assert!(document.contains("object_kind \"literal\""));
}

#[test]
fn annotations_use_real_character_offsets() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;
    // First sentence spans characters 0..28 ("Старик поймал золотую рыбку.").
    assert!(document.contains("span \"0:28\""));
    assert!(document.contains("text \"Старик поймал золотую рыбку.\""));
}

#[test]
fn formalization_is_deterministic() {
    // The fetched-text == fallback-text invariant the planner relies on: the same
    // input always yields byte-identical Links Notation.
    let first = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let second = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    assert_eq!(first.links_notation, second.links_notation);
    assert_eq!(first.summary, second.summary);
}

#[test]
fn arbitrary_text_still_produces_a_valid_knowledge_base() {
    // Open-domain text the lexicon does not recognise: every sentence still
    // becomes an annotation plus a natural-language assertion. No work matched,
    // so there are no lexicon-sourced concepts/procedures/contexts — and we do
    // not pretend otherwise.
    let formalized = formalize_text_to_links("A cat sat on a mat. Then it slept.", "doc:demo");
    let summary = &formalized.summary;

    assert_eq!(summary.doc_id, "doc:demo");
    assert_eq!(summary.annotations, 2);
    assert_eq!(summary.assertions, 2);
    assert_eq!(summary.procedures, 0);
    assert_eq!(summary.contexts, 0);
    assert!(!summary.covers_all_nine());
    assert!(formalized
        .links_notation
        .contains("predicate \"pred:states\""));
    assert!(formalized
        .links_notation
        .contains("natural_language \"A cat sat on a mat.\""));
    // Language detection falls back to English for non-Cyrillic input.
    assert!(formalized.links_notation.contains("language \"en\""));
}

#[test]
fn explicit_doc_id_overrides_the_default() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "kb:custom");
    assert_eq!(formalized.summary.doc_id, "kb:custom");
    assert!(formalized
        .links_notation
        .starts_with("knowledge_base\n  id \"kb:custom\""));
}

// --- Deterministic agentic planner (the server's "brain") -------------------

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

/// Append the assistant `tool_calls` turn the planner produced plus the tool's
/// `result`, mirroring what an agentic CLI feeds back on the next request.
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
fn planner_ignores_non_formalization_tasks() {
    // A request unrelated to issue #468 yields no plan, so the server falls
    // through to its ordinary symbolic solver and agentic coding stays opt-in.
    let messages = vec![ChatMessage::user("What is the capital of France?")];
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    assert_eq!(plan_chat_step(&messages, &tools), None);
}

#[test]
fn planner_walks_the_full_search_fetch_write_run_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(
        "Please formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.",
    )];

    // Step 1: search for the source text.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_search");
    assert!(call.arguments.contains(SEARCH_QUERY));
    answer_tool_call(&mut messages, &call, "1. ru.wikisource.org — full text");

    // Step 2: fetch the canonical source.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_fetch");
    assert!(call.arguments.contains(CANONICAL_SOURCE_URL));
    answer_tool_call(&mut messages, &call, CANONICAL_FISHERMAN_SYNOPSIS);

    // Step 3: write the formalized knowledge base.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(KB_PATH));
    assert!(call.arguments.contains("knowledge_base"));
    answer_tool_call(&mut messages, &call, "wrote knowledge-base.lino");

    // Step 4: verify by reading the file back.
    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(KB_PATH));
    answer_tool_call(
        &mut messages,
        &call,
        "knowledge_base\n  id \"tale:fisherman-and-fish\"",
    );

    // Step 5: the recipe is exhausted — the final answer carries the KB inline.
    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("nine protocol primitives"));
            assert!(answer.contains("knowledge_base"));
            assert!(answer.contains(KB_PATH));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn planner_skips_capabilities_no_advertised_tool_provides() {
    // A CLI that only exposes a write tool: the planner skips search/fetch/run
    // and writes the canonical-synopsis knowledge base immediately, then ends.
    let tools = ["write_file"];
    let mut messages = vec![ChatMessage::user("formalize the fisherman tale")];

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    answer_tool_call(&mut messages, &call, "ok");

    assert!(matches!(
        plan_chat_step(&messages, &tools),
        Some(AgenticPlan::Final(_))
    ));
}

#[test]
fn planner_completes_directly_when_no_tools_are_advertised() {
    // No tools at all: the planner cannot act, so it answers directly with the
    // canonical knowledge base rather than stalling.
    let messages = vec![ChatMessage::user("formalize the fisherman tale")];
    match plan_chat_step(&messages, &[]) {
        Some(AgenticPlan::Final(answer)) => assert!(answer.contains("knowledge_base")),
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn planner_formalizes_the_fetched_text_when_fetch_succeeds() {
    // A successful fetch returning real text is used as the formalization source.
    let tools = ["web_fetch", "write_file"];
    let mut messages = vec![ChatMessage::user("formalize the fisherman tale")];

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_fetch");
    answer_tool_call(&mut messages, &call, "Старик поймал золотую рыбку.");

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    let expected = formalize_text_to_links("Старик поймал золотую рыбку.", "").links_notation;
    assert_eq!(written["content"], expected);
}

#[test]
fn planner_falls_back_to_the_synopsis_when_fetch_errors() {
    // The fetch tool returns an error string. The planner does not trust it as
    // source text; the written knowledge base is the canonical-synopsis one, so
    // the loop still completes with a stable, all-nine-primitive document.
    let tools = ["web_fetch", "write_file"];
    let mut messages = vec![ChatMessage::user("formalize the fisherman tale")];

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_fetch");
    answer_tool_call(&mut messages, &call, "Error: 404 Not Found");

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    let expected = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "").links_notation;
    assert_eq!(written["content"], expected);
}

#[test]
fn planner_classifies_results_by_tool_call_id_when_name_is_absent() {
    // Some CLIs omit the tool result's `name`; the planner then maps the
    // `tool_call_id` back to the originating assistant `tool_calls` turn.
    let tools = ["web_search", "write_file"];
    let mut messages = vec![ChatMessage::user("formalize the fisherman tale")];

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "web_search");

    let id = "call_named_none";
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id,
        call.tool,
        call.arguments,
    )]));
    let mut result = ChatMessage::new("tool", "ru.wikisource.org");
    result.tool_call_id = Some(String::from(id)); // name deliberately left None
    messages.push(result);

    // Search is now recognised as done, so the planner advances to write.
    let next = expect_single_call(&messages, &tools);
    assert_eq!(next.tool, "write_file");
}

#[test]
fn server_emits_tool_calls_for_a_formalization_task_in_agent_mode() {
    // End-to-end through the OpenAI-compatible entry point: in agent mode, with a
    // permitted tool advertised, a formalization task makes the server emit a
    // `tool_calls` assistant turn rather than plain text. `web_search` is granted
    // by the default associative package, so the permission gate lets it through.
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [{
            "role": "user",
            "content": "Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base."
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web",
                "parameters": {"type": "object"}
            }
        }]
    }))
    .unwrap();

    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let completion = create_chat_completion_with_solver(&request, &solver);
    let choice = &completion.choices[0];
    assert_eq!(choice.finish_reason, "tool_calls");
    assert_eq!(choice.message.tool_calls.len(), 1);
    let call = &choice.message.tool_calls[0];
    assert_eq!(call.function.name, "web_search");
    assert!(call.function.arguments.contains(SEARCH_QUERY));
    // The assistant turn requesting tool calls carries no textual content.
    assert!(choice.message.content.plain_text().is_empty());
}

#[test]
fn server_returns_final_knowledge_base_once_the_recipe_is_exhausted() {
    // After the only advertised tool (web_search) has produced a result, the
    // planner has nothing left to call, so the server completes with the
    // knowledge base inline and finish_reason "stop".
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [
            {"role": "user", "content": "Formalize the fisherman tale into links notation."},
            {"role": "assistant", "tool_calls": [{
                "id": "call_1",
                "type": "function",
                "function": {"name": "web_search", "arguments": "{\"query\":\"...\"}"}
            }]},
            {"role": "tool", "tool_call_id": "call_1", "name": "web_search",
             "content": "ru.wikisource.org"}
        ],
        "tools": [{
            "type": "function",
            "function": {"name": "web_search", "parameters": {"type": "object"}}
        }]
    }))
    .unwrap();

    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let completion = create_chat_completion_with_solver(&request, &solver);
    let choice = &completion.choices[0];
    assert_eq!(choice.finish_reason, "stop");
    assert!(choice.message.tool_calls.is_empty());
    let body = choice.message.content.plain_text();
    assert!(body.contains("knowledge_base"));
    assert!(body.contains("nine protocol primitives"));
}
