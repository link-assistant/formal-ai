//! Issue #559 (R335): the recursive meta core, self-described as grounded data.
//!
//! `data/meta/recursive-core-recipe.lino` is the meta algorithm describing
//! *itself*: the eleven ordered steps that turn any message into a solved,
//! link-native knowledge base, plus the live functions that implement each step.
//! These tests keep that recipe grounded — they load it and assert the real
//! source still defines every named function and lists eleven contiguous steps —
//! so the self-description can never drift from the code that actually runs. This
//! is the concrete sense in which the core can "reason about itself": its own
//! algorithm exists as data the engine can read, pinned to the implementation.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/recursive-core-recipe.lino";

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
fn recursive_core_recipe_steps_are_complete_and_ordered() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "recursive_core");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "the recipe must describe how to generalise the core to new task classes"
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
        (1..=11).collect::<Vec<_>>(),
        "the recursive meta core must list eleven contiguously ordered steps"
    );
}

#[test]
fn recursive_core_recipe_functions_exist_in_named_source() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(
        functions.len() >= 10,
        "the recipe must pin the core's implementing functions"
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
fn recursive_core_recipe_steps_cite_readable_seed_files() {
    let records = records();
    for step in of_kind(&records, "meta_step") {
        let source = step.require("source_file");
        let contents = read(source);
        assert!(
            !contents.is_empty(),
            "step `{}` cites empty source file {source}",
            step.require("id")
        );
        assert!(
            !step.require("detail").is_empty(),
            "step `{}` must explain how it advances the meta loop",
            step.require("id")
        );
    }
}
