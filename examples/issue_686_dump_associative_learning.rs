//! Emit the deterministic Agent CLI transcript for issue #686.
//!
//! Regenerate the committed evidence with:
//!
//! ```sh
//! cargo run --example issue_686_dump_associative_learning \
//!   > docs/case-studies/issue-686/agent-cli-session-associative-learning.json
//! ```

use formal_ai::agentic_coding::{run_agentic_task, ASSOCIATIVE_LEARNING_TASK};

fn main() {
    let outcome = run_agentic_task(ASSOCIATIVE_LEARNING_TASK)
        .expect("the associative-learning Agent CLI scenario should complete");
    println!(
        "{}",
        serde_json::to_string_pretty(&outcome.session_json())
            .expect("the Agent CLI transcript should serialize")
    );
}
