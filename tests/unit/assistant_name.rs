use formal_ai::FormalAiEngine;

struct NamePrompt {
    language: &'static str,
    prompt: &'static str,
    /// The exact answer this prompt produces. Issue #960 (R234-2): a test is
    /// documentation, so the expected reply is spelled out here in full rather
    /// than approximated by a substring — a reader learns what the system says
    /// by reading the test, not by running it.
    answer: &'static str,
}

const NAME_PROMPTS: &[NamePrompt] = &[
    NamePrompt {
        language: "en",
        prompt: "What is your name?",
        answer: "I'm formal AI, and currently I don't have a name. But you can name me as you like.",
    },
    NamePrompt {
        language: "ru",
        prompt: "Как твое имя?",
        answer: "Я formal AI, и сейчас у меня нет имени. Но вы можете назвать меня как хотите.",
    },
    NamePrompt {
        language: "ru",
        prompt: "Как тебя зовут?",
        answer: "Я formal AI, и сейчас у меня нет имени. Но вы можете назвать меня как хотите.",
    },
    NamePrompt {
        language: "hi",
        prompt: "आपका नाम क्या है?",
        answer: "मैं formal AI हूँ, और अभी मेरा कोई नाम नहीं है। लेकिन आप मुझे अपनी पसंद का नाम दे सकते हैं।",
    },
    NamePrompt {
        language: "zh",
        prompt: "你叫什么名字?",
        answer: "我是 formal AI,目前还没有名字。不过您可以按自己的喜好给我起名。",
    },
];

#[test]
fn reported_russian_name_question_is_answered() {
    let response = FormalAiEngine.answer("Как твое имя?");

    assert_eq!(response.intent, "assistant_name");
    // R234-2: the exact answer, verbatim, before the looser guards below.
    assert_eq!(
        response.answer,
        "Я formal AI, и сейчас у меня нет имени. Но вы можете назвать меня как хотите."
    );
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
            response.answer, case.answer,
            "{} prompt {:?} should answer verbatim",
            case.language, case.prompt,
        );
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
