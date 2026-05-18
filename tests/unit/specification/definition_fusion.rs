//! Cross-language definition fusion tests.
//!
//! Issue #63 asks formal-ai to go beyond a single Wikipedia summary by
//! combining definitions for the same concept across language editions. The
//! implemented contract is intentionally deterministic: merge only concept
//! records that share the same seed/Wikidata anchor, preserve source-language
//! labels, deduplicate exact repeated facts, and expose the provenance in the
//! evidence links.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn definition_merge_combines_multiple_wikipedia_language_blocks() {
    let response = answer("Merge Wikipedia definitions of IIR");

    assert_eq!(response.intent, "definition_merge");
    assert!(
        response
            .answer
            .contains("Merged definition of infinite impulse response (IIR)"),
        "merged answer should name the shared concept anchor: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Source languages: en, ru, hi, zh"),
        "merged answer should disclose every source language: {}",
        response.answer
    );
    assert!(
        response.answer.contains("recursive digital filter")
            && response
                .answer
                .contains("Фильтр с бесконечной импульсной характеристикой"),
        "merged answer should include facts from more than one Wikipedia language block: {}",
        response.answer
    );
}

#[test]
fn definition_merge_keeps_shared_anchor_and_source_evidence() {
    let response = answer("Combine translated definitions for IIR");

    assert_eq!(response.intent, "definition_merge");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q740073"),
        "merged definition should preserve the shared Wikidata anchor: {:?}",
        response.evidence_links
    );
    assert!(
        response.evidence_links.iter().any(|link| link
            .starts_with("source:http:https://en.wikipedia.org/wiki/Infinite_impulse_response")),
        "merged definition should cite the English Wikipedia source: {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("definition_merge:language:")),
        "merged definition should record language-level fusion events: {:?}",
        response.evidence_links
    );
}
