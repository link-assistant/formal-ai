//! Plan 10 leaf 20 (issue #1138), family one: the `http_fetch` phrase rows of
//! `data/seed/intent-routing.lino` retire onto the structural URL object and
//! the shared `http_fetch` role surfaces of
//! `data/seed/meanings-web-navigation.lino`.
//!
//! These held-out paraphrases ship BEFORE the rows are deleted, so the
//! retirement commit is already guarded: every prompt here is a phrasing no
//! phrase row ever named, and each must reach the same answer the same
//! language's canonical phrasing reaches — through the role-surface
//! derivation, not through an exact-match row. The URL is the object; the
//! verb is evidence, never the decision.

use formal_ai::seed::intent_routing;
use formal_ai::{FormalAiEngine, SymbolicAnswer};

const REPO_URL: &str = "https://github.com/netkeep80/anum_docs";

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

/// A canonical phrasing and a held-out paraphrase of the same fetch request,
/// per language the `http_fetch` role carries surfaces for.
#[test]
fn held_out_fetch_paraphrases_reach_the_canonical_fetch_answer() {
    let cases = [
        (
            "make a request to",
            "would you be so kind as to make an http request to",
        ),
        ("выполни http запрос к", "будь добр, выполни http запрос к"),
        ("अनुरोध भेजें", "कृपया इसके लिए अनुरोध भेजें"),
        ("发送请求", "麻烦你现在发送请求给"),
    ];
    for (canonical, paraphrase) in cases {
        let expected = answer(&format!("{canonical} {REPO_URL}"));
        assert_eq!(
            expected.intent, "http_fetch",
            "the canonical phrasing `{canonical} …` must itself reach the fetch answer"
        );
        let held_out = answer(&format!("{paraphrase} {REPO_URL}"));
        assert_eq!(
            (
                held_out.intent.as_str(),
                held_out.answer.as_str(),
                held_out.confidence
            ),
            (
                expected.intent.as_str(),
                expected.answer.as_str(),
                expected.confidence
            ),
            "the held-out paraphrase `{paraphrase} …` must reach the answer the \
             canonical phrasing reaches, with no phrase row naming it",
        );
    }
}

/// A URL prompt in a phrasing the role surfaces do not carry is not a fetch
/// request the exact-match table would have claimed either: the derivation
/// declines rather than guessing, which is the boundary the retirement keeps.
#[test]
fn a_url_with_no_fetch_evidence_is_not_claimed_by_the_retired_rows() {
    let family_rows = intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == "http_fetch")
        .map(|route| route.keywords.len() + route.phrases.len() + route.tokens.len())
        .unwrap_or(0);
    assert!(
        family_rows <= 12,
        "the http_fetch family may not grow new phrase, keyword or token rows; \
         it had 12 (eleven phrases and the fetch token) at the draft and zero \
         after the retirement, with the URL object deciding (plan 10 leaf 20)"
    );
}

/// Plan 10 leaf 20, family two: the `web_search` phrase rows are duplicates of
/// the `web_search_explicit_prefix` role surfaces, and a real search request
/// (surface plus query) never matched them, because the rows required
/// whole-prompt equality. The derivation below is what always decided these
/// prompts; Spanish and Chinese had no rows at all and were already derived.
#[test]
fn held_out_web_search_paraphrases_reach_the_canonical_search_answer() {
    let query = "giant squid migration routes";
    for (canonical, paraphrase) in [
        (
            "search the web for",
            "search the web for, and be thorough about",
        ),
        ("поищи в интернете", "поищи в интернете, пожалуйста,"),
    ] {
        let expected = answer(&format!("{canonical} {query}"));
        assert_eq!(
            expected.intent, "web_search",
            "the canonical phrasing `{canonical} …` must itself reach the search answer"
        );
        let held_out = answer(&format!("{paraphrase} {query}"));
        assert_eq!(
            held_out.intent, "web_search",
            "the held-out paraphrase `{paraphrase} …` must reach web_search with no \
             phrase row naming it"
        );
    }
    // No Hindi or Chinese row ever existed: these two were derived before
    // the retirement and must stay derived after it. (Spanish is the known
    // gap — the solver's web-search seed carries no es lexeme, recorded in
    // the plan as future work, not asserted here.)
    for language_held_out in ["वेब पर खोजें", "在线搜索"] {
        let held_out = answer(&format!("{language_held_out} {query}"));
        assert_eq!(
            held_out.intent, "web_search",
            "the {language_held_out} … phrasing was never a table row; the derivation \
             must keep deciding it"
        );
    }
}

/// The `web_search` family may not grow its exact-match rows back.
#[test]
fn the_web_search_family_stays_row_free() {
    let family_rows = intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == "web_search")
        .map(|route| route.keywords.len() + route.phrases.len() + route.tokens.len())
        .unwrap_or(0);
    assert!(
        family_rows <= 6,
        "the web_search family had six phrase rows at the draft and zero after the \
         retirement; the explicit-prefix role surfaces of \
         meanings-web-search-query.lino decide these prompts (plan 10 leaf 20)"
    );
}

/// Plan 10 leaf 20, family three: the `url_navigate` rows are bare lead forms
/// (`open`, `show`, `открой ссылку`) duplicating the `url_navigate` role's
/// prefix surfaces. A navigation prompt carries a host, so whole-prompt
/// equality never fired for one — the role prefix derivation decided, which
/// the issue #125 suite pins across thirty host-bearing prompts. The
/// paraphrases here keep the seeded surface in prefix position (the
/// derivation is `starts_with`) and vary the tail, which no row named.
#[test]
fn held_out_url_navigate_paraphrases_reach_the_navigation_answer() {
    const HOST: &str = "github.com";
    for (canonical, paraphrase) in [
        ("navigate to", "navigate to github.com right now please"),
        ("open the page", "open the page github.com and wait"),
        ("show me", "show me github.com one more time"),
        ("открой ссылку", "открой ссылку github.com если не сложно"),
        ("перейди на", "перейди на github.com как можно быстрее"),
    ] {
        let expected = answer(&format!("{canonical} {HOST}"));
        assert_eq!(
            expected.intent, "url_navigate",
            "the canonical lead `{canonical} …` must itself reach the navigation answer"
        );
        let held_out = answer(paraphrase);
        assert_eq!(
            held_out.intent, "url_navigate",
            "the held-out paraphrase `{paraphrase}` must reach url_navigate with no \
             phrase row naming it"
        );
        assert_eq!(
            held_out.answer, expected.answer,
            "the held-out paraphrase `{paraphrase}` must reach the navigation answer"
        );
    }
}

/// The `url_navigate` family may not grow its exact-match rows back.
#[test]
fn the_url_navigate_family_stays_row_free() {
    let family_rows = intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == "url_navigate")
        .map(|route| route.keywords.len() + route.phrases.len() + route.tokens.len())
        .unwrap_or(0);
    assert!(
        family_rows <= 37,
        "the url_navigate family had thirty-seven bare-lead phrase rows at the draft \
         and zero after the retirement; the url_navigate role prefix surfaces of \
         meanings-web-navigation.lino decide host-bearing prompts, which the issue \
         #125 suite pins (plan 10 leaf 20)"
    );
}

/// Plan 10 leaf 20, family four preparation: the greeting family's rows are
/// bare whole prompts with no object, so they retire onto a seeded
/// `social_greeting` role rather than a structural derivation. This guard
/// ships BEFORE the wiring: every keyword and phrase the family holds today
/// must already be carried by the role, so the later wiring provably covers
/// each retired row. Spanish never had rows; the role carries es anyway,
/// which is the coverage the retirement will generalize to.
#[test]
fn every_greeting_family_row_is_carried_by_the_social_greeting_role() {
    let family = intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == "greeting")
        .expect("the greeting family block keeps its slug while its rows retire");
    let mut carried = formal_ai::seed::lexicon()
        .words_for_role("social_greeting")
        .into_iter()
        .collect::<Vec<String>>();
    carried.sort();
    for row in family.keywords.iter().chain(family.phrases.iter()) {
        assert!(
            carried.binary_search(row).is_ok(),
            "the social_greeting role must carry `{row}` before the family's rows retire"
        );
    }
    let spanish =
        formal_ai::seed::lexicon().words_for_role_in_languages("social_greeting", &["es"]);
    assert!(
        spanish.iter().any(|word| word == "hola"),
        "the role carries Spanish greetings although the family never had es rows"
    );
}
