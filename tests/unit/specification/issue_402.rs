use formal_ai::FormalAiEngine;

// Issue #402: a Russian personal small-talk prompt used to fall through to
// unknown. It needs a concrete, localized answer family instead of the
// missing-rule guide.
#[test]
fn russian_free_time_prompt_returns_assistant_free_time_answer() {
    let response = FormalAiEngine.answer("Что делаешь в свободное время?");

    assert_eq!(response.intent, "assistant_free_time");
    assert!(
        response.answer.contains("свободного времени"),
        "expected localized free-time answer, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "response:assistant_free_time"),
        "response should cite response:assistant_free_time, got {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:ru"),
        "response should keep Russian language evidence, got {:?}",
        response.evidence_links
    );
}
