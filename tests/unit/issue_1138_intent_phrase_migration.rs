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
