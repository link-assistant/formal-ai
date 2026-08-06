//! Replay the issue #908 run: drive the Python hello-world recipe through the
//! qwen shell envelopes the harness actually sent, printing what Formal AI
//! plans after each one.
//!
//! Before the fix the `Exit Code: 0` verification step ended the run with
//! "The agentic CLI harness could not complete `main.py`", because the
//! envelope's unconditional `Error: (none)` line read as an error.

use formal_ai::agentic_coding::{plan_symbolic_command_reroute, AgenticPlan};
use formal_ai::protocol::ChatMessage;
use formal_ai::solver::{SolverConfig, UniversalSolver};

const PROMPT: &str = "Write a hello world program in Python.";

/// The harness envelope, byte-shaped like the one quoted in the issue.
fn envelope(command: &str, output: &str, exit_code: i32) -> String {
    format!(
        "Command: {command}\nDirectory: (root)\nOutput: {output}\nError: (none)\n\
         Exit Code: {exit_code}\nSignal: 0\nProcess Group PGID: 685377"
    )
}

fn main() {
    let answer = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
    .solve(PROMPT);
    let tools = ["write_file", "run_shell_command"];
    // What the harness reported for each step of the run, in order.
    let results = [
        String::from("Successfully overwrote file: main.py"),
        envelope("python3 -m py_compile main.py", "(empty)", 0),
        envelope("python3 main.py", "Hello, World!", 0),
    ];

    let mut messages = vec![ChatMessage::user(PROMPT)];
    for (step, result) in results.iter().enumerate() {
        match plan_symbolic_command_reroute(&messages, &tools, &answer) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                println!("=== step {step}: {} {}", calls[0].tool, calls[0].arguments);
                let id = format!("call_{step}");
                messages.push(ChatMessage::tool_result(
                    &id,
                    &calls[0].tool,
                    result.clone(),
                ));
            }
            Some(AgenticPlan::Final(text)) => {
                println!("=== step {step}: run ended early\n{text}");
                return;
            }
            None => {
                println!("=== step {step}: no reroute");
                return;
            }
        }
    }

    match plan_symbolic_command_reroute(&messages, &tools, &answer) {
        Some(AgenticPlan::Final(text)) => println!("=== final answer\n{text}"),
        other => println!("=== expected a final answer, got {other:?}"),
    }
}
