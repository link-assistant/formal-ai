use formal_ai::FormalAiEngine;

fn answer(prompt: &str) -> formal_ai::SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn assert_metatheory(prompt: &str, expect_in_answer: &str) {
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
        response.answer.contains(expect_in_answer),
        "{prompt:?} should define metatheory, got: {}",
        response.answer
    );
}

// Issue #436: the bare prompt "theory of theory" used to fall through to the
// unknown intent. It is the literal reading of "metatheory", so it now resolves
// to the seeded concept across every supported language (english, russian,
// hindi, chinese).
#[test]
fn theory_of_theory_resolves_to_metatheory_concept_in_every_language() {
    // english
    assert_metatheory("theory of theory", "metatheory");
    assert_metatheory("metatheory", "metatheory");
    assert_metatheory("what is metatheory?", "metatheory");
    // russian — the issue came from a Russian-locale session.
    assert_metatheory("теория теории", "Метатеория");
    assert_metatheory("метатеория", "Метатеория");
    // hindi
    assert_metatheory("सिद्धांत का सिद्धांत", "मेटा-सिद्धांत");
    assert_metatheory("परासिद्धांत", "मेटा-सिद्धांत");
    // chinese
    assert_metatheory("理论的理论", "元理论");
    assert_metatheory("元理论", "元理论");
}

// The new general metatheory concept must not shadow the Link Foundation
// "Links meta-theory" product concept, which keeps its own routing.
#[test]
fn links_meta_theory_routing_is_preserved() {
    for prompt in ["theory of links", "links theory", "теория связей"] {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "concept_lookup:hit:concept_links_meta_theory"),
            "{prompt:?} must still resolve to the links meta-theory concept, got {:?}",
            response.evidence_links
        );
    }
}
