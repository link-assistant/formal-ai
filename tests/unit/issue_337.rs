use formal_ai::FormalAiEngine;

#[test]
fn github_repository_extraction_prompt_does_not_route_to_configuration_capability() {
    // Regression test for issue #337: the phrase "available tools" plus
    // "programming language" is part of the requested GitHub repository facts,
    // not a question about message-driven configuration.
    let prompt = "Navigate to github.com/link-assistant/formal-ai. Extract information about:\n\
1. The main programming language used\n\
2. Number of stars\n\
3. Last commit date\n\
4. List all available tools mentioned in the README\n\n\
Then format this as a JSON object.";

    let response = FormalAiEngine.answer(prompt);

    assert_ne!(
        response.intent, "capabilities",
        "GitHub repository extraction should not be answered as feature capability: {}",
        response.answer
    );
    assert_ne!(
        response.intent, "unknown",
        "GitHub repository extraction should route as a repository request: {}",
        response.answer
    );
    assert!(
        !response
            .answer
            .to_lowercase()
            .contains("message-driven configuration"),
        "GitHub repository extraction should not mention configuration capability: {}",
        response.answer
    );
}

#[test]
fn direct_feature_capability_questions_still_work_across_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "Is configuration available?",
        },
        Case {
            language: "ru",
            prompt: "доступна ли настройка?",
        },
        Case {
            language: "hi",
            prompt: "क्या settings उपलब्ध हैं?",
        },
        Case {
            language: "zh",
            prompt: "设置可用吗?",
        },
    ];

    for case in cases {
        let response = FormalAiEngine.answer(case.prompt);

        assert_eq!(
            response.intent, "capabilities",
            "{} feature capability prompt should still route to capabilities: {}",
            case.language, response.answer
        );
    }
}
