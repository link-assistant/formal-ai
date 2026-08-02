//! Links-network terminology meta-algorithm grounding (issue #664).
//!
//! Issue #664 (E45) asked us to keep the product's associative surface a *links
//! network*, not a "graph": `GET /v1/network` is canonical, `/v1/graph` survives
//! only as a deprecated alias, the `self_source_graph`/`source_graph` modules are
//! renamed to `*_links`, the UI says "links network view", the docs speak
//! links-network vocabulary, and a repository-hygiene lint blocks *new*
//! graph-named public routes and Rust modules so the cleanup cannot regress.
//! `data/meta/links-network-terminology-recipe.lino` is that recipe: it names
//! every ordered step, handler function, module rename, allowlist entry, CI
//! wiring, and pinning test that make up the cleanup.
//!
//! These tests keep the recipe *grounded*: they load it and assert the live
//! source still matches. If the recipe and the code drift apart, CI fails — so
//! the terminology cleanup is itself a reproducible artifact of the
//! meta-algorithm, not a one-off hand edit. The parser mirrors
//! `tests/unit/specification/market_price_verification_meta_algorithm.rs`.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/links-network-terminology-recipe.lino";

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
    assert_eq!(recipe[0].require("topic"), "links_network_terminology");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how to generalise the terminology cleanup"
    );
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the links-network cleanup"
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
        (1..=steps.len()).collect::<Vec<_>>(),
        "the meta-algorithm steps must be contiguously ordered from 1"
    );

    for step in &steps {
        assert!(
            !step.require("detail").is_empty(),
            "meta_step {} must describe what it does",
            step.require("id"),
        );
        let seed_file = step.require("seed_file");
        assert!(
            repo_root().join(seed_file).exists(),
            "meta_step {} seed_file {seed_file} should exist",
            step.require("id"),
        );
    }
}

#[test]
fn meta_recipe_functions_exist_in_named_source() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(
        functions.len() >= 8,
        "expected the endpoint, alias, lint, and projection functions to be documented"
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
fn meta_recipe_renames_landed_and_old_names_are_gone() {
    let records = records();
    let renames = of_kind(&records, "meta_rename");
    assert!(
        renames.len() >= 2,
        "expected the self_source and agentic source module renames"
    );
    for rename in renames {
        let from_file = rename.require("from_file");
        let to_file = rename.require("to_file");
        assert!(
            repo_root().join(to_file).exists(),
            "renamed module {to_file} should exist",
        );
        assert!(
            !repo_root().join(from_file).exists(),
            "old graph-named module {from_file} should be gone",
        );
        // The new module must actually be declared under its new name.
        let declaration = read(rename.require("declaration_file"));
        let new_module = rename.require("new_module");
        assert!(
            declaration.contains(&format!("mod {new_module}")),
            "{} should declare mod {new_module}",
            rename.require("declaration_file"),
        );
    }
}

#[test]
fn meta_recipe_allowlist_entries_are_present_in_the_lint() {
    let records = records();
    let allowlist = of_kind(&records, "meta_allowlist");
    assert!(
        allowlist.len() >= 3,
        "expected the deprecated-alias, knowledge_graph, and citation-host exceptions"
    );
    for entry in allowlist {
        let token = entry.require("token");
        let lint = read(entry.require("lint_file"));
        assert!(
            lint.contains(token),
            "the terminology lint should allowlist {token}",
        );
        assert!(
            !entry.require("reason").is_empty(),
            "allowlist entry {token} should explain why it is exempt",
        );
    }
}

#[test]
fn meta_recipe_lint_is_wired_into_ci() {
    let records = records();
    let ci = of_kind(&records, "meta_ci");
    assert_eq!(ci.len(), 1, "exactly one CI wiring record expected");
    let entry = ci[0];
    let workflow = read(entry.require("workflow_file"));
    let needle = entry.require("needle");
    assert!(
        workflow.contains(needle),
        "{} should run the {needle} lint",
        entry.require("workflow_file"),
    );
    let suite = read(entry.require("suite_file"));
    assert!(
        suite.contains(needle),
        "{} should register the {needle} embedded tests",
        entry.require("suite_file"),
    );
}

#[test]
fn meta_recipe_tests_pin_the_behaviour() {
    let records = records();
    let tests = of_kind(&records, "meta_test");
    assert!(
        tests.len() >= 3,
        "expected the alias integration, lint, and links-network spec pins"
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
        "data/meta/links-network-terminology-recipe.lino",
        "tests/unit/specification/links_network_terminology_meta_algorithm.rs",
        "links network",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}
