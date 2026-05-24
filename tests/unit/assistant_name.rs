use formal_ai::FormalAiEngine;

struct NamePrompt {
    language: &'static str,
    prompt: &'static str,
}

const NAME_PROMPTS: &[NamePrompt] = &[
    NamePrompt {
        language: "en",
        prompt: "What is your name?",
    },
    NamePrompt {
        language: "ru",
        prompt: "Как твое имя?",
    },
    NamePrompt {
        language: "ru",
        prompt: "Как тебя зовут?",
    },
    NamePrompt {
        language: "hi",
        prompt: "आपका नाम क्या है?",
    },
    NamePrompt {
        language: "zh",
        prompt: "你叫什么名字?",
    },
];

#[test]
fn reported_russian_name_question_is_answered() {
    let response = FormalAiEngine.answer("Как твое имя?");

    assert_eq!(response.intent, "assistant_name");
    assert!(
        response.answer.contains("formal AI") || response.answer.contains("formal-ai"),
        "name answer should mention formal AI, got: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("имени") || response.answer.contains("звать"),
        "Russian name answer should explain the current name state, got: {}",
        response.answer,
    );
}

#[test]
fn assistant_name_questions_are_supported_across_languages() {
    for case in NAME_PROMPTS {
        let response = FormalAiEngine.answer(case.prompt);
        assert_eq!(
            response.intent, "assistant_name",
            "{} prompt {:?} should resolve as assistant_name, got {} -> {}",
            case.language, case.prompt, response.intent, response.answer,
        );
        assert_ne!(
            response.intent, "unknown",
            "{} prompt {:?} must not fall through to unknown",
            case.language, case.prompt,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:assistant_name"),
            "{} prompt {:?} should cite response:assistant_name, got {:?}",
            case.language,
            case.prompt,
            response.evidence_links,
        );
    }
}
