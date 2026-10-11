//! Issue #800: the amazon.in charger search that died in research.
//!
//! Reported dialog (agentic session, 2026):
//!
//! ```text
//! U: Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in
//! A: Research completed for мне зарядку для ноутбука acer aspire 3 a325 45
//!    на amazon in, but the tool returned no content.
//! ```
//!
//! The request is a shopping search: a marketplace, a product noun, a
//! brand and exact model code. The product-search handler composes the
//! site-scoped query those terms describe and answers honestly in offline
//! mode. These tests pin the exact reported string, the composed link, the
//! extracted model code, and the charger-specific verification advice.

use formal_ai::event_log::EventLog;
use formal_ai::handle_product_search;
use formal_ai::web_engine_core::normalize_prompt;

/// The handler answer for a raw prompt.
fn handled(prompt: &str) -> Option<formal_ai::engine::SymbolicAnswer> {
    handle_product_search(prompt, &normalize_prompt(prompt), &mut EventLog::new())
}

/// The exact answer for the reported prompt: the matched noun's surface
/// stem, the site-scoped link (stem, other product terms, model code, brand
/// words, percent-encoded), no stated constraints, and the charger advice.
const EXPECTED_ANSWER: &str = concat!(
    "Here is the search I composed for зарядк on amazon in:\n",
    "\n",
    "https://www.amazon.in/s?k=%D0%B7%D0%B0%D1%80%D1%8F%D0%B4%D0%BA%20laptop%20A325-45%20Aspire%20Acer\n",
    "\n",
    "Constraints you asked for: (none stated)\n",
    "Before you buy: match the laptop model exactly (the model number is on the underside), then the connector type and the wattage; a lower-wattage charger throttles or refuses to charge\n",
    "This was not fetched — the link is the exact query I would run, and I ran it in offline mode.",
);

#[test]
fn the_reported_amazon_in_prompt_composes_the_search() {
    let answer = handled("Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in")
        .expect("the reported shopping request must be handled");
    assert_eq!(answer.answer, EXPECTED_ANSWER);
    assert!(
        answer.answer.contains("amazon.in/s?k="),
        "the site-scoped link must be composed: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("A325-45"),
        "the exact model code must survive into the query: {}",
        answer.answer
    );
}

#[test]
fn the_answer_states_offline_honesty_and_buying_advice() {
    let answer = handled("Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in")
        .expect("handled");
    assert_eq!(answer.answer, EXPECTED_ANSWER);
    assert!(
        answer.answer.contains("не загружал") || answer.answer.contains("not fetched"),
        "offline honesty is mandatory: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("wattage") || answer.answer.contains("мощност"),
        "charger advice must name wattage/connector: {}",
        answer.answer
    );
}

#[test]
fn a_shopping_request_without_a_marketplace_is_not_claimed() {
    assert!(
        handled("Найди мне хороший подарок сестре").is_none(),
        "no marketplace phrase means the handler declines"
    );
}

#[test]
fn a_non_shopping_prompt_is_not_claimed() {
    assert!(
        handled("What is the variance of 2, 4, 4, 4, 5").is_none(),
        "the handler must not claim ordinary questions"
    );
}
