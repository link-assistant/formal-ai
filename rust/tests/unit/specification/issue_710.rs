//! Conversational regressions re-verified by issue #710.
//!
//! These are deliberately end-to-end specification tests. Each prompt enters
//! through `UniversalSolver`, so passing requires routing, decomposition,
//! dialog memory, localization, and final answer projection to agree.

use std::collections::BTreeSet;

use formal_ai::{ConversationTurn, UniversalSolver};

const ENGLISH_COMPOUND_ANSWER: &str = concat!(
    "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.",
    "\n\n",
    "I am formal-ai, a deterministic symbolic AI. Here is what I can do:\n\n- **Greetings**: respond to «Hi», «Hello», and similar.\n- **Hello World**: generate programs in Rust, Python, JavaScript, Go, C, and more.\n- **Web search**: search the internet through DuckDuckGo, Wikipedia, and Wikidata when available.\n- **Concept lookup**: explain terms — try «What is Wikipedia?»\n- **Arithmetic**: evaluate expressions — try «What is 2 + 2?»\n- **Translation**: translate phrases between languages.\n- **Memory**: recall context within the current session.\n- **Behavior rules**: send `List behavior rules` to see the built-in routing rules, and `Show behavior rule unknown` to read one in Links Notation.\n- **Teach this dialog**: send «When I say `your prompt`, answer `your answer`» to add a dialog-local rule for the current conversation.\n- **Self facts**: send `List all facts you know about yourself` to see what I know about myself.\n- **Report a missing rule**: use the top-bar **Report issue** button; unknown-prompt message links include the diagnostic trace for maintainers.\n- **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export from messages.\n\nI run on local symbolic rules, without any neural network inference.",
    "\n\n2 + 2 = 4",
);

const DOCUMENTED_FREE_TIME_ANSWERS: &[&str] = &[
    "I do not have free time the way a person does. Between prompts I am idle; when the dialog is active, I help with tasks, rules, and explanations.",
    "When nobody is using me, I am not doing anything in the background. When you write, I spend the moment helping with the current task.",
    "My quiet time is basically waiting state. In active dialog I work on reasoning, local rules, code, and useful explanations.",
    "I do not have hobbies, but in conversation I can help analyze ideas, write programs, check rules, and organize information.",
    "У меня нет свободного времени в человеческом смысле. Между запросами я бездействую; когда диалог активен, помогаю с задачами, правилами и объяснениями.",
    "Когда меня не спрашивают, я ничего не делаю в фоне. Когда вы пишете, я разбираю текущую задачу и отвечаю по правилам.",
    "Мое свободное от запросов время - это состояние ожидания. В активном диалоге я помогаю рассуждать, писать код и упорядочивать знания.",
    "Хобби у меня нет, но в разговоре я могу помогать с идеями, программами, правилами и объяснениями.",
    "मेरे पास इंसानों जैसा खाली समय नहीं होता। संदेशों के बीच मैं निष्क्रिय रहता हूँ; संवाद सक्रिय हो तो मैं काम, नियम और व्याख्याओं में मदद करता हूँ।",
    "जब कोई मुझसे बात नहीं कर रहा होता, मैं पृष्ठभूमि में कुछ नहीं करता। संदेश आने पर मैं उसी काम पर ध्यान देता हूँ।",
    "मेरा खाली समय मूलतः प्रतीक्षा की स्थिति है। बातचीत में मैं तर्क, कोड, नियम और जानकारी व्यवस्थित करने में मदद करता हूँ।",
    "我没有人类意义上的空闲时间。两次提问之间我处于空闲状态；对话开始后,我会帮助处理任务、规则和解释。",
    "没有人使用我时,我不会在后台做别的事。你发来消息后,我就专注处理当前任务。",
    "我的空闲时间基本就是等待状态。在对话中,我可以帮助推理、写代码、检查规则和整理信息。",
];

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
        if language == "en" {
            assert_eq!(response.answer, ENGLISH_COMPOUND_ANSWER);
        }
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
            assert!(
                DOCUMENTED_FREE_TIME_ANSWERS.contains(&first.answer.as_str()),
                "{language} answer for {prompt:?} must be one of the documented public variants: {}",
                first.answer
            );
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
