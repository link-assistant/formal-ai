//! Issue #1172: factual Q&A answered questions about the wrong subject.
//!
//! `FactRecord::matches_normalized` matched subject aliases and question
//! keywords as raw substrings, so the United States alias "us" fired inside
//! "a**us**tralia" and "What is the capital of Australia?" was answered with
//! Washington, D.C. from the USA fact — even though no Australia fact is
//! seeded. The matcher now requires whole-word (token-run) matches, mirrored
//! by `matchesFactPhrase` in `js/worker/formal_ai_worker_05.js`.
//!
//! These tests pin the boundary contract from three angles:
//! - `FactRecord::contains_word_sequence` unit semantics (word boundaries,
//!   consecutive multi-word runs, CJK substring path, empty inputs),
//! - the seeded records' `matches_normalized` verdicts in all four supported
//!   languages, and
//! - the full engine dispatch, so an unseeded subject falls through to a
//!   non-`fact_lookup` answer instead of the wrong country's capital.
//!
//! The prompts are normalized with `web_engine_core::normalize_prompt`, the
//! public twin of the engine's `engine::normalize_prompt` (identical for
//! these prompts, which contain no `c++`/`c#` spellings that the engine
//! additionally canonicalizes).

use formal_ai::seed::{FactRecord, facts};
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::FormalAiEngine;

/// Fetch a seeded fact record by slug (e.g. `fact_capital_usa`).
fn fact(slug: &str) -> FactRecord {
    facts()
        .into_iter()
        .find(|record| record.slug == slug)
        .unwrap_or_else(|| panic!("seeded fact {slug} should exist in facts.lino"))
}

#[test]
fn contains_word_sequence_requires_whole_word_boundaries() {
    // The defect itself: the two-letter alias "us" must not fire inside
    // "australia" (issue #1172) — only as a standalone token.
    assert!(!FactRecord::contains_word_sequence(
        "what is the capital of australia",
        "us"
    ));
    assert!(FactRecord::contains_word_sequence("capital of usa", "usa"));

    // Multi-word aliases match as consecutive token runs, in order.
    assert!(FactRecord::contains_word_sequence(
        "name the united states capital",
        "united states"
    ));
    assert!(!FactRecord::contains_word_sequence(
        "the states united name",
        "united states"
    ));

    // Question keywords get the same boundary treatment: "capital" must not
    // match inside "capitalism".
    assert!(!FactRecord::contains_word_sequence(
        "is capitalism an economic system",
        "capital"
    ));

    // Empty phrases never match, and no phrase matches empty text.
    assert!(!FactRecord::contains_word_sequence(
        "what is the capital of australia",
        ""
    ));
    assert!(!FactRecord::contains_word_sequence("", "us"));

    // CJK phrases have no inter-word spaces to honor, so they keep substring
    // matching (mirroring `seed::meanings::surface_present`, issue #386).
    assert!(FactRecord::contains_word_sequence(
        "美国的首都是什么",
        "美国"
    ));
}

#[test]
fn usa_fact_no_longer_answers_australia_prompts() {
    let record = fact("fact_capital_usa");
    let normalized = normalize_prompt("What is the capital of Australia?");
    assert_eq!(normalized, "what is the capital of australia");
    assert!(
        !record.matches_normalized(&normalized),
        "the alias \"us\" must not match inside \"australia\" (issue #1172)"
    );

    // No seeded fact at all claims Australia, so the store must resolve the
    // prompt to "unanswerable" — the precondition for the live/web fallback
    // the planner's open-web gate (issue #989) releases.
    assert!(
        facts()
            .iter()
            .all(|record| !record.matches_normalized(&normalized)),
        "no seeded fact should match an Australia prompt"
    );
}

#[test]
fn usa_fact_still_matches_its_own_prompts() {
    let record = fact("fact_capital_usa");
    for prompt in [
        "What is the capital of the United States?",
        "Which city is the capital of the USA?",
        "capital of usa",
        "Какова столица США?",
        "अमेरिका की राजधानी क्या है?",
        "美国的首都是什么？",
    ] {
        let normalized = normalize_prompt(prompt);
        assert!(
            record.matches_normalized(&normalized),
            "prompt {prompt:?} (normalized {normalized:?}) should match the USA capital fact"
        );
    }
}

#[test]
fn japan_fact_still_matches_multilingual_prompts() {
    let record = fact("fact_capital_japan");
    for prompt in [
        "What is the capital of Japan?",
        "Which city is Japan's capital?",
        "Какова столица Японии?",
        "जापान की राजधानी क्या है?",
    ] {
        let normalized = normalize_prompt(prompt);
        assert!(
            record.matches_normalized(&normalized),
            "prompt {prompt:?} (normalized {normalized:?}) should match the Japan capital fact"
        );
    }
}

#[test]
fn keyword_inside_a_longer_word_does_not_route_to_a_fact() {
    // The alias side matches ("usa" is a whole token) but the keyword
    // "capital" only appears inside "capitalism", so the conjunction fails
    // and the prompt must not be treated as a seeded capital question.
    let record = fact("fact_capital_usa");
    let normalized = normalize_prompt("Is capitalism the system of the usa?");
    assert!(!record.matches_normalized(&normalized));
}

#[test]
fn engine_answers_seeded_capitals_from_their_own_facts() {
    let japan = FormalAiEngine.answer("What is the capital of Japan?");
    assert_eq!(japan.intent, "fact_lookup");
    assert!(
        japan.answer.contains("Tokyo"),
        "Japan prompt should answer with the seeded summary, got: {}",
        japan.answer
    );

    let usa = FormalAiEngine.answer("What is the capital of the United States?");
    assert_eq!(usa.intent, "fact_lookup");
    assert!(
        usa.answer.contains("Washington"),
        "United States prompt should answer with the seeded summary, got: {}",
        usa.answer
    );
}

#[test]
fn engine_no_longer_answers_australia_with_washington() {
    // Australia has no seeded fact, so the prompt must fall through the
    // fact store to a non-fact_lookup answer — never the USA record's
    // Washington, D.C. summary the substring alias match used to produce.
    let response = FormalAiEngine.answer("What is the capital of Australia?");
    assert_ne!(
        response.intent, "fact_lookup",
        "an unseeded subject must not route to a seeded fact (issue #1172)"
    );
    assert!(
        !response.answer.contains("Washington"),
        "Australia prompt must not be answered from the USA fact, got: {}",
        response.answer
    );
}
