//! P/Q-id formalization tests.
//!
//! Issue #248 extends the Links formalization layer beyond the seed concept
//! aliases: arbitrary prompt fragments should collapse to language-independent
//! Wikidata properties/items when possible and to explicit fallbacks otherwise.

use formal_ai::translation::{
    formalize_prompt, formalize_prompt_candidates, select_formalization_candidate,
    softmax_formalization_scores, FormalizationAnchorKind, FormalizationDecision,
    FormalizationRole, FormalizationSelectionConfig, FormalizationSelectionReason,
};
use formal_ai::{FormalAiEngine, SolverConfig, UniversalSolver};

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

#[test]
fn temperature_selection_is_deterministic_for_fixed_config() {
    let candidates = formalize_prompt_candidates("apple is a fruit", "en");
    assert!(
        candidates.len() >= 2,
        "ambiguous relation phrasing should expose competing candidates"
    );

    let probabilities = softmax_formalization_scores(&candidates, 0.7);
    let total = probabilities.iter().sum::<f32>();
    assert!(
        (total - 1.0).abs() < 0.000_1,
        "softmax probabilities must sum to one, got {probabilities:?}"
    );

    let config = FormalizationSelectionConfig {
        temperature: 0.7,
        guess_probability: 1.0,
        questioning_rigor: 0.4,
    };
    let first = select_formalization_candidate(&candidates, config, "apple is a fruit");
    let second = select_formalization_candidate(&candidates, config, "apple is a fruit");

    assert_eq!(first.selected_index(), second.selected_index());
    assert_eq!(first.probabilities.len(), second.probabilities.len());
    for (left, right) in first.probabilities.iter().zip(&second.probabilities) {
        assert!((*left - *right).abs() < f32::EPSILON);
    }
}

#[test]
fn high_rigor_low_margin_asks_smallest_clarifying_question() {
    let config = SolverConfig {
        temperature: 0.7,
        guess_probability: 0.0,
        questioning_rigor: 1.0,
        ..SolverConfig::default()
    };
    let response = UniversalSolver::new(config).solve("apple is a fruit");

    assert_eq!(response.intent, "clarify_interpretation");
    assert!(
        response.answer.contains("instance of"),
        "{}",
        response.answer
    );
    assert!(
        response.answer.contains("subclass of"),
        "{}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "policy:clarify_under_ambiguity"),
        "clarify policy must be explicit, got {:?}",
        response.evidence_links
    );
}

#[test]
fn low_rigor_ambiguous_prompt_guesses_and_records_policy() {
    let candidates = formalize_prompt_candidates("apple is a fruit", "en");
    let selection = select_formalization_candidate(
        &candidates,
        FormalizationSelectionConfig {
            temperature: 0.7,
            guess_probability: 0.0,
            questioning_rigor: 0.0,
        },
        "apple is a fruit",
    );
    assert!(matches!(
        selection.decision,
        FormalizationDecision::Selected {
            reason: FormalizationSelectionReason::GuessedUnderAmbiguity,
            ..
        }
    ));

    let config = SolverConfig {
        temperature: 0.7,
        guess_probability: 0.0,
        questioning_rigor: 0.0,
        ..SolverConfig::default()
    };
    let response = UniversalSolver::new(config).solve("apple is a fruit");
    assert_ne!(response.intent, "clarify_interpretation");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "policy:guessed_under_ambiguity"),
        "guess policy must be explicit, got {:?}",
        response.evidence_links
    );
}

#[test]
fn same_prompt_and_config_produce_same_interpretation_choice() {
    let config = SolverConfig {
        temperature: 0.9,
        guess_probability: 0.3,
        questioning_rigor: 0.1,
        ..SolverConfig::default()
    };
    let solver = UniversalSolver::new(config);

    let first = solver.solve("apple is a fruit");
    let second = solver.solve("apple is a fruit");

    assert_eq!(first, second);
    assert!(
        first
            .evidence_links
            .iter()
            .filter(|link| link.starts_with("candidate:"))
            .count()
            >= 2,
        "candidate formalizations must be visible in evidence"
    );
}

#[test]
fn temperature_selection_preserves_supported_language_candidates() {
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
    let config = FormalizationSelectionConfig {
        temperature: 0.7,
        guess_probability: 0.8,
        questioning_rigor: 0.4,
    };

    for Case { language, prompt } in cases {
        let candidates = formalize_prompt_candidates(prompt, language);
        let selection = select_formalization_candidate(&candidates, config, prompt);
        assert!(
            selection.selected_candidate().is_some(),
            "{language} prompt should still select a formalization"
        );
    }
}
