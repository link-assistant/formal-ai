//! Issue #535 — grounded statement verification, end to end.
//!
//! Comment 4754747438 asks us to *"use our web search to check for each
//! statement in the text"* and to weigh those statements with
//! relative-meta-logic: assume a statement true, raise its probability with
//! trusted original-first sources, lower it with contradicting evidence, and
//! ignore unoriginal reposts.
//!
//! This example demonstrates the whole deterministic, offline plan the solver
//! records for a document-verification request:
//!   1. split a text sample into checkable statements across scripts,
//!   2. build the grounding web-search query for each, and
//!   3. show how the assumed-true prior moves under different tiers of evidence.
//!
//! Run with: `cargo run --example issue_535_extract_probe`

use formal_ai::relative_meta_logic::{
    RelativeEvidence, SourceTier, Stance, StatementAssessment, TruthValue, ASSUMED_TRUE_PRIOR,
};
use formal_ai::statement_verification::StatementVerificationPlan;

fn main() {
    // 1) Statement extraction across scripts. The Chinese sample shows the
    //    char-count gate keeping space-free sentences while dropping fragments.
    let samples = [
        "The tower opened in 1889. It stands 300 metres tall.",
        "埃菲尔铁塔于1889年开放。它有三百多米高。",
    ];
    for sample in samples {
        let plan = StatementVerificationPlan::from_sample(sample);
        println!("sample: {sample}");
        println!("  statements: {}", plan.len());
        for statement_plan in &plan.statements {
            println!("    - statement: {}", statement_plan.statement);
            println!("      query:     {}", statement_plan.query);
            println!(
                "      assessment: {}",
                statement_plan.assessment.trace_payload()
            );
        }
    }

    // 2) How the assumed-true prior moves under the trusted-source policy. A
    //    statement starts likely (0.6) and is raised by original first sources,
    //    lowered by contradicting originals, and left untouched by reposts.
    println!("\nrelative-meta-logic evidence weighing (prior = {ASSUMED_TRUE_PRIOR:.6}):");
    let scenarios: &[(&str, Vec<RelativeEvidence>)] = &[
        ("no evidence (offline)", vec![]),
        (
            "original first-party supports",
            vec![RelativeEvidence::new(
                "agency.gov",
                SourceTier::OriginalFirstParty,
                Stance::Supports,
                TruthValue::new(0.9),
            )],
        ),
        (
            "original journalism contradicts",
            vec![RelativeEvidence::new(
                "gazette.example",
                SourceTier::OriginalJournalism,
                Stance::Contradicts,
                TruthValue::new(0.8),
            )],
        ),
        (
            "unoriginal repost supports (ignored)",
            vec![RelativeEvidence::new(
                "aggregator.example",
                SourceTier::Unoriginal,
                Stance::Supports,
                TruthValue::new(1.0),
            )],
        ),
    ];
    for (label, evidence) in scenarios {
        let assessment = StatementAssessment::assess(
            "The tower opened in 1889".to_owned(),
            TruthValue::new(ASSUMED_TRUE_PRIOR),
            evidence,
        );
        println!("  {label:38} -> {}", assessment.trace_payload());
    }
}
