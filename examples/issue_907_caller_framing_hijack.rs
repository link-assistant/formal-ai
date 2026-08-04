//! Reproduce the issue-#907 caller-framing hijack over the Gemini surface and
//! print the function call agent mode emits for each framing.
//!
//! The gemini CLI prefixes every turn with a `<session_context>` block whose
//! second line is *"Today's date is Sunday, August 2, 2026 (formatted according
//! to the user's locale)."* Before the fix Formal AI answered **that sentence**:
//! it emitted `run_shell_command({"command":"date"})` and never acted on the
//! request that followed, so every `gemini` run through agent mode was hijacked.
//! After the fix the caller's context block is not the user's request, a
//! declarative "X is Y" statement is not a question, and the actual request —
//! *"Write a hello world program in Python."* — is what gets planned.
//!
//! This is the in-process twin of the report's `repro-intent-hijack.sh`: same
//! request, same two tool declarations, same protocol; the only variable is one
//! line of caller context.
//!
//! Usage: `cargo run --example issue_907_caller_framing_hijack`

use formal_ai::gemini::{
    create_gemini_generate_content_response_with_solver_and_memory, GeminiGenerateContentRequest,
};
use formal_ai::{SolverConfig, UniversalSolver};

/// The context block the gemini CLI really sends, verbatim from the report.
const GEMINI_SESSION_CONTEXT: &str = "\
<session_context>
This is the Gemini CLI. We are setting up the context for our chat.
Today's date is Sunday, August 2, 2026 (formatted according to the user's locale).
My operating system is: linux
</session_context>";

/// The same block with only the date sentence removed.
const GEMINI_SESSION_CONTEXT_WITHOUT_DATE: &str = "\
<session_context>
This is the Gemini CLI. We are setting up the context for our chat.
My operating system is: linux
</session_context>";

const REQUEST: &str = "Write a hello world program in Python.";

fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

/// The function call the Gemini surface emits for `parts`, rendered like the
/// report's script renders it.
fn emitted(parts: &[&str]) -> String {
    let request: GeminiGenerateContentRequest = serde_json::from_value(serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": parts.iter().map(|text| serde_json::json!({"text": text})).collect::<Vec<_>>(),
        }],
        "tools": [{"functionDeclarations": [
            {
                "name": "run_shell_command",
                "description": "Run a shell command",
                "parameters": {"type": "OBJECT", "properties": {"command": {"type": "STRING"}}, "required": ["command"]}
            },
            {
                "name": "write_file",
                "description": "Write a file",
                "parameters": {"type": "OBJECT", "properties": {"file_path": {"type": "STRING"}, "content": {"type": "STRING"}}, "required": ["file_path", "content"]}
            }
        ]}]
    }))
    .expect("gemini request");

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
        .map_or_else(
            || String::from("text-only"),
            |call| {
                format!(
                    "{}({})",
                    call["name"].as_str().unwrap_or_default(),
                    call["args"].to_string().chars().take(60).collect::<String>()
                )
            },
        )
}

fn main() {
    println!("== the same request under different caller framing");
    for context in [
        GEMINI_SESSION_CONTEXT,
        GEMINI_SESSION_CONTEXT_WITHOUT_DATE,
        "Today's date is Sunday, August 2, 2026.",
        "The current time is 20:00.",
        "The date is Sunday.",
        "Today is Sunday, August 2, 2026.",
        "date",
        "My operating system is: linux",
        "",
    ] {
        let label = context.replace('\n', " ⏎ ");
        let parts: Vec<&str> = if context.is_empty() {
            vec![REQUEST]
        } else {
            vec![context, REQUEST]
        };
        println!(
            "  {:<52} -> {}",
            label.chars().take(50).collect::<String>(),
            emitted(&parts)
        );
    }

    println!();
    println!("== a user who really does ask for the date still gets it");
    for question in ["what is the date?", "show me today's date", "run date"] {
        println!("  {question:<52} -> {}", emitted(&[question]));
    }
}
