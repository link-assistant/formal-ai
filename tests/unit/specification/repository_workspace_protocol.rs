//! Issue #1138 B7 (plan 03, L8): the repository protocol, grounded against the live source.
//!
//! `data/meta/repository-workspace-protocol.lino` is the ordered protocol
//! written down as data, so adding a step is an edit to the document rather than
//! an edit to Rust. These tests keep it grounded the way
//! `agentic_meta_algorithm.rs` keeps the agentic recipe grounded: every
//! `source_file` exists, `order` is contiguous, every `id` is matched by a
//! `ProtocolStep` the loader produces, and regenerating the document reproduces
//! the committed content id.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::repository_workspace::WorkspaceProtocol;
use formal_ai::source_fetch::sha256_hex;

const PROTOCOL: &str = "data/meta/repository-workspace-protocol.lino";

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
    let text = read(PROTOCOL);
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

fn declared_steps() -> Vec<Record> {
    records()
        .into_iter()
        .filter(|record| record.kind == "meta_step")
        .collect()
}

/// The document describes code that exists, in an order with no gaps, and every
/// step it names is a step the loader actually produces.
#[test]
fn protocol_document_matches_the_live_source() {
    let declared = declared_steps();
    assert_eq!(
        declared.len(),
        6,
        "the protocol is clone, locate, read, edit, verify, diff"
    );

    let mut orders: Vec<usize> = declared
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
        (1..=declared.len()).collect::<Vec<usize>>(),
        "step orders must be contiguous from one"
    );

    for step in &declared {
        let source_file = step.require("source_file");
        assert!(
            repo_root().join(source_file).exists(),
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
    }

    let loaded = WorkspaceProtocol::load();
    let loaded_ids: Vec<String> = loaded
        .steps()
        .iter()
        .map(|step| step.id.clone())
        .collect();
    let declared_ids: Vec<String> = declared
        .iter()
        .map(|step| step.require("id").to_owned())
        .collect();
    assert_eq!(
        loaded_ids, declared_ids,
        "the loader must produce exactly the steps the document declares, in order"
    );
}

/// The document is derived, so deleting it and regenerating it reproduces the
/// committed content id.
#[test]
fn protocol_document_is_rediscoverable() {
    let committed = read(PROTOCOL);
    assert_eq!(
        sha256_hex(WorkspaceProtocol::regenerate_document().as_bytes()),
        sha256_hex(committed.as_bytes()),
        "regenerating the protocol document must reproduce the committed content id"
    );
}
