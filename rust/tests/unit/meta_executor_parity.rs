//! Issue #1550: the native recursive procedure emits the same event sequence
//! as the browser (JavaScript) one for the same request and host, on the
//! shared fixture `rust/tests/fixtures/meta-executor-parity.json`
//! (acceptance criterion e).
//!
//! The fixture is written from the JavaScript module by
//! `node experiments/issue-1550/parity-fixture.mjs --write`; the JavaScript
//! test `rust/tests/web/issue-1550-meta-executor.test.mjs` replays the same
//! file, so a drift on either side fails one of the suites.

use formal_ai::meta_executor::{
    BoundOverrides, Draft, MetaHost, Request, RunOptions, TaskNode, decomposition_shape,
    execute_procedure, task_leaves,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("../fixtures/meta-executor-parity.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("parity fixture is JSON")
}

fn field<'a>(entry: &'a Value, name: &str) -> &'a str {
    entry[name]
        .as_str()
        .unwrap_or_else(|| panic!("fixture entry lacks string field {name:?}"))
}

#[allow(clippy::cast_possible_truncation)]
fn draft_of(row: &Value) -> Option<Draft> {
    if !row.is_object() {
        return None;
    }
    Some(Draft {
        intent: field(row, "intent").to_owned(),
        content: field(row, "content").to_owned(),
        confidence: row["confidence"].as_f64().map_or(0.0, |value| value as f32),
    })
}

/// The table host of one case: `answers[text]` is the direct derivation,
/// `cached[text]` the stored row.
struct TableHost<'a> {
    entry: &'a Value,
}

impl MetaHost for TableHost<'_> {
    fn draft(&mut self, text: &str, _task: &TaskNode) -> Option<Draft> {
        draft_of(&self.entry["answers"][text])
    }

    fn cached(&mut self, text: &str, _task: &TaskNode) -> Option<Draft> {
        draft_of(&self.entry["cached"][text])
    }
}

fn bound(entry: &Value, name: &str) -> Option<u32> {
    entry["bounds"][name]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("fixture array")
        .iter()
        .map(|item| item.as_str().expect("fixture string").to_owned())
        .collect()
}

#[test]
fn every_fixture_case_replays_to_the_javascript_event_sequence() {
    let fixture = fixture();
    assert_eq!(fixture["schema"], "meta-executor-parity/v1");
    let cases = fixture["cases"].as_array().expect("fixture cases");
    assert!(cases.len() >= 10, "the fixture covers the acceptance cases");
    for entry in cases {
        let name = field(entry, "name");
        let options = RunOptions {
            bounds: BoundOverrides {
                depth: bound(entry, "depth"),
                breadth: bound(entry, "breadth"),
                budget: bound(entry, "budget"),
            },
            withheld_tests: strings(&entry["withheld"]),
        };
        let mut host = TableHost { entry };
        let run = execute_procedure(
            Request {
                text: field(entry, "text"),
                language: field(entry, "language"),
            },
            &mut host,
            &options,
        );
        let expected = &entry["expected"];
        assert_eq!(
            decomposition_shape(&run.root),
            field(expected, "shape"),
            "{name}: shape"
        );
        assert_eq!(
            task_leaves(&run.root),
            strings(&expected["leaves"]),
            "{name}: leaves"
        );
        assert_eq!(run.outcome.status, field(expected, "status"), "{name}");
        assert_eq!(
            run.outcome
                .candidate
                .as_ref()
                .map(|candidate| candidate.draft.content.as_str()),
            expected["content"].as_str(),
            "{name}: content"
        );
        let events: Vec<String> = run
            .events
            .iter()
            .map(|event| format!("{} {}", event.kind, event.payload))
            .collect();
        assert_eq!(events, strings(&expected["events"]), "{name}: events");
    }
}
