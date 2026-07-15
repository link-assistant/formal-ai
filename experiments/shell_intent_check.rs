//! Manual check: does a semantic shell *intent* route to the right command
//! `tool_call`? Run with `cargo run --example shell_intent_check` after adding it
//! to Cargo.toml, or paste into a scratch test. Kept here for reuse (issue #680).

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::ChatMessage;

fn main() {
    let cases = [
        "Show me what's in this folder",
        "Print the current working directory",
        "Tell me today's date using the shell",
        "How much disk space is free?",
        "Show the running processes",
        "Count the number of lines in Cargo.toml",
        "Create a directory called build",
        "What is my username?",
        // Negative: must NOT be a shell command.
        "read the file alpha.txt",
    ];
    for prompt in cases {
        let messages = vec![ChatMessage::user(prompt)];
        let plan = plan_chat_step(&messages, &["bash", "read_file"]);
        match plan {
            Some(AgenticPlan::ToolCalls(calls)) => {
                println!("{prompt:?} -> {} {}", calls[0].tool, calls[0].arguments);
            }
            other => println!("{prompt:?} -> {other:?}"),
        }
    }
}
