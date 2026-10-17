use super::*;
use crate::formalization::page::fact_survives;
use crate::formalization::text_statements::{Word, WordKind, clause_statement, formalize_text};
use crate::summarization::dependency::restates;
#[test]
fn retained_typed_facts_entail_only_matching_later_evidence() {
    let earlier = formalize_text("1969 and 1972", "en").remove(0);
    let later = formalize_text("1972", "en").remove(0);
    assert!(typed_meaning_entails(&earlier.dialogue, &later.dialogue));
    assert!(!typed_meaning_entails(&later.dialogue, &earlier.dialogue));
    assert!(restates(&later, &earlier));
    let neutral = formalize_text("web interface", "en").remove(0);
    let quantified = formalize_text("10 web interface examples", "en").remove(0);
    assert!(restates(&neutral, &quantified));
    let yes = formalize_dialogue("yes", "en");
    let no = formalize_dialogue("no", "en");
    assert!(!typed_meaning_entails(&yes, &no));
    assert!(!typed_meaning_entails(&no, &yes));
    let currency = formalize_dialogue("450000 RUB", "en");
    let other = formalize_dialogue("450000 USD", "en");
    assert!(!typed_meaning_entails(&currency, &other));
}
#[test]
fn declarative_context_does_not_invent_question_or_location_evidence() {
    for text in [
        "The Moon is a body, which reflects light.",
        "The cat is an animal, is a small domesticated mammal.",
    ] {
        assert!(
            formalize_text(text, "en")
                .iter()
                .all(|statement| statement.dialogue.question.is_none())
        );
    }
    assert!(
        formalize_dialogue("Which body reflects light?", "en")
            .question
            .is_some()
    );
    assert!(formalize_dialogue("Из звёзд", "ru").answer.is_none());
    assert!(formalize_dialogue("In botany", "en").answer.is_none());
    assert_eq!(
        formalize_dialogue("In Paris", "en")
            .answer
            .expect("location evidence")
            .locations,
        ["Paris"]
    );
    let previous = Word {
        kind: WordKind::Number,
        id: "number:17".to_owned(),
        surface: "17".to_owned(),
    };
    let inherited = clause_statement("It supplies energy.", "en", Some(&previous), true);
    assert_eq!(inherited.dialogue.quantities[0].value, "17");
    assert_eq!(inherited.dialogue.quantities[0].source_components.len(), 0);
    assert!(fact_survives(&inherited));
}

#[test]
fn logical_numeric_groups_keep_original_source_token_weights_once() {
    let vectors: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/formalization-native-parity.json"
    ))
    .unwrap();
    for vector in vectors["counts"].as_array().unwrap() {
        let statement = formalize_text(
            vector["text"].as_str().unwrap(),
            vector["language"].as_str().unwrap(),
        )
        .remove(0);
        let terms = crate::formalization::text_statements::statement_terms(&statement);
        let repeated = (0..vector["repeat"].as_u64().unwrap())
            .flat_map(|_| terms.iter().copied())
            .collect::<Vec<_>>();
        assert_eq!(
            count_source_terms(&statement, &repeated),
            usize::try_from(vector["expected"].as_u64().unwrap())
                .expect("fixture count fits platform")
        );
    }
}
