use formal_ai::FormalAiEngine;

fn answer(prompt: &str) -> formal_ai::SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// Issue #436: the bare prompt "theory of theory" used to fall through to the
// unknown intent. It is the literal reading of "metatheory", so it now resolves
// to the seeded concept.
#[test]
fn theory_of_theory_resolves_to_metatheory_concept() {
    for prompt in ["theory of theory", "metatheory", "what is metatheory?"] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "concept_lookup",
            "{prompt:?} should resolve from the seed, got {} -> {}",
            response.intent, response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "concept_lookup:hit:concept_metatheory"),
            "{prompt:?} should cite the metatheory seed record, got {:?}",
            response.evidence_links
        );
        assert!(
            response.answer.to_lowercase().contains("metatheory"),
            "{prompt:?} should define metatheory, got: {}",
            response.answer
        );
    }
}

// Issue #436 came from a Russian-locale session; the localized aliases
// "теория теории" / "метатеория" must resolve too, without shadowing the
// Link Foundation "теория связей" concept.
#[test]
fn russian_theory_of_theory_resolves_to_metatheory_concept() {
    let response = answer("теория теории");
    assert_eq!(
        response.intent, "concept_lookup",
        "Russian metatheory prompt should resolve from the seed, got {} -> {}",
        response.intent, response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "concept_lookup:hit:concept_metatheory"),
        "answer should cite the metatheory seed record, got {:?}",
        response.evidence_links
    );
    assert!(
        response.answer.to_lowercase().contains("метатеория"),
        "Russian answer should define the term, got: {}",
        response.answer
    );

    // The Links meta-theory product question stays on its own concept.
    let links = answer("теория связей");
    assert!(
        links
            .evidence_links
            .iter()
            .any(|link| link == "concept_lookup:hit:concept_links_meta_theory"),
        "links theory must still resolve to its own concept, got {:?}",
        links.evidence_links
    );
}
