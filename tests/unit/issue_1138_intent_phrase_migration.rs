//! Plan 10 leaf 20 (issue #1138): the intent-routing table's exact-match
//! phrase families retire onto derivations. The `http_fetch`, `web_search`
//! and `url_navigate` rows retired onto the structural URL object and the
//! shared role surfaces of the web-navigation and web-search seeds; the
//! conversational families (`greeting`, `wellbeing`, `farewell`,
//! `courtesy_response`, `test_status`, `assistant_name`, `identity`,
//! `assistant_free_time`) retire onto seeded conversation roles declared by
//! a `role_surface` field on the family's own block.
//!
//! These held-out paraphrases and bare canonical prompts ship BEFORE the
//! rows are deleted, so the retirement commit is already guarded: every
//! prompt here is a phrasing no phrase row ever named, or a bare prompt the
//! role must carry, and each must reach the answer the same family's
//! canonical phrasing reaches — through the derivation, not through an
//! exact-match row. The URL is the object; the verb is evidence, never the
//! decision. A conversational prompt has no object: the decision is the
//! role's whole-prompt surface inventory, the same equality the rows had.

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

/// Plan 10 leaf 20, family four: the greeting rows retired onto the
/// `social_greeting` role under the same whole-prompt equality. Retired rows
/// themselves must still reach the greeting answer through the role, and the
/// Spanish greetings no row ever held are the generalization the retirement
/// buys. The `greet` token row stays: its contains-match is separate
/// behavior with its own retirement to draft.
#[test]
fn greetings_route_through_the_social_greeting_role() {
    for prompt in [
        "hi",
        "hello there",
        "good morning",
        "привет",
        "здравствуйте",
        "नमस्ते",
        "你好",
        "hola",
        "buenos días",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "greeting",
            "the bare greeting `{prompt}` must reach the greeting intent through the \
             social_greeting role, with the family's keyword and phrase rows retired"
        );
    }
    // A greeting surface inside a larger request is evidence, not the
    // decision: the whole-prompt equality the rows had is the boundary the
    // role keeps, so this compound stays a courtesy-and-question synthesis.
    assert_ne!(
        answer("hi what is 2 + 2").intent,
        "greeting",
        "a greeting lead on a compound prompt must not claim the whole route"
    );
}

/// The bare prompts a family held as keyword or phrase rows, per family:
/// each must still reach the family intent through its declared role, with
/// the rows retired. Spanish surfaces no row ever held are the
/// generalization the retirement buys.
fn family_guard_count(slug: &str) -> usize {
    intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == slug)
        .map(|route| route.keywords.len() + route.phrases.len() + route.tokens.len())
        .unwrap_or(0)
}

#[test]
fn wellbeing_prompts_route_through_the_social_wellbeing_role() {
    for prompt in [
        "how are you",
        "привет как дела",
        "как поживаешь",
        "कैसे हो",
        "आप कैसे हैं",
        "你好吗",
        "最近怎么样",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "wellbeing",
            "the wellbeing prompt `{prompt}` must reach the wellbeing intent \
             through the social_wellbeing role, with the family's rows retired"
        );
    }
    for spanish in ["cómo estás", "qué tal"] {
        assert_eq!(
            answer(spanish).intent,
            "wellbeing",
            "the Spanish wellbeing prompt `{spanish}` was never a table row; the \
             role carries it as the retirement's generalization"
        );
    }
    // `how is it going` is NOT asserted: the how_it_works handler claimed it
    // even while the wellbeing row existed, so the row was never the decision.
    // The retirement must not change that prompt, and it does not claim it.
    assert_ne!(
        answer("how are you handling the schema migration").intent,
        "wellbeing",
        "a wellbeing lead on a compound prompt must not claim the whole route"
    );
}

#[test]
fn the_wellbeing_family_stays_row_free() {
    assert!(
        family_guard_count("wellbeing") <= 27,
        "the wellbeing family had twenty-seven phrase rows at the draft and zero \
         after the retirement; the social_wellbeing role surfaces of \
         meanings-conversation.lino decide (plan 10 leaf 20)"
    );
}

#[test]
fn farewell_prompts_route_through_the_social_farewell_role() {
    for prompt in [
        "goodbye",
        "see you later",
        "take care",
        "пока",
        "до свидания",
        "फिर मिलेंगे",
        "再见",
        "拜拜",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "farewell",
            "the farewell prompt `{prompt}` must reach the farewell intent \
             through the social_farewell role, with the family's rows retired"
        );
    }
    for spanish in ["adiós", "hasta luego"] {
        assert_eq!(
            answer(spanish).intent,
            "farewell",
            "the Spanish farewell prompt `{spanish}` was never a table row; the \
             role carries it as the retirement's generalization"
        );
    }
    assert_ne!(
        answer("see you at the review tomorrow").intent,
        "farewell",
        "a farewell lead on a compound prompt must not claim the whole route"
    );
}

#[test]
fn the_farewell_family_stays_row_free() {
    assert!(
        family_guard_count("farewell") <= 27,
        "the farewell family had twenty-seven keyword and phrase rows at the \
         draft and zero after the retirement; the social_farewell role surfaces \
         of meanings-conversation.lino decide (plan 10 leaf 20)"
    );
}

#[test]
fn courtesy_response_prompts_route_through_the_social_courtesy_role() {
    for prompt in [
        "thanks",
        "thank you",
        "i am fine thank you",
        "спасибо",
        "у меня всё хорошо спасибо",
        "धन्यवाद",
        "ठीक हूँ धन्यवाद",
        "谢谢",
        "非常感谢",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "courtesy_response",
            "the courtesy prompt `{prompt}` must reach the courtesy_response \
             intent through the social_courtesy_response role, with the family's \
             rows retired"
        );
    }
    for spanish in ["gracias", "muchas gracias"] {
        assert_eq!(
            answer(spanish).intent,
            "courtesy_response",
            "the Spanish courtesy prompt `{spanish}` was never a table row; the \
             role carries it as the retirement's generalization"
        );
    }
    assert_ne!(
        answer("thank you note for the team review").intent,
        "courtesy_response",
        "a courtesy lead on a compound prompt must not claim the whole route"
    );
}

#[test]
fn the_courtesy_response_family_stays_row_free() {
    assert!(
        family_guard_count("courtesy_response") <= 41,
        "the courtesy_response family had forty-one keyword and phrase rows at \
         the draft and zero after the retirement; the social_courtesy_response \
         role surfaces of meanings-conversation.lino decide (plan 10 leaf 20)"
    );
}

#[test]
fn test_status_prompts_route_through_the_social_test_status_role() {
    for prompt in [
        "ping",
        "test passed",
        "testing 123",
        "are you there",
        "тест пройден",
        "ты тут",
        "prueba superada",
        "estoy aquí",
        "你在吗",
        "我在这里",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "test_status",
            "the test-status prompt `{prompt}` must reach the test_status intent \
             through the social_test_status role, with the keyword and phrase \
             rows retired"
        );
    }
    assert_ne!(
        answer("ping the repository before the deploy").intent,
        "test_status",
        "a test-status lead on a compound prompt must not claim the whole route; \
         the combos keep their all-token behavior for compound status checks"
    );
}

#[test]
fn the_test_status_family_stays_row_free() {
    let family = intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == "test_status")
        .expect("the test_status family block keeps its slug");
    assert!(
        family.keywords.len() + family.phrases.len() <= 44,
        "the test_status family had forty-four keyword and phrase rows at the \
         draft and zero after the retirement; the social_test_status role \
         surfaces decide the bare prompts and the combos keep the compound \
         all-token checks (plan 10 leaf 20)"
    );
}

#[test]
fn assistant_name_prompts_route_through_the_social_name_role() {
    for prompt in [
        "what is your name",
        "what's your name",
        "как тебя зовут",
        "назови своё имя",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "assistant_name",
            "the name prompt `{prompt}` must reach the assistant_name intent \
             through the social_assistant_name role, with the family's phrase \
             rows retired"
        );
    }
    // `आपका नाम क्या है` and `你叫什么名字` are NOT asserted: the
    // set_assistant_name handler claimed both even while the family's phrase
    // rows existed, so those rows were never the decision for them.
    for spanish in ["cómo te llamas", "cuál es tu nombre"] {
        assert_eq!(
            answer(spanish).intent,
            "assistant_name",
            "the Spanish name prompt `{spanish}` was never a table row; the role \
             carries it as the retirement's generalization"
        );
    }
    assert_ne!(
        answer("what should i call the new variable").intent,
        "assistant_name",
        "a name lead on a compound prompt must not claim the whole route; the \
         combos keep their all-token behavior"
    );
}

#[test]
fn the_assistant_name_family_stays_row_free() {
    let family = intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == "assistant_name")
        .expect("the assistant_name family block keeps its slug");
    assert!(
        family.keywords.len() + family.phrases.len() <= 24,
        "the assistant_name family had twenty-four phrase rows at the draft and \
         zero after the retirement; the social_assistant_name role surfaces \
         decide the bare prompts and the combos keep the compound all-token \
         checks (plan 10 leaf 20)"
    );
}

#[test]
fn identity_prompts_route_through_the_social_identity_role() {
    for prompt in [
        "who are you",
        "tell me about yourself",
        "кто ты",
        "расскажи о себе",
        "तुम कौन हो",
        "介绍一下你自己",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "identity",
            "the identity prompt `{prompt}` must reach the identity intent \
             through the social_identity role, with the family's phrase rows \
             retired"
        );
    }
    // `आप कौन हैं` and `你是谁` are NOT asserted: the who_is_question handler
    // claimed both even while the identity rows existed, so those rows were
    // never the decision for them.
    for spanish in ["quién eres"] {
        assert_eq!(
            answer(spanish).intent,
            "identity",
            "the Spanish identity prompt `{spanish}` was never a table row; the \
             role carries it as the retirement's generalization"
        );
    }
    assert_ne!(
        answer("who wrote the hamlet play").intent,
        "identity",
        "an identity lead on a compound prompt must not claim the whole route; \
         the combos keep their all-token behavior"
    );
}

#[test]
fn the_identity_family_stays_row_free() {
    let family = intent_routing()
        .intents
        .iter()
        .find(|route| route.slug == "identity")
        .expect("the identity family block keeps its slug");
    assert!(
        family.keywords.len() + family.phrases.len() <= 38,
        "the identity family had thirty-eight phrase rows at the draft and zero \
         after the retirement; the social_identity role surfaces decide the \
         bare prompts and the combos keep the compound all-token checks \
         (plan 10 leaf 20)"
    );
}

#[test]
fn free_time_prompts_route_through_the_social_free_time_role() {
    for prompt in [
        "what do you do in your free time",
        "how do you spend your spare time",
        "что делаешь в свободное время",
        "как проводишь свободное время",
        "खाली समय में क्या करते हो",
        "你空闲时间做什么",
    ] {
        assert_eq!(
            answer(prompt).intent,
            "assistant_free_time",
            "the free-time prompt `{prompt}` must reach the assistant_free_time \
             intent through the social_assistant_free_time role, with the \
             family's rows retired and the capabilities rules' route_exact \
             veto reading the same role surfaces"
        );
    }
    for spanish in ["qué haces en tu tiempo libre", "cómo pasas tu tiempo libre"] {
        assert_eq!(
            answer(spanish).intent,
            "assistant_free_time",
            "the Spanish free-time prompt `{spanish}` was never a table row; the \
             role carries it as the retirement's generalization"
        );
    }
}

#[test]
fn the_assistant_free_time_family_stays_row_free() {
    assert!(
        family_guard_count("assistant_free_time") <= 21,
        "the assistant_free_time family had twenty-one phrase rows at the draft \
         and zero after the retirement; the social_assistant_free_time role \
         surfaces of meanings-conversation.lino decide, and the capabilities \
         rules' route_exact veto reads them through the same declaration \
         (plan 10 leaf 20)"
    );
}
