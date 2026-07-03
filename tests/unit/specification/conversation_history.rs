//! Natural-language access to prior dialog turns (issue #509).

use formal_ai::{
    answer_memory_recall, create_chat_completion, solve_with_history, ChatCompletionRequest,
    ChatMessage, ConversationTurn, MemoryEvent, SymbolicAnswer,
};

fn has_evidence(response: &SymbolicAnswer, expected: &str) -> bool {
    response
        .evidence_links
        .iter()
        .any(|link| link.starts_with(expected))
}

#[test]
fn solve_with_history_searches_english_dialog_history_by_term() {
    let history = [
        ConversationTurn::user("What is Rust?"),
        ConversationTurn::assistant("Rust is a systems programming language."),
        ConversationTurn::user("What is Wikipedia?"),
        ConversationTurn::assistant("Wikipedia is an encyclopedia."),
    ];

    let response = solve_with_history("When did I mention Rust?", &history);

    assert_eq!(response.intent, "conversation_recall");
    assert!(response.answer.contains("Rust"), "{}", response.answer);
    assert!(
        response.answer.contains("user: What is Rust?"),
        "{}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("assistant: Rust is a systems programming language."),
        "{}",
        response.answer
    );
    assert!(
        !response.answer.contains("Wikipedia is an encyclopedia."),
        "{}",
        response.answer
    );
    assert!(has_evidence(&response, "filter:memory_query"));
    assert!(has_evidence(&response, "memory_match"));
}

#[test]
fn solve_with_history_reports_no_dialog_history_matches() {
    let history = [
        ConversationTurn::user("What is Rust?"),
        ConversationTurn::assistant("Rust is a systems programming language."),
    ];

    let response = solve_with_history("When did I ask about Haskell?", &history);

    assert_eq!(response.intent, "conversation_recall");
    assert!(
        response.answer.contains("No mentions of \"haskell\" found"),
        "{}",
        response.answer
    );
    assert!(has_evidence(&response, "filter:memory_matches"));
}

#[test]
fn solve_with_history_searches_dialog_history_in_russian() {
    let history = [
        ConversationTurn::user("Что такое Википедия?"),
        ConversationTurn::assistant("Википедия - свободная энциклопедия."),
        ConversationTurn::user("Что такое Rust?"),
    ];

    let response = solve_with_history("Когда я спрашивал про Википедия?", &history);

    assert_eq!(response.intent, "conversation_recall");
    assert!(response.answer.contains("Википедия"), "{}", response.answer);
    assert!(
        response.answer.contains("user: Что такое Википедия?"),
        "{}",
        response.answer
    );
    assert!(
        !response.answer.contains("Что такое Rust?"),
        "{}",
        response.answer
    );
}

#[test]
fn russian_previous_user_question_skips_assistant_and_meta_recall_turns() {
    let unknown = "Я не уверен, как на это ответить.";
    let first_history = [
        ConversationTurn::user("Поставь мне встречу с мамукой на 10:00"),
        ConversationTurn::assistant(unknown),
    ];

    let first = solve_with_history("Что я спрашивал в прошлом сообщении?", &first_history);

    assert_eq!(first.intent, "recall_last_question");
    assert!(
        first
            .answer
            .contains("Поставь мне встречу с мамукой на 10:00"),
        "{}",
        first.answer
    );
    assert!(!first.answer.contains(unknown), "{}", first.answer);

    let followup_history = [
        ConversationTurn::user("Поставь мне встречу с мамукой на 10:00"),
        ConversationTurn::assistant(unknown),
        ConversationTurn::user("Что я спрашивал в прошлом сообщении?"),
        ConversationTurn::assistant(&first.answer),
    ];

    let followup = solve_with_history("а я что спрашивал?", &followup_history);

    assert_eq!(followup.intent, "recall_last_question");
    assert!(
        followup
            .answer
            .contains("Поставь мне встречу с мамукой на 10:00"),
        "{}",
        followup.answer
    );
    assert!(
        !followup
            .answer
            .contains("Что я спрашивал в прошлом сообщении?"),
        "{}",
        followup.answer
    );
}

#[test]
fn previous_user_question_recall_skips_meta_turns_in_supported_languages() {
    struct RecallCase<'a> {
        language: &'a str,
        request: &'a str,
        unknown: &'a str,
        first_prompt: &'a str,
        followup_prompt: &'a str,
    }

    let cases = [
        RecallCase {
            language: "en",
            request: "Schedule a meeting with Mamuka at 10:00",
            unknown: "I do not know how to answer that.",
            first_prompt: "What did I ask in the previous message?",
            followup_prompt: "What did I ask?",
        },
        RecallCase {
            language: "ru",
            request: "Поставь мне встречу с мамукой на 10:00",
            unknown: "Я не уверен, как на это ответить.",
            first_prompt: "Что я спрашивал в прошлом сообщении?",
            followup_prompt: "а я что спрашивал?",
        },
        RecallCase {
            language: "hi",
            request: "कल 10:00 बजे मामूका के साथ मीटिंग रखो",
            unknown: "मुझे नहीं पता कि इसका उत्तर कैसे दूं.",
            first_prompt: "मैंने पिछले संदेश में क्या पूछा था",
            followup_prompt: "मैंने क्या पूछा था",
        },
        RecallCase {
            language: "zh",
            request: "明天10点安排和妈妈的会议",
            unknown: "我不知道该怎么回答.",
            first_prompt: "我之前问了什么",
            followup_prompt: "我刚才问了什么",
        },
    ];

    for case in cases {
        let first_history = [
            ConversationTurn::user(case.request),
            ConversationTurn::assistant(case.unknown),
        ];

        let first = solve_with_history(case.first_prompt, &first_history);

        assert_eq!(first.intent, "recall_last_question", "{}", case.language);
        assert!(
            first.answer.contains(case.request),
            "{}: {}",
            case.language,
            first.answer
        );
        assert!(
            !first.answer.contains(case.unknown),
            "{}: {}",
            case.language,
            first.answer
        );

        let followup_history = [
            ConversationTurn::user(case.request),
            ConversationTurn::assistant(case.unknown),
            ConversationTurn::user(case.first_prompt),
            ConversationTurn::assistant(&first.answer),
        ];

        let followup = solve_with_history(case.followup_prompt, &followup_history);

        assert_eq!(followup.intent, "recall_last_question", "{}", case.language);
        assert!(
            followup.answer.contains(case.request),
            "{}: {}",
            case.language,
            followup.answer
        );
        assert!(
            !followup.answer.contains(case.first_prompt),
            "{}: {}",
            case.language,
            followup.answer
        );
    }
}

#[test]
fn solve_with_history_searches_dialog_history_in_hindi() {
    let history = [
        ConversationTurn::user("विकिपीडिया क्या है?"),
        ConversationTurn::assistant("विकिपीडिया एक मुक्त ज्ञानकोश है."),
        ConversationTurn::user("Rust क्या है?"),
    ];

    let response = solve_with_history("मेरी बातचीत में खोजो विकिपीडिया", &history);

    assert_eq!(response.intent, "conversation_recall");
    assert!(
        response.answer.contains("विकिपीडिया"),
        "{}",
        response.answer
    );
    assert!(
        response.answer.contains("user: विकिपीडिया क्या है?"),
        "{}",
        response.answer
    );
    assert!(
        !response.answer.contains("Rust क्या है?"),
        "{}",
        response.answer
    );
}

#[test]
fn solve_with_history_searches_dialog_history_in_chinese() {
    let history = [
        ConversationTurn::user("维基百科是什么?"),
        ConversationTurn::assistant("维基百科是一个自由百科全书."),
        ConversationTurn::user("Rust 是什么?"),
    ];

    let response = solve_with_history("我什么时候提到维基百科?", &history);

    assert_eq!(response.intent, "conversation_recall");
    assert!(response.answer.contains("维基百科"), "{}", response.answer);
    assert!(
        response.answer.contains("user: 维基百科是什么?"),
        "{}",
        response.answer
    );
    assert!(
        !response.answer.contains("Rust 是什么?"),
        "{}",
        response.answer
    );
}

#[test]
fn solve_with_history_accepts_other_conversation_query_forms() {
    let history = [
        ConversationTurn::user("What is Rust?"),
        ConversationTurn::assistant("Rust is a systems programming language."),
    ];

    let response = solve_with_history("Find Rust in another conversation", &history);

    assert_eq!(response.intent, "conversation_recall");
    assert!(response.answer.contains("Rust"), "{}", response.answer);
    assert!(has_evidence(&response, "filter:memory_scope"));
}

#[test]
fn chat_completion_supports_natural_language_history_search() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![
            ChatMessage::user("What is Rust?"),
            ChatMessage::assistant("Rust is a systems programming language."),
            ChatMessage::user("What is Wikipedia?"),
            ChatMessage::assistant("Wikipedia is an encyclopedia."),
            ChatMessage::user("When did I mention Rust?"),
        ],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };

    let completion = create_chat_completion(&request);
    let content = completion.choices[0].message.content.plain_text();

    assert!(content.contains("Rust"), "{content}");
    assert!(content.contains("user: What is Rust?"), "{content}");
    assert!(
        !content.contains("Wikipedia is an encyclopedia."),
        "{content}"
    );
}

#[test]
fn natural_language_query_searches_persisted_memory_events() {
    let events = vec![
        memory_event("a1", "user", "conv-a", "Rust Notes", "What is Rust?"),
        memory_event(
            "a2",
            "assistant",
            "conv-a",
            "Rust Notes",
            "Rust is a systems programming language.",
        ),
        memory_event(
            "b1",
            "user",
            "conv-b",
            "Wikipedia Notes",
            "What is Wikipedia?",
        ),
    ];

    let response = answer_memory_recall(
        "Find Rust in another conversation",
        &events,
        Some("current-conversation"),
    )
    .expect("recall query should be recognized");

    assert_eq!(response.intent, "conversation_recall");
    assert!(
        response.answer.contains("Rust Notes"),
        "{}",
        response.answer
    );
    assert!(
        response.answer.contains("user: What is Rust?"),
        "{}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("assistant: Rust is a systems programming language."),
        "{}",
        response.answer
    );
    assert!(
        !response.answer.contains("What is Wikipedia?"),
        "{}",
        response.answer
    );
    assert!(has_evidence(&response, "filter:memory_conversations"));
    assert!(has_evidence(&response, "memory_match"));
}

#[test]
fn natural_language_query_searches_whole_memory_event_fields() {
    let events = vec![
        memory_event("a1", "user", "conv-a", "Rust Notes", "What is Rust?"),
        tool_memory_event(
            "tool-1",
            "conv-tools",
            "Tool Trace",
            "web_search",
            "{\"query\":\"rust memory\"}",
            "Found Rust memory references.",
        ),
    ];

    let response = answer_memory_recall("recall web_search", &events, Some("current-conversation"))
        .expect("recall query should be recognized");

    assert_eq!(response.intent, "conversation_recall");
    assert!(
        response.answer.contains("Tool Trace"),
        "{}",
        response.answer
    );
    assert!(
        response.answer.contains("tool: web_search"),
        "{}",
        response.answer
    );
    assert!(
        response.answer.contains("intent: web_search"),
        "{}",
        response.answer
    );
    assert!(
        !response.answer.contains("What is Rust?"),
        "{}",
        response.answer
    );
}

#[test]
fn natural_language_whole_memory_field_recall_covers_supported_languages() {
    struct RecallCase {
        language: &'static str,
        prompt: &'static str,
    }

    let events = vec![tool_memory_event(
        "tool-1",
        "conv-tools",
        "Tool Trace",
        "web_search",
        "{\"query\":\"rust memory\"}",
        "Found Rust memory references.",
    )];

    for case in [
        RecallCase {
            language: "en",
            prompt: "recall web_search",
        },
        RecallCase {
            language: "ru",
            prompt: "Когда я спрашивал про web_search?",
        },
        RecallCase {
            language: "hi",
            prompt: "मेरी बातचीत में खोजो web_search",
        },
        RecallCase {
            language: "zh",
            prompt: "我什么时候提到web_search?",
        },
    ] {
        let response = answer_memory_recall(case.prompt, &events, None)
            .unwrap_or_else(|| panic!("{} recall query should be recognized", case.language));

        assert_eq!(response.intent, "conversation_recall", "{}", case.language);
        assert!(
            response.answer.contains("tool: web_search"),
            "{} answer: {}",
            case.language,
            response.answer
        );
    }
}

#[test]
fn natural_language_query_searches_memory_link_projection() {
    let events = vec![tool_memory_event(
        "tool-1",
        "conv-tools",
        "Tool Trace",
        "web_search",
        "{\"query\":\"rust memory\"}",
        "Found Rust memory references.",
    )];

    let response = answer_memory_recall("recall field:tool", &events, None)
        .expect("recall query should be recognized");

    assert_eq!(response.intent, "conversation_recall");
    assert!(
        response.answer.contains("Tool Trace"),
        "{}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("link: field:tool -> value:web_search"),
        "{}",
        response.answer
    );
}

fn memory_event(
    id: &str,
    role: &str,
    conversation_id: &str,
    conversation_title: &str,
    content: &str,
) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        kind: Some(String::from("message")),
        role: Some(role.to_owned()),
        content: Some(content.to_owned()),
        conversation_id: Some(conversation_id.to_owned()),
        conversation_title: Some(conversation_title.to_owned()),
        ..MemoryEvent::default()
    }
}

fn tool_memory_event(
    id: &str,
    conversation_id: &str,
    conversation_title: &str,
    tool: &str,
    inputs: &str,
    outputs: &str,
) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        kind: Some(String::from("tool_call")),
        role: Some(String::from("assistant")),
        intent: Some(tool.to_owned()),
        tool: Some(tool.to_owned()),
        inputs: Some(inputs.to_owned()),
        outputs: Some(outputs.to_owned()),
        conversation_id: Some(conversation_id.to_owned()),
        conversation_title: Some(conversation_title.to_owned()),
        evidence: vec![format!("tool:{tool}")],
        ..MemoryEvent::default()
    }
}
