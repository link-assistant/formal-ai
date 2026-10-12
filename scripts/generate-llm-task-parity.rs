#!/usr/bin/env rust-script
//! One generated LLM-task-parity surface (issue #1171, E136).
//!
//! `docs/llm-task-parity.md` compares every class of task people bring to
//! LLMs with Formal AI's formal version of the answer: what LLMs do well,
//! the public benchmark that measures it, the white-box formal route, the
//! latest measured verdict, the evidence pointer and the tracking issue.
//! The document is generated, never hand-edited, exactly like
//! `docs/status.md` (modelled on `scripts/render-status.rs`).
//!
//! Input: `data/meta/llm-task-classes.lino`, the data-owned class registry
//! from issue #1171 R1. The generator has no embedded class table, so each
//! parity row and benchmark citation is edited in one place.
//!
//! Usage:
//!   rust-script scripts/generate-llm-task-parity.rs --write
//!   rust-script scripts/generate-llm-task-parity.rs --check
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::fs;
use std::path::{Path, PathBuf};

const DOCUMENT: &str = "docs/llm-task-parity.md";
const REGISTRY: &str = "data/meta/llm-task-classes.lino";
const ISSUE_URL_PREFIX: &str = "https://github.com/link-assistant/formal-ai/issues/";

/// One LLM task class parsed from the data registry.
struct Class {
    id: String,
    name: String,
    llm_strength: String,
    benchmarks: Vec<(String, Option<String>)>,
    formal_version: String,
    status: String,
    tracking_issue: String,
    evidence: String,
}


/// Splits a field line into its `key value` pairs. A quoted value may contain
/// spaces and may be followed by further pairs on the same line
/// (`benchmark "Natural Questions" benchmark_url "https://…"`), so each value
/// is taken as either the span between one pair of quotes or the next bare
/// word.
fn field_pairs(line: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut rest = line.trim();
    while let Some((key, after)) = rest.split_once(' ') {
        let after = after.trim_start();
        if let Some(quoted) = after.strip_prefix('"')
            && let Some(end) = quoted.find('"')
        {
            pairs.push((key.to_owned(), quoted[..end].to_owned()));
            rest = quoted[end + 1..].trim_start();
        } else {
            let (value, tail) = after
                .split_once(' ')
                .unwrap_or((after, ""));
            pairs.push((key.to_owned(), value.to_owned()));
            rest = tail.trim_start();
        }
    }
    pairs
}

/// Parses the registry schema: a `llm_task_classes` root, `class <id>`
/// children at two spaces, fields at four. `benchmark` and `benchmark_url`
/// share one line, so each field line is split into key/value pairs while
/// text remains.
fn parse_classes(source: &str) -> Result<Vec<Class>, String> {
    let mut classes = Vec::new();
    let mut current: Option<Class> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(id) = line.strip_prefix("  class ") {
            if let Some(done) = current.take() {
                classes.push(done);
            }
            current = Some(Class {
                id: id.trim().to_owned(),
                name: String::new(),
                llm_strength: String::new(),
                benchmarks: Vec::new(),
                formal_version: String::new(),
                status: String::new(),
                tracking_issue: String::new(),
                evidence: String::new(),
            });
        } else if current.is_some() && line.starts_with("    ") && !line.starts_with("      ") {
            let class = current.as_mut().expect("a class is open");
            for (key, value) in field_pairs(trimmed) {
                match key.as_str() {
                    "name" => class.name = value,
                    "llm_strength" => class.llm_strength = value,
                    "benchmark" => class.benchmarks.push((value, None)),
                    "benchmark_url" => {
                        if let Some(last) = class.benchmarks.last_mut() {
                            last.1 = Some(value);
                        }
                    }
                    "formal_version" => class.formal_version = value,
                    "status" => class.status = value,
                    "tracking_issue" => class.tracking_issue = value,
                    "evidence" => class.evidence = value,
                    _ => {}
                }
            }
        }
    }
    if let Some(done) = current.take() {
        classes.push(done);
    }
    for class in &classes {
        for field in [
            ("name", &class.name),
            ("llm_strength", &class.llm_strength),
            ("formal_version", &class.formal_version),
            ("status", &class.status),
            ("tracking_issue", &class.tracking_issue),
            ("evidence", &class.evidence),
        ] {
            if field.1.is_empty() {
                return Err(format!("class {}: empty {}", class.id, field.0));
            }
        }
    }
    if classes.is_empty() {
        Err("registry has no class rows".to_owned())
    } else {
        Ok(classes)
    }
}

fn benchmark_cell(class: &Class) -> String {
    if class.benchmarks.is_empty() {
        return "no dedicated public benchmark; probed in-repo".to_owned();
    }
    class
        .benchmarks
        .iter()
        .map(|(name, url)| match url {
            Some(url) => format!("[{name}]({url})"),
            None => name.clone(),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn tracking_cell(class: &Class) -> String {
    format!("[#{0}]({ISSUE_URL_PREFIX}{0})", class.tracking_issue)
}

fn evidence_cell(class: &Class) -> String {
    if let Some(number) = class.evidence.strip_prefix(ISSUE_URL_PREFIX) {
        format!("[issue {number}]({})", class.evidence)
    } else if class.evidence.starts_with("http") {
        let label = class
            .evidence
            .rsplit('/')
            .next()
            .unwrap_or("evidence")
            .to_owned();
        format!("[{label}]({})", class.evidence)
    } else {
        format!("`{}`", class.evidence)
    }
}

fn class_cell(class: &Class) -> String {
    format!("`{}` — {}", class.id, class.name)
}

fn document(classes: &[Class], input_source: &str) -> String {
    let mut output = format!(
        "<!-- Generated by `rust-script scripts/generate-llm-task-parity.rs --write`. Never edited \
         by hand. -->\n\
         Input: {input_source}\n\n\
         # LLM task parity\n\n\
         One row per class of task people bring to LLMs, with what current LLMs do well, the \
         public benchmark that measures it, and the formal version of the answer Formal AI \
         targets. The status column carries the latest measured verdict; the first measurement \
         (formal-ai 0.347.0, 34 probes) found 0 of 34 useful answers and 3 wrong ones, recorded \
         in the history section below.\n\n\
         | Task class | What LLMs do well | Public benchmark | Formal version of the answer | \
         Status | Evidence | Tracking |\n\
         | --- | --- | --- | --- | --- | --- | --- |\n"
    );
    for class in classes {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            class_cell(class),
            class.llm_strength,
            benchmark_cell(class),
            class.formal_version,
            class.status,
            evidence_cell(class),
            tracking_cell(class)
        ));
    }
    output.push_str(
        "\n## History\n\n\
         ### formal-ai 0.347.0 — first measurement, 34 probes\n\n\
         0 of 34 useful; 3 wrong: `factual_qa` (the US capital for an Australia question), \
         `units_and_dates` (the day arithmetic ignored the stated offset) and `repository_qa` \
         (a self-description instead of the asked function). 22 further probes returned the \
         canned web-search paragraph, and the rest were honest refusals or surface-word \
         misroutes. Full transcripts: issue #1171.\n",
    );
    output
}

fn input_source(root: &Path) -> Result<(String, Vec<Class>), String> {
    let registry_path = root.join(REGISTRY);
    let source = fs::read_to_string(&registry_path)
        .map_err(|error| format!("failed to read {REGISTRY}: {error}"))?;
    Ok((
        format!("`{REGISTRY}`"),
        parse_classes(&source).map_err(|error| format!("{REGISTRY}: {error}"))?,
    ))
}

fn main() {
    let root = std::env::current_dir().expect("current directory");
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "--check".to_owned());
    if !matches!(mode.as_str(), "--write" | "--check") {
        eprintln!("generate-llm-task-parity: unknown mode {mode}; expected --write or --check");
        std::process::exit(2);
    }
    let (input_source_value, classes) =
        input_source(&root).unwrap_or_else(|error| {
            eprintln!("generate-llm-task-parity: {error}");
            std::process::exit(1);
        });
    let path = PathBuf::from(DOCUMENT);
    let content = document(&classes, &input_source_value);
    if mode == "--write" {
        fs::write(&path, &content)
            .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        println!("rendered {DOCUMENT} from {input_source_value}");
        return;
    }
    if fs::read_to_string(&path).ok().as_ref() == Some(&content) {
        println!("{DOCUMENT} is current");
    } else {
        eprintln!("stale generated document: {DOCUMENT}");
        eprintln!("run rust-script scripts/generate-llm-task-parity.rs --write");
        std::process::exit(1);
    }
}
