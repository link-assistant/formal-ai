//! Regression coverage for issue #467: Russian Air India stroller baggage
//! questions should resolve to a grounded fact lookup.

use formal_ai::{SymbolicAnswer, UniversalSolver};

const REPORTED_PROMPT: &str = "Даёт airindia бесплатную детскую коляску к багажу?";

struct StrollerCase {
    language: &'static str,
    prompt: &'static str,
    expected_fragments: &'static [&'static str],
}

#[test]
fn reported_russian_air_india_stroller_baggage_prompt_is_fact_lookup() {
    let solver = UniversalSolver::default();
    let response = solver.solve(REPORTED_PROMPT);

    assert_air_india_stroller_policy(
        &response,
        "ru",
        REPORTED_PROMPT,
        &["Air India", "коляск", "бесплат", "нельзя брать в салон"],
    );
}

#[test]
fn air_india_stroller_baggage_variants_route_to_same_fact() {
    let solver = UniversalSolver::default();

    for case in [
        StrollerCase {
            language: "en",
            prompt: "Does Air India allow a free stroller for an infant?",
            expected_fragments: &["Air India", "stroller", "free", "not allowed onboard"],
        },
        StrollerCase {
            language: "ru",
            prompt: "Можно ли бесплатно сдать детскую коляску на рейсе Air India?",
            expected_fragments: &["Air India", "коляск", "бесплат", "нельзя брать в салон"],
        },
        StrollerCase {
            language: "hi",
            prompt: "क्या एयर इंडिया शिशु के लिए मुफ्त बेबी स्ट्रॉलर सामान में देती है?",
            expected_fragments: &[
                "Air India",
                "बेबी स्ट्रॉलर",
                "मुफ्त",
                "केबिन में ले जाने की अनुमति नहीं",
            ],
        },
        StrollerCase {
            language: "zh",
            prompt: "印度航空允许婴儿推车免费作为行李吗?",
            expected_fragments: &["Air India", "婴儿推车", "免费", "不能带上客舱"],
        },
    ] {
        let response = solver.solve(case.prompt);
        assert_air_india_stroller_policy(
            &response,
            case.language,
            case.prompt,
            case.expected_fragments,
        );
    }
}

fn assert_air_india_stroller_policy(
    response: &SymbolicAnswer,
    language: &str,
    prompt: &str,
    expected_fragments: &[&str],
) {
    assert_eq!(
        response.intent, "fact_lookup",
        "[{language}] {prompt:?} should route to fact_lookup, got {} -> {}",
        response.intent, response.answer
    );
    assert!(
        response
            .thinking_steps
            .iter()
            .any(|step| step.source_event == "fact_lookup:hit"
                && step.detail == "fact_air_india_infant_stroller_allowance"),
        "[{language}] {prompt:?} should select the Air India stroller allowance fact, got {:?}",
        response.thinking_steps
    );
    for expected in expected_fragments {
        assert!(
            response.answer.contains(expected),
            "[{language}] {prompt:?} answer should contain {expected:?}, got: {}",
            response.answer
        );
    }
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q69906"),
        "[{language}] {prompt:?} should keep the Air India Wikidata anchor, got {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source:")),
        "[{language}] {prompt:?} should record a source event, got {:?}",
        response.evidence_links
    );
    assert!(
        response.links_notation.contains(
            "https://www.airindia.com/in/en/travel-information/baggage-guidelines/checked-baggage-allowance.html"
        ),
        "[{language}] {prompt:?} trace should include the official Air India baggage source, got {}",
        response.links_notation
    );
}
