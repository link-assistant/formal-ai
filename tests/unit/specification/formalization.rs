//! P/Q-id formalization tests.
//!
//! Issue #248 extends the Links formalization layer beyond the seed concept
//! aliases: arbitrary prompt fragments should collapse to language-independent
//! Wikidata properties/items when possible and to explicit fallbacks otherwise.

use formal_ai::translation::{formalize_prompt, FormalizationAnchorKind, FormalizationRole};
use formal_ai::FormalAiEngine;

#[test]
fn arbitrary_statement_maps_predicate_and_nouns_to_wikidata_ids() {
    let candidate = formalize_prompt("apple is a fruit", "en");

    let subject = candidate
        .slot(FormalizationRole::Subject)
        .expect("subject slot");
    let predicate = candidate
        .slot(FormalizationRole::Predicate)
        .expect("predicate slot");
    let object = candidate
        .slot(FormalizationRole::Object)
        .expect("object slot");

    assert_eq!(subject.anchor.kind, FormalizationAnchorKind::WikidataItem);
    assert_eq!(subject.anchor.id, "wikidata:Q89");
    assert_eq!(
        predicate.anchor.kind,
        FormalizationAnchorKind::WikidataProperty
    );
    assert_eq!(predicate.anchor.id, "wikidata:P31");
    assert_eq!(object.anchor.kind, FormalizationAnchorKind::WikidataItem);
    assert_eq!(object.anchor.id, "wikidata:Q3314483");

    let lino = candidate.to_links_notation();
    assert!(lino.contains("subject_q \"wikidata:Q89\""), "{lino}");
    assert!(lino.contains("predicate_p \"wikidata:P31\""), "{lino}");
    assert!(lino.contains("object_q \"wikidata:Q3314483\""), "{lino}");
}

#[test]
fn action_prompt_maps_translation_verb_to_wikidata_property() {
    let candidate = formalize_prompt("translate apple to Russian", "en");

    let predicate = candidate
        .slot(FormalizationRole::Predicate)
        .expect("predicate slot");
    assert_eq!(
        predicate.anchor.kind,
        FormalizationAnchorKind::WikidataProperty
    );
    assert_eq!(predicate.anchor.id, "wikidata:P5972");

    assert!(
        candidate
            .slots
            .iter()
            .any(|slot| slot.anchor.id == "wikidata:Q89"),
        "expected apple to anchor to Q89, got {candidate:?}",
    );
}

#[test]
fn supported_language_prompts_map_local_surfaces_to_same_language_independent_ids() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "translate apple to Russian",
        },
        Case {
            language: "ru",
            prompt: "переведи яблоко на английский",
        },
        Case {
            language: "hi",
            prompt: "सेब का हिंदी में अनुवाद करो",
        },
        Case {
            language: "zh",
            prompt: "把 苹果 翻译成中文",
        },
    ];

    for Case { language, prompt } in cases {
        let candidate = formalize_prompt(prompt, language);

        assert_eq!(
            candidate
                .slot(FormalizationRole::Predicate)
                .expect("predicate slot")
                .anchor
                .id,
            "wikidata:P5972",
            "{language} prompt should map the translation verb to P5972",
        );
        assert_eq!(
            candidate
                .slot(FormalizationRole::Object)
                .expect("object slot")
                .anchor
                .id,
            "wikidata:Q89",
            "{language} prompt should map the translated surface to Q89",
        );
    }
}

#[test]
fn unmodeled_dictionary_terms_fall_back_to_wiktionary_surfaces() {
    let candidate = formalize_prompt("what does digress mean?", "en");

    let term = candidate
        .slot(FormalizationRole::Subject)
        .expect("dictionary term slot");
    assert_eq!(term.surface, "digress");
    assert_eq!(term.anchor.kind, FormalizationAnchorKind::WiktionaryEntry);
    assert_eq!(term.anchor.id, "wiktionary:en:digress");
    assert!(
        candidate.unresolved_terms.is_empty(),
        "Wiktionary fallback should remain anchored, got {candidate:?}",
    );
}

#[test]
fn unanchored_unknown_terms_are_flagged_for_later_translation_gaps() {
    let candidate = formalize_prompt("define zzqxqv", "en");

    let term = candidate
        .slot(FormalizationRole::Subject)
        .expect("unknown term slot");
    assert_eq!(term.surface, "zzqxqv");
    assert_eq!(term.anchor.kind, FormalizationAnchorKind::RawText);
    assert_eq!(term.anchor.id, "raw:zzqxqv");
    assert_eq!(candidate.unresolved_terms, vec!["zzqxqv"]);
    assert!(
        candidate
            .to_links_notation()
            .contains("formalization_unresolved"),
        "unresolved terms must be visible in Links Notation: {}",
        candidate.to_links_notation(),
    );
}

#[test]
fn solver_evidence_exposes_formalization_ids_for_downstream_selection() {
    let response = FormalAiEngine.answer("translate apple to Russian");

    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "formalization:predicate_p:wikidata:P5972"),
        "translation predicate should be available to E4/E6 consumers, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "formalization:object_q:wikidata:Q89"),
        "object Q-id should be available to E4/E6 consumers, got {:?}",
        response.evidence_links,
    );
}
