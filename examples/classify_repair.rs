//! Classify each class of failure and show which part Formal AI would repair (issue
//! #558, `R558-02`).
//!
//! The self-healing loop repairs a single canonical failure by synthesising a solver
//! method. This example exercises the *general* front of the loop: it classifies three
//! canonical failure traces — one whose repair is a solver method, one a data record,
//! and one a test — and prints the grounded, human-gated repair strategy the classifier
//! composes for each. Every classification and plan is a deterministic function of the
//! trace; neural inference is not used, and nothing is applied — each strategy is a
//! reviewable plan.
//!
//! Usage: `cargo run --example classify_repair`. The one-line summary of each strategy
//! prints to stderr; the full repair-strategies document (Links Notation) prints to
//! stdout.

use formal_ai::agentic_coding::repair_strategy::render_document;
use formal_ai::canonical_strategies;

fn main() {
    for strategy in canonical_strategies() {
        eprintln!("[{}] {}", strategy.target.slug(), strategy.summary());
    }

    print!("{}", render_document());
}
