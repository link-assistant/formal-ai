//! Curated Link Assistant / Link Foundation project lookup tests.
//!
//! These tests pin down the "what is <project>?" behavior for the curated
//! registry described in `data/seed/projects.lino` and the
//! formalize-summarize-deformalize pipeline in `src/summarization.rs`.
//! Splitting them out of `prompt_variations.rs` keeps each test file under
//! the repository's 1000-line cap (see `scripts/check-file-size.rs`).

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn russian_hive_mind_prompt_prefers_link_assistant_project() {
    let response = answer("Что такое Hive Mind?");
    assert_eq!(
        response.intent, "hive_mind_lookup",
        "Hive Mind should not fall through to a generic concept or Wikipedia-style answer: {}",
        response.answer,
    );
    assert!(response.answer.contains("link-assistant/hive-mind"));
    // Russian localized description comes from data/seed/projects.lino via the
    // summarization pipeline ("Hive Mind — это ИИ, который ...").
    assert!(
        response.answer.contains("ИИ"),
        "Russian Hive Mind answer should carry the localized ИИ description, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("hive_mind:preferred:")),
        "expected preferred Hive Mind evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn english_hive_mind_prompt_prefers_link_assistant_project() {
    let response = answer("What is Hive Mind?");
    assert_eq!(
        response.intent, "hive_mind_lookup",
        "Hive Mind should resolve to the curated project answer, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(response.answer.contains("link-assistant/hive-mind"));
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("ai that controls ais")
            || lower.contains("orchestrates")
            || lower.contains("the ai"),
        "English Hive Mind answer should describe the project, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("hive_mind:preferred:")),
        "expected preferred Hive Mind evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn curated_project_concept_prompt_routes_to_project_lookup() {
    let response = answer("What is link-cli?");
    assert_eq!(
        response.intent, "project_lookup",
        "curated link-cli concept lookup should route through project_lookup, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("link-foundation/link-cli"),
        "link-cli answer should link to the canonical repository, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project:preferred:")),
        "expected curated project evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn curated_project_lookup_records_summarization_evidence() {
    let response = answer("What is command-stream?");
    assert_eq!(response.intent, "project_lookup");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("summarization:mode")),
        "project lookup should log a summarization mode event, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("summarization:language")),
        "project lookup should log a summarization language event, got {:?}",
        response.evidence_links,
    );
}
