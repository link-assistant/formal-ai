//! Behavioural pins for the Links Notation formalizer (issue #468).
//!
//! These lock the meta-language formalization: text in, Links Notation out, with
//! all nine protocol primitives realised as links — and the honest, grounded
//! behaviour on text the closed lexicon does not recognise.

use formal_ai::agentic_coding::{
    corpus, coverage_line, formalize_text_to_links, plan_chat_step, run_agentic_task, AgenticPlan,
    PlannedToolCall, CANONICAL_FISHERMAN_SYNOPSIS, CANONICAL_SOURCE_URL, DRIVER_TOOLS,
    FISHERMAN_DOC_ID, KB_PATH, PRIMITIVE_KINDS, SEARCH_QUERY,
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
fn issue_607_planner_maps_shell_tools_to_ls_command() {
    for tool in ["bash", "shell", "run_command"] {
        let messages = vec![ChatMessage::user(
            "Run the ls command to list files in the current directory.",
        )];

        let call = expect_single_call(&messages, &[tool]);
        assert_eq!(call.tool, tool);
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
        assert_eq!(arguments["command"], "ls");
    }
}

#[test]
fn issue_607_server_emits_tool_calls_for_shell_request_in_agent_mode() {
    for tool in ["bash", "shell", "run_command"] {
        let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
            "model": "formal-ai",
            "messages": [{
                "role": "user",
                "content": "Run the ls command to list files in the current directory."
            }],
            "tools": [{
                "type": "function",
                "function": {
                    "name": tool,
                    "description": "Run a shell command",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {"type": "string"}
                        },
                        "required": ["command"]
                    }
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
        assert_eq!(call.function.name, tool);
        let arguments: serde_json::Value = serde_json::from_str(&call.function.arguments).unwrap();
        assert_eq!(arguments["command"], "ls");
        assert!(choice.message.content.plain_text().is_empty());
    }
}

#[test]
fn issue_676_planner_maps_execute_pwd_to_pwd_command() {
    // The flagship regression from issue #676: `execute pwd` (and every other seed
    // shell token, not just `ls`) must reach the CLI's shell tool as the named command.
    for (prompt, expected) in [
        ("execute pwd", "pwd"),
        ("Execute pwd", "pwd"),
        ("run pwd", "pwd"),
        ("please run pwd for me", "pwd"),
        ("execute `pwd`", "pwd"),
        ("run git status", "git status"),
        ("execute cargo test", "cargo test"),
        ("run whoami", "whoami"),
    ] {
        let messages = vec![ChatMessage::user(prompt)];
        let call = expect_single_call(&messages, &["bash"]);
        assert_eq!(call.tool, "bash", "{prompt}");
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
        assert_eq!(arguments["command"], expected, "{prompt}");
    }
}

#[test]
fn issue_676_planner_maps_natural_language_file_listing_to_ls() {
    // The second reported failure: "give me a list of files in current folder" and its
    // many phrasings must resolve to `ls`.
    for prompt in [
        "give me a list of files in current folder",
        "give me a list of files in the current folder",
        "show me the files in this directory",
        "list all files here",
        "can you list the files in the current directory?",
        "what files are in the current folder?",
    ] {
        let messages = vec![ChatMessage::user(prompt)];
        let call = expect_single_call(&messages, &["bash"]);
        assert_eq!(call.tool, "bash", "{prompt}");
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
        assert_eq!(arguments["command"], "ls", "{prompt}");
    }
}

#[test]
fn issue_676_planner_ignores_shell_tokens_without_run_context() {
    // A bare mention of a shell token in prose (no run verb / terminal phrase, no
    // listing request) must not be mistaken for a command to execute.
    for prompt in [
        "what does pwd mean?",
        "explain how git works",
        "is npm a package manager?",
    ] {
        let messages = vec![ChatMessage::user(prompt)];
        let tools = ["bash", "web_search", "web_fetch", "write_file"];
        // Either the planner declines (None) or it routes elsewhere, but it must never
        // emit a shell tool call for these.
        if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &tools) {
            for call in calls {
                assert_ne!(call.tool, "bash", "{prompt} should not run a shell command");
            }
        }
    }
}

#[test]
fn issue_624_planner_maps_natural_language_directory_listing_to_ls() {
    for prompt in [
        "what files are in this folder?",
        "show me the contents of this directory",
        "can you check which files exist in the current folder?",
        "print a directory listing of the current working directory",
    ] {
        let messages = vec![ChatMessage::user(prompt)];
        let call = expect_single_call(&messages, &["bash"]);

        assert_eq!(call.tool, "bash", "{prompt}");
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
        assert_eq!(arguments["command"], "ls", "{prompt}");
    }
}

#[test]
fn issue_624_server_emits_tool_calls_for_natural_language_directory_listing() {
    for prompt in [
        "what files are in this folder?",
        "show me the contents of this directory",
        "can you check which files exist in the current folder?",
        "print a directory listing of the current working directory",
    ] {
        let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
            "model": "formal-ai",
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "tools": [{
                "type": "function",
                "function": {
                    "name": "bash",
                    "description": "Execute a shell command",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {"type": "string"}
                        },
                        "required": ["command"]
                    }
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
        assert_eq!(choice.finish_reason, "tool_calls", "{prompt}");
        assert_eq!(choice.message.tool_calls.len(), 1, "{prompt}");
        let call = &choice.message.tool_calls[0];
        assert_eq!(call.function.name, "bash", "{prompt}");
        let arguments: serde_json::Value = serde_json::from_str(&call.function.arguments).unwrap();
        assert_eq!(arguments["command"], "ls", "{prompt}");
        assert!(choice.message.content.plain_text().is_empty(), "{prompt}");
    }
}

#[test]
fn issue_607_server_summarizes_shell_tool_result_after_ls_runs() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [
            {
                "role": "user",
                "content": "Run the ls command to list files in the current directory."
            },
            {
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {"name": "bash", "arguments": "{\"command\":\"ls\"}"}
                }]
            },
            {
                "role": "tool",
                "tool_call_id": "call_1",
                "name": "bash",
                "content": "Cargo.toml\nsrc\n"
            }
        ],
        "tools": [{
            "type": "function",
            "function": {
                "name": "bash",
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
    assert_eq!(choice.finish_reason, "stop");
    assert!(choice.message.tool_calls.is_empty());
    let body = choice.message.content.plain_text();
    assert!(body.contains("`ls`"));
    assert!(body.contains("Cargo.toml"));
    assert!(body.contains("src"));
}

#[test]
fn issue_607_driver_executes_ls_inside_the_sandbox_workspace() {
    let outcome = run_agentic_task("Run the ls command to list files here.")
        .expect("the sandbox workspace should be created");

    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    assert_eq!(outcome.steps.len(), 1);
    let step = &outcome.steps[0];
    assert_eq!(step.tool, "run_command");
    let arguments: serde_json::Value = serde_json::from_str(&step.arguments).unwrap();
    assert_eq!(arguments["command"], "ls");
    assert!(outcome.final_answer.contains("`ls`"));
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
        "model": "formal-ai",
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
        "model": "formal-ai",
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

// --- Offline web corpus (what web_search/web_fetch resolve against) ---------

#[test]
fn corpus_search_surfaces_the_canonical_source() {
    // A query mentioning the tale matches the corpus page and lists its url so the
    // agent can fetch it next.
    let results = corpus::web_search("Пушкин Сказка о рыбаке и рыбке полный текст");
    assert!(results.contains(CANONICAL_SOURCE_URL));
    assert!(results.contains("Викитека"));
}

#[test]
fn corpus_search_reports_no_results_for_an_unrelated_query() {
    let results = corpus::web_search("weather forecast tomorrow");
    assert!(results.starts_with("web_search: no results"));
}

#[test]
fn corpus_fetch_returns_the_canonical_synopsis_for_the_known_url() {
    // The fetch body is exactly the formalizer's fallback text, so a successful
    // fetch and the offline fallback produce the same knowledge base.
    assert_eq!(
        corpus::web_fetch(CANONICAL_SOURCE_URL),
        CANONICAL_FISHERMAN_SYNOPSIS
    );
}

#[test]
fn corpus_fetch_reports_a_404_for_an_unknown_url() {
    // An unknown url yields an error string the planner's heuristic recognises,
    // exercising the "understand errors from tools" requirement.
    let body = corpus::web_fetch("https://example.com/missing");
    assert!(body.contains("404"));
    assert!(body.contains("https://example.com/missing"));
}

// --- In-repo agentic driver (the offline "agentic CLI") ---------------------

#[test]
fn driver_runs_the_full_search_fetch_write_run_loop_to_a_final_answer() {
    // The driver plays an external agentic CLI: it advertises the four tools,
    // executes every tool call the server emits against the offline corpus and a
    // sandboxed workspace, feeds results back, and loops until the server returns
    // the finished knowledge base. This is the end-to-end "agentic coding mode".
    let outcome = run_agentic_task(
        "Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base \
         covering all nine protocol primitives.",
    )
    .expect("the sandbox workspace should be created");

    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");

    // The recipe executes exactly the four tools, in canonical order.
    let executed: Vec<&str> = outcome
        .steps
        .iter()
        .map(|step| step.tool.as_str())
        .collect();
    assert_eq!(executed, DRIVER_TOOLS.to_vec());

    // The final answer is the formalizer's report plus the knowledge base inline.
    assert!(outcome.final_answer.contains("nine protocol primitives"));
    assert!(outcome.final_answer.contains("knowledge_base"));
    assert!(outcome.final_answer.contains(KB_PATH));
    // One server round-trip per tool call plus the final answer turn.
    assert_eq!(outcome.turns, outcome.steps.len() + 1);
}

#[test]
fn driver_fetches_the_canonical_source_and_writes_the_knowledge_base() {
    let outcome = run_agentic_task(
        "Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.",
    )
    .expect("the sandbox workspace should be created");

    // Step 2 fetches the canonical url and gets the synopsis back, not a 404.
    let fetch = &outcome.steps[1];
    assert_eq!(fetch.tool, "web_fetch");
    assert!(fetch.arguments.contains(CANONICAL_SOURCE_URL));
    assert!(fetch.result.contains("Старик поймал золотую рыбку."));

    // Step 3 writes the formalized knowledge base to the expected path.
    let write = &outcome.steps[2];
    assert_eq!(write.tool, "write_file");
    assert!(write.arguments.contains(KB_PATH));
    assert!(write.result.starts_with("wrote "));

    // Step 4 reads the file back and sees the knowledge-base header — proof the
    // command ran inside the workspace and observed the written bytes.
    let run = &outcome.steps[3];
    assert_eq!(run.tool, "run_command");
    assert!(run.result.contains("knowledge_base"));
    assert!(run.result.contains("tale:fisherman-and-fish"));
}

#[test]
fn driver_is_deterministic() {
    // No clock, no randomness, no network: the same task yields byte-identical
    // transcripts and final answers run to run.
    let task = "formalize the fisherman tale into links notation";
    let first = run_agentic_task(task).expect("workspace");
    let second = run_agentic_task(task).expect("workspace");
    assert_eq!(first.steps, second.steps);
    assert_eq!(first.final_answer, second.final_answer);
    assert_eq!(first.turns, second.turns);
}
