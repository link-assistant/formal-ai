//! Keep source query delimiters and referent clarification distinct from factual questions.
use formal_ai::event_log::EventLog;
use formal_ai::question_necessity::{QuestionClass, classify_question, enforce_questions};

#[test]
fn structured_source_query_parameters_remain_observations() {
    for body in [
        "Research trail: phrases=isogram;attempts=compose:failed;consulted_sources=source=stackexchange;status=offline_cache_miss;detail=https://api.stackexchange.com/2.3/search/advanced?order=desc&q=isogram",
        "Research trail: phrases=reverse;attempts=compose:failed;consulted_sources=source=wikipedia;status=offline_cache_miss;detail=url=https://en.wikipedia.org/w/api.php?action=query&search=reverse",
        "Capture [publisher](https://publisher.example/search?q=method).",
    ] {
        let mut log = EventLog::new();
        assert_eq!(enforce_questions(body, &mut log), body);
        assert!(
            log.events()
                .iter()
                .all(|event| !event.kind.starts_with("question_necessity:"))
        );
    }
    let mut log = EventLog::new();
    let body = "source=https://publisher.example/search?q=method\nWhat is the answer?";
    assert_eq!(
        enforce_questions(body, &mut log),
        "source=https://publisher.example/search?q=method"
    );
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "question_necessity:refused")
    );
}

#[test]
fn selecting_an_ambiguous_referent_is_a_requirement_question() {
    for question in [
        "Which `join` method do you mean?",
        "Which interpretation of that term do you mean?",
        "Какой метод `join` вы имеете в виду?",
        "आप किस `join` विधि की बात कर रहे हैं?",
        "你指的是哪个 `join` 方法？",
        "¿A qué método `join` te refieres?",
    ] {
        let mut log = EventLog::new();
        assert_eq!(
            classify_question(question).class,
            QuestionClass::Requirement,
            "{question}"
        );
        assert_eq!(enforce_questions(question, &mut log), question);
        assert!(
            log.events()
                .iter()
                .any(|event| event.kind == "question_necessity:asked")
        );
    }
    assert_eq!(
        classify_question("What is the signature of the join method?").class,
        QuestionClass::Factual
    );
}
