//! Issue #1138: the binary-tree Agent CLI ladder collapsed from 15/32 to 0/32
//! through two server-side behaviors this file pins.
//!
//! * The client's conversation-compaction request (a summarization prompt with
//!   no tools advertised) was answered with a capability-gap refusal -- the
//!   table read "which files we're working on" as a workspace enumeration and
//!   reported a missing `shell` tool to a client that had advertised fourteen.
//!   The client stored that refusal as its compaction summary, the compacted
//!   conversation lost the leaf task, and every later turn ran blind: the proof
//!   and effect files were never written (`missing_proof`, 14 leaves).
//! * The recall turn ("What did we do so far?") was stolen whole-sentence by
//!   the web router and answered with a web search.
//! * The evidence-record route delivered the residual's *failure* prose --
//!   verification-failure renderings and solver refusals -- as the content of
//!   the effect file it was asked to write, so the file's `result=` line
//!   carried a status template instead of the requested marker
//!   (`unverified_leaf_result`, 17 leaves).
//! * The client pings "Continue if you have next steps" after every tool
//!   result, and each ping opened a fresh evidence window: the write state
//!   machine lost sight of its own read and write, restarted the pair on every
//!   round, and the leaf never reached its verification, its effect file, or a
//!   completion. A continuation cue resumes the standing run (issue #1095) --
//!   it must not erase what the run has already done.

use formal_ai::protocol::{ChatMessage, MessageContent, ToolCall};
use formal_ai::{
    ChatCompletion, ChatCompletionRequest, SolverConfig, UniversalSolver,
    create_chat_completion_with_solver,
};

const AGENT_CLI_TOOLS: [&str; 14] = [
    "bash",
    "batch",
    "codesearch",
    "edit",
    "glob",
    "grep",
    "list",
    "read",
    "task",
    "todoread",
    "todowrite",
    "webfetch",
    "websearch",
    "write",
];

fn tool_declarations() -> Vec<serde_json::Value> {
    AGENT_CLI_TOOLS
        .iter()
        .map(|name| {
            serde_json::json!({
                "type": "function",
                "function": {"name": name, "description": format!("{name} tool"), "parameters": {"type": "object", "properties": {}}}
            })
        })
        .collect()
}

fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

fn agent_step(messages: Vec<ChatMessage>) -> ChatCompletion {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "agent-cli",
        "messages": messages,
        "tools": tool_declarations(),
    }))
    .expect("valid chat completion request");
    create_chat_completion_with_solver(&request, &agent_solver())
}

fn no_tool_step(messages: Vec<ChatMessage>) -> ChatCompletion {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "agent-cli",
        "messages": messages,
    }))
    .expect("valid chat completion request");
    create_chat_completion_with_solver(&request, &UniversalSolver::default())
}

fn final_text(completion: &ChatCompletion) -> String {
    match &completion.choices[0].message.content {
        MessageContent::Text(text) => text.clone(),
        MessageContent::Parts(_) => String::new(),
    }
}

fn planned_call(completion: &ChatCompletion) -> Option<(String, String)> {
    completion.choices.first().and_then(|choice| {
        choice
            .message
            .tool_calls
            .first()
            .map(|call| (call.function.name.clone(), call.function.arguments.clone()))
    })
}

fn read_call(id: &str, path: &str) -> ChatMessage {
    ChatMessage::assistant_tool_calls(vec![ToolCall {
        id: id.to_owned(),
        kind: "function".to_owned(),
        function: formal_ai::protocol::FunctionCall {
            name: "read".to_owned(),
            arguments: format!("{{\"filePath\":\"{path}\"}}"),
        },
    }])
}

fn write_call(id: &str, path: &str, content: &str) -> ChatMessage {
    ChatMessage::assistant_tool_calls(vec![ToolCall {
        id: id.to_owned(),
        kind: "function".to_owned(),
        function: formal_ai::protocol::FunctionCall {
            name: "write".to_owned(),
            arguments: serde_json::json!({"filePath": path, "content": content}).to_string(),
        },
    }])
}

fn tool_reply(id: &str, name: &str, result: &str) -> ChatMessage {
    ChatMessage::tool_result(id.to_owned(), name.to_owned(), result)
}

const ADD_TO_LIST_LEAF: &str = "Atomic task L01: Edit the tracked file `rust/src/web_search_core.rs`: add \"wikiquote\" to the WEB_SEARCH_PROVIDERS list. Change only that file and keep it valid Rust.

This is recursive binary-tree node 1.1.1.1.1 at depth 5. Solve only this node's task in this fresh temporary repository. Its harness-evaluated completion criterion is: the tracked file carries the requested marker. Apply the change to the tracked file itself -- the file has to end up modified in the Git worktree, and nothing else may change. Then create `agent-ladder-effects/node-1.1.1.1.1.lino` with these exact field lines: `node_path=1.1.1.1.1`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by at least four words that state the change you made and that contain the exact text \"wikiquote\". Leave supporting evidence in .agent-ladder/node-1.1.1.1.1-proof.md. The first line must be exactly node_path=1.1.1.1.1 and the body must state the concrete result. The harness rejects proof without the separate Git effect. Use web research when it materially improves factual accuracy. Do not claim success without evidence.";

const COMPACTION_SYSTEM: &str = "You are a helpful AI assistant tasked with summarizing conversations.

When asked to summarize, provide a detailed but concise summary of the conversation. Focus on information that would be helpful for continuing the conversation, including what was done, what is currently being worked on, which files are being modified, and what needs to be done next.";

const COMPACTION_ASK: &str = "Provide a detailed but concise summary of our conversation above. Focus on information that would be helpful for continuing the conversation, including what we did and what we're doing, which files we're working on, and what we're going to do next.";

/// The client's compaction request: system summarization preamble, the leaf
/// conversation, and the summary ask -- with no tools advertised. The answer
/// must be a summary that carries the task, never a capability-gap refusal
/// naming tools the surface never offered.
#[test]
fn compaction_summary_request_gets_a_summary_not_a_gap_refusal() {
    let completion = no_tool_step(vec![
        ChatMessage::new("system", COMPACTION_SYSTEM),
        ChatMessage::user(ADD_TO_LIST_LEAF),
        ChatMessage::assistant("Let me open rust/src/web_search_core.rs and read what it says."),
        tool_reply(
            "r1",
            "read",
            "pub const WEB_SEARCH_PROVIDERS: [&str; 3] = [\"duckduckgo\", \"brave\", \"startpage\"];",
        ),
        ChatMessage::assistant("Let me update rust/src/web_search_core.rs for you."),
        tool_reply("w1", "write", ""),
        ChatMessage::user("What did we do so far?"),
        ChatMessage::user(COMPACTION_ASK),
    ]);

    let answer = final_text(&completion);
    let lowered = answer.to_lowercase();
    assert!(
        lowered.contains("wikiquote") || lowered.contains("web_search_providers"),
        "the compaction summary must carry the task, got: {answer}"
    );
    assert!(
        !lowered.contains("list_dir") && !lowered.contains("does not expose"),
        "a summarization request must not be answered with a capability-gap refusal: {answer}"
    );
}

/// The recall turn over the full toolset: a question about what the
/// conversation did is answered from the conversation, not by searching the web
/// for the question itself.
#[test]
fn recall_turn_over_the_full_toolset_is_answered_from_history() {
    let completion = agent_step(vec![
        ChatMessage::user(ADD_TO_LIST_LEAF),
        read_call("r1", "rust/src/web_search_core.rs"),
        tool_reply(
            "r1",
            "read",
            "pub const WEB_SEARCH_PROVIDERS: [&str; 3] = [\"duckduckgo\", \"brave\", \"startpage\"];",
        ),
        ChatMessage::user("What did we do so far?"),
    ]);

    if let Some((name, arguments)) = planned_call(&completion) {
        assert_ne!(
            name, "websearch",
            "a recall question is not a web search: {arguments}"
        );
        assert_ne!(name, "webfetch", "a recall question is not a fetch");
    } else {
        let answer = final_text(&completion).to_lowercase();
        assert!(
            answer.contains("wikiquote") || answer.contains("web_search_providers"),
            "the recall answer must carry the conversation's task, got: {answer}"
        );
    }
}

/// The replace leaf must reach the edit tool after its read, and no turn may
/// deliver a write whose content is failure prose -- the effect file's
/// `result=` line is the leaf's verified content, not a status template.
#[test]
fn replace_leaf_edits_and_never_writes_failure_prose() {
    let source = "pub const HEADER: &str = \"request_history\";\npub fn memory() -> String { String::new() }\n";
    let replace_leaf = "Atomic task L12: In the file rust/src/protocol_memory.rs, replace \"request_history\" with \"conversation_history\". Change only that file and keep it valid Rust.\n\nThis is recursive binary-tree node 1.2.1.1.1 at depth 5. Solve only this node's task. Then create `agent-ladder-effects/node-1.2.1.1.1.lino` with these exact field lines: `node_path=1.2.1.1.1`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by at least four words that state the change you made and that contain the exact text \"conversation_history\". Leave supporting evidence in .agent-ladder/node-1.2.1.1.1-proof.md.";

    let mut messages = vec![ChatMessage::user(replace_leaf.to_owned())];
    let mut saw_edit = false;
    for _ in 0..4 {
        let completion = agent_step(messages.clone());
        let Some((name, arguments)) = planned_call(&completion) else {
            break;
        };
        assert!(
            !arguments.contains("Verification failed")
                && !arguments.contains("could not be recorded"),
            "failure prose must not be delivered as file content: {name} {arguments}"
        );
        if name == "edit" {
            saw_edit = true;
            assert!(
                arguments.contains("request_history") && arguments.contains("conversation_history"),
                "the edit must carry the old and new text: {arguments}"
            );
        }
        let id = format!("c{}", messages.len());
        let result = if name == "read" {
            source.to_owned()
        } else if name == "edit" {
            "The file rust/src/protocol_memory.rs was edited.".to_owned()
        } else {
            "done".to_owned()
        };
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall {
            id: id.clone(),
            kind: "function".to_owned(),
            function: formal_ai::protocol::FunctionCall {
                name: name.clone(),
                arguments: arguments.clone(),
            },
        }]));
        messages.push(tool_reply(&id, &name, &result));
    }
    assert!(saw_edit, "the replace leaf must reach the edit tool");
}

/// After a client that reports empty write results completes the tracked-file
/// change, the continue turn must not claim nothing is in progress: the effect
/// and proof deliveries are still outstanding.
#[test]
fn continue_turn_after_an_empty_result_write_resumes_the_delivery() {
    let updated = "pub const WEB_SEARCH_PROVIDERS: [&str; 4] = [\"duckduckgo\", \"brave\", \"startpage\", \"wikiquote\"];\n";
    let completion = agent_step(vec![
        ChatMessage::user(ADD_TO_LIST_LEAF),
        read_call("r1", "rust/src/web_search_core.rs"),
        tool_reply(
            "r1",
            "read",
            "pub const WEB_SEARCH_PROVIDERS: [&str; 3] = [\"duckduckgo\", \"brave\", \"startpage\"];",
        ),
        write_call("w1", "rust/src/web_search_core.rs", updated),
        tool_reply("w1", "write", ""),
        ChatMessage::user("Continue if you have next steps"),
    ]);

    if let Some((name, arguments)) = planned_call(&completion) {
        assert!(
            !arguments.contains("Verification failed")
                && !arguments.contains("could not be recorded"),
            "failure prose must not be delivered as file content: {name} {arguments}"
        );
    } else {
        let answer = final_text(&completion);
        assert_ne!(
            answer, "Nothing is in progress to continue. Tell me the task and I will start it.",
            "the leaf's deliveries are outstanding; the continue turn must not claim nothing is in progress"
        );
    }
}

/// Drive the L01 leaf the way the harness does: run the planned call, answer a
/// write with the client's empty acknowledgement, and ping "Continue if you
/// have next steps" after every round. The tracked-file edit is only the first
/// deliverable -- the run must advance to its verification command or the
/// effect-file delivery rather than restarting the read--write pair on every
/// ping, which is what held all 32 leaves at zero.
#[test]
fn continuation_pings_do_not_restart_the_leaf_write_state_machine() {
    let source =
        "pub const WEB_SEARCH_PROVIDERS: [&str; 3] = [\"duckduckgo\", \"brave\", \"startpage\"];\n";
    let updated = "pub const WEB_SEARCH_PROVIDERS: [&str; 4] = [\"duckduckgo\", \"brave\", \"startpage\", \"wikiquote\"];\n";
    let mut messages = vec![ChatMessage::user(ADD_TO_LIST_LEAF.to_owned())];
    let mut steps: Vec<(String, String)> = Vec::new();
    let mut advanced = false;
    for _ in 0..10 {
        let completion = agent_step(messages.clone());
        if let Some((name, arguments)) = planned_call(&completion) {
            let repeats = steps
                .iter()
                .filter(|prior| prior.0 == name && prior.1 == arguments)
                .count();
            assert!(
                repeats < 2,
                "a continuation ping restarted the {name} step for the third time: {arguments}"
            );
            if arguments.contains("agent-ladder-effects")
                || (name == "bash" && arguments.contains("rust/src/web_search_core.rs"))
            {
                advanced = true;
            }
            steps.push((name.clone(), arguments.clone()));
            let id = format!("c{}", messages.len());
            let result = if name == "read" {
                source.to_owned()
            } else if name == "bash" && arguments.contains("cat") {
                updated.to_owned()
            } else {
                String::new()
            };
            messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall {
                id: id.clone(),
                kind: "function".to_owned(),
                function: formal_ai::protocol::FunctionCall {
                    name: name.clone(),
                    arguments: arguments.clone(),
                },
            }]));
            messages.push(tool_reply(&id, &name, &result));
        } else {
            messages.push(ChatMessage::assistant(final_text(&completion)));
        }
        messages.push(ChatMessage::user("Continue if you have next steps"));
    }
    assert!(
        advanced,
        "the leaf must advance past its tracked-file edit to the verification command or the effect-file delivery; planned steps: {steps:?}"
    );
}

/// The same ping pattern against a literal-file creation: the general plan
/// writes its plan record, writes the requested file, and verifies it with the
/// request-derived command. Every ping resetting the evidence window restarted
/// the plan-record write instead, and the requested file was never verified or
/// reported complete.
#[test]
fn continuation_pings_do_not_restart_a_literal_file_plan() {
    let task = "Create the file notes/welcome.txt containing the single line hello wikiquote.";
    let mut messages = vec![ChatMessage::user(task.to_owned())];
    let mut steps: Vec<(String, String)> = Vec::new();
    let mut completed = false;
    for _ in 0..10 {
        let completion = agent_step(messages.clone());
        if let Some((name, arguments)) = planned_call(&completion) {
            let repeats = steps
                .iter()
                .filter(|prior| prior.0 == name && prior.1 == arguments)
                .count();
            assert!(
                repeats < 2,
                "a continuation ping restarted the {name} step for the third time: {arguments}"
            );
            steps.push((name.clone(), arguments.clone()));
            let id = format!("c{}", messages.len());
            let result = if name == "bash" && arguments.contains("cat") {
                "hello wikiquote\n".to_owned()
            } else {
                String::new()
            };
            messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall {
                id: id.clone(),
                kind: "function".to_owned(),
                function: formal_ai::protocol::FunctionCall {
                    name: name.clone(),
                    arguments: arguments.clone(),
                },
            }]));
            messages.push(tool_reply(&id, &name, &result));
        } else {
            let answer = final_text(&completion);
            let lowered = answer.to_lowercase();
            if lowered.contains("welcome.txt") || lowered.contains("wikiquote") {
                completed = true;
            }
            messages.push(ChatMessage::assistant(answer));
        }
        messages.push(ChatMessage::user("Continue if you have next steps"));
    }
    assert!(
        completed,
        "the literal-file plan must reach a completion that reports the requested file; planned steps: {steps:?}"
    );
}
