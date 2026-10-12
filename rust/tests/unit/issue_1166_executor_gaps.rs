//! Issue #1166 R1166-3/R1166-4: executors bind and report from the graph.
//!
//! The program executor's output operands come from the obligation graph's
//! clauses (`bound_output_literals`, read by
//! `program_contract::explicit_stdout`), and every executor that reads the
//! graph records `obligation_gap_lines` as `obligation_gap` events: the
//! clauses no rule can read and the output literals the graph demands but no
//! binding carries. The cases live in
//! `tests/fixtures/issue-1166/executor-gaps.json`, which the JavaScript twin
//! (`rust/tests/web/issue-1166-executor-gaps.test.mjs`) reads too, so both
//! roots are pinned to the same lines.

use formal_ai::EventLog;
use formal_ai::intent_formalization::{
    OBLIGATION_GAP_KIND, ObligationKind, bound_output_literals, formalize_request,
    obligation_gap_lines, record_obligation_gaps,
};
use formal_ai::program_contract;
use serde_json::Value;

const CASES: &str = include_str!("../fixtures/issue-1166/executor-gaps.json");

/// The `(id, prompt, expected lines)` triples of one fixture section.
fn cases(section: &str) -> Vec<(String, String, Vec<String>)> {
    let root: Value = serde_json::from_str(CASES).expect("the fixture is JSON");
    root[section]
        .as_array()
        .expect("the fixture section is an array")
        .iter()
        .map(|case| {
            let text = |key: &str| case[key].as_str().expect("a string field").to_owned();
            let expected = case["expected"]
                .as_array()
                .expect("expected is an array")
                .iter()
                .map(|line| line.as_str().expect("a string line").to_owned())
                .collect();
            (text("id"), text("prompt"), expected)
        })
        .collect()
}

/// The `obligation_gap` payloads a log carries, in order.
fn gap_events(log: &EventLog) -> Vec<String> {
    log.events()
        .iter()
        .filter(|event| event.kind == OBLIGATION_GAP_KIND)
        .map(|event| event.payload.clone())
        .collect()
}

/// Every binding case: the literals the executor binds, in request order.
#[test]
fn output_literals_are_bound_from_the_obligation_clauses() {
    let binding = cases("bindingCases");
    assert_eq!(binding.len(), 4, "every binding case is read");
    for (id, prompt, expected) in binding {
        assert_eq!(
            bound_output_literals(&prompt),
            expected,
            "binding case {id}"
        );
        let joined = (!expected.is_empty()).then(|| expected.join("\n"));
        assert_eq!(
            program_contract::explicit_stdout(&prompt),
            joined,
            "explicit_stdout case {id}"
        );
    }
}

/// Every gap case: the exact lines, recorded one event each.
#[test]
fn executors_record_every_obligation_gap_line() {
    let gaps = cases("gapCases");
    assert_eq!(gaps.len(), 2, "every gap case is read");
    for (id, prompt, expected) in gaps {
        assert_eq!(obligation_gap_lines(&prompt), expected, "gap case {id}");
        let mut log = EventLog::default();
        assert_eq!(record_obligation_gaps(&prompt, &mut log), expected);
        assert_eq!(gap_events(&log), expected, "recorded events of {id}");
    }
}

/// R1166-4: a literal the graph demands but the binding does not carry is
/// reported against its own node, never silently dropped from the program.
#[test]
fn unbound_output_literal_is_reported_against_its_node() {
    let prompt = "Write a Python program that prints \"A\"\nThen write \"B\"";
    let graph = formalize_request(prompt);
    let bound = bound_output_literals(prompt);
    assert_eq!(bound, ["A"]);
    let unbound = graph.unbound_output_report(&bound);
    assert_eq!(unbound.len(), 1, "only the unbound literal is reported");
    let node = graph
        .obligations_of_kind(ObligationKind::OutputLiteral)
        .into_iter()
        .find(|node| graph.literal_for(node) == Some("B"))
        .expect("the graph demands the second literal");
    assert!(unbound[0].contains(&node.node_id), "{}", unbound[0]);
    assert!(
        graph
            .unbound_output_report(&["A".to_owned(), "B".to_owned()])
            .is_empty(),
        "a fully bound graph reports no unbound literal"
    );
}

/// The program-contract executor records the same lines in its run.
#[test]
fn program_contract_run_carries_the_gap_lines() {
    let prompt = "Write a Python program that prints \"A\"\nThen write \"B\"";
    let mut log = EventLog::default();
    let answer = program_contract::answer(prompt, &mut log).expect("a program contract answer");
    assert!(answer.execution_recipe.is_some());
    assert_eq!(gap_events(&log), obligation_gap_lines(prompt));
}

/// The classifier's vocabularies are seed roles, so the canonical body's
/// style, naming and badge clauses still formalize with no code-side table.
#[test]
fn authoring_kinds_are_read_from_seed_roles() {
    let body = concat!(
        "First, add clear comments\n",
        "Also, use a meaningful name for the workflow file\n",
        "Finally, a CI badge\n",
    );
    let graph = formalize_request(body);
    assert_eq!(
        graph.obligation_kinds(),
        [
            ObligationKind::CodeStyle,
            ObligationKind::FileNaming,
            ObligationKind::CiBadge
        ]
    );
}

/// R1166-4: a catalog `write_program` request that carries work or CI
/// obligations records the fixture's lines in its run, the lines the browser
/// worker records for the same request
/// (`rust/tests/web/issue-1166-worker-obligation-gaps.test.mjs`); a plain
/// catalog request records none.
#[test]
fn catalog_requests_record_the_worker_gap_lines() {
    let catalog = cases("catalogCases");
    assert_eq!(catalog.len(), 6, "every catalog case is read");
    for (id, prompt, expected) in catalog {
        let response = formal_ai::UniversalSolver::default().solve(&prompt);
        if expected.is_empty() {
            assert!(
                !response.links_notation.contains(OBLIGATION_GAP_KIND),
                "catalog case {id} records no gap: {}",
                response.links_notation
            );
            continue;
        }
        assert_eq!(obligation_gap_lines(&prompt), expected, "catalog case {id}");
        for gap in &expected {
            assert!(
                response
                    .links_notation
                    .contains(&format!("{OBLIGATION_GAP_KIND} {gap}")),
                "catalog case {id}: {gap} is missing from {}",
                response.links_notation
            );
        }
    }
}
