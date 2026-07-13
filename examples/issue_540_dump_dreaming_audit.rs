//! Regenerate the committed issue-#540 dreaming-audit artifacts.
//!
//! Runs the real agentic loop on `DREAMING_AUDIT_TASK` and prints either the
//! session JSON or the generated gap-analysis document, exactly as
//! `tests/unit/issue_540_agent_cli.rs` pins them:
//!
//! ```sh
//! cargo run --example issue_540_dump_dreaming_audit -- session \
//!   > docs/case-studies/issue-540/agent-cli-session-dreaming-audit.json
//! cargo run --example issue_540_dump_dreaming_audit -- document \
//!   > docs/case-studies/issue-540/dreaming-gap-analysis.lino
//! ```

use formal_ai::agentic_coding::{dreaming_audit, run_agentic_task, DREAMING_AUDIT_TASK};

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    if mode == "document" {
        print!("{}", dreaming_audit::render_document());
    } else {
        let outcome = run_agentic_task(DREAMING_AUDIT_TASK).expect("workspace");
        println!(
            "{}",
            serde_json::to_string_pretty(&outcome.session_json()).unwrap()
        );
    }
}
