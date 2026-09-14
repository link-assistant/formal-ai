use formal_ai::FormalAiEngine;

const ENGLISH_TEST_STATUS_PROMPTS: &[&str] = &[
    "Test",
    "test passed",
    "I'm here",
    "I am here",
    "test passed, I'm here",
    "testing 123",
    "are you there?",
];
// "Ты тут?" is the exact liveness probe from issue #979's reported dialog: it
// fell through intent routing and came back as web-search noise.
const RUSSIAN_TEST_STATUS_PROMPTS: &[&str] = &[
    "тест",
    "тест пройден",
    "я здесь",
    "я тут",
    "ты здесь",
    "Ты тут?",
    "вы здесь",
    "вы тут",
];
const HINDI_TEST_STATUS_PROMPTS: &[&str] = &[
    "टेस्ट",
    "परीक्षण",
    "परीक्षण सफल रहा",
    "मैं यहाँ हूँ",
    "क्या आप वहाँ हैं?",
];
const CHINESE_TEST_STATUS_PROMPTS: &[&str] =
    &["测试", "测试通过", "我在这里", "你在吗?", "您在吗?"];
// Spanish liveness probes keep the es locale in the same routed matrix as the
// other registered languages, with and without accents.
const SPANISH_TEST_STATUS_PROMPTS: &[&str] = &[
    "prueba",
    "prueba superada",
    "estoy aquí",
    "estoy aqui",
    "¿Estás ahí?",
    "estas ahi",
    "¿Sigues ahí?",
];

// Issue #149: the health-check prompt "Test" was reported as unknown in the
// browser demo. Keep the short smoke-test phrases and their combinations on a
// dedicated intent so users get an explicit liveness acknowledgement.
#[test]
fn test_status_prompts_are_recognized() {
    for prompt in ENGLISH_TEST_STATUS_PROMPTS {
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

#[test]
fn test_status_matrix_is_classified_across_languages() {
    for prompts in [
        ENGLISH_TEST_STATUS_PROMPTS,
        RUSSIAN_TEST_STATUS_PROMPTS,
        HINDI_TEST_STATUS_PROMPTS,
        CHINESE_TEST_STATUS_PROMPTS,
        SPANISH_TEST_STATUS_PROMPTS,
    ] {
        for prompt in prompts {
            let response = FormalAiEngine.answer(prompt);
            assert_eq!(
                response.intent, "test_status",
                "prompt {prompt:?} should yield test_status, got intent={} answer={}",
                response.intent, response.answer,
            );
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link == "response:test_status"),
                "prompt {prompt:?} response should cite response:test_status",
            );
            assert!(
                !response
                    .answer
                    .contains("cannot answer that from local links rules"),
                "prompt {prompt:?} should not return the unknown fallback: {}",
                response.answer,
            );
        }
    }
}
