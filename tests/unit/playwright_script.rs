use formal_ai::{FormalAiEngine, SolverConfig, UniversalSolver};

#[test]
fn russian_playwright_script_prompt_returns_starter_example() {
    // Regression test for issue #135: the reported Russian prompt with the
    // common "Playright" typo was routed to the unknown fallback.
    let response = FormalAiEngine.answer("Можешь написать мне Playright скрипт?");

    assert_eq!(
        response.intent, "playwright_script",
        "answer was: {}",
        response.answer
    );
    assert!(response.answer.contains("Playwright"));
    assert!(response.answer.contains("```typescript"));
    assert!(response.answer.contains("@playwright/test"));
    assert!(response
        .answer
        .contains("https://playwright.dev/docs/writing-tests"));
    assert_ne!(response.intent, "unknown");
}

#[test]
fn correctly_spelled_playwright_prompt_does_not_claim_typo_correction() {
    let response = FormalAiEngine.answer("Can you write a Playwright script?");

    assert_eq!(
        response.intent, "playwright_script",
        "answer was: {}",
        response.answer
    );
    assert!(response.answer.contains("@playwright/test"));
    assert!(!response.answer.contains("`Playright`"));
}

#[test]
fn playwright_script_prompts_route_across_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "Can you write a Playwright script?",
        },
        Case {
            language: "ru",
            prompt: "Можешь написать мне Playright скрипт?",
        },
        Case {
            language: "hi",
            prompt: "क्या तुम Playwright script लिख सकते हो?",
        },
        Case {
            language: "zh",
            prompt: "可以写一个 Playwright script 吗？",
        },
    ];

    for case in cases {
        let response = FormalAiEngine.answer(case.prompt);

        assert_eq!(
            response.intent, "playwright_script",
            "language: {}, answer was: {}",
            case.language, response.answer
        );
        assert!(response.answer.contains("@playwright/test"));
        assert_ne!(response.intent, "unknown");
    }
}

#[test]
fn low_guess_playwright_script_prompt_asks_for_scope() {
    let solver = UniversalSolver::new(SolverConfig {
        guess_probability: 0.1,
        ..SolverConfig::default()
    });
    let response = solver.solve("Можешь написать мне Playright скрипт?");

    assert_eq!(
        response.intent, "playwright_script_clarification",
        "answer was: {}",
        response.answer
    );
    assert!(response.answer.contains("URL"));
    assert!(response.answer.contains("Playwright"));
    assert!(!response.answer.contains("```typescript"));
    assert_ne!(response.intent, "unknown");
}
