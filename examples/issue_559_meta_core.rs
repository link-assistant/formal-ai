//! Emit the issue #559 meta-core link artifacts for a sample prompt (R330–R337).
//!
//! Run with:
//!
//! ```text
//! cargo run --example issue_559_meta_core
//! ```
//!
//! This drives the general meta core directly — no neural inference, no network —
//! and prints, in Links Notation, each artifact the solver now records as a
//! trace-only loop event for every request:
//!
//! 1. the problem frame (R330): the request as an explicit set of needs;
//! 2. the recursive, bounded work-unit tree (R332): decompose until atomic;
//! 3. the need-satisfaction ledger (R333): one row per need with its status;
//! 4. the method registry (R331): the catalogue derived from live dispatch;
//! 5. the white-box recursive reasoning (R337): the downward/upward thought per
//!    step, so the box is inspectable, not just the predicate;
//! 6. the solution evidence (R334): the join `need → leaf → status → method`.
//!
//! The self-describing recipe (R335) lives as data in
//! `data/meta/recursive-core-recipe.lino`. Together these are the "deep
//! case-study analysis" data for `docs/case-studies/issue-559`.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::{NeedLedger, ProblemFrame, WorkUnit};
use formal_ai::meta_reasoning::WorkUnitReasoning;
use formal_ai::method_registry::MethodRegistry;
use formal_ai::solution_evidence::SolutionEvidence;
use formal_ai::translation::formalize_prompt;

fn dump(prompt: &str) {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);
    let root = WorkUnit::from_formalization(&formalization, 4);
    let ledger = NeedLedger::resolve(&frame, &root);
    let registry = MethodRegistry::from_dispatch();
    let reasoning = WorkUnitReasoning::for_unit(&root, &registry);
    let evidence = SolutionEvidence::assemble(&frame, &ledger, &registry);

    println!("================================================================");
    println!("PROMPT: {prompt}");
    println!("================================================================");
    println!("\n# (R330) problem frame\n{}", frame.to_links_notation());
    println!(
        "\n# (R332) recursive work-unit tree\n{}",
        root.to_links_notation()
    );
    println!(
        "\n# (R333) need-satisfaction ledger\n{}",
        ledger.to_links_notation()
    );
    println!(
        "\n# (R337) white-box recursive reasoning ({} steps)\n{}",
        reasoning.step_count(),
        reasoning.to_links_notation()
    );
    println!(
        "\n# (R334) solution evidence (accounted_for={}, fully_resolved={})\n{}",
        evidence.accounted_for(),
        evidence.fully_resolved(),
        evidence.to_links_notation()
    );
}

fn main() {
    // A routed single need, a conjunction of two needs, and an unroutable need:
    // the three shapes the ledger and evidence must account for honestly.
    dump("translate apple to Russian");
    dump("translate apple to Russian and write a hello world program in Python");
    dump("zzqqx unfathomable gibberish token");

    // (R331) the method catalogue is the same for every request; print it once.
    let registry = MethodRegistry::from_dispatch();
    println!("================================================================");
    println!(
        "# (R331) method registry — {} methods derived from live dispatch",
        registry.method_count()
    );
    println!("================================================================");
    println!("{}", registry.to_links_notation());
}
