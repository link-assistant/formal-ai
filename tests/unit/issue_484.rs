use formal_ai::FormalAiEngine;

// Concept lookup should treat a trailing language directive as the requested
// answer language, not as part of the concept term or context.
#[test]
fn telegram_ads_concept_lookup_respects_explicit_response_language() {
    let cases = [
        (
            "Tell me about Telegram Ads in Russian",
            "Реклама в Telegram",
            "официальная рекламная платформа",
            "concept_lookup:response-language:ru",
        ),
        (
            "Расскажи за Telegram Ads на английском",
            "Telegram Ads",
            "native advertising platform",
            "concept_lookup:response-language:en",
        ),
    ];

    for (prompt, expected_term, expected_summary, expected_evidence) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "concept_lookup",
            "[{prompt}] expected concept_lookup, got intent: {}",
            response.intent
        );
        assert!(
            response.answer.contains(expected_term),
            "[{prompt}] answer should use localized term {expected_term:?}, got: {}",
            response.answer
        );
        assert!(
            response.answer.contains(expected_summary),
            "[{prompt}] answer should use requested language summary, got: {}",
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == expected_evidence),
            "[{prompt}] evidence should include {expected_evidence}, got {:?}",
            response.evidence_links
        );
    }
}
