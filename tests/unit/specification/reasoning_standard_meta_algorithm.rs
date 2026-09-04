//! Issue #1073 (R1073-6): the reasoning standard, self-described as grounded data.
//!
//! `data/meta/reasoning-standard-recipe.lino` describes how the standard is
//! produced: the ordered steps, the gates each one installs, the evaluator that
//! decides each gate, the fixture the harness replays, and the tests that pin it.
//! These tests keep that description grounded against the live source, so the
//! recipe cannot drift into prose that merely claims the standard exists. The
//! recipe is the machine-readable answer to requirement 6 — the procedure stated
//! formally enough that following it, without a model, reaches the same gates.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/reasoning-standard-recipe.lino";

struct Record {
    kind: String,
    fields: Vec<(String, String)>,
}

impl Record {
    fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn require(&self, name: &str) -> &str {
        self.field(name)
            .unwrap_or_else(|| panic!("{} record missing field `{name}`", self.kind))
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
}

fn records() -> Vec<Record> {
    let text = read(RECIPE);
    let mut records = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(parse_record(&current));
            current.clear();
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(parse_record(&current));
    }
    records
}

fn parse_record(lines: &[&str]) -> Record {
    let mut kind = String::new();
    let mut fields = Vec::new();
    for line in lines.iter().skip(1) {
        let trimmed = line.trim();
        if let Some((name, raw)) = trimmed.split_once(' ') {
            let value = unquote(raw.trim());
            if name == "record_type" {
                kind = value;
            } else {
                fields.push((name.to_owned(), value));
            }
        }
    }
    Record { kind, fields }
}

fn unquote(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
}

fn of_kind<'a>(records: &'a [Record], kind: &str) -> Vec<&'a Record> {
    records
        .iter()
        .filter(|record| record.kind == kind)
        .collect()
}

#[test]
fn reasoning_standard_recipe_steps_are_complete_and_ordered() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "reasoning_standard");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "the recipe must say how a new reasoning requirement becomes a gate"
    );

    let steps = of_kind(&records, "meta_step");
    let mut orders: Vec<usize> = steps
        .iter()
        .map(|step| {
            step.require("order")
                .parse()
                .expect("step order must be an integer")
        })
        .collect();
    orders.sort_unstable();
    assert_eq!(
        orders,
        (1..=8).collect::<Vec<_>>(),
        "the reasoning standard must list eight contiguously ordered steps"
    );
    for step in steps {
        let source = step.require("seed_file");
        assert!(
            !read(source).is_empty(),
            "step `{}` cites empty source file {source}",
            step.require("id")
        );
        assert!(
            !step.require("detail").is_empty(),
            "step `{}` must explain how it produces part of the standard",
            step.require("id")
        );
    }
}

#[test]
fn reasoning_standard_recipe_gates_match_the_declared_standard() {
    let records = records();
    let declared = formal_ai::reasoning_standard::standard().expect("the standard must load");
    let gates = of_kind(&records, "meta_gate");
    assert_eq!(
        gates.len(),
        declared.gates.len(),
        "the recipe must document every gate the standard declares"
    );

    let evaluators = read("src/reasoning_standard/mod.rs");
    let mut orders: Vec<usize> = Vec::new();
    for gate in gates {
        let slug = gate.require("gate");
        let declared_gate = declared
            .gates
            .iter()
            .find(|record| record.slug == slug)
            .unwrap_or_else(|| panic!("data/meta/reasoning-standard.lino should declare {slug}"));
        assert_eq!(
            gate.require("order"),
            declared_gate.order.to_string(),
            "gate {slug} must keep the order the standard gives it"
        );
        assert_eq!(
            gate.require("trigger"),
            declared_gate.trigger.slug(),
            "gate {slug} must keep the trigger the standard gives it"
        );
        assert_eq!(
            gate.require("source_file"),
            "src/reasoning_standard/mod.rs",
            "gate evaluators live with the audit"
        );
        let evaluator = gate.require("evaluator");
        assert!(
            evaluators.contains(&format!("fn {evaluator}")),
            "src/reasoning_standard/mod.rs should define fn {evaluator} for gate {slug}"
        );
        orders.push(gate.require("order").parse().expect("gate order integer"));
    }
    orders.sort_unstable();
    assert_eq!(
        orders,
        (1..=orders.len()).collect::<Vec<_>>(),
        "the gates must be contiguously ordered"
    );
}

#[test]
fn reasoning_standard_recipe_functions_exist_in_named_source() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(
        functions.len() >= 8,
        "the recipe must pin the functions that implement the standard"
    );
    for function in functions {
        let name = function.require("function");
        let source = read(function.require("source_file"));
        assert!(
            source.contains(&format!("fn {name}")),
            "{} should define fn {name}",
            function.require("source_file")
        );
    }
}

#[test]
fn reasoning_standard_recipe_pins_its_fixture_registry_and_tests() {
    let records = records();

    let fixtures = of_kind(&records, "meta_fixture");
    assert!(!fixtures.is_empty(), "the recipe must pin its fixture");
    for fixture in fixtures {
        let path = fixture.require("fixture");
        assert!(
            read(path).contains("reasoning_episode"),
            "{path} should hold a reasoning episode"
        );
        assert!(!fixture.require("purpose").is_empty());
    }

    for registry in of_kind(&records, "meta_registry") {
        let path = registry.require("registry");
        let field = registry.require("field");
        let text = read(path);
        assert!(
            text.contains(&format!("{field} ")),
            "{path} should carry the {field} the recipe says its trust is derived from"
        );
    }

    let tests = of_kind(&records, "meta_test");
    assert!(
        tests.len() >= 2,
        "the recipe must pin the tests that hold it"
    );
    for test in tests {
        let path = test.require("test_file");
        let source = read(path);
        assert!(
            source.contains("#[test]"),
            "{path} should contain the tests the recipe cites"
        );
        assert!(!test.require("pins").is_empty());
    }
}

#[test]
fn the_meta_core_runs_the_audit_with_no_mode_in_front_of_it() {
    let core = read("src/meta_core.rs");
    assert!(
        core.contains("crate::reasoning_standard::record_reasoning_standard("),
        "record_meta_core must audit every request"
    );
    let call = core
        .split("crate::reasoning_standard::record_reasoning_standard(")
        .next()
        .expect("the call site should follow the pipeline body");
    let tail = call
        .rsplit_once("let _skills")
        .map_or(call, |(_, after)| after);
    assert!(
        !tail.contains("if "),
        "the reasoning-standard audit must not sit behind a mode gate: {tail}"
    );
}
