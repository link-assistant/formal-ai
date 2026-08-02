use super::*;

#[test]
fn relation_prompt_extracts_subject_predicate_object() {
    let candidate = formalize_prompt("apple is a fruit", "en");
    assert_eq!(
        candidate
            .slot(FormalizationRole::Subject)
            .unwrap()
            .anchor
            .id,
        "wikidata:Q89"
    );
    assert_eq!(
        candidate
            .slot(FormalizationRole::Predicate)
            .unwrap()
            .anchor
            .id,
        "wikidata:P31"
    );
    assert_eq!(
        candidate.slot(FormalizationRole::Object).unwrap().anchor.id,
        "wikidata:Q3314483"
    );
}

#[test]
fn russian_translation_prompt_uses_multilingual_label_table() {
    let candidate = formalize_prompt("переведи яблоко на английский", "ru");
    assert_eq!(
        candidate
            .slot(FormalizationRole::Predicate)
            .unwrap()
            .anchor
            .id,
        "wikidata:P5972"
    );
    assert_eq!(
        candidate.slot(FormalizationRole::Object).unwrap().anchor.id,
        "wikidata:Q89"
    );
}

#[test]
fn source_first_translation_formalizes_only_the_source_proposition() {
    assert_eq!(
        parse_translation_object(
            "любая формальная система либо неполна, либо противоречива - translate to english",
        )
        .as_deref(),
        Some("любая формальная система либо неполна, либо противоречива"),
    );
}
