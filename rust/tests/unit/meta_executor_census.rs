//! Issue #1550 (acceptance criterion c): no answer of the specification corpus
//! skips the recursive procedure in the native runtime.
//!
//! The fixture `rust/tests/fixtures/procedure-census-prompts.json` holds every
//! prompt the specification corpus asks (written by
//! `node experiments/issue-1550/census-fixture.mjs --write`; the JavaScript test
//! `rust/tests/web/issue-1550-procedure-census.test.mjs` fails when it is
//! stale). Every prompt is solved by the production entry and the trace of the
//! turn must end with the procedure's `meta_exec:result` event, so a new early
//! return that bypasses `run_meta_procedure` fails here.

use formal_ai::UniversalSolver;
use formal_ai::meta_executor::{last_turn_trace, remember_trace};
use serde_json::Value;

const FIXTURE: &str = include_str!("../fixtures/procedure-census-prompts.json");

#[test]
fn every_specification_prompt_runs_the_recursive_procedure() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("census fixture is JSON");
    assert_eq!(fixture["schema"], "procedure-census-prompts/v1");
    let prompts = fixture["prompts"]
        .as_array()
        .expect("census fixture lists prompts");
    assert!(!prompts.is_empty(), "the census fixture lists no prompt");
    let solver = UniversalSolver::default();
    let mut bypassed = Vec::new();
    for prompt in prompts.iter().filter_map(Value::as_str) {
        remember_trace(&[]);
        let _answer = solver.solve(prompt);
        let trace = last_turn_trace();
        if !trace.iter().any(|event| event.kind == "meta_exec:result") {
            bypassed.push(prompt.to_owned());
        }
    }
    assert!(
        bypassed.is_empty(),
        "{} specification prompt(s) were answered without the recursive procedure; \
         make the new early return a method of the cascade the procedure records: {bypassed:?}",
        bypassed.len(),
    );
}
