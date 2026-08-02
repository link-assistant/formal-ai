//! Probe the `plan_chat_step` route order after merging issue #715's recipes
//! with main's literal-write probe (PR #731).
//!
//! Main moved the `compose_general_change_plan` write probe ahead of the keyword
//! recipes so arbitrary filenames containing "issue"/"report"/"learning" are not
//! misrouted. Issue #715 added `code_rewrite_learning` and `code_artifact`
//! recipes that key on such names, so the two changes could collide. This probe
//! prints the chosen route for each canonical task to confirm they still reach
//! their own recipes.
//!
//! Run: copy to `examples/` and `cargo run --example probe_planner_route_order`.
//!
//! Result (2026-07-16, merge of origin/main into issue-715 branch): the
//! canonical learning task reaches its own recipe, and a literal write whose
//! filename contains "learning" reaches main's general-change plan. The two
//! route sets do not collide.

use formal_ai::agentic_coding::planner::plan_chat_step;
use formal_ai::agentic_coding::CODE_REWRITE_LEARNING_TASK;
use formal_ai::protocol::ChatMessage;

fn probe(label: &str, task: &str) {
    let messages = vec![ChatMessage {
        role: String::from("user"),
        content: serde_json::from_value(serde_json::json!(task)).unwrap(),
        ..Default::default()
    }];
    let tools = ["write_file", "read_file", "run_command", "edit_file"];
    match plan_chat_step(&messages, &tools) {
        Some(plan) => {
            let rendered = format!("{plan:?}");
            let head: String = rendered.chars().take(160).collect();
            println!("[{label}]\n  -> {head}\n");
        }
        None => println!("[{label}]\n  -> None (falls through to ordinary solver)\n"),
    }
}

fn main() {
    // The canonical recipe task: names the report path but no literal content,
    // so the write probe must decline and the learning recipe must win.
    probe("code_rewrite_learning canonical", CODE_REWRITE_LEARNING_TASK);

    // A literal write whose filename contains "learning": main's probe must win.
    probe(
        "literal write with learning-ish name",
        "Write the text hello world to notes/learning-report.txt",
    );
}
