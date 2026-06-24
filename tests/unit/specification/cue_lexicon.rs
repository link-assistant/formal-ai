//! Issue #559 (R341): natural-language recognition cues as grounded link data.
//!
//! The meta core's first step turns a message into a problem frame, recognizing
//! which handler family a phrase points at. Those cue lists used to be inline Rust
//! string literals in `intent_formalization` (arithmetic operators, web-search
//! verbs, the fourteen text-manipulation operations, calendar fallback verbs, …).
//! R341 lifts them into `data/meta/cue-lexicon.lino`. These tests keep the data
//! grounded and prove the migration is behavior-preserving (R13):
//!
//! 1. every cue set the Rust code consults exists in the data, with a known mode;
//! 2. the migrated cue *contents* are exactly the lists they replaced (pinned);
//! 3. the match modes behave correctly — token matching keeps "book" from matching
//!    inside "books", substring matching catches embedded operations, prefix
//!    matching anchors at the start;
//! 4. routing is unchanged: representative prompts still surface the same handler
//!    relevants they did before the cues moved out of Rust.

use formal_ai::cue_lexicon::{cue_set, cues, matches, CueMatch};
use formal_ai::intent_formalization::formalize_intent;
use formal_ai::translation::formalize_prompt;

/// Every cue set the migrated Rust call sites look up, with the mode it must carry.
const CONSULTED_SETS: &[(&str, CueMatch)] = &[
    ("execution_failure_prompt", CueMatch::Substring),
    ("execution_failure_normalized", CueMatch::Substring),
    ("arithmetic_operators", CueMatch::Substring),
    ("web_search", CueMatch::Token),
    ("procedural_how_to", CueMatch::Prefix),
    ("proof_request", CueMatch::Token),
    ("write_script", CueMatch::Token),
    ("software_project", CueMatch::Token),
    ("concept_lookup", CueMatch::Prefix),
    ("calendar_fallback_verbs", CueMatch::Token),
    ("calendar_digit_actions", CueMatch::Token),
    ("calendar_ru_date_marker", CueMatch::Substring),
    ("text_manipulation", CueMatch::Substring),
];

#[test]
fn every_consulted_cue_set_is_present_with_the_expected_mode() {
    for (name, mode) in CONSULTED_SETS {
        let set = cue_set(name)
            .unwrap_or_else(|| panic!("cue set `{name}` must exist in data/meta/cue-lexicon.lino"));
        assert_eq!(
            set.match_mode,
            *mode,
            "cue set `{name}` must use the {} match mode the Rust call site relies on",
            mode.slug()
        );
        assert!(
            !set.cues.is_empty(),
            "cue set `{name}` must list at least one cue"
        );
        assert!(
            !set.handler.is_empty(),
            "cue set `{name}` must name the handler family it recognizes"
        );
    }
}

#[test]
fn migrated_cue_contents_match_the_lists_they_replaced() {
    // These are the exact literals that used to live in append_prompt_relevants /
    // looks_arithmetic / looks_like_text_manipulation. Pinning them here is the
    // behavior-preservation contract: the data must reproduce the old Rust lists.
    assert_eq!(
        cues("arithmetic_operators"),
        ["+", "-", "*", "/", "plus", "minus", "times", "divided"]
    );
    assert_eq!(cues("web_search"), ["search", "google", "find"]);
    assert_eq!(cues("procedural_how_to"), ["how to "]);
    assert_eq!(cues("proof_request"), ["prove", "proof"]);
    assert_eq!(cues("write_script"), ["script", "code"]);
    assert_eq!(
        cues("software_project"),
        ["build", "create", "implement", "develop"]
    );
    assert_eq!(cues("concept_lookup"), ["what is ", "define "]);
    assert_eq!(
        cues("calendar_fallback_verbs"),
        ["забей", "поставь", "schedule", "book"]
    );
    assert_eq!(cues("calendar_digit_actions"), ["schedule", "book", "add"]);
    assert_eq!(cues("calendar_ru_date_marker"), ["число"]);
    assert_eq!(cues("execution_failure_prompt"), ["undefined_function"]);
    assert_eq!(cues("execution_failure_normalized"), ["undefined function"]);
    assert_eq!(
        cues("text_manipulation"),
        [
            "uppercase",
            "lowercase",
            "replace",
            "remove text",
            "append text",
            "prepend text",
            "extract email",
            "count occurrences",
            "count unique words",
            "deduplicate lines",
            "sort lines",
            "trim whitespace",
            "normalize whitespace",
            "reverse words",
        ]
    );
}

#[test]
fn token_mode_respects_word_boundaries() {
    // The reason calendar verbs use token mode: "book" must not match inside "books"
    // (e.g. a "free-programming-books" mention), but must match as a standalone word.
    assert!(matches("calendar_fallback_verbs", "book a meeting"));
    assert!(!matches(
        "calendar_fallback_verbs",
        "free programming books"
    ));
    // Substring mode, by contrast, catches embedded operations as the old code did.
    assert!(matches("text_manipulation", "please uppercase this"));
    // Prefix mode anchors at the start: "what is" only as an opener.
    assert!(matches("concept_lookup", "what is a monad"));
    assert!(!matches("concept_lookup", "tell me what is a monad"));
}

#[test]
fn a_missing_set_never_matches() {
    assert!(!matches("no_such_cue_set", "anything at all"));
    assert!(cues("no_such_cue_set").is_empty());
}

/// The relevants list is what routing consumes; if a handler cue fires it appears
/// as `handler:<name>`. Asserting these proves the data-sourced cues drive routing
/// exactly as the old inline literals did.
fn relevants_for(prompt: &str) -> Vec<String> {
    let candidate = formalize_prompt(prompt, "en");
    formalize_intent(prompt, "en", Some(&candidate)).relevants
}

#[test]
fn routing_is_unchanged_for_representative_prompts() {
    let cases: &[(&str, &str)] = &[
        ("what is 2 + 2", "handler:arithmetic"),
        ("search the web for rust releases", "handler:web_search"),
        ("how to bake bread", "handler:procedural_how_to"),
        ("prove that 2 is even", "handler:proof_request"),
        ("write a script to sort files", "handler:write_script"),
        ("build a todo app", "handler:software_project"),
        ("what is a monad", "handler:concept_lookup"),
        ("uppercase this sentence", "handler:text_manipulation"),
    ];
    for (prompt, expected) in cases {
        let relevants = relevants_for(prompt);
        assert!(
            relevants.iter().any(|r| r == expected),
            "prompt {prompt:?} must surface {expected}; got {relevants:?}"
        );
    }
}

#[test]
fn book_word_boundary_does_not_misroute_a_books_link() {
    // Regression mirror of the calendar token-boundary comment: the word "book"
    // inside "books" must not promote a calendar create-event handler.
    let relevants = relevants_for("a list of free programming books to read");
    assert!(
        !relevants
            .iter()
            .any(|r| r == "handler:calendar_create_event"),
        "an embedded 'book' must not trigger calendar routing; got {relevants:?}"
    );
}
