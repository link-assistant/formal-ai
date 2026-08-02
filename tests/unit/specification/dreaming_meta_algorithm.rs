//! Issue #540: the dreaming planner, self-described as grounded data.
//!
//! `data/meta/dreaming-recipe.lino` is the meta-algorithm describing how the
//! low-priority dreaming planner is produced: the thirteen ordered steps that turn
//! stored experience into a maintenance-and-generalization plan, plus the live
//! functions and constants that implement each step. These tests keep that
//! recipe grounded — they load it and assert the real source still defines every
//! named function, declares every named constant, lists thirteen contiguous steps,
//! and is pinned by real tests — so the self-description can never drift from the
//! code that actually runs. The parser mirrors
//! `tests/unit/specification/recursive_core_recipe.rs`.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/dreaming-recipe.lino";

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
fn meta_recipe_steps_are_complete_and_ordered() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "dreaming");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how dreaming generalizes across the task class"
    );
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the dreaming planner"
    );

    let mut orders: Vec<usize> = of_kind(&records, "meta_step")
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
        (1..=13).collect::<Vec<_>>(),
        "the meta-algorithm must list thirteen contiguously ordered steps"
    );

    for step in of_kind(&records, "meta_step") {
        let source_file = step.require("source_file");
        assert!(
            repo_root().join(source_file).exists(),
            "meta_step {} source_file {source_file} should exist",
            step.require("id"),
        );
    }
}

#[test]
fn meta_recipe_functions_exist_in_named_source() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(
        functions.len() >= 13,
        "expected the classify/learn/replay/apply/storage/runtime chain"
    );
    for function in &functions {
        let name = function.require("function");
        let source = read(function.require("source_file"));
        assert!(
            source.contains(&format!("fn {name}")),
            "{} should define fn {name}",
            function.require("source_file"),
        );
    }
}

#[test]
fn meta_recipe_constants_are_declared_in_source() {
    let records = records();
    let constants = of_kind(&records, "meta_constant");
    assert!(
        constants.len() >= 2,
        "expected the free-space reserve and amendment-kind constants to be documented"
    );
    for constant in constants {
        let token = constant.require("const");
        let source = read(constant.require("source_file"));
        assert!(
            source.contains(token),
            "{} should reference {token}",
            constant.require("source_file"),
        );
        assert!(
            !constant.require("purpose").is_empty(),
            "meta_constant {token} should describe its purpose",
        );
    }
}

#[test]
fn meta_recipe_tests_pin_the_behaviour() {
    let records = records();
    let tests = of_kind(&records, "meta_test");
    assert!(
        tests.len() >= 3,
        "expected the maintenance, docs-traceability, and self-grounding pins"
    );
    for test in tests {
        let test_file = test.require("test_file");
        assert!(
            repo_root().join(test_file).exists(),
            "meta_test {} should reference an existing file {test_file}",
            test.require("suite"),
        );
        assert!(
            !test.require("pins").is_empty(),
            "meta_test {} should describe what it pins",
            test.require("suite"),
        );
    }
}

#[test]
fn meta_algorithm_doc_explains_and_links_the_recipe() {
    let doc = read("docs/meta-algorithm.md");
    for needle in [
        "# Meta-Algorithm",
        "data/meta/dreaming-recipe.lino",
        "tests/unit/specification/dreaming_meta_algorithm.rs",
        "dreaming",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}
