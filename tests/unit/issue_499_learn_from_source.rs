//! Issue #499: a user teaching the engine where to learn from — e.g.
//! "Обратясь сюда ты узнаешь актуальные темы <Google Trends URL>" — must be
//! recognized as a *learn from this data source* directive and routed into the
//! matching auto-learning capability, instead of returning `intent: unknown`.
//!
//! These tests pin one behaviour per requirement plus one whole-task check, and
//! deliberately vary the natural-language wording each time (CONTRIBUTING rule 4)
//! so a passing run proves the routing is general, not hardcoded to one phrase.

use formal_ai::agentic_coding::{
    google_trends_learning as recipe, is_google_trends_learning_task, run_agentic_task,
    GOOGLE_TRENDS_LEARNING_PATH,
};
use formal_ai::engine::FormalAiEngine;
use formal_ai::{response_for, seed, supported_languages, trending_learning_report};

/// The exact prompt reported in issue #499 (Russian, with the Google Trends URL).
const REPORTED_PROMPT: &str =
    "Обратясь сюда ты узнаешь актуальные темы https://trends.google.com/trending?hl=ru&&geo=US";

fn answer(prompt: &str) -> formal_ai::SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// R499-1: the reported prompt no longer falls to the unknown opener.
#[test]
fn the_reported_prompt_no_longer_returns_unknown() {
    let result = answer(REPORTED_PROMPT);
    assert_ne!(
        result.intent, "unknown",
        "issue #499: the reported learn-from-source directive must be recognized",
    );
    assert_eq!(result.intent, "learn_from_source");
    assert!(
        result.confidence >= 0.9,
        "a directly recognized source directive should answer with high confidence",
    );
}

// R499-2: the directive is recognized across every supported language, each with a
// different wording, so the routing is not tied to a single phrase or locale.
#[test]
fn learn_from_source_is_recognized_in_every_supported_language() {
    let prompts = [
        // en — "learn from" cue + host
        "Learn from popular Google searches at https://trends.google.com/trending/rss?geo=US",
        // ru — "тут видны" cue + host
        "Тут видны темы кототорые интересуют людей https://trends.google.com/trending?hl=ru&&geo=US",
        // hi — "यहाँ से सीख" cue + host
        "यहाँ से सीख सकते हो कि लोग क्या खोज रहे हैं https://trends.google.com/trending?geo=IN",
        // zh — "在这里了解" cue + "谷歌趋势" keyword + host
        "在这里了解谷歌趋势的热门话题 https://trends.google.com/trending?geo=CN",
    ];
    for prompt in prompts {
        assert_eq!(
            answer(prompt).intent,
            "learn_from_source",
            "every supported language must route the directive: {prompt}",
        );
    }
    // Every supported language also has its own localized acknowledgement, so no
    // locale silently falls back to English.
    for language in supported_languages() {
        assert!(
            response_for("learn_from_source", &language).is_some(),
            "the learn_from_source response must exist for language {language}",
        );
    }
}

// R499-3: routing is data-driven — it needs both a learning-directive cue *and* a
// reference to a seed-declared source. A directive alone, or a source alone, does
// not trigger it, and unrelated prompts are untouched.
#[test]
fn routing_requires_both_a_directive_and_a_known_source() {
    // A bare navigation request to the same host carries no learning cue → not ours.
    assert_ne!(
        answer("Open https://trends.google.com/trending?geo=US").intent,
        "learn_from_source",
        "a plain open/navigate request must not be treated as a learning directive",
    );
    // A learning cue with no known data source is not a learn-from-source request.
    assert_ne!(
        answer("You can learn a lot from reading books").intent,
        "learn_from_source",
        "a learning cue without a declared source must not trigger the handler",
    );
    // The registry actually declares the source the handler keys on, and the
    // production match is on the declared capability slug, never a literal URL.
    let registry = seed::learning_sources();
    let google = registry
        .sources
        .iter()
        .find(|source| source.id == "google_trends")
        .expect("the seed must declare the Google Trends learnable source");
    assert_eq!(google.capability, "google_trends_learning");
    assert_eq!(google.host, "trends.google.com");
    assert!(!registry.directive_cues.is_empty());
}

// R499-4: the acknowledgement is rendered in the prompt's language.
#[test]
fn the_acknowledgement_is_localized_to_the_prompt_language() {
    let russian = answer(REPORTED_PROMPT);
    assert!(
        russian.answer.contains("источник данных"),
        "a Russian directive should be acknowledged in Russian: {}",
        russian.answer,
    );
    let english = answer(
        "Here you can learn the current trending topics https://trends.google.com/trending?geo=US",
    );
    assert!(
        english.answer.contains("data source I can learn from"),
        "an English directive should be acknowledged in English: {}",
        english.answer,
    );
}

// R499-5 / whole task: the directive routes into the Google Trends learning
// frontier — the same human-gated, proposal-only loop the rest of the system uses
// — and the answer reflects that report faithfully (counts, and adopts nothing).
#[test]
fn the_directive_routes_into_the_human_gated_learning_frontier() {
    let report = trending_learning_report();
    assert_eq!(report.adopted_count(), 0, "the loop stays proposal-only");

    let result = answer(REPORTED_PROMPT);
    assert_eq!(result.intent, "learn_from_source");
    // The answer names Google Trends and quotes the report's own coverage split,
    // proving it is derived from trending_learning_report(), not a canned string.
    assert!(result.answer.contains("Google Trends"));
    assert!(result.answer.contains(&report.total_prompts.to_string()));
    assert!(result.answer.contains(&report.frontier_count().to_string()));
    assert!(
        result.answer.contains("human-gated"),
        "the answer must state the frontier flows into the human-gated loop: {}",
        result.answer,
    );
}

// R499-6: the *same* teaching directive that the chat handler recognizes also
// drives the Agent CLI learning recipe — one seed-declared source, two entry
// points. The directive routes into the same `write_file -> run_command -> final`
// loop the operator-worded task uses and writes the committed learning artifact.
#[test]
fn the_reported_directive_drives_the_learning_recipe_via_agent_cli() {
    // The literal user directive is enough to select the learning recipe; no
    // English operator framing ("learning frontier" / "self-improvement loop") is
    // required, because routing consults the seed `learning_sources` registry.
    assert!(
        is_google_trends_learning_task(REPORTED_PROMPT),
        "the reported learn-from-source directive must drive the Agent CLI learning recipe",
    );
    // A learning cue with no declared source must not select the recipe, so the
    // Agent CLI stays as data-driven as the chat handler.
    assert!(!is_google_trends_learning_task(
        "Here you can learn a lot from reading books"
    ));

    let outcome = run_agentic_task(REPORTED_PROMPT).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("the directive must drive a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["path"], GOOGLE_TRENDS_LEARNING_PATH);
    assert_eq!(written["content"], recipe::render_document());
}

// R499-7: the Agent CLI session driven by the reported directive is pinned
// byte-for-byte, so a routing or report change can never silently drift the
// committed transcript the case study links to.
#[test]
fn committed_agent_cli_session_matches_a_fresh_learn_from_source_run() {
    let committed =
        include_str!("../../docs/case-studies/issue-499/agent-cli-session-learn-from-source.json");
    let fresh = run_agentic_task(REPORTED_PROMPT).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed issue-499 learn-from-source Agent CLI session is stale; regenerate it with \
         `cargo run --example issue_499_dump_agent_cli_session`",
    );
}
