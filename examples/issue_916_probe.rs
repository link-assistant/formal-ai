//! Probe for issue #916: what does the server plan for a prompt that carries
//! caller framing (issue #907), and what commands does the python hello-world
//! execution recipe carry (issue #908)?
//!
//! Run with `cargo run --example issue_916_probe`.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::ChatMessage;
use formal_ai::solver::{SolverConfig, UniversalSolver};

const GEMINI_SESSION_CONTEXT: &str = "<session_context>\n\
     Today's date is Sunday, August 2, 2026 (formatted according to the user's locale).\n\
     My operating system is: linux\n\
     I'm currently working in the directory: /tmp/work\n\
     </session_context>\n";

fn main() {
    let tools = ["write_file", "run_shell_command"];
    let prompts = [
        "Write a hello world program in Python.".to_owned(),
        format!("{GEMINI_SESSION_CONTEXT}Write a hello world program in Python."),
        "Today's date is Sunday, August 2, 2026.\nWrite a hello world program in Python."
            .to_owned(),
        "What is the date?".to_owned(),
        "What's the date?".to_owned(),
        "How much disk space is free?".to_owned(),
    ];
    for prompt in prompts {
        let messages = vec![ChatMessage::user(&prompt)];
        let plan = plan_chat_step(&messages, &tools);
        let rendered = match plan {
            Some(AgenticPlan::ToolCalls(calls)) => calls
                .iter()
                .map(|call| format!("{}({})", call.tool, call.arguments))
                .collect::<Vec<_>>()
                .join(", "),
            Some(AgenticPlan::Final(answer)) => format!("FINAL: {answer}"),
            None => "NONE".to_owned(),
        };
        println!("--- {prompt:?}\n    {rendered}\n");
    }

    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve("Write a hello world program in Python.");
    let recipe = answer.execution_recipe.as_ref().expect("execution recipe");
    println!("recipe path: {}", recipe.path);
    println!("recipe commands: {:?}", recipe.commands);
}
