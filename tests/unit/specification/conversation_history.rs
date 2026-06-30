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
