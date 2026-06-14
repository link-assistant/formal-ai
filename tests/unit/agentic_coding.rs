//! Behavioural pins for the Links Notation formalizer (issue #468).
//!
//! These lock the meta-language formalization: text in, Links Notation out, with
//! all nine protocol primitives realised as links — and the honest, grounded
//! behaviour on text the closed lexicon does not recognise.

use formal_ai::agentic_coding::{
    coverage_line, formalize_text_to_links, CANONICAL_FISHERMAN_SYNOPSIS, FISHERMAN_DOC_ID,
    PRIMITIVE_KINDS,
};

#[test]
fn canonical_synopsis_covers_all_nine_primitives() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let summary = &formalized.summary;

    assert!(
        summary.covers_all_nine(),
        "expected all nine primitives, got: {}",
        coverage_line(summary)
    );
    assert_eq!(summary.covered.len(), PRIMITIVE_KINDS.len());
    // The covered list is reported in canonical primitive order.
    assert_eq!(summary.covered, PRIMITIVE_KINDS.to_vec());

    // Pinned counts for the co-designed synopsis + lexicon.
    assert_eq!(summary.doc_id, FISHERMAN_DOC_ID);
    assert_eq!(summary.concepts, 3, "greed (lexicon) + ransom + wish");
    assert_eq!(
        summary.entities, 4,
        "old_man, old_woman, golden_fish, trough"
    );
    assert_eq!(summary.predicates, 6);
    assert_eq!(summary.assertions, 7);
    assert_eq!(summary.procedures, 1);
    assert_eq!(summary.contexts, 2);
    assert_eq!(summary.temporals, 3);
    assert_eq!(summary.modals, 3);
    assert_eq!(summary.annotations, 7);
    assert_eq!(summary.total_records(), 37);
}

#[test]
fn every_output_record_is_links_notation_not_a_rust_struct() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;

    // The header and each primitive kind appears as a Links Notation record head.
    assert!(document.starts_with("knowledge_base\n  id \"tale:fisherman-and-fish\""));
    for kind in PRIMITIVE_KINDS {
        assert!(
            document.contains(&format!("{kind}\n  id ")),
            "missing `{kind}` record in document"
        );
    }
    // Indentation is two spaces (the meta-language convention), never tabs.
    assert!(!document.contains('\t'));
    assert!(document.contains("\n  id \"a:0\""));
}

#[test]
fn grounded_svo_extraction_is_faithful() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;

    // "Старик поймал золотую рыбку." — subject, predicate, object all grounded,
    // with the predicate's temporal and the scene's context attached as links.
    assert!(document.contains("subject \"ent:old_man\""));
    assert!(document.contains("predicate \"pred:catch\""));
    assert!(document.contains("object \"ent:golden_fish\""));
    assert!(document.contains("time \"temporal:в-начале-сказки\""));
    assert!(document.contains("context \"ctx:seaside\""));

    // Modality is carried as a link to a modal record.
    assert!(document.contains("modal \"modal:commitment\""));
    assert!(document.contains("modal:commitment"));

    // Provenance ties each assertion back to a character span of the source.
    assert!(document.contains("provenance \"tale:fisherman-and-fish@0:28\""));
}

#[test]
fn unmatched_object_falls_back_to_an_honest_literal() {
    // "Старуха потребовала стать владычицей морской." — the demand's object is
    // not in the closed lexicon, so it is recorded as a literal rather than an
    // invented entity. The recogniser never hallucinates a relation it cannot
    // ground.
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;
    assert!(document.contains("object \"стать владычицей морской\""));
    assert!(document.contains("object_kind \"literal\""));
}

#[test]
fn annotations_use_real_character_offsets() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let document = &formalized.links_notation;
    // First sentence spans characters 0..28 ("Старик поймал золотую рыбку.").
    assert!(document.contains("span \"0:28\""));
    assert!(document.contains("text \"Старик поймал золотую рыбку.\""));
}

#[test]
fn formalization_is_deterministic() {
    // The fetched-text == fallback-text invariant the planner relies on: the same
    // input always yields byte-identical Links Notation.
    let first = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    let second = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");
    assert_eq!(first.links_notation, second.links_notation);
    assert_eq!(first.summary, second.summary);
}

#[test]
fn arbitrary_text_still_produces_a_valid_knowledge_base() {
    // Open-domain text the lexicon does not recognise: every sentence still
    // becomes an annotation plus a natural-language assertion. No work matched,
    // so there are no lexicon-sourced concepts/procedures/contexts — and we do
    // not pretend otherwise.
    let formalized = formalize_text_to_links("A cat sat on a mat. Then it slept.", "doc:demo");
    let summary = &formalized.summary;

    assert_eq!(summary.doc_id, "doc:demo");
    assert_eq!(summary.annotations, 2);
    assert_eq!(summary.assertions, 2);
    assert_eq!(summary.procedures, 0);
    assert_eq!(summary.contexts, 0);
    assert!(!summary.covers_all_nine());
    assert!(formalized
        .links_notation
        .contains("predicate \"pred:states\""));
    assert!(formalized
        .links_notation
        .contains("natural_language \"A cat sat on a mat.\""));
    // Language detection falls back to English for non-Cyrillic input.
    assert!(formalized.links_notation.contains("language \"en\""));
}

#[test]
fn explicit_doc_id_overrides_the_default() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "kb:custom");
    assert_eq!(formalized.summary.doc_id, "kb:custom");
    assert!(formalized
        .links_notation
        .starts_with("knowledge_base\n  id \"kb:custom\""));
}
