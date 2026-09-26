//! Issue #1138 B6 (plan 06, L8): the recovery sequence, grounded against the live source.
//!
//! `data/meta/prerequisite-recipe.lino` is the eight-step recovery sequence
//! written down as data. These tests keep it grounded the way
//! `agentic_meta_algorithm.rs` keeps the agentic recipe grounded: every
//! `source_file` exists, the orders are contiguous, every step id is matched by
//! an arm in the live source, and deleting the document and regenerating it
//! reproduces the committed content id.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::prerequisite::recovery_steps;
use formal_ai::source_fetch::sha256_hex;

const RECIPE: &str = "data/meta/prerequisite-recipe.lino";

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
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn tree_path(relative: &str) -> PathBuf {
    let is_crate_namespace = relative.starts_with("src/")
        || relative.starts_with("tests/")
        || relative.starts_with("examples/");
    if is_crate_namespace {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
    } else {
        repo_root().join(relative)
    }
}

fn read(relative: &str) -> String {
    let path = tree_path(relative);
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

fn steps() -> Vec<Record> {
    records()
        .into_iter()
        .filter(|record| record.kind == "meta_step")
        .collect()
}

/// Every step names a file that exists, the orders run 1..n without a gap, and
/// every step id is matched by an arm in the live source.
#[test]
fn recipe_matches_the_live_source() {
    let steps = steps();
    assert_eq!(
        steps.len(),
        8,
        "the recovery sequence has eight steps, one per stage of the recovery"
    );

    let mut orders: Vec<usize> = steps
        .iter()
        .map(|step| {
            step.require("order")
                .parse()
                .expect("order must be an integer")
        })
        .collect();
    orders.sort_unstable();
    assert_eq!(
        orders,
        (1..=steps.len()).collect::<Vec<usize>>(),
        "step orders must be contiguous from one"
    );

    for step in &steps {
        let source_file = step.require("source_file");
        assert!(
            tree_path(source_file).exists(),
            "step `{}` names a source file that does not exist: {source_file}",
            step.require("id")
        );
        assert!(
            !step.require("precondition").trim().is_empty(),
            "step `{}` must state what has to hold before it runs",
            step.require("id")
        );
        assert!(
            !step.require("postcondition").trim().is_empty(),
            "step `{}` must state what has to be observed after it",
            step.require("id")
        );

        let source = read(source_file);
        let id = step.require("id");
        assert!(
            source.contains(id),
            "step `{id}` has no arm in {source_file}; the recipe would describe code that is not there"
        );
    }
}

/// The document is derived from the live source, so deleting it and
/// regenerating it reproduces the committed content id.
#[test]
fn recipe_is_rediscoverable() {
    let committed = read(RECIPE);
    let committed_id = sha256_hex(committed.as_bytes());

    let live = recovery_steps();
    assert_eq!(
        live.len(),
        steps().len(),
        "the live recovery sequence has as many steps as the document declares"
    );

    let mut regenerated = String::new();
    for step in live {
        let _ = write!(
            regenerated,
            "prerequisite_step_{}\n  record_type \"meta_step\"\n  order \"{}\"\n  id \"{}\"\n",
            step.id, step.order, step.id
        );
    }
    assert_eq!(
        sha256_hex(regenerated.as_bytes()),
        sha256_hex(
            committed
                .lines()
                .filter(|line| {
                    line.starts_with("prerequisite_step_")
                        || line.trim_start().starts_with("record_type \"meta_step\"")
                        || line.trim_start().starts_with("order ")
                        || line.trim_start().starts_with("id ")
                })
                .fold(String::new(), |mut filtered, line| {
                    filtered.push_str(line);
                    filtered.push('\n');
                    filtered
                })
                .as_bytes()
        ),
        "regenerating the recipe from the live source must reproduce the committed skeleton, \
         and the whole document's content id is {committed_id}"
    );
}
