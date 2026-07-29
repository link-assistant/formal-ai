//! Regression coverage for the task-ladder false greens found by issue #842.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::ChatMessage;

fn one_call(prompt: &str) -> PlannedToolCall {
    let plan = plan_chat_step(
        &[ChatMessage::user(prompt)],
        &["bash", "websearch", "request_user_input"],
    )
    .expect("the task should be planned");
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("expected one tool call, got {plan:?}");
    };
    assert_eq!(calls.len(), 1, "{calls:?}");
    calls.into_iter().next().unwrap()
}

#[test]
fn definition_example_requests_route_to_research_in_every_supported_language() {
    for (prompt, subject) in [
        ("Give one example of a flarb", "flarb"),
        (
            "Приведи один пример препарата, который называют фуфломицином",
            "фуфломицин",
        ),
        ("फ्लार्ब का एक उदाहरण दें", "फ्लार्ब"),
        ("举一个弗拉布的例子", "弗拉布"),
    ] {
        let call = one_call(prompt);
        assert_eq!(call.tool, "websearch", "{prompt}: {call:?}");
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("search arguments");
        let query = arguments["query"].as_str().expect("search query");
        assert!(query.contains(subject), "{prompt}: {query}");
    }
}

#[test]
fn isolated_contextual_pronoun_question_does_not_search_the_web() {
    let prompt = "Что означает слово это в предложении: так что это такое то?";
    let plan = plan_chat_step(
        &[ChatMessage::user(prompt)],
        &["bash", "websearch", "request_user_input"],
    )
    .expect("the contextual question should be handled");
    let AgenticPlan::Final(answer) = plan else {
        panic!("an unresolved contextual pronoun should ask for context: {plan:?}");
    };
    assert!(answer.to_lowercase().contains("имеете в виду"), "{answer}");
}
