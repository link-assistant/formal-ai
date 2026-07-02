//! Issue #538 — "Make our meanings and words more detailed".
//!
//! The tomato meaning must expose, *from the seed data*, whether each of its
//! surfaces is the singular or the plural way to name the concept, what part of
//! speech it is, and — in the reverse direction — which meaning every surface
//! denotes. Before this issue помидор carried a plural (помидоры) while its
//! synonym томат did not, and no surface recorded its grammatical number at all.

use formal_ai::seed::{lexicon, Meaning, WordForm};

/// The tomato surface whose spelling is exactly `text`, in any language.
fn tomato_form<'a>(meaning: &'a Meaning, text: &str) -> &'a WordForm {
    meaning
        .word_forms()
        .find(|form| form.text == text)
        .unwrap_or_else(|| panic!("tomato meaning must lexicalise the surface `{text}`"))
}

#[test]
fn tomato_surfaces_pin_their_grammatical_number() {
    let lex = lexicon();
    let tomato = lex.meaning("tomato").expect("tomato meaning must exist");

    // (surface, expected grammatical number). Both the Russian помидор/помидоры
    // pair and its synonym томат/томаты pair now carry singular *and* plural,
    // closing the asymmetry the issue reported, and English gains its plural too.
    let cases = [
        ("tomato", "singular"),
        ("tomatoes", "plural"),
        ("помидор", "singular"),
        ("помидоры", "plural"),
        ("томат", "singular"),
        ("томаты", "plural"),
    ];

    for (surface, number) in cases {
        let form = tomato_form(tomato, surface);
        assert_eq!(
            form.grammatical_number(),
            Some(number),
            "`{surface}` must be tagged as the {number} form of tomato"
        );
    }
}

#[test]
fn tomato_surfaces_expose_part_of_speech_from_data() {
    let lex = lexicon();
    let tomato = lex.meaning("tomato").expect("tomato meaning must exist");

    for surface in [
        "tomato",
        "tomatoes",
        "помидор",
        "помидоры",
        "томат",
        "томаты",
        "टमाटर",
        "番茄",
    ] {
        let form = tomato_form(tomato, surface);
        assert_eq!(
            form.part_of_speech(),
            Some("noun"),
            "`{surface}` must record its part of speech (noun) in the seed"
        );
    }
}

#[test]
fn every_tomato_surface_denotes_the_tomato_meaning() {
    // The reverse ("direct") dictionary direction: a word points back at the
    // meaning it expresses, so the reference is bidirectional.
    let lex = lexicon();
    let tomato = lex.meaning("tomato").expect("tomato meaning must exist");

    for surface in [
        "tomato",
        "tomatoes",
        "помидор",
        "помидоры",
        "томат",
        "томаты",
    ] {
        let form = tomato_form(tomato, surface);
        assert!(
            form.denotations().any(|meaning| meaning == "tomato"),
            "`{surface}` must denote the tomato meaning (bidirectional reference)"
        );
    }
}

#[test]
fn tomato_singular_and_plural_are_distinct_forms_in_each_language() {
    // Regression guard for the reported gap: the plural must exist *and* differ
    // from the singular for both Russian synonyms, not only for помидор.
    let lex = lexicon();
    let tomato = lex.meaning("tomato").expect("tomato meaning must exist");

    let pairs = [
        ("tomato", "tomatoes"),
        ("помидор", "помидоры"),
        ("томат", "томаты"),
    ];

    for (singular, plural) in pairs {
        let singular_form = tomato_form(tomato, singular);
        let plural_form = tomato_form(tomato, plural);
        assert_eq!(singular_form.grammatical_number(), Some("singular"));
        assert_eq!(plural_form.grammatical_number(), Some("plural"));
        assert_ne!(
            singular_form.text, plural_form.text,
            "the {singular}/{plural} pair must be two distinct surfaces"
        );
    }
}

#[test]
fn grammatical_number_meanings_are_grounded_and_multilingual() {
    // The `singular`/`plural` values a surface points at are themselves seed
    // meanings, grounded in Wikidata and lexicalised in every supported
    // language, so the grammatical detail is understood, not an opaque tag.
    let lex = lexicon();
    for (slug, wikidata) in [
        ("grammatical_number", "Q104083"),
        ("singular", "Q110786"),
        ("plural", "Q146786"),
    ] {
        let meaning = lex
            .meaning(slug)
            .unwrap_or_else(|| panic!("{slug} meaning must exist"));
        assert_eq!(
            meaning.wikidata, wikidata,
            "{slug} must be grounded in {wikidata}"
        );
        for language in ["en", "ru", "hi", "zh"] {
            assert!(
                meaning.word_in(language).is_some(),
                "{slug} must be lexicalised in {language}"
            );
        }
    }
}
