//! Issue #1175 R3: claim routing through the capability table.
//!
//! The `claim` rows of `data/seed/capability-routing.lino` are consulted after
//! formalization and before a handler runs: a handler with a row is offered
//! the prompt only when one of its `admits_on` evidence kinds holds in the
//! prompt's structure, so a surface word alone cannot claim it. Mirrored by
//! `rust/tests/web/issue-1175-claim-routing.test.mjs`.

use formal_ai::capability_routing::{
    CLAIM_EVIDENCE_KINDS, ClaimRow, claim_admitted, claim_evidence_holds, claim_rows,
    claim_rows_from,
};

#[test]
fn the_claim_rows_are_read_from_the_capability_table() {
    let rows: Vec<(&str, &str, Vec<&str>)> = claim_rows()
        .iter()
        .map(|row| {
            (
                row.handler.as_str(),
                row.browser_handler.as_str(),
                row.admits_on.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                "software_project",
                "trySoftwareProjectRequest",
                vec!["object_phrase_artifact", "approval_of_a_proposal"],
            ),
            (
                "terminal_command",
                "tryTerminalCommand",
                vec!["shell_command_shape", "semantic_shell_task"],
            ),
            ("repository_lineage", "", vec!["repository_subject"]),
        ]
    );
}

#[test]
fn every_row_names_a_known_handler_and_known_evidence() {
    let precedence = formal_ai::seed::handler_precedence();
    for row in claim_rows() {
        assert!(
            row.handler == "terminal_command" || precedence.iter().any(|name| name == &row.handler),
            "claim row names a handler the precedence table does not: {row:?}"
        );
        assert!(
            !row.admits_on.is_empty(),
            "a row admits on something: {row:?}"
        );
        for kind in &row.admits_on {
            assert!(
                CLAIM_EVIDENCE_KINDS.contains(&kind.as_str()),
                "unknown evidence kind `{kind}` in {row:?}"
            );
            assert!(claim_evidence_holds(kind, "ls -la", "ls -la").is_some());
        }
        assert!(
            !row.because.is_empty(),
            "every row states its reason: {row:?}"
        );
    }
    assert_eq!(claim_evidence_holds("surface_word", "ls", "ls"), None);
}

#[test]
fn a_surface_word_alone_is_not_admitted() {
    for (handler, prompt) in [
        (
            "terminal_command",
            "Make a 3-day itinerary for a first visit to Rome.",
        ),
        (
            "terminal_command",
            "Find the bug: def average(xs): return sum(xs) / len(xs)",
        ),
        (
            "software_project",
            "Write a regex for a US ZIP code with an optional 4-digit extension",
        ),
        ("repository_lineage", "Which issue introduced this feature?"),
    ] {
        assert!(
            !claim_admitted(handler, prompt, &prompt.to_lowercase()),
            "`{handler}` must not be offered `{prompt}`"
        );
    }
}

#[test]
fn the_structural_evidence_admits() {
    for (handler, prompt) in [
        ("terminal_command", "find . -name '*.log' -size +10M"),
        ("software_project", "Build a web app for tracking habits"),
        (
            "repository_lineage",
            "Which issue introduced scripts/check-self-development-release.rs?",
        ),
        (
            "repository_lineage",
            "Why does the self-development status fail?",
        ),
        ("repository_lineage", "What does `evaluate_calculation` do?"),
    ] {
        assert!(
            claim_admitted(handler, prompt, &prompt.to_lowercase()),
            "`{handler}` must be offered `{prompt}`"
        );
    }
}

#[test]
fn a_handler_without_a_row_is_admitted_and_a_fixture_row_routes_without_code() {
    assert!(claim_admitted(
        "concept_lookup",
        "What is a monad?",
        "what is a monad?"
    ));
    let fixture = "capability_routing\n  claim\n    handler concept_lookup\n    admits_on shell_command_shape\n    because \"fixture\"\n";
    assert_eq!(
        claim_rows_from(fixture),
        vec![ClaimRow {
            handler: "concept_lookup".to_owned(),
            browser_handler: String::new(),
            admits_on: vec!["shell_command_shape".to_owned()],
            because: "fixture".to_owned(),
        }]
    );
}
