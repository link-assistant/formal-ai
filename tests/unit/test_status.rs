use formal_ai::FormalAiEngine;

// Issue #149: the health-check prompt "Test" was reported as unknown in the
// browser demo. Keep the short smoke-test phrases and their combinations on a
// dedicated intent so users get an explicit liveness acknowledgement.
#[test]
fn test_status_prompts_are_recognized() {
    let cases = [
        "Test",
        "test passed",
        "I'm here",
        "I am here",
        "test passed, I'm here",
        "testing 123",
        "are you there?",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "test_status",
            "prompt {:?} should be recognized as test_status, got intent {:?} and answer {:?}",
            prompt, response.intent, response.answer,
        );
        assert!(
            response.answer.contains("Test passed") && response.answer.contains("I'm here"),
            "prompt {prompt:?} should get an explicit liveness answer, got: {}",
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:test_status"),
            "prompt {prompt:?} response should cite response:test_status",
        );
    }
}
