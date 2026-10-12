//! Issue #1176 R1176-1: a unit is resolved to its Wikidata item and the
//! linear factor is derived at answer time from the two items' captured
//! P2370 ("conversion to SI unit") statements in
//! `data/seed/wikidata-conversion-to-si.lino`; the answer cites both items,
//! the property and the quotient. A unit the capture does not cover keeps the
//! seed factor and carries no Wikidata sentence. The browser twin is pinned
//! by `rust/tests/web/issue-1176-wikidata-grounding.test.mjs`; the capture is
//! re-read against Wikidata by `scripts/refresh-wikidata-conversion-to-si.mjs`.

use formal_ai::FormalAiEngine;

#[test]
fn english_conversion_cites_both_items_and_the_p2370_quotient() {
    let response = FormalAiEngine.answer("How many meters are 5 feet?");
    assert_eq!(response.intent, "unit_conversion");
    assert_eq!(
        response.answer,
        "5 feet is 1.524 meters. 5 × 0.3048 = 1.524, because 1 feet = 0.3048 meters. Wikidata grounds the factor: feet is Q3710 and meters is Q11573, whose conversion to SI unit (P2370, captured 2026-10-07) is 0.3048 and 1 of Q11573, so 0.3048 ÷ 1 = 0.3048."
    );
}

#[test]
fn russian_conversion_cites_the_items_in_russian() {
    let response = FormalAiEngine.answer("Сколько километров в 26.2 милях?");
    assert_eq!(response.intent, "unit_conversion");
    assert_eq!(
        response.answer,
        "26.2 милях — это 42.1648128 километров. 26.2 × 1.609344 = 42.1648128, потому что 1 милях = 1.609344 километров. Коэффициент из Викиданных: милях — это Q253276, километров — это Q828224; их перевод в единицу СИ (P2370, снимок от 2026-10-07) равен 1609.344 и 1000 от Q11573, поэтому 1609.344 ÷ 1000 = 1.609344."
    );
}

#[test]
fn an_uncaptured_amount_keeps_the_seed_factor_without_a_wikidata_sentence() {
    // The ounce's P2370 amount (0.028349523125) lies past the exact decimal
    // bounds, so the seed factor is used and no item is cited.
    let response = FormalAiEngine.answer("How many grams are 2 ounces?");
    assert_eq!(response.intent, "unit_conversion");
    assert_eq!(
        response.answer,
        "2 ounces is 56.69904625 grams. 2 × 28.349523125 = 56.69904625, because 1 ounces = 28.349523125 grams."
    );
}

#[test]
fn hindi_and_chinese_conversions_cite_the_items_in_their_language() {
    let hindi = FormalAiEngine.answer("26.2 मील कितने किलोमीटर हैं?");
    assert_eq!(
        hindi.answer,
        "26.2 मील = 42.1648128 किलोमीटर। 26.2 × 1.609344 = 42.1648128, क्योंकि 1 मील = 1.609344 किलोमीटर। गुणांक विकिडेटा से: मील Q253276 है और किलोमीटर Q828224 है; SI इकाई में उनका रूपांतरण (P2370, 2026-10-07 को लिया गया) Q11573 का 1609.344 और 1000 है, इसलिए 1609.344 ÷ 1000 = 1.609344।"
    );
    let chinese = FormalAiEngine.answer("26.2英里是多少公里？");
    assert_eq!(
        chinese.answer,
        "26.2 英里 是 42.1648128 公里。26.2 × 1.609344 = 42.1648128，因为 1 英里 = 1.609344 公里。系数来自维基数据：英里 是 Q253276，公里 是 Q828224；它们换算为国际单位（P2370，2026-10-07 采集）分别是 Q11573 的 1609.344 和 1000，所以 1609.344 ÷ 1000 = 1.609344。"
    );
}
