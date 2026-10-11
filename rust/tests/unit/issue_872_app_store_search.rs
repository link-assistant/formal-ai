//! Issue #872: the App Store request with constraints that fell to unknown.
//!
//! Reported dialog (agentic CLI, 0.304.0, 2026-07-26):
//!
//! ```text
//! U: игры для малышей с открытым исходным кодом (абсолютно и полностью
//!    бесплатные) в App Store (iOS)
//! A: … Я не смог определить … (unknown fallback)
//! ```
//!
//! A bare noun phrase with a marketplace is a shopping request without a
//! request verb: the structural shape (marketplace phrase + product noun)
//! must carry it, and the qualifiers — open source, completely free — are
//! constraints the composed search states and advises on, not noise.
//! These tests pin the exact reported string, the App Store link, both
//! constraints, and the toddler-app advice.

use formal_ai::event_log::EventLog;
use formal_ai::handle_product_search;
use formal_ai::web_engine_core::normalize_prompt;

/// The handler answer for a raw prompt.
fn handled(prompt: &str) -> Option<formal_ai::engine::SymbolicAnswer> {
    handle_product_search(prompt, &normalize_prompt(prompt), &mut EventLog::new())
}

/// The exact answer for the reported prompt: the user's own noun phrase as
/// the query, all three matched constraints in catalogue order, and the
/// toddler advice followed by each constraint's advice.
const EXPECTED_ANSWER: &str = concat!(
    "Here is the search I composed for игры для малышей on app store ios:\n",
    "\n",
    "https://www.apple.com/us/search/%D0%B8%D0%B3%D1%80%D1%8B%20%D0%B4%D0%BB%D1%8F%20%D0%BC%D0%B0%D0%BB%D1%8B%D1%88%D0%B5%D0%B9?src=globalnav\n",
    "\n",
    "Constraints you asked for: free, fully_free, open_source\n",
    "Before you buy: check the age band on the listing and the permissions the app asks for; offline-capable listings avoid ads served to children filter the store listing to price zero; a paid ranking can still surface, so read the price on the page check the listing for in-app purchases and ads; fully free and open source usually also means the source repository is linked verify the linked source repository exists and matches the store listing; stores do not enforce this label\n",
    "This was not fetched — the link is the exact query I would run, and I ran it in offline mode.",
);

#[test]
fn the_reported_app_store_prompt_composes_the_search() {
    let answer = handled(
        "игры для малышей с открытым исходным кодом (абсолютно и полностью бесплатные) в App Store (iOS)",
    )
    .expect("the bare noun phrase with a marketplace must be handled");
    assert_eq!(answer.answer, EXPECTED_ANSWER);
    assert!(
        answer.answer.contains("apple.com/us/search/"),
        "the App Store deep link must be composed: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("%D0%B8%D0%B3%D1%80%D1%8B") || answer.answer.contains("игры"),
        "the query keeps the user's own words: {}",
        answer.answer
    );
}

#[test]
fn both_qualifiers_survive_as_stated_constraints() {
    let answer = handled(
        "игры для малышей с открытым исходным кодом (абсолютно и полностью бесплатные) в App Store (iOS)",
    )
    .expect("handled");
    assert_eq!(answer.answer, EXPECTED_ANSWER);
    assert!(
        answer.answer.contains("fully_free") && answer.answer.contains("open_source"),
        "open source and completely-free must both be named: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("in-app purchases") || answer.answer.contains("source repository"),
        "the constraint advice must say what to verify: {}",
        answer.answer
    );
}

#[test]
fn the_toddler_advice_names_age_band_and_permissions() {
    let answer = handled(
        "игры для малышей с открытым исходным кодом (абсолютно и полностью бесплатные) в App Store (iOS)",
    )
    .expect("handled");
    assert_eq!(answer.answer, EXPECTED_ANSWER);
    assert!(
        answer.answer.contains("age band") && answer.answer.contains("permissions"),
        "kids-app advice must name the age band and the permissions: {}",
        answer.answer
    );
}

#[test]
fn a_bare_noun_phrase_without_a_marketplace_still_declines() {
    assert!(
        handled("игры для малышей с открытым исходным кодом").is_none(),
        "no marketplace means no shopping shape"
    );
}
