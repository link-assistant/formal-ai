//! Print the plan Formal AI's agent-mode planner produces for a decomposition
//! node instruction (issue #1066).
//!
//! The issue-#1028 ladder hands each node a task *and* an evidence obligation:
//! "… Leave observable evidence in <path>. The first line must be exactly
//! <marker>." This tool shows what the planner decides for such an instruction,
//! so a routing regression is visible without standing up a server and the real
//! Agent CLI.
//!
//! Usage: `cargo run --example issue_1066_ladder_node_plan -- ["<prompt>"]`

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};

const LADDER_TOOLS: [&str; 14] = [
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

fn main() {
    let prompts: Vec<String> = {
        let supplied: Vec<String> = std::env::args().skip(1).collect();
        if supplied.is_empty() {
            vec![ladder_node_prompt(
                "Atomic task L01: Inspect the existing task-decomposition data model and identify where a node stores its children.",
                "1.1.1.1.1",
                5,
                "observable evidence exists",
            )]
        } else {
            supplied
        }
    };

    for prompt in &prompts {
        println!("=== prompt ===\n{prompt}\n");
        match plan_chat_step(&[ChatMessage::user(prompt)], &LADDER_TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                for call in calls {
                    println!("tool: {}\nargs: {}", call.tool, call.arguments);
                }
            }
            Some(AgenticPlan::Final(answer)) => println!("final: {answer}"),
            None => println!("no plan"),
        }
        println!();
    }
}

/// Reproduce the instruction `experiments/issue_1028_agent_cli_ladder/run.sh`
/// sends to one node, verbatim in shape.
fn ladder_node_prompt(task: &str, id: &str, depth: u32, criterion: &str) -> String {
    format!(
        "{task}\n\nThis is recursive binary-tree node {id} at depth {depth}. Solve only this \
         node's task in this fresh temporary repository. Its completion criterion is: \
         {criterion}. Leave observable evidence in .agent-ladder/node-{id}-proof.md. The first \
         line must be exactly node_path={id}. Use web research when it materially improves \
         factual accuracy. Do not claim success without evidence.\n"
    )
}
