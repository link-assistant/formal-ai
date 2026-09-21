//! Probe (issue #907): what the planner and the Gemini surface do with the
//! report's phrasings, used to pin the tests to the real routing behaviour.

use formal_ai::agentic_coding::plan_chat_step;
use formal_ai::gemini::{
    GeminiGenerateContentRequest, create_gemini_generate_content_response_with_solver_and_memory,
};
use formal_ai::protocol::ChatMessage;
use formal_ai::{SolverConfig, UniversalSolver};

const PROMPTS: [&str; 9] = [
    "Write a hello world program in Python.",
    "Create a file main.py that prints Hello, world!",
    "我的用户名是什么",
    "Today's date is Sunday, August 2, 2026 (formatted according to the user's locale).\n\nWrite a hello world program in Python.",
    "The current time is 20:00.\n\nWrite a hello world program in Python.",
    "Текущее время — 20:00.\n\nWrite a hello world program in Python.",
    "今天的日期是2026年8月2日。\n\nWrite a hello world program in Python.",
    "<session_context>\nThis is the Gemini CLI. We are setting up the context for our chat.\nToday's date is Sunday, August 2, 2026 (formatted according to the user's locale).\n</session_context>\n\nWrite a hello world program in Python.",
    "what is the date?",
];

fn gemini_call(prompt: &str) -> String {
    let request: GeminiGenerateContentRequest = serde_json::from_value(serde_json::json!({
        "contents": [{"role": "user", "parts": [{"text": prompt}]}],
        "tools": [{"functionDeclarations": [
            {"name": "run_shell_command", "description": "Run a shell command",
             "parameters": {"type": "OBJECT", "properties": {"command": {"type": "STRING"}},
                            "required": ["command"]}},
            {"name": "write_file", "description": "Write a file",
             "parameters": {"type": "OBJECT",
                            "properties": {"file_path": {"type": "STRING"},
                                           "content": {"type": "STRING"}},
                            "required": ["file_path", "content"]}}
        ]}]
    }))
    .expect("valid Gemini generateContent request");
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let response = create_gemini_generate_content_response_with_solver_and_memory(
        &request,
        "formal-ai",
        &solver,
        &[],
    );
    response["candidates"][0]["content"]["parts"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|part| part.get("functionCall"))
        .map_or_else(|| "<no function call>".to_owned(), ToString::to_string)
}

fn main() {
    let tools = ["run_shell_command", "write_file"];
    for prompt in PROMPTS.iter().chain(
        [
            "The current time is 20:00.",
            "The current time is 20:00.\n\nWrite a hello world program in Python.",
        ]
        .iter(),
    ) {
        let prompt = *prompt;
        let messages = vec![ChatMessage::user(prompt)];
        let planned = plan_chat_step(&messages, &tools);
        println!(
            "--- {prompt:?}\n  plan_chat_step: {planned:?}\n  gemini: {}",
            gemini_call(prompt)
        );
    }
}
