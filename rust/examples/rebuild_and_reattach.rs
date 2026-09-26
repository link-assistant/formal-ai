//! Recompile Formal AI and reattach the improved worker to the UI (issue #558,
//! `R558-06`).
//!
//! Once a change is accepted (a green benchmark gate *and* a human approval), the final
//! step of the self-change loop is to rebuild Formal AI and reattach the improved
//! WebAssembly worker to the browser UI. This example composes that plan for the
//! canonical accepted change and prints the ordered, reversible rebuild-and-reattach
//! pipeline. Nothing is rebuilt or restarted — the plan is the reviewable product, and
//! every step is observable and reversible, so keeping the swap stays a human decision.
//! Neural inference is not used; the plan is a deterministic function of the accepted
//! change and the grounded UI artifacts.
//!
//! Usage: `cargo run --example rebuild_and_reattach`. The one-line summary and each
//! step prints to stderr; the full rebuild-and-reattach plan (Links Notation) prints to
//! stdout.

use formal_ai::agentic_coding::rebuild_plan::render_document;
use formal_ai::canonical_rebuild_plan;

fn main() {
    let plan = canonical_rebuild_plan();
    eprintln!("{}", plan.summary());
    for step in &plan.steps {
        eprintln!("  {}. {}", step.ordinal, step.action);
    }

    print!("{}", render_document());
}
