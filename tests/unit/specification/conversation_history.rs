//! Natural-language access to prior dialog turns (issue #509).

use formal_ai::{
    create_chat_completion, solve_with_history, ChatCompletionRequest, ChatMessage,
    ConversationTurn, SymbolicAnswer,
};

fn has_evidence(response: &SymbolicAnswer, expected: &str) -> bool {
    response
        .evidence_links
        .iter()
        .any(|link| link.starts_with(expected))
}

#[test]
fn solve_with_history_searches_dialog_history_by_term() {
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
