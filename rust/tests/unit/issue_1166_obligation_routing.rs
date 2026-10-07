//! Issue #1166 (E131): the request is formalized into obligations instead of
//! matched phrases.
//!
//! Covers request formalization on the canonical Kotlin Hello World issue
//! body, the coreference pass that repairs the doubled output of #1156,
//! multilingual and paraphrase parity of the obligation graph, the
//! never-discarded `Underivable` reporting, and the terminal-routing guard
//! that stops a leading shell token from claiming a sentence about building
//! something. The issue's three planned modules
//! (`issue_1166_request_formalization`, `issue_1166_coreference`,
//! `issue_1166_multilingual_parity`) consolidate here because the graph, the
//! merge, and the parity fixtures exercise one module:
//! `intent_formalization::obligations`.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::intent_formalization::{
    ObligationKind, formalize_request, request_carries_work_obligations,
};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue-1166")
}

fn fixture(name: &str) -> String {
    fs::read_to_string(fixture_dir().join(name)).unwrap_or_else(|error| {
        panic!("fixture {name} missing under tests/fixtures/issue-1166: {error}")
    })
}

const CANONICAL: &str = "hello-world-kotlin-en.txt";

/// The canonical body yields the output literal the issue pins: the value
/// appears once even though the body quotes it three times (intro, "print
/// exactly", Expected Output) — the structural fix for the doubled output of
/// #1156.
#[test]
fn canonical_yields_unique_output_literal() {
    let graph = formalize_request(&fixture(CANONICAL));
    assert_eq!(
        graph.unique_output_literal(),
        Some("Hello, World!"),
        "the quoted literal must be anchored exactly once"
    );
}

/// R1166-2: mentions of the same quoted literal collapse to one node, so the
/// output-literal node count is one for three mentions.
#[test]
fn two_identical_output_clauses_produce_one_node() {
    let graph = formalize_request(&fixture(CANONICAL));
    let output_nodes = graph.obligations_of_kind(ObligationKind::OutputLiteral);
    assert_eq!(
        output_nodes.len(),
        1,
        "coreference must merge the repeated literal mentions"
    );
}

/// Distinct literals stay distinct nodes, and the graph reports no *unique*
/// literal when two different values are demanded.
#[test]
fn two_output_clauses_with_distinct_literals_produce_two_nodes() {
    let body = "First, print exactly: \"Alpha\"\nThen, print exactly: \"Beta\"\n";
    let graph = formalize_request(body);
    assert_eq!(
        graph
            .obligations_of_kind(ObligationKind::OutputLiteral)
            .len(),
        2,
        "distinct literals must not merge"
    );
    assert_eq!(
        graph.unique_output_literal(),
        None,
        "two distinct demanded values means no unique literal"
    );
}

/// A filename clause is a naming obligation, never an output literal, so the
/// coreference pass cannot merge it with the output-literal node even when
/// the strings coincide.
#[test]
fn coreference_does_not_merge_filename_with_output_literal() {
    let body = concat!(
        "First, print exactly: \"test-hello-world.yml\"\n",
        "Also, use a meaningful name for the workflow file\n"
    );
    let graph = formalize_request(body);
    assert_eq!(
        graph
            .obligations_of_kind(ObligationKind::OutputLiteral)
            .len(),
        1
    );
    assert_eq!(
        graph.obligations_of_kind(ObligationKind::FileNaming).len(),
        1
    );
    let naming = graph
        .obligations_of_kind(ObligationKind::FileNaming)
        .pop()
        .expect("naming node");
    assert!(
        graph.literal_for(naming).is_none(),
        "a naming node carries no output literal"
    );
}

/// The canonical body yields the CI-workflow obligation the old phrase match
/// looked for — now derived from the graph, with the role mention as an
/// internal detail of the classifier.
#[test]
fn canonical_yields_ci_workflow_node() {
    let graph = formalize_request(&fixture(CANONICAL));
    assert!(
        graph.has_obligation(ObligationKind::CiWorkflow),
        "the GitHub Actions clause must formalize to a CI obligation"
    );
}

/// The canonical body yields the file-naming obligation the executor used to
/// miss entirely (it wrote `run.yml` instead of `test-hello-world.yml`).
#[test]
fn canonical_yields_file_naming_node() {
    let graph = formalize_request(&fixture(CANONICAL));
    assert!(
        graph.has_obligation(ObligationKind::FileNaming),
        "the meaningful-name clause must formalize to a naming obligation"
    );
}

/// Style and badge clauses formalize too, so "add clear comments" and the
/// optional badge are no longer silently dropped requirements.
#[test]
fn canonical_yields_style_and_badge_nodes() {
    let graph = formalize_request(&fixture(CANONICAL));
    assert!(graph.has_obligation(ObligationKind::CodeStyle));
    assert!(graph.has_obligation(ObligationKind::CiBadge));
}

/// R1166-5: a clause nothing classifies keeps its node and its `Underivable`
/// expectation with a byte span — reported, never discarded (R710-R9).
#[test]
fn unknown_requirement_clause_yields_underivable_node_not_panic() {
    let body = "First, print exactly: \"Hi\"\nThen, harmonize the quantum flux\n";
    let graph = formalize_request(body);
    let underivable = graph.underivable();
    assert!(
        !underivable.is_empty(),
        "the unreadable clause must stay in the graph"
    );
    assert!(
        underivable.iter().all(|node| node.span.1 > node.span.0),
        "every reported gap carries its byte span"
    );
    assert!(
        graph
            .gap_report()
            .iter()
            .all(|line| line.contains("underivable")),
        "gap lines name the reason"
    );
}

/// R1166-6: a translated body (same meaning, ru surface) yields the same
/// obligation count and the same output literal as English.
#[test]
fn kotlin_issue_ru_yields_same_obligation_count_as_en() {
    let en = formalize_request(&fixture(CANONICAL));
    let ru = formalize_request(&fixture("hello-world-kotlin-ru.txt"));
    assert_eq!(ru.nodes().len(), en.nodes().len());
    assert_eq!(
        ru.unique_output_literal(),
        en.unique_output_literal(),
        "the literal is language-independent"
    );
}

/// Hindi, Chinese, and Spanish parity for the same body.
#[test]
fn kotlin_issue_hi_zh_es_yield_same_graph_modulo_language_tag() {
    let en = formalize_request(&fixture(CANONICAL));
    for language in ["hi", "zh", "es"] {
        let body = formalize_request(&fixture(&format!("hello-world-kotlin-{language}.txt")));
        assert_eq!(
            body.obligation_kinds(),
            en.obligation_kinds(),
            "{language} must formalize to the same obligation kinds"
        );
        assert_eq!(
            body.unique_output_literal(),
            en.unique_output_literal(),
            "{language} must anchor the same literal"
        );
    }
    assert_eq!(
        formalize_request(&fixture("hello-world-kotlin-ru.txt")).obligation_kinds(),
        en.obligation_kinds()
    );
}

/// R1166-7: a paraphrase of the canonical body produces the same obligation
/// kinds and the same literal.
#[test]
fn paraphrase_en_yields_same_obligation_count_as_canonical_en() {
    let canonical = formalize_request(&fixture(CANONICAL));
    let paraphrase = formalize_request(&fixture("hello-world-kotlin-paraphrase-en.txt"));
    assert_eq!(paraphrase.nodes().len(), canonical.nodes().len());
    assert_eq!(paraphrase.obligation_kinds(), canonical.obligation_kinds());
    assert_eq!(
        paraphrase.unique_output_literal(),
        canonical.unique_output_literal()
    );
}

/// The terminal guard: a request that formalizes into work obligations is not
/// a command line, including in languages whose marker words the English-only
/// argument check of #1175 does not know.
#[test]
fn work_obligation_requests_are_not_terminal_commands() {
    assert!(request_carries_work_obligations(
        "make a small Kotlin app that prints \"Hello, World!\"; add a GitHub Actions workflow"
    ));
    assert!(request_carries_work_obligations(
        "сделай приложение на Kotlin, которое выводит \"Привет\"; добавь workflow GitHub Actions"
    ));
    assert!(!request_carries_work_obligations("git status"));
    assert!(!request_carries_work_obligations("echo \"Hello, World!\""));
}

/// The graph renders as one Links Notation record, so parity fixtures and
/// event logs can pin expected graphs.
#[test]
fn graph_renders_links_notation() {
    let graph = formalize_request(&fixture(CANONICAL));
    let notation = graph.links_notation();
    assert!(notation.starts_with("request_obligation_graph"));
    assert!(notation.contains("kind output_literal"));
    assert!(notation.contains("kind ci_workflow"));
    assert!(notation.contains("literal \"Hello, World!\""));
}
