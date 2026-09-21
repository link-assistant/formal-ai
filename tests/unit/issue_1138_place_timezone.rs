//! Plan 10 leaf 16 (issue #869): a place mention anchors a scheduled event to
//! its IANA zone through the entity registry, never a hand-typed alias table.
//!
//! The registry's `timezone` fields are grounded (Wikidata country chain plus
//! the tz database's country-to-zone table), so the same matcher resolves an
//! exact name (`hora de Berlín`), a CJK compound (`柏林时间` contains `柏林`)
//! and an inflected form (`по Грузии`) without a seed row per form. A create
//! request that names no place keeps the default zone and says so in its
//! `calendar:timezone_origin` evidence.

use formal_ai::FormalAiEngine;

fn answer(prompt: &str) -> formal_ai::SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn a_berlin_mention_anchors_the_event_in_every_supported_language() {
    for prompt in [
        "Pencil in a review with Dmitri on Tuesday at 09:30, Berlin time.",
        "Поставь разбор с Дмитрием во вторник на 09:30 по Берлину.",
        "मंगलवार 09:30 बर्लिन समय दिमित्री के साथ समीक्षा रख दीजिए।",
        "周二 09:30 柏林时间，和德米特里安排一次评审。",
        "Agenda una revisión con Dmitri el martes a las 09:30, hora de Berlín.",
    ] {
        let response = answer(prompt);
        assert_eq!(response.intent, "calendar_create_event", "{prompt}");
        assert!(
            response.answer.contains("TZID=Europe/Berlin"),
            "{prompt} must anchor in Europe/Berlin; got: {}",
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("calendar:timezone_origin")),
            "{prompt} must record where its zone came from; links: {:?}",
            response.evidence_links
        );
    }
}

#[test]
fn a_tbilisi_mention_resolves_through_the_city_entity() {
    // "по тбилиси" used to live in the retired alias role; the city entity
    // carries the same zone from its own grounding now.
    let response = answer("Поставь встречу с Леваном на 17:00 по тбилиси");
    assert_eq!(response.intent, "calendar_create_event");
    assert!(
        response.answer.contains("TZID=Asia/Tbilisi"),
        "the tbilisi entity grounds Asia/Tbilisi; got: {}",
        response.answer
    );
}

#[test]
fn a_georgia_mention_resolves_through_the_country_entity() {
    // The inflected "по Грузии" is one edit from the grounded surface
    // "Грузия", inside the same budget name correction uses.
    let response = answer("Поставь мне встречу с Леваном на 5 часов по Грузии");
    assert_eq!(response.intent, "calendar_create_event");
    assert!(
        response.answer.contains("TZID=Asia/Tbilisi"),
        "the georgia entity grounds Asia/Tbilisi; got: {}",
        response.answer
    );
}

#[test]
fn a_create_request_without_a_place_says_it_used_the_default_zone() {
    let response = answer("Создай встречу на 10 часов с Марией");
    assert_eq!(response.intent, "calendar_create_event");
    assert!(
        response.answer.contains("UTC"),
        "no place named means the default zone; got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("calendar:timezone_origin")),
        "the default zone must be recorded as an origin, not passed silently; links: {:?}",
        response.evidence_links
    );
}
