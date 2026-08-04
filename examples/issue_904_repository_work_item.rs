//! Reproduce the issue-#904 repository work item end to end and print what the
//! run records and claims.
//!
//! Before the fix this printed a `goal` holding the caller's system-prompt
//! preamble, a `verification_command` of `cat .formal-ai/general-change-plan.lino`
//! (the file the run itself had just written), and a final answer that read as a
//! success ("Recorded and verified …"). After the fix the goal is the objective
//! the caller stated after the documented lead, the plan names no verification
//! command, and the run ends in a "planned, not executed" state.
//!
//! Usage: `cargo run --example issue_904_repository_work_item`

use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
use formal_ai::agentic_coding::run_agentic_task;

const HARNESS_PROMPT: &str = "\
You are an AI issue solver using @link-assistant/agent.
General guidelines.
   - When you execute commands and the output becomes large, save the logs to files.
   - When you test assumptions, keep experiment scripts in ./experiments.
Issue to solve: https://github.com/link-assistant/formal-ai/issues/904
Implement the fix in the repository and verify it with tests.";

fn main() {
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("repository work-item plan");
    println!("--- plan record ---\n{}", plan.links_notation());
    println!("goal: {}", plan.goal);
    println!("verification_command: {:?}", plan.verification_command);

    let outcome = run_agentic_task(HARNESS_PROMPT).expect("agentic replay");
    println!(
        "--- tools used ---\n{:?}",
        outcome
            .steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>()
    );
    println!("--- final answer ---\n{}", outcome.final_answer);
}
