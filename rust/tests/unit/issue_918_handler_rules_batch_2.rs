//! Issue #918 (E71, R914-6), second migration file: the `diagnostic` prelude
//! row and the `coreference` row read only seed data in both runtimes.
//!
//! `diagnostic` recognized its per-message marker as a literal of
//! `rust/src/meta_method_dispatch.rs`. The marker is now the
//! `diagnostic_marker` table of `data/seed/handler-rules.lino`, matched in the
//! prompt itself and stripped from it, so recognition and stripping agree: an
//! upper-case marker used to be recognized in the lower-cased text but not
//! stripped from the prompt, so the inner solve met the same marker again.
//! A lower-case marker answers exactly as before.
//!
//! `coreference` already read its pronouns, antecedents and bodies from
//! `data/seed/coreference.lino`; its browser twin read only the last user turn
//! and answered only through a seeded fact. Both now resolve the nearest
//! earlier user turn naming an antecedent and answer the seeded body. The
//! browser pins are `rust/tests/web/issue-0918-handler-rules-batch-2.test.mjs`.
//!
//! `nl_tool` recognized its tool calls through seeded roles already; its seven
//! report and refusal templates are now seeded `nl_tool_*` responses. The
//! English answers below are byte-identical to the deleted format strings. The
//! refusals now speak the prompt's language (a deliberate change: a Russian
//! tool call used to be refused in English); the tool reports stay English.
//!
//! `installation_conversion` read its request cues, source and target
//! markers, prose function words and verb-to-step map from inline arrays and
//! wrote every sentence as a format string. They are now the
//! `installation_*` tables and policy of `data/seed/handler-rules.lino` and
//! seeded `installation_*` responses in both runtimes; every answer is
//! byte-identical, as the documented answers of
//! `rust/tests/unit/installation_conversion.rs` pin.
//!
//! `translation` recognized its requests through seeded roles already; its two
//! gap answers, the program-translation heading, the program gap comment and
//! the two clarifying questions of `rust/src/translation/selection.rs` are now
//! seeded English `translation_*` responses, byte-identical to the deleted
//! format strings (the gap answers stay pinned by
//! `rust/tests/unit/specification/translation_via_links.rs`).

use std::fmt::Write as _;

use formal_ai::rule_interpreter::{handler_table_rows, handler_table_value};
use formal_ai::web_search_core::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};
use formal_ai::{ConversationTurn, FormalAiEngine, SolverConfig, SymbolicAnswer, UniversalSolver};

use super::installation_conversion::{DocumentedFormat, documented_conversion_answer};

const RUST_BODY: &str = "`it` resolves to Rust from your prior turn. Compared with C, Rust adds ownership, borrowing, and stronger compile-time checks so memory-safety errors are caught before the program runs while retaining native-code performance.";

#[test]
fn the_diagnostic_marker_is_a_seed_table_row() {
    assert_eq!(
        handler_table_rows("diagnostic_marker"),
        [("[diagnostic]".to_owned(), "section".to_owned())]
    );
}

#[test]
fn a_marked_prompt_appends_its_trace_under_the_seeded_marker() {
    // The diagnostic row solves the unmarked prompt with a fresh solver of the
    // same configuration and appends its links, evidence and trace under the
    // marker, so the expected answer is assembled from that same solve.
    let inner = UniversalSolver::default().solve("Hi");
    let marked = UniversalSolver::default().solve("[diagnostic] Hi");
    let mut expected = format!(
        "{}\n\n[diagnostic]\n{}\n",
        inner.answer,
        inner.links_notation.trim_end()
    );
    for link in &inner.evidence_links {
        let _ = writeln!(expected, "evidence: {link}");
    }
    let _ = writeln!(expected, "trace: {}", inner.intent);
    assert_eq!(marked.intent, inner.intent);
    assert_eq!(marked.answer, expected);
    assert!(
        marked
            .evidence_links
            .iter()
            .any(|link| link == "diagnostic_mode:active")
    );
}

#[test]
fn an_upper_case_marker_is_not_the_seeded_marker() {
    // Recognition and stripping read the same exact seeded marker, so an
    // upper-case one is ordinary prompt text and the turn is no diagnostic.
    let response = UniversalSolver::default().solve("[DIAGNOSTIC] Hi");
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link == "diagnostic_mode:active"),
        "{:?}",
        response.evidence_links
    );
}

#[test]
fn a_pronoun_answers_the_seeded_body_of_the_nearest_antecedent() {
    let history = [
        ConversationTurn::user("I love Rust."),
        ConversationTurn::assistant("Rust is a systems programming language."),
        ConversationTurn::user("The weather is pleasant."),
        ConversationTurn::assistant("It is."),
    ];
    let response =
        UniversalSolver::default().solve_with_history("Why is it safer than C?", &history);
    assert_eq!(response.intent, "coreference_rust");
    assert_eq!(response.answer, RUST_BODY);
    // The native log keeps the resolution event by id; the browser spells its payload.
    for link in [
        "coreference:resolved:",
        "wikidata:Q575650",
        "response:coreference",
    ] {
        assert!(
            response
                .evidence_links
                .iter()
                .any(|candidate| candidate.starts_with(link)),
            "{link} in {:?}",
            response.evidence_links
        );
    }
}

#[test]
fn a_pronoun_without_an_earlier_antecedent_is_not_a_coreference() {
    let history = [
        ConversationTurn::user("The weather is pleasant."),
        ConversationTurn::assistant("It is."),
    ];
    let response =
        UniversalSolver::default().solve_with_history("Why is it safer than C?", &history);
    assert!(
        !response.intent.starts_with("coreference"),
        "{}",
        response.intent
    );
}

/// A seeded context keeps its spaces (PR #1188 T470): `" program "` never
/// matches "programming", so a search request is no coreference.
#[test]
fn a_word_that_only_contains_a_pronoun_is_not_a_coreference() {
    let history = [
        ConversationTurn::user("Find detailed information about Rust programming"),
        ConversationTurn::assistant("Search results for Rust programming."),
    ];
    let response = UniversalSolver::default()
        .solve_with_history("Rust programming के बारे में जानकारी खोजो", &history);
    assert!(
        !response.intent.starts_with("coreference"),
        "{}",
        response.intent
    );
}

fn agent_answer(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
    .solve(prompt)
}

#[test]
fn a_tool_call_outside_agent_mode_is_refused_from_the_seed() {
    let response = UniversalSolver::default().solve("Call the calculator API with `2 + 2`");
    assert_eq!(response.intent, "tool_call_refused");
    assert_eq!(
        response.answer,
        "Execution status: refused. Natural-language tool calls require explicit agent mode before `tool:calculator` can run."
    );
}

#[test]
fn a_russian_tool_call_is_refused_in_russian() {
    let response = UniversalSolver::default().solve("Вызови инструмент калькулятор с `2 + 2`");
    assert_eq!(response.intent, "tool_call_refused");
    assert_eq!(
        response.answer,
        "Статус выполнения: отказ. Вызовы инструментов на естественном языке требуют явно включённого режима агента, прежде чем `tool:calculator` сможет запуститься."
    );
}

#[test]
fn an_allowed_calculator_call_renders_the_seeded_report() {
    let response = agent_answer("Call the calculator API with `2 + 2`");
    assert_eq!(response.intent, "natural_language_api_call");
    assert_eq!(
        response.answer,
        "Execution status: executed.\nTool call: calculator\nInput: `2 + 2`\nResult: 4"
    );
}

#[test]
fn an_allowed_web_search_call_renders_the_seeded_plan() {
    let response = agent_answer("Call the web_search API with query `Rust ownership`");
    let providers = WEB_SEARCH_PROVIDERS.join(", ");
    let expected = format!(
        "Execution status: planned.\nTool call: web_search\nQuery: `Rust ownership`\nResult: search plan recorded with providers {providers}; combined ranking uses reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})."
    );
    assert_eq!(response.intent, "natural_language_api_call");
    assert_eq!(response.answer, expected);
}

#[test]
fn an_unpermitted_tool_names_the_package_it_needs() {
    let response = agent_answer("Call the local_shell tool with `ls`");
    assert_eq!(response.intent, "tool_call_refused");
    assert_eq!(
        response.answer,
        "Execution status: refused. Tool calls are not allowed for `tool:local_shell`: no installed associative package grants tool:local_shell. Install or import an associative package that grants this capability before enabling the tool."
    );
}

#[test]
fn installation_steps_are_described_through_the_seeded_verb_table() {
    assert_eq!(
        handler_table_value("installation_verb_action", "pushd"),
        Some("enter_directory")
    );
    assert_eq!(
        handler_table_value("installation_verb_action", "launch"),
        Some("run")
    );
    assert_eq!(
        handler_table_value("installation_verb_action", "deno"),
        None
    );
}

#[test]
fn an_installation_guide_converts_to_both_scripts_from_the_seed_tables() {
    let prompt = "Convert this installation guide into both sh and PowerShell scripts:\n- `make`\n- `./app --help`\n- `deno`";
    assert_eq!(
        FormalAiEngine.answer(prompt).answer,
        documented_conversion_answer(
            DocumentedFormat::Markdown,
            &[DocumentedFormat::Shell, DocumentedFormat::PowerShell],
            "the project",
            &[
                ("Build the project", "make"),
                ("Verify the installation", "./app --help"),
                ("Run deno", "deno"),
            ],
        )
    );
}

#[test]
fn translation_wording_is_the_seeded_english_responses() {
    for (intent, text) in [
        (
            "translation_gap_no_source",
            "I could not identify a source phrase to translate from {source} to {target}.",
        ),
        (
            "translation_gap_surface",
            "I could not translate \"{surface}\" from {source} to {target} with the available formalization data. I recorded this as a translation gap for follow-up.",
        ),
        (
            "translation_program_heading",
            "Translated `{code}` from {source} to {target}:",
        ),
        (
            "translation_code_gap_comment",
            "translation gap for `{subject}` from {source} to {target}",
        ),
        (
            "translation_clarify_interpretation",
            "Which interpretation did you mean: {top} or {other}?",
        ),
        (
            "translation_clarify_reading",
            concat!(
                "Should I read \"",
                "{text}",
                "\" as \"",
                "{top}",
                "\" or \"",
                "{other}",
                "\"?"
            ),
        ),
    ] {
        assert_eq!(
            formal_ai::seed::localized_response(intent, "en").as_deref(),
            Some(text),
            "{intent}"
        );
    }
}
