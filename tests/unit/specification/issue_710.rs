//! Conversational regressions re-verified by issue #710.
//!
//! These are deliberately end-to-end specification tests. Each prompt enters
//! through `UniversalSolver`, so passing requires routing, decomposition,
//! dialog memory, localization, and final answer projection to agree.

use std::collections::BTreeSet;

use formal_ai::{ConversationTurn, UniversalSolver};

#[test]
fn localized_response_additions_cover_registered_languages() {
    let registered_languages = ["en", "ru", "hi", "zh", "es"];

    for language in registered_languages {
        for intent in [
            "set_assistant_name",
            "assistant_name_recall",
            "ambiguous_modification_clarification",
        ] {
            assert!(
                formal_ai::seed::response_for(intent, language).is_some(),
                "{intent} should be localized for {language}"
            );
        }
        assert!(
            formal_ai::seed::response_variant_for("assistant_free_time", language, "probe")
                .is_some(),
            "assistant_free_time should be localized for {language}"
        );
    }
}

#[test]
fn independent_questions_are_answered_in_source_order_in_every_language() {
    let cases = [
        ("en", "Who are you? What can you do? What is 2 + 2?"),
        ("ru", "Кто ты? Что ты умеешь? Сколько будет 2 + 2?"),
        ("hi", "तुम कौन हो? आप क्या कर सकते हैं? 2 + 2 कितना है?"),
        ("zh", "你是谁？你能做什么？2 + 2 等于多少？"),
    ];
    let solver = UniversalSolver::default();

    for (language, prompt) in cases {
        let response = solver.solve(prompt);
        assert_eq!(
            response.intent, "compound_response",
            "{language} should compose independently solved questions, got {}: {}",
            response.intent, response.answer
        );
        assert!(
            response.answer.trim_end().ends_with('4'),
            "{language} arithmetic answer should remain the third result: {}",
            response.answer
        );
        assert_eq!(
            response
                .evidence_links
                .iter()
                .filter(|link| link.starts_with("sub_impulse:"))
                .count(),
            3,
            "{language} should expose all decomposed questions: {:?}",
            response.evidence_links
        );
    }
}

#[test]
fn assistant_name_can_be_set_and_recalled_in_every_language() {
    let cases = [
        ("en", "Now your name is Ada.", "What is your name?", "Ada"),
        (
            "ru",
            "Теперь тебя зовут Инеффа.",
            "Как тебя зовут?",
            "Инеффа",
        ),
        ("hi", "अब तुम्हारा नाम इनेफ़ा है।", "तुम्हारा नाम क्या है?", "इनेफ़ा"),
        ("zh", "现在你叫伊内法。", "你叫什么名字？", "伊内法"),
    ];
    let solver = UniversalSolver::default();

    for (language, assignment, question, expected_name) in cases {
        let acknowledgement = solver.solve(assignment);
        assert_eq!(
            acknowledgement.intent, "set_assistant_name",
            "{language} should recognize assistant renaming, got {}: {}",
            acknowledgement.intent, acknowledgement.answer
        );
        assert!(acknowledgement.answer.contains(expected_name));

        let history = [
            ConversationTurn::user(assignment),
            ConversationTurn::assistant(acknowledgement.answer),
        ];
        let recall = solver.solve_with_history(question, &history);
        assert_eq!(
            recall.intent, "assistant_name",
            "{language} should recall the assigned name, got {}: {}",
            recall.intent, recall.answer
        );
        assert!(
            recall.answer.contains(expected_name),
            "{language} recall should contain {expected_name:?}: {}",
            recall.answer
        );
    }
}

#[test]
fn ambiguous_modifications_ask_exactly_one_question_in_every_language() {
    let cases = [
        ("en", "Reverse it."),
        ("ru", "Измени это."),
        ("hi", "इसे बदलो।"),
        ("zh", "修改它。"),
    ];
    let solver = UniversalSolver::default();

    for (language, prompt) in cases {
        let response = solver.solve(prompt);
        let question_count = response
            .answer
            .chars()
            .filter(|character| matches!(character, '?' | '？'))
            .count();

        assert_eq!(
            response.intent, "ambiguous_modification_clarification",
            "{language} should clarify a target-less modification, got {}: {}",
            response.intent, response.answer
        );
        assert_eq!(
            question_count, 1,
            "{language} should ask exactly one clarifying question: {}",
            response.answer
        );
    }
}

#[test]
fn free_time_answers_are_prompt_stable_but_not_one_canned_reply() {
    let cases = [
        (
            "en",
            [
                "What do you do in your free time?",
                "How do you spend your free time?",
                "What do you do when you are not working?",
            ],
        ),
        (
            "ru",
            [
                "Что делаешь в свободное время?",
                "Чем занимаешься в свободное время?",
                "Что делаешь когда свободен?",
            ],
        ),
        (
            "hi",
            [
                "खाली समय में क्या करते हो?",
                "आप खाली समय में क्या करते हैं?",
                "फुर्सत में क्या करते हो?",
            ],
        ),
        (
            "zh",
            [
                "你空闲时间做什么?",
                "你有空的时候做什么?",
                "你业余时间做什么?",
            ],
        ),
    ];
    let solver = UniversalSolver::default();

    for (language, prompts) in cases {
        let mut distinct = BTreeSet::new();
        for prompt in prompts {
            let first = solver.solve(prompt);
            let replay = solver.solve(prompt);
            assert_eq!(first.intent, "assistant_free_time", "{language}: {prompt}");
            assert_eq!(
                first.answer, replay.answer,
                "{language} variation should be deterministic for {prompt:?}"
            );
            distinct.insert(first.answer);
        }
        assert!(
            distinct.len() >= 2,
            "{language} should expose multiple deterministic variants, got {distinct:?}"
        );
    }
}
