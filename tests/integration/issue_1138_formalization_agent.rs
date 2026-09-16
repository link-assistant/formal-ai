//! Issue #1138, plan 04 L16 — the deep formalizer through a real agent process.
//!
//! Plans 06 and 07 of the issue #710 case study each recorded a *hand-run*
//! `formal-ai agent` probe and wrote the finding into prose ("0 concepts, only
//! preserved spans"). These two tests replace both probes with always-run
//! coverage: a real `formal-ai agent` process over a held-out requirement must
//! report the concepts it grounded, and must report the needs it could not
//! ground rather than claiming coverage.

use std::process::Command;

/// A held-out requirement: `isogram` appears in no seed file, so the concept
/// can only come from a source.
const HELD_OUT_REQUIREMENT: &str =
    "Formalize this requirement: an isogram check must reject any word that repeats a letter.";

/// A requirement naming an operation no trusted source defines, so the honest
/// outcome is an ungrounded need rather than a confident formalization.
const UNGROUNDABLE_REQUIREMENT: &str =
    "Formalize this requirement: a blorptide check must reject any sequence that is not a blorptide.";

fn agent(task: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["--silent", "agent", "--task", task])
        .output()
        .expect("failed to execute the formal-ai binary");
    assert!(
        output.status.success(),
        "agent run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn an_agent_run_over_an_unfamiliar_requirement_reports_grounded_concepts_not_only_spans() {
    let transcript = agent(HELD_OUT_REQUIREMENT);

    assert!(
        transcript.contains("concept:isogram"),
        "the agent must ground the requirement's key concept: {transcript}"
    );
    assert!(
        transcript.contains("source_url") && transcript.contains("sha256"),
        "a grounded concept carries the provenance of the bytes that ground it: {transcript}"
    );
    assert!(
        !transcript.contains("tale:fisherman-and-fish"),
        "the seeded fairy tale is a regression corpus, not the answer: {transcript}"
    );
}

#[test]
fn an_agent_run_reports_its_ungrounded_needs_rather_than_claiming_coverage() {
    let transcript = agent(UNGROUNDABLE_REQUIREMENT);

    assert!(
        transcript.contains("needs_raised"),
        "the run states how many needs it raised: {transcript}"
    );
    assert!(
        transcript.contains("unsatisfiable") || transcript.contains("needs_grounded 0"),
        "an ungrounded need is reported, never silently dropped: {transcript}"
    );
    assert!(
        !transcript.contains("blorptide is"),
        "a word no source defines never acquires a meaning: {transcript}"
    );
}
