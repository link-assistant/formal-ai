//! Multilingual prompts reported after the surface shipped.
//!
//! The tests above this module pin the multilingual surface as it was
//! specified: greetings, identity, concept lookup and (concept, context)
//! disambiguation in every registered language. The ones here arrived the other
//! way round — each is a prompt someone typed at the shipped system and
//! reported, from issue #21's unreadable percent-encoded Wikipedia link to
//! issue #41's circular Russian joke. They live in their own file for the same
//! reason `check-file-size.rs` asks of any Rust file that nears a thousand
//! lines: two people editing two reports should not be editing one file.

use super::{answer, humanize_url};

// ---------------------------------------------------------------------------
// Issue #21: URLs with non-ASCII characters must be displayed in human-readable
// IRI form across every surface, while remaining functional (the encoded URI
// must still resolve when clicked). These tests pin down the helper that every
// formal-ai surface uses to render Wikipedia and concept-lookup sources.
// ---------------------------------------------------------------------------

#[test]
fn humanize_url_renders_cyrillic_wikipedia_link_readably() {
    // The exact URL pattern from issue #21.
    let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4";
    assert_eq!(
        humanize_url(encoded),
        "https://ru.wikipedia.org/wiki/Изумруд",
        "Cyrillic Wikipedia URL must display as readable IRI",
    );
}

#[test]
fn humanize_url_handles_every_supported_language() {
    let cases = [
        (
            "https://hi.wikipedia.org/wiki/%E0%A4%A8%E0%A4%AE%E0%A4%B8%E0%A5%8D%E0%A4%A4%E0%A5%87",
            "https://hi.wikipedia.org/wiki/नमस्ते",
        ),
        (
            "https://zh.wikipedia.org/wiki/%E4%BD%A0%E5%A5%BD",
            "https://zh.wikipedia.org/wiki/你好",
        ),
        (
            "https://ja.wikipedia.org/wiki/%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF",
            "https://ja.wikipedia.org/wiki/こんにちは",
        ),
        (
            "https://ar.wikipedia.org/wiki/%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
            "https://ar.wikipedia.org/wiki/مرحبا",
        ),
    ];
    for (encoded, expected) in cases {
        assert_eq!(
            humanize_url(encoded),
            expected,
            "humanize_url failed for {encoded}",
        );
    }
}

#[test]
fn humanize_url_preserves_functional_link_target() {
    // The encoded form must round-trip cleanly: encode(humanize(x)) ≈ x for
    // every URL we ship. We approximate the cycle by asserting that the
    // humanized form, when fed through Rust's standard percent-encoding via
    // the path crate (or by ensuring it contains the original Unicode chars),
    // does not lose information.
    let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4";
    let humanized = humanize_url(encoded);
    assert!(humanized.contains("Изумруд"));
    assert!(humanized.starts_with("https://ru.wikipedia.org/wiki/"));
    // ASCII URLs must round-trip untouched.
    let ascii = "https://en.wikipedia.org/wiki/Albert_Einstein";
    assert_eq!(humanize_url(ascii), ascii);
}

// ---------------------------------------------------------------------------
// Issue #44: Russian prompts with no matching rule return unknown + Russian
// reply.
// ---------------------------------------------------------------------------

#[test]
fn russian_nonsensical_question_returns_unknown_intent() {
    let response = answer("куда плешивый спрятал сахар?");
    assert_eq!(response.intent, "unknown");
}

#[test]
fn russian_mixed_units_question_returns_unit_incompatibility_intent() {
    let response = answer("Сколько метров в килобайте?");
    assert_eq!(response.intent, "unit_incompatibility");
}

#[test]
fn russian_trick_riddle_returns_unknown_intent() {
    let response = answer(
        "Стоит четырёхэтажный дом, в каждом этаже по восьми окон, \
         на крыше — два слуховых окна и две трубы, в каждом этаже \
         по два квартиранта. А теперь скажите, господа, в каком году \
         умерла у швейцара его бабушка?",
    );
    assert_eq!(response.intent, "unknown");
}

#[test]
fn russian_unknown_reply_is_in_russian() {
    let response = answer("куда плешивый спрятал сахар?");
    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("символьного правила")
            || response.answer.contains("Links Notation"),
        "Russian unknown reply should be in Russian or reference Links Notation, got: {}",
        response.answer
    );
}

#[test]
fn russian_unknown_reply_uses_russian_rule_configuration_examples() {
    let response = answer(
        "посмотри понял в чем смысл? если понял, то я тебе скину для теста \
         следующим сообщением тестовую картинку",
    );
    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("Покажи правила поведения"),
        "Russian unknown reply should explain rule listing in Russian, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Покажи правило unknown"),
        "Russian unknown reply should explain rule inspection in Russian, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Когда я скажу"),
        "Russian unknown reply should explain dialog-local teaching in Russian, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("List behavior rules")
            && !response.answer.contains("Show behavior rule unknown")
            && !response.answer.contains("When I say"),
        "Russian unknown reply should not switch to English command examples, got: {}",
        response.answer
    );
    assert!(
        !response
            .answer
            .contains("локальным правилам Links Notation"),
        "Unknown fallback should describe links rules rather than Links Notation rules, got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// Issue #29: "не понял" and other clarification prompts should be handled
// with a helpful clarification response, not the generic "unknown" fallback.
// ---------------------------------------------------------------------------

#[test]
fn russian_did_not_understand_returns_clarification_intent() {
    let response = answer("не понял");
    assert_eq!(
        response.intent, "clarification",
        "\"не понял\" should map to clarification intent, got: {}",
        response.intent
    );
}

#[test]
fn russian_clarification_reply_is_in_russian() {
    let response = answer("не понял");
    assert!(
        !response.answer.contains("symbolic rule")
            && !response.answer.contains("Links Notation fact"),
        "clarification reply must not be the generic unknown-intent fallback, got: {}",
        response.answer
    );
}

#[test]
fn english_did_not_understand_returns_clarification_intent() {
    let response = answer("I don't understand");
    assert_eq!(
        response.intent, "clarification",
        "\"I don't understand\" should map to clarification intent, got: {}",
        response.intent
    );
}

#[test]
fn english_dont_understand_variant_returns_clarification_intent() {
    let response = answer("I didn't understand");
    assert_eq!(
        response.intent, "clarification",
        "\"I didn't understand\" should map to clarification intent, got: {}",
        response.intent
    );
}

// ---------------------------------------------------------------------------
// Issue #30: "назови цвет" — "назови " prefix must route to concept_lookup,
// and "цвет" must resolve to the color concept record.
// The reporter's exact prompt was "назови цвет" which returned intent:unknown.
// ---------------------------------------------------------------------------

#[test]
fn russian_nazovi_prefix_routes_to_concept_lookup() {
    // "назови X" is a Russian imperative meaning "name X / tell me X".
    // It must be recognized as a concept_lookup prefix (issue #30).
    let response = answer("назови цвет");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "\"назови цвет\" should route to concept_lookup, got: {}",
        response.intent
    );
}

#[test]
fn russian_nazovi_tsvet_answer_references_color() {
    // The resolved answer must reference the color concept.
    let response = answer("назови цвет");
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("цвет") || lower.contains("color") || lower.contains("colour"),
        "\"назови цвет\" answer should describe a color, got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// Issue #41: "Купи слона" — well-known Russian circular-joke idiom.
// The phrase should be recognized and answered with the traditional reply,
// not fall through to the "unknown" catch-all intent.
// ---------------------------------------------------------------------------

#[test]
fn kupi_slona_returns_dedicated_idiom_intent() {
    // Issue #41 reporter's exact prompt.
    let response = answer("Купи слона");
    assert_ne!(
        response.intent, "unknown",
        "\"Купи слона\" must not fall through to unknown intent; got: {}",
        response.intent
    );
    assert_eq!(
        response.intent, "kupi_slona",
        "\"Купи слона\" must map to the kupi_slona intent, got: {}",
        response.intent
    );
}

#[test]
fn kupi_slona_answer_includes_traditional_reply() {
    let response = answer("Купи слона");
    let lower = response.answer.to_lowercase();
    // The traditional comeback is "у всех есть слон, а у меня нет"
    // (everyone has an elephant, but I don't) and similar variants.
    assert!(
        lower.contains("слон") || lower.contains("всех"),
        "\"Купи слона\" answer should reference the elephant, got: {}",
        response.answer
    );
}

#[test]
fn kupi_slona_answer_is_in_russian() {
    let response = answer("Купи слона");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:ru"),
        "\"Купи слона\" should be tagged as Russian, got evidence links: {:?}",
        response.evidence_links
    );
}
