//! Computer-use meta-algorithm grounding (issue #707).
//!
//! `data/meta/computer-use-recipe.lino` is the recipe that produced the
//! computer-use stage: the ordered observe → induce → bind → synthesize →
//! verify → refuse loop, the seed roles recognition reads instead of hardcoded
//! words, the functions that implement each step, the data it learns from, and
//! the benchmarks that ratchet it.
//!
//! These tests keep the recipe *grounded*. Every file it names must exist,
//! every function it names must be defined in the file it names, every seed role
//! constant must be declared, and every benchmark test it cites must exist under
//! that name. If the recipe and the code drift apart, CI fails — so the recipe
//! stays an accurate description of how the code was produced rather than stale
//! documentation. The parser mirrors
//! `tests/unit/specification/budget_search_meta_algorithm.rs`.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/computer-use-recipe.lino";

struct Record {
    id: String,
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
            .unwrap_or_else(|| panic!("{} record `{}` missing field `{name}`", self.kind, self.id))
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
    let id = lines[0].trim().to_owned();
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
    Record { id, kind, fields }
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
fn meta_recipe_states_the_topic_and_how_it_generalises() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "computer_use");
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the computer-use stage"
    );
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how the loop generalises beyond computer use"
    );
}

#[test]
fn meta_recipe_steps_are_contiguously_ordered_and_carry_an_invariant() {
    let records = records();
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
        "the meta-algorithm must list contiguously ordered steps"
    );
    assert!(
        steps.len() >= 9,
        "the loop needs at least taxonomy, record, meanings, partition, induce, \
         bind, synthesize, verify, and refuse"
    );
    for step in &steps {
        assert!(
            !step.require("operation").is_empty(),
            "meta_step {} must say what it does",
            step.id
        );
        assert!(
            !step.require("invariant").is_empty(),
            "meta_step {} must state the invariant it preserves",
            step.id
        );
    }
}

#[test]
fn every_seed_role_the_recipe_names_is_declared_in_the_seed_file() {
    let records = records();
    let roles = of_kind(&records, "meta_role");
    assert!(
        roles.len() >= 3,
        "operation, resource, and gap cues expected"
    );
    let source = read("src/computer_use/lexicon.rs") + &read("src/computer_use/mod.rs");
    for role in roles {
        let seed_file = role.require("seed_file");
        let seed = read(seed_file);
        let name = role.require("role");
        assert!(
            seed.contains(name),
            "seed file {seed_file} does not declare role {name}"
        );
        let konstant = role.require("const");
        assert!(
            source.contains(konstant),
            "no computer-use source reads {konstant}"
        );
    }
}

#[test]
fn every_function_the_recipe_names_is_defined_where_it_says() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(functions.len() >= 8, "the loop needs its handlers named");
    for entry in functions {
        let source_file = entry.require("source_file");
        let source = read(source_file);
        let function = entry.require("function");
        assert!(
            source.contains(&format!("fn {function}")),
            "{source_file} does not define `{function}`"
        );
    }
}

#[test]
fn every_data_and_evidence_file_the_recipe_names_exists() {
    let records = records();
    let referenced = of_kind(&records, "meta_data")
        .into_iter()
        .chain(of_kind(&records, "meta_evidence"))
        .collect::<Vec<_>>();
    assert!(
        referenced.len() >= 5,
        "taxonomy, corpus, meanings, held-out set, evidence"
    );
    for entry in referenced {
        let source_file = entry.require("source_file");
        assert!(
            repo_root().join(source_file).exists(),
            "{} names a missing file {source_file}",
            entry.id
        );
        assert!(
            !entry.require("purpose").is_empty(),
            "{} must say why the file is part of the recipe",
            entry.id
        );
    }
}

#[test]
fn every_benchmark_the_recipe_cites_exists_under_that_name() {
    let records = records();
    let benchmarks = of_kind(&records, "meta_benchmark");
    assert!(
        benchmarks.len() >= 6,
        "recorded, held-out, parity, drift, anti-memorisation, and boundary slices"
    );
    for entry in benchmarks {
        let source_file = entry.require("source_file");
        let source = read(source_file);
        let test = entry.require("test");
        assert!(
            source.contains(&format!("fn {test}")),
            "{source_file} does not define the cited test `{test}`"
        );
        assert!(
            !entry.require("coverage").is_empty(),
            "{} must say what slice it covers",
            entry.id
        );
    }
}

#[test]
fn both_external_client_parity_runs_are_wired_into_release_ci() {
    let records = records();
    let parity = of_kind(&records, "meta_parity");
    assert_eq!(
        parity.len(),
        2,
        "recorded and held-out slices run externally"
    );
    let workflow = read(".github/workflows/release.yml");
    for entry in parity {
        for field in ["script", "verifier"] {
            let file = entry.require(field);
            assert!(
                repo_root().join(file).exists(),
                "{} names a missing {field} {file}",
                entry.id
            );
        }
        let script = entry.require("script");
        assert!(
            workflow.contains(script),
            "release.yml never runs {script}, so the parity claim is unenforced"
        );
    }
}
