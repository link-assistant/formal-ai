//! Drive the full agentic-coding loop offline and print the transcript plus the
//! finished knowledge base (issue #468).
//!
//! Run with:
//!
//! ```text
//! cargo run --example issue_468_agentic_loop
//! ```
//!
//! The in-repo driver plays the role of an external agentic CLI against our
//! OpenAI-compatible server: it advertises tools, executes every emitted tool call
//! — `web_search` / `web_fetch` against an offline corpus, `write_file` /
//! `run_command` in a sandboxed workspace — feeds each result back, and loops
//! until the server returns the finished Links Notation knowledge base. This is
//! the "solve such tasks in agentic mode" behaviour the issue asks for, running
//! deterministically with no network and no neural inference.

use formal_ai::agentic_coding::run_agentic_task;

fn main() {
    let task = "Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base \
                covering all nine protocol primitives.";

    let outcome = run_agentic_task(task).expect("the sandbox workspace should be created");

    println!("# Agentic transcript\n{}", outcome.transcript());
    println!("# Final answer\n{}", outcome.final_answer);
}
