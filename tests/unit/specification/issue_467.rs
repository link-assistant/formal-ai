//! Regression coverage for issue #467: Russian Air India stroller baggage
//! questions should resolve to a grounded fact lookup.

use formal_ai::{SymbolicAnswer, UniversalSolver};

const REPORTED_PROMPT: &str = "Даёт airindia бесплатную детскую коляску к багажу?";

#[test]
fn reported_russian_air_india_stroller_baggage_prompt_is_fact_lookup() {
    let solver = UniversalSolver::default();
    let response = solver.solve(REPORTED_PROMPT);

    assert_air_india_stroller_policy(&response, REPORTED_PROMPT);
}

#[test]
fn air_india_stroller_baggage_variants_route_to_same_fact() {
    let solver = UniversalSolver::default();

    for prompt in [
        "Does Air India allow a free stroller for an infant?",
        "Air India free collapsible stroller baggage allowance",
        "Можно ли бесплатно сдать детскую коляску на рейсе Air India?",
        "Даёт ли Air India бесплатную складную коляску сверх багажа?",
    ] {
        let response = solver.solve(prompt);
        assert_air_india_stroller_policy(&response, prompt);
    }
}

fn assert_air_india_stroller_policy(response: &SymbolicAnswer, prompt: &str) {
    assert_eq!(
        response.intent, "fact_lookup",
        "{prompt:?} should route to fact_lookup, got {} -> {}",
        response.intent, response.answer
    );
    assert!(
        response
            .thinking_steps
            .iter()
            .any(|step| step.source_event == "fact_lookup:hit"
                && step.detail == "fact_air_india_infant_stroller_allowance"),
        "{prompt:?} should select the Air India stroller allowance fact, got {:?}",
        response.thinking_steps
    );
    assert!(
        response.answer.contains("Air India"),
        "{prompt:?} answer should name Air India, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("коляск") || response.answer.to_lowercase().contains("stroller"),
        "{prompt:?} answer should mention the stroller allowance, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("бесплат") || response.answer.to_lowercase().contains("free"),
        "{prompt:?} answer should say the stroller allowance is free, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("нельзя брать в салон")
            || response
                .answer
                .to_lowercase()
                .contains("not allowed onboard"),
        "{prompt:?} answer should mention the onboard stroller limit, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q69906"),
        "{prompt:?} should keep the Air India Wikidata anchor, got {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source:")),
        "{prompt:?} should record a source event, got {:?}",
        response.evidence_links
    );
    assert!(
        response.links_notation.contains(
            "https://www.airindia.com/in/en/travel-information/baggage-guidelines/checked-baggage-allowance.html"
        ),
        "{prompt:?} trace should include the official Air India baggage source, got {}",
        response.links_notation
    );
}
