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

const STATUS_FIXTURE: &str = include_str!("../fixtures/meta-executor-status-parity.json");
const CUE_FIXTURE: &str = include_str!("../fixtures/meta-executor-cue-parity.json");

fn fixture() -> Value {
    let mut fixture: Value = serde_json::from_str(FIXTURE).expect("parity fixture is JSON");
    for (name, source) in [("status", STATUS_FIXTURE), ("cue", CUE_FIXTURE)] {
        let supplement: Value = serde_json::from_str(source).expect(name);
        assert_eq!(supplement["schema"], fixture["schema"]);
        fixture["cases"]
            .as_array_mut()
            .expect("fixture cases")
            .extend(
                supplement["cases"]
                    .as_array()
                    .expect("supplement cases")
                    .iter()
                    .cloned(),
            );
    }
    fixture
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
        evidence: row.get("evidence").map_or_else(Vec::new, strings),
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

#[test]
fn production_composition_preserves_concrete_output_obligations() {
    for intent in [
        "write_program_skill_gap",
        "write_program_unspecified",
        "write_program",
    ] {
        let root = Draft {
            intent: intent.to_owned(),
            content: "Source unavailable".to_owned(),
            confidence: 0.0,
            evidence: Vec::new(),
        };
        let mut child = |_: &str| {
            Some(Draft {
                intent: "read_file".to_owned(),
                content: "A reading tool is needed".to_owned(),
                confidence: 1.0,
                evidence: Vec::new(),
            })
        };
        let run = formal_ai::meta_executor::solve_answer(
            Request {
                text: "Create source and import assert",
                language: "en",
            },
            root,
            &mut child,
            &RunOptions::default(),
        )
        .expect("outer production turn");
        assert!(
            formal_ai::meta_executor::adopted_composition(&run).is_none(),
            "{intent}"
        );
        assert!(
            run.events
                .iter()
                .any(|event| event.kind == "meta_exec:compose")
        );
    }
}

#[test]
fn production_composition_still_solves_untyped_independent_tasks() {
    let mut child = |text: &str| {
        Some(Draft {
            intent: "answered".to_owned(),
            content: text.to_owned(),
            confidence: 1.0,
            evidence: Vec::new(),
        })
    };
    let run = formal_ai::meta_executor::solve_answer(
        Request {
            text: "First task and second task",
            language: "en",
        },
        Draft {
            intent: "unknown".to_owned(),
            content: String::new(),
            confidence: 0.0,
            evidence: Vec::new(),
        },
        &mut child,
        &RunOptions::default(),
    )
    .expect("outer production turn");
    let composed = formal_ai::meta_executor::adopted_composition(&run).expect("valid composition");
    assert!(composed.content.contains("First task"));
    assert!(composed.content.contains("second task"));
}

#[test]
fn validated_prime_candidates_keep_their_intent_in_every_response_language() {
    let solver = formal_ai::UniversalSolver::new(formal_ai::SolverConfig {
        offline: true,
        temperature: 0.0,
        ..formal_ai::SolverConfig::default()
    });
    for (prompt, expected) in [
        ("Choose a prime between 8 and 12", "11"),
        ("Выбери простое число от 24 до 30", "29"),
        ("Elige un número primo entre 32 y 40", "37"),
        ("选择8到12之间的质数", "11"),
        ("0 और 3 के बीच अभाज्य संख्या चुनो", "2"),
    ] {
        let answer = solver.solve(prompt);
        assert_eq!(answer.answer, expected, "{prompt}");
        assert_eq!(answer.intent, "number_property_prime", "{prompt}");
    }
}

#[test]
fn production_composition_retains_only_selected_child_evidence() {
    for (connective, expected) in [
        (
            "and",
            vec!["source:first", "source:shared", "source:second"],
        ),
        ("or", vec!["source:first", "source:shared"]),
    ] {
        let mut child = |text: &str| {
            let source = if text == "First task" {
                "source:first"
            } else {
                "source:second"
            };
            Some(Draft {
                intent: "answered".to_owned(),
                content: text.to_owned(),
                confidence: 1.0,
                evidence: vec![source.to_owned(), "source:shared".to_owned()],
            })
        };
        let prompt = format!("First task {connective} second task");
        let run = formal_ai::meta_executor::solve_answer(
            Request {
                text: &prompt,
                language: "en",
            },
            Draft {
                intent: "unknown".to_owned(),
                content: String::new(),
                confidence: 0.0,
                evidence: vec!["source:unselected-root".to_owned()],
            },
            &mut child,
            &RunOptions::default(),
        )
        .expect("outer production turn");
        let composed =
            formal_ai::meta_executor::adopted_composition(&run).expect("composition selected");
        assert_eq!(composed.evidence, expected, "{connective}");
    }
}

// Issue #1580: main rejected source-operand compositions because separately
// solved assignment fragments were unresolved. Replay through the real solver,
// including the renumbered case, in the existing isolated native target.
#[test]
fn production_algebra_substitution_replays_main_regressions() {
    let solver = formal_ai::UniversalSolver::new(formal_ai::SolverConfig {
        offline: true,
        execution_surface: formal_ai::ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..formal_ai::SolverConfig::default()
    });
    for (prompt, expected) in [
        (
            "If x = 2 and y = 5, then what is the value of (x^4 + 2y^2) / 6?",
            "11",
        ),
        ("Given y = 4 and x = 3, evaluate (x^2 + 2y).", "17"),
        ("Given x = 3 and y = 4, evaluate (x^2 + 2y).", "17"),
        ("If x = 3 and y = 4, what is x*x + 2*y?", "17"),
        ("If x = 4 and y = 6, what is x*x + 2*y?", "28"),
    ] {
        let response = solver.solve(prompt);
        assert_eq!(
            response.intent, "algebra_substitution",
            "{prompt}: {}",
            response.answer
        );
        assert_eq!(response.answer, expected, "{prompt}");
        for kind in [
            "sub_impulse:",
            "sub_result:",
            "composition:substitution:",
            "composition:evaluation:",
        ] {
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link.starts_with(kind)),
                "{prompt}: missing {kind}"
            );
        }
        assert!(
            response.links_notation.contains("composition:evaluation"),
            "{prompt}"
        );
    }
}

// Replay the main-only synthesis contracts through the real offline solver.
#[test]
fn production_remainder_sale_recomputes_source_operands() {
    let solver = formal_ai::UniversalSolver::new(formal_ai::SolverConfig {
        offline: true,
        execution_surface: formal_ai::ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..formal_ai::SolverConfig::default()
    });
    for (total, price, expected) in [(24, 3, 36), (30, 4, 72)] {
        let prompt = format!(
            "Marisol's hens lay {total} eggs per day. She eats five for breakfast every morning and bakes cakes for her neighbors every day with seven. She sells the remainder at the market daily for ${price} per fresh egg. How much in dollars does she make every day at the market?"
        );
        let response = solver.solve(&prompt);
        assert_eq!(
            response.intent, "arithmetic_word_problem",
            "{}",
            response.answer
        );
        assert_eq!(response.answer, expected.to_string(), "{prompt}");
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("composition:remainder:")),
            "{prompt}"
        );
        let evaluation = format!("({total} - 12) * {price} = {expected}");
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("composition:evaluation:")),
            "{prompt}: {:?}",
            response.evidence_links
        );
        assert!(response.links_notation.contains(&evaluation), "{prompt}");
    }
}

#[test]
fn production_object_count_replays_membership_sources() {
    let solver = formal_ai::UniversalSolver::new(formal_ai::SolverConfig {
        offline: true,
        execution_surface: formal_ai::ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..formal_ai::SolverConfig::default()
    });
    let prompt =
        "I have a clarinet, a violin, and a flute. How many musical instruments do I have?";
    let response = solver.solve(prompt);
    assert_eq!(response.intent, "verifiable_task", "{}", response.answer);
    assert_eq!(response.answer, "3", "{prompt}");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("verifiable_task:executed:")),
        "{:?}",
        response.evidence_links
    );
    assert!(
        response
            .links_notation
            .contains("verifiable_task:membership_source"),
        "{}",
        response.links_notation
    );
}
