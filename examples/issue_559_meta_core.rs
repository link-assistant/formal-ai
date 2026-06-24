//! Emit the issue #559 meta-core link artifacts for a sample prompt (R330–R342).
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
//! 6. the upward construction pass (R338): the post-order leaf→root walk that
//!    composes each answer back up, the construction half of the recursion;
//! 7. the solution evidence (R334): the join `need → leaf → status → method`;
//! 8. the method-selection comparison (R339): per atomic leaf, the legacy method
//!    versus the registry-resolved one, classified and counted (shown in the
//!    `compare` mode so both authorities and the agreement are visible);
//! 9. the skill-accumulation ledger (R342): each satisfied need distilled into a
//!    proposed reusable skill and each blocked need into a curriculum item, with
//!    nothing ever auto-promoted (the promotable count is always zero).
//!
//! The self-describing recipe (R335) lives as data in
//! `data/meta/recursive-core-recipe.lino`. The meta self-improvement loop (R340)
//! reads that recipe against the live pipeline and prints the proposed updated
//! algorithm — here a no-op, proving the recipe describes every stage. Together
//! these are the "deep case-study analysis" data for `docs/case-studies/issue-559`.

use formal_ai::cue_lexicon::cue_sets;
use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_construction::UpwardConstruction;
use formal_ai::meta_frame::{NeedLedger, ProblemFrame, WorkUnit};
use formal_ai::meta_reasoning::WorkUnitReasoning;
use formal_ai::meta_self_improvement::{MetaSelfImprovement, SelfImprovementMode};
use formal_ai::method_registry::MethodRegistry;
use formal_ai::selection::{SelectionComparison, SelectionMode};
use formal_ai::skill_ledger::SkillLedger;
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
    let construction = UpwardConstruction::for_unit(&root, &registry);
    let evidence = SolutionEvidence::assemble(&frame, &ledger, &registry);
    let selection = SelectionComparison::for_unit(&root, &registry);
    let skills = SkillLedger::from_evidence(&evidence);

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
        "\n# (R338) upward construction pass ({} steps)\n{}",
        construction.step_count(),
        construction.to_links_notation()
    );
    println!(
        "\n# (R334) solution evidence (accounted_for={}, fully_resolved={})\n{}",
        evidence.accounted_for(),
        evidence.fully_resolved(),
        evidence.to_links_notation()
    );
    println!(
        "\n# (R339) method-selection comparison \
         (leaves={}, agree={}, registry_rescues={}, contradict={})\n{}",
        selection.leaf_count(),
        selection.agreement_count(),
        selection.rescue_count(),
        selection.contradiction_count(),
        selection.to_links_notation(SelectionMode::Compare)
    );
    println!(
        "\n# (R342) skill-accumulation ledger \
         (proposed={}, curriculum={}, promotable={})\n{}",
        skills.proposed_count(),
        skills.curriculum_count(),
        skills.promotable_count(),
        skills.to_links_notation()
    );
}

fn main() {
    // A routed single need, a conjunction of two needs, and an unroutable need:
    // the three shapes the ledger and evidence must account for honestly.
    dump("translate apple to Russian");
    dump("translate apple to Russian and write a hello world program in Python");
    dump("zzqqx unfathomable gibberish token");

    // (R341) the recognition cue lexicon: the hardcoded natural-language cues that
    // used to be inline Rust literals, now reviewable link data grouped into named
    // cue sets with their match mode. Print the catalogue once.
    let sets = cue_sets();
    println!("================================================================");
    println!(
        "# (R341) cue lexicon — {} cue sets lifted out of Rust into data",
        sets.len()
    );
    println!("================================================================");
    for set in sets {
        println!(
            "{} [{}] handler={}: {}",
            set.name,
            set.match_mode.slug(),
            set.handler,
            set.cues.join(", ")
        );
    }

    // (R331) the method catalogue is the same for every request; print it once.
    let registry = MethodRegistry::from_dispatch();
    println!("================================================================");
    println!(
        "# (R331) method registry — {} methods derived from live dispatch",
        registry.method_count()
    );
    println!("================================================================");
    println!("{}", registry.to_links_notation());

    // (R340) the meta algorithm reading itself: compare the recipe (the algorithm
    // as link data) against the live pipeline (the algorithm as code), and emit the
    // proposed updated recipe as links. On the checked-in sources this is a no-op,
    // proving the self-description matches what the pipeline actually runs.
    let proposal = MetaSelfImprovement::from_repo().propose();
    println!("================================================================");
    println!(
        "# (R340) meta self-improvement proposal (self_consistent={}, changes={})",
        proposal.is_self_consistent(),
        proposal.change_count()
    );
    println!("# {}", proposal.summary());
    println!("================================================================");
    println!(
        "{}",
        proposal.to_links_notation(SelfImprovementMode::Propose)
    );
}
