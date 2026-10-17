use super::dialogue::{Dialogue, formalize_dialogue};
use super::text_statements::{Statement, read_words};
use unicode_normalization::UnicodeNormalization;
/// Canonical semantic evidence excludes source spelling and token custody.
#[must_use]
pub fn typed_meaning(dialogue: &Dialogue) -> Option<String> {
    use serde_json::json;
    if dialogue.question.is_none() && dialogue.answer.is_none() && dialogue.quantities.is_empty() {
        return None;
    }
    let sorted = |values: &[String]| {
        values
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    };
    let question = dialogue.question.as_ref().map(|question| {
        json!([
            ["kind", question.kind],
            ["topic", question.topic],
            ["topics", sorted(&question.topics)],
            ["subject", question.subject],
            ["answer_type", question.answer_type],
            ["currencies", sorted(&question.currencies)],
            ["unit", question.unit],
        ])
    });
    let answer = dialogue.answer.as_ref().map(|answer| {
        json!([
            ["kind", answer.kind],
            ["polarity", answer.polarity],
            [
                "locations",
                answer
                    .locations
                    .iter()
                    .map(|value| value
                        .nfkc()
                        .flat_map(char::to_lowercase)
                        .collect::<String>())
                    .collect::<Vec<_>>()
            ],
        ])
    });
    let quantities = dialogue
        .quantities
        .iter()
        .map(|quantity| {
            json!([
                ["value", quantity.value],
                ["comparison", quantity.comparison],
                ["currency", quantity.currency],
                ["unit", quantity.unit],
                ["period", quantity.period],
                ["net", quantity.net],
                ["uncertainty", quantity.uncertainty],
            ])
        })
        .collect::<Vec<_>>();
    Some(
        json!([
            ["schema", "dialogue-meaning/v1"],
            ["question", question],
            ["answer", answer],
            ["quantities", quantities],
        ])
        .to_string(),
    )
}

/// Full semantic identity includes typed evidence and legacy term structure.
#[must_use]
pub fn statement_identity(statement: &Statement) -> String {
    let subject = statement
        .subject
        .as_ref()
        .map_or("", |subject| subject.id.as_str());
    let others: std::collections::BTreeSet<&str> = statement
        .terms
        .iter()
        .map(|term| term.id.as_str())
        .filter(|id| *id != subject)
        .collect();
    let legacy = format!(
        "{}|{subject}|{}",
        statement.polarity.slug(),
        others.into_iter().collect::<Vec<_>>().join(" ")
    );
    typed_meaning(&statement.dialogue)
        .map_or_else(|| legacy.clone(), |typed| format!("{legacy}|{typed}"))
}
/// Source-faithful rendering when the structural view would lose typed meaning.
#[must_use]
pub fn preserve_typed_rendering(
    statement: &Statement,
    language: &str,
    rendered: String,
    subject_surface: &str,
) -> String {
    if language != statement.language
        || typed_meaning(&statement.dialogue)
            == typed_meaning(&formalize_dialogue(&rendered, language))
    {
        return rendered;
    }
    let mut original = statement.text.clone();
    if statement.dialogue.question.is_some()
        && formalize_dialogue(&original, language).question.is_none()
    {
        original.push('?');
    }
    let first = read_words(&original, language, false)
        .into_iter()
        .find(|word| word.kind.is_term());
    if statement
        .subject
        .as_ref()
        .is_some_and(|subject| first.as_ref().is_none_or(|word| word.id != subject.id))
    {
        original = format!("{subject_surface} {original}");
    }
    original
}

/// Earlier retained facts must satisfy every later typed obligation without conversions.
#[must_use]
pub fn typed_meaning_entails(earlier: &Dialogue, later: &Dialogue) -> bool {
    use serde_json::Value;
    fn facts(dialogue: &Dialogue) -> std::collections::BTreeMap<String, Value> {
        typed_meaning(dialogue)
            .and_then(|text| serde_json::from_str::<Vec<(String, Value)>>(&text).ok())
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
    fn fields(value: Option<&Value>) -> std::collections::BTreeMap<String, Value> {
        value
            .cloned()
            .and_then(|value| serde_json::from_value::<Vec<(String, Value)>>(value).ok())
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
    let held = facts(earlier);
    let needed = facts(later);
    if held.get("question").unwrap_or(&Value::Null)
        != needed.get("question").unwrap_or(&Value::Null)
    {
        return false;
    }
    let answer = fields(needed.get("answer"));
    let previous = fields(held.get("answer"));
    if answer.get("kind").and_then(Value::as_str) == Some("boolean") && answer != previous {
        return false;
    }
    if answer.get("kind").and_then(Value::as_str) == Some("location") {
        if previous.get("kind").and_then(Value::as_str) != Some("location") {
            return false;
        }
        if answer
            .get("locations")
            .and_then(Value::as_array)
            .is_some_and(|values| {
                values.iter().any(|value| {
                    !previous
                        .get("locations")
                        .and_then(Value::as_array)
                        .is_some_and(|held| held.contains(value))
                })
            })
        {
            return false;
        }
    }
    needed
        .get("quantities")
        .and_then(Value::as_array)
        .is_none_or(|values| {
            values.iter().all(|value| {
                held.get("quantities")
                    .and_then(Value::as_array)
                    .is_some_and(|held| held.contains(value))
            })
        })
}

/// Logical evidence inherited from a numeric subject has no local source-token spans.
#[must_use]
pub fn inherit_numeric_evidence(
    mut dialogue: Dialogue,
    subject: Option<&super::text_statements::Word>,
    language: &str,
) -> Dialogue {
    let Some(subject) =
        subject.filter(|word| word.kind == super::text_statements::WordKind::Number)
    else {
        return dialogue;
    };
    let Some(mut quantity) = super::dialogue::dialogue_quantities(&subject.surface, language)
        .into_iter()
        .next()
    else {
        return dialogue;
    };
    if dialogue
        .quantities
        .iter()
        .any(|value| value.value == quantity.value)
    {
        return dialogue;
    }
    quantity.source_components.clear();
    dialogue.quantities.insert(0, quantity);
    if dialogue.answer.is_none() && dialogue.question.is_none() {
        dialogue.answer = Some(super::dialogue::Answer {
            kind: "quantity".to_owned(),
            polarity: None,
            quantities: dialogue.quantities.clone(),
            locations: Vec::new(),
        });
    }
    dialogue
}

#[cfg(test)]
#[path = "../../tests/source/source_tests/formalization/typed_meaning-semantic_entailment_tests.rs"]
mod semantic_entailment_tests;
