use super::*;

#[test]
fn property_questions_do_not_supply_assistant_identity() {
    for (text, language, topic) in [
        ("Where do you reside?", "en", "location"),
        ("Какой у вас стаж с GraphQL?", "ru", "experience"),
        (
            "What compensation do you expect in EUR?",
            "en",
            "compensation",
        ),
    ] {
        let actual = formalize_dialogue(text, language);
        assert_eq!(
            actual
                .question
                .as_ref()
                .and_then(|question| question.topic.as_deref()),
            Some(topic)
        );
        assert!(blocks_self_description(text, language));
    }
    assert!(!blocks_self_description("Tell me about yourself", "en"));
    assert!(!blocks_self_description("Can you run JavaScript?", "en"));
}

#[test]
fn polar_answers_and_currency_mismatch_preserve_obligations() {
    let where_question = formalize_dialogue("Где вы живёте?", "ru");
    let denial = formalize_dialogue("Нет.", "ru");
    assert_eq!(
        denial
            .answer
            .as_ref()
            .and_then(|answer| answer.polarity.as_deref()),
        Some("denied")
    );
    assert_eq!(
        answer_compatibility(where_question.question.as_ref(), denial.answer.as_ref()).status,
        "incompatible"
    );
    let salary = formalize_dialogue("What salary in USD?", "en");
    let amount = formalize_dialogue("От 987 654 ₽ в месяц на руки.", "ru");
    assert_eq!(amount.quantities[0].value, "987654");
    assert_eq!(amount.quantities[0].comparison, "at-least");
    assert_eq!(amount.quantities[0].currency.as_deref(), Some("RUB"));
    assert_eq!(
        answer_compatibility(salary.question.as_ref(), amount.answer.as_ref()).status,
        "incompatible"
    );
}

#[test]
fn numeric_evidence_never_uses_binary_float_or_a_task_answer() {
    let amount = dialogue_quantities("At least €9,007,199,254,740,993 net per month", "en");
    assert_eq!(amount[0].value, "9007199254740993");
    assert_eq!(amount[0].currency.as_deref(), Some("EUR"));
    assert_eq!(amount[0].period.as_deref(), Some("month"));
    assert!(amount[0].net);
}
