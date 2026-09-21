use super::*;

#[test]
fn renders_the_requested_language_and_substitutes_fields() {
    let rendered = thinking_prose(
        "thinking_step_detect_language",
        "ru",
        &[("language", "русский")],
    );
    assert_eq!(
        rendered.as_deref(),
        Some("Определить язык запроса: русский.")
    );
}

#[test]
fn falls_back_to_english_for_an_unregistered_language() {
    let rendered = thinking_prose("thinking_step_memory", "qq", &[]);
    assert_eq!(rendered.as_deref(), Some("Update the local memory bundle."));
}

#[test]
fn reports_a_missing_intent_rather_than_inventing_prose() {
    assert!(thinking_prose("thinking_step_not_a_real_step", "en", &[]).is_none());
}

#[test]
fn names_languages_in_the_answer_language() {
    assert_eq!(language_label("ru", "en"), "английский");
    assert_eq!(language_label("en", "ru"), "Russian");
    assert_eq!(language_label("es", "zh"), "chino");
    assert_eq!(language_label("en", ""), "an unrecognized language");
}
