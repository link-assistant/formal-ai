//! Issue #907: the caller's framing is not the user's request.
//!
//! The gemini CLI prefixes every turn with a `<session_context>` block whose
//! second line reads *"Today's date is Sunday, August 2, 2026 (formatted
//! according to the user's locale)."* Intent routing matched that sentence
//! anywhere in the incoming request, so **every** agent-mode gemini run emitted
//! `run_shell_command({"command":"date"})` and silently dropped the real task.
//!
//! One test per requirement of the issue:
//!
//! 1. intent matching reads the user's request, not client-injected context;
//! 2. a declarative *"X is Y"* statement never fires an intent, in any language;
//! 3. a turn that carries a task gets the task, even when an intent also matches;
//!
//! followed by the whole-task test: the report's exact gemini request, over the
//! real Gemini surface, must write the requested program.
//!
//! Each case uses a *different* phrasing so a passing run proves the routing is
//! general rather than memorised (CONTRIBUTING rule 4).

use formal_ai::gemini::{
    create_gemini_generate_content_response_with_solver_and_memory, GeminiGenerateContentRequest,
};
use formal_ai::protocol::{latest_user_request, ChatMessage};
use formal_ai::seed;
use formal_ai::{SolverConfig, UniversalSolver};

/// The two capabilities the report's gemini session advertises.
const TOOLS: [&str; 2] = ["run_shell_command", "write_file"];

const REQUEST: &str = "Write a hello world program in Python.";

/// The context block the gemini CLI really sends, verbatim from the report.
const GEMINI_SESSION_CONTEXT: &str = "<session_context>\nThis is the Gemini CLI. We are setting up the context for our chat.\nToday's date is Sunday, August 2, 2026 (formatted according to the user's locale).\nMy operating system is: linux\n</session_context>";

fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

/// The declarations the report's gemini session advertises, as the CLI sends them.
fn function_declarations() -> serde_json::Value {
    serde_json::json!([{"functionDeclarations": [
        {
            "name": TOOLS[0],
            "description": "Run a shell command",
            "parameters": {
                "type": "OBJECT",
                "properties": {"command": {"type": "STRING"}},
                "required": ["command"]
            }
        },
        {
            "name": TOOLS[1],
            "description": "Write a file",
            "parameters": {
                "type": "OBJECT",
                "properties": {
                    "file_path": {"type": "STRING"},
                    "content": {"type": "STRING"}
                },
                "required": ["file_path", "content"]
            }
        }
    ]}])
}

/// The function call the Gemini surface emits for a turn made of `parts`.
///
/// Routing is exercised over the surface the report uses, so a passing test is
/// evidence about the reported behaviour rather than about an inner helper.
fn gemini_call(parts: &[&str]) -> serde_json::Value {
    let request: GeminiGenerateContentRequest = serde_json::from_value(serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": parts.iter().map(|text| serde_json::json!({"text": text})).collect::<Vec<_>>(),
        }],
        "tools": function_declarations(),
    }))
    .expect("valid Gemini generateContent request");

    let response = create_gemini_generate_content_response_with_solver_and_memory(
        &request,
        "formal-ai",
        &agent_solver(),
        &[],
    );
    response["candidates"][0]["content"]["parts"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|part| part.get("functionCall"))
        .cloned()
        .unwrap_or_else(|| panic!("expected a function call for {parts:?}, got {response}"))
}

/// Requirement 1: what a client wraps in its own context block is the client
/// talking. Every registered marker is checked with a different client's real
/// framing, so the separation is not one CLI's special case.
#[test]
fn client_injected_context_is_not_the_users_request() {
    for (tag, context) in [
        (
            "session_context",
            "This is the Gemini CLI. We are setting up the context for our chat.\nToday's date is Sunday, August 2, 2026 (formatted according to the user's locale).",
        ),
        (
            "system-reminder",
            "As you answer the user's questions, you can use the following context:\n# currentDate\nToday's date is 2026-08-02.",
        ),
        (
            "environment_context",
            "<cwd>/tmp/workspace</cwd>\n<approval_policy>on-request</approval_policy>",
        ),
        ("env", "Working directory: /tmp/workspace\nToday's date: 2026-08-02"),
    ] {
        let framed = format!("<{tag}>\n{context}\n</{tag}>\n\n{REQUEST}");
        let messages = vec![ChatMessage::user(&framed)];

        assert_eq!(
            latest_user_request(&messages).as_deref(),
            Some(REQUEST),
            "<{tag}> is the client talking, not the user"
        );
        let call = gemini_call(&[&framed]);
        assert_eq!(call["name"], "write_file", "<{tag}> must not decide the turn");
    }
}

/// Requirement 1, seed side: the markers are declared as data, so a maintainer
/// adds the next client by editing `data/seed/caller-context.lino`.
#[test]
fn caller_context_markers_live_in_seed_data() {
    let vocabulary = seed::caller_context_vocabulary();
    for (tag, client) in [
        ("session_context", "gemini"),
        ("system-reminder", "claude"),
        ("system-reminder", "qwen"),
        ("environment_context", "codex"),
        ("env", "opencode"),
    ] {
        let block = vocabulary
            .injected_blocks
            .iter()
            .find(|block| block.tag == tag)
            .unwrap_or_else(|| panic!("missing injected block {tag}"));
        assert_eq!(block.open(), format!("<{tag}>"));
        assert_eq!(block.close(), format!("</{tag}>"));
        assert!(
            block.clients.iter().any(|known| known == client),
            "{tag} should record {client}"
        );
    }
}

/// Requirement 2: stating a fact about the date is not asking for it — in any
/// of the languages the date intent is declared in.
#[test]
fn a_declarative_statement_does_not_fire_an_intent() {
    for statement in [
        "Today's date is Sunday, August 2, 2026 (formatted according to the user's locale).",
        "The current date is 2026-08-02.",
        "The current time is 20:00.",
        "Текущее время — 20:00.",
        "आज की तारीख 2 अगस्त 2026 है।",
        "今天的日期是2026年8月2日。",
    ] {
        let call = gemini_call(&[&format!("{statement}\n\n{REQUEST}")]);
        assert_eq!(
            call["name"], "write_file",
            "a statement must not route: {statement:?} planned {call}"
        );
    }
}

/// Requirement 2, the other direction: really asking still routes to `date`.
/// A guard that silenced the intent altogether would pass the test above.
#[test]
fn asking_for_the_date_still_runs_date() {
    for question in [
        "what is the date?",
        "show me today's date",
        "what day is it",
        "print the current date",
        "какое сегодня число?",
        "आज की तारीख क्या है?",
        "今天的日期是什么？",
    ] {
        let call = gemini_call(&[question]);
        assert_eq!(call["name"], "run_shell_command", "{question}");
        assert_eq!(call["args"]["command"], "date", "{question}");
    }
}

/// Requirement 3: when a turn carries a task, the task wins — an intent cue
/// riding along in the same turn does not replace it.
#[test]
fn a_turn_that_carries_a_task_gets_the_task() {
    for prompt in [
        "Show me the current date. Write a hello world program in Python please.",
        "Today's date is Sunday. Create a file main.py that prints Hello, world!",
        "The current time is 20:00. Write a Python script that prints Hello, world!",
    ] {
        let call = gemini_call(&[prompt]);
        assert_eq!(call["name"], "write_file", "{prompt} planned {call}");
        assert!(
            call["args"]["content"]
                .as_str()
                .is_some_and(|content| content.contains("Hello, world!")),
            "{prompt} planned {call}"
        );
    }
}

/// The whole task: the report's request, over the real Gemini surface, with the
/// caller framing the gemini CLI actually sends.
#[test]
fn the_gemini_cli_session_context_no_longer_hijacks_the_request() {
    let call = gemini_call(&[GEMINI_SESSION_CONTEXT, REQUEST]);

    assert_eq!(call["name"], "write_file", "{call}");
    assert_eq!(call["args"]["file_path"], "main.py", "{call}");
    assert!(
        call["args"]["content"]
            .as_str()
            .is_some_and(|content| content.contains("Hello, world!")),
        "{call}"
    );
}
