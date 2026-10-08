//! R1188-U19 (extends R526-1): the best translation survives the round trip.
//!
//! Source -> meta -> target -> meta -> source, for words and sentences. The
//! JavaScript twin is `rust/tests/web/round-trip-translation.test.mjs`; both
//! pin the same outputs.

use formal_ai::formalization::statement_rendering::{resolve_surface, surfaces_in};
use formal_ai::formalization::text_statements::formalize_text;
use formal_ai::translation::round_trip::{best_surface, round_trip, translate_text};

#[test]
fn the_first_listed_surface_loses_to_the_one_whose_meaning_comes_back() {
    let statements = formalize_text("fix", "en");
    let subject = statements
        .first()
        .and_then(|statement| statement.subject.clone())
        .expect("a subject");
    let candidates = surfaces_in(&subject.id, "ru");
    assert_eq!(candidates.first().map(String::as_str), Some("добавь"));
    assert_ne!(
        resolve_surface("добавь", "ru").as_deref(),
        Some(subject.id.as_str())
    );
    let best = best_surface(&subject, "en", "ru").expect("a Russian surface");
    assert_ne!(candidates.first(), Some(&best));
    assert!(candidates.contains(&best), "{best}");
    assert_eq!(
        resolve_surface(&best, "ru").as_deref(),
        Some(subject.id.as_str())
    );
}

#[test]
fn a_word_crosses_to_every_language_and_comes_back_as_itself() {
    for (target, surface) in [
        ("ru", "яблоко"),
        ("hi", "सेब"),
        ("zh", "苹果"),
        ("es", "manzana"),
    ] {
        let trip = round_trip("apple", "en", target);
        assert_eq!(
            (trip.forward.as_str(), trip.backward.as_str(), trip.survives),
            (surface, "apple", true),
            "{target}"
        );
    }
}

#[test]
fn a_sentence_crosses_statement_by_statement_and_keeps_its_meanings() {
    let trip = round_trip("La manzana es una fruta.", "es", "en");
    assert_eq!(trip.forward, "apple instance of fruit");
    assert_eq!(trip.backward, "manzana es una fruta");
    assert_eq!(
        (trip.survives, trip.surviving_terms, trip.known_terms),
        (true, 3, 3)
    );
    assert_eq!(
        translate_text("The apple is a fruit.", "en", "zh"),
        "苹果是水果"
    );
}

#[test]
fn an_unknown_word_stays_in_its_source_form_and_the_sentence_does_not_survive() {
    let trip = round_trip("The apple is red.", "en", "ru");
    assert_eq!(trip.forward, "яблоко red");
    assert!(!trip.survives);
    assert_eq!((trip.surviving_terms, trip.known_terms), (1, 1));
}
