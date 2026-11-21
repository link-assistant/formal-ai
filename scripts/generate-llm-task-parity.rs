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

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DOCUMENT: &str = "docs/llm-task-parity.md";
const REGISTRY: &str = "data/meta/llm-task-classes.lino";
const RELEASE: &str = "v0.356.0";
const MEASUREMENT: &str = "data/benchmarks/llm-task-parity/v0.356.0-verdicts.tsv";
const ISSUE_URL_PREFIX: &str = "https://github.com/link-assistant/formal-ai/issues/";

/// One LLM task class parsed from the data registry.
struct Class {
    id: String,
    name: String,
    llm_strength: String,
    benchmarks: Vec<(String, Option<String>)>,
    formal_version: String,
    structured_output: String,
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
            let (value, tail) = after.split_once(' ').unwrap_or((after, ""));
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
    let default_structured_output = source
        .lines()
        .find(|line| line.starts_with("  default-structured-output "))
        .and_then(|line| field_pairs(line.trim()).into_iter().next())
        .map_or_else(String::new, |(_, value)| value);
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
                structured_output: default_structured_output.clone(),
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
                    "structured-output" => class.structured_output = value,
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
            ("structured-output", &class.structured_output),
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

#[derive(Clone, Copy)]
struct Probe<'a> {
    id: &'a str,
    en: &'a str,
    ru: &'a str,
    observations: &'a str,
}

fn parse_measurement<'a>(
    source: &'a str,
    classes: &[Class],
) -> Result<BTreeMap<&'a str, Vec<Probe<'a>>>, String> {
    let mut result = BTreeMap::<&str, Vec<Probe<'_>>>::new();
    for (index, line) in source.lines().enumerate().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        let [class, id, en, ru, observations] = fields.as_slice() else {
            return Err(format!(
                "line {} must contain five tab-separated fields",
                index + 1
            ));
        };
        if !classes.iter().any(|item| item.id == *class)
            || ![en, ru]
                .iter()
                .all(|verdict| matches!(**verdict, "useful" | "partial" | "miss"))
            || observations.is_empty()
        {
            return Err(format!("invalid probe at line {}", index + 1));
        }
        let probes = result.entry(class).or_default();
        if probes.iter().any(|probe| probe.id == *id) {
            return Err(format!("duplicate probe {class}/{id}"));
        }
        probes.push(Probe {
            id,
            en,
            ru,
            observations,
        });
    }
    for class in classes {
        if !result.contains_key(class.id.as_str()) {
            return Err(format!("no bilingual probe for {}", class.id));
        }
    }
    Ok(result)
}

fn verdict_cell(probes: &[Probe<'_>]) -> String {
    let en = probes.iter().filter(|probe| probe.en == "useful").count();
    let ru = probes.iter().filter(|probe| probe.ru == "useful").count();
    format!("EN {en}/{} · RU {ru}/{}", probes.len(), probes.len())
}

fn render_measurement(
    output: &mut String,
    classes: &[Class],
    measurement: &BTreeMap<&str, Vec<Probe<'_>>>,
) {
    let verdicts: Vec<_> = measurement
        .values()
        .flatten()
        .flat_map(|probe| [probe.en, probe.ru])
        .collect();
    let useful = verdicts
        .iter()
        .filter(|verdict| **verdict == "useful")
        .count();
    let partial = verdicts
        .iter()
        .filter(|verdict| **verdict == "partial")
        .count();
    let count = verdicts.len();
    output.push_str(&format!(
        "\n## Release measurement\n\n\
         On 2026-11-21, the published Linux x86-64 musl binary for [{RELEASE}](https://github.com/link-assistant/formal-ai/releases/tag/{RELEASE}) answered {count} probes across {} classes. The run produced {useful} useful answers, {partial} partial answers and {} misses. Each case uses one English and one Russian prompt.\n\n\
         The archive `formal-ai-cli-x86_64-unknown-linux-musl.tar.gz` matched the published SHA-256 digest `05305d18986ad303e78357e973d058a56b196a1422ea6ec5a0924e1876b0a5fb`.\n\n\
         The published CLI served `/v1/chat/completions` in its default mode with a fresh home, no prior conversation beyond the fixture, and normal provider access. The agentic coding probe records the default tool policy. Benchmarks in the table above describe related public tasks. This run uses only the synthetic fixtures below.\n\n\
         A useful response answers the request correctly and meets its material constraints. A partial response contains usable evidence or an incomplete artifact. Refusals, misroutes, incorrect answers and empty results count as misses. The per-case notes are manually reviewed.\n\n\
         [Prompt fixtures](../data/benchmarks/llm-task-parity/prompts.jsonl) · [Captured responses](../data/benchmarks/llm-task-parity/{RELEASE}.jsonl) · [Verdicts](../{MEASUREMENT}) · [Capture script](../experiments/llm-task-parity/capture.mjs)\n\n\
         ```sh\n\
         env HOME=\"$(mktemp -d)\" formal-ai serve --silent --host 127.0.0.1 --port 8080\n\
         # In a second shell, from the repository root\n\
         node experiments/llm-task-parity/capture.mjs http://127.0.0.1:8080 {RELEASE} > responses.jsonl\n\
         ```\n\n\
         | Task class | Case | English | Russian | Observation |\n\
         | --- | --- | --- | --- | --- |\n",
        classes.len(), count - useful - partial
    ));
    for class in classes {
        for probe in &measurement[class.id.as_str()] {
            output.push_str(&format!(
                "| `{}` | `{}` | {} | {} | {} |\n",
                class.id, probe.id, probe.en, probe.ru, probe.observations
            ));
        }
    }
}

fn document(
    classes: &[Class],
    input_source: &str,
    measurement: &BTreeMap<&str, Vec<Probe<'_>>>,
) -> String {
    let mut output = format!(
        "<!-- Generated by `rust-script scripts/generate-llm-task-parity.rs --write`. Never edited \
         by hand. -->\n\
         Input: {input_source}\n\n\
         # LLM task parity\n\n\
         One row per class of task people bring to LLMs, with what current LLMs do well, the \
         public benchmark that measures it, and the formal version of the answer Formal AI \
         targets. The status column counts useful answers by prompt language on release {RELEASE}. \
         The first measurement (formal-ai 0.347.0, 34 probes) found 0 of 34 useful answers \
         and 3 wrong ones, recorded in the history section below.\n\n\
         | Task class | What LLMs do well | Public benchmark | Formal version of the answer | \
         Structured output | Status | Evidence | Tracking |\n\
         | --- | --- | --- | --- | --- | --- | --- | --- |\n"
    );
    for class in classes {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            class_cell(class),
            class.llm_strength,
            benchmark_cell(class),
            class.formal_version,
            class.structured_output,
            verdict_cell(&measurement[class.id.as_str()]),
            evidence_cell(class),
            tracking_cell(class)
        ));
    }
    render_measurement(&mut output, classes, measurement);
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
    let (input_source_value, classes) = input_source(&root).unwrap_or_else(|error| {
        eprintln!("generate-llm-task-parity: {error}");
        std::process::exit(1);
    });
    let path = PathBuf::from(DOCUMENT);
    let verdicts = fs::read_to_string(root.join(MEASUREMENT))
        .unwrap_or_else(|error| panic!("failed to read {MEASUREMENT}: {error}"));
    let measurement = parse_measurement(&verdicts, &classes)
        .unwrap_or_else(|error| panic!("{MEASUREMENT}: {error}"));
    let content = document(&classes, &input_source_value, &measurement);
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
