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
