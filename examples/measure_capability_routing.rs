//! Measure the capability-routing corpus (issue #1138 B10, plan 10 leaves 1-3).
//!
//! `data/meta/capability-routing-ratchet.lino` records `cross_tool_misroutes`
//! and `silent_unknowns` as *measurements*. This example is what produces them:
//! it runs every committed case of `data/benchmarks/capability-routing/`
//! through `formal_ai::capability_routing::route` and prints the counts, so a
//! number in the ledger is one a run produced rather than one an author chose.
//!
//! Run it with `--verbose` to list every case that did not reach its expected
//! capability, which is how the honest-failure text of each frontier class is
//! kept honest between commits.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::capability_routing::{RoutingOutcome, acts, locus_of, object_type, route};

/// Everything the decision table's rows can name, so a miss is a routing
/// failure rather than an unadvertised tool.
const ADVERTISED: &[&str] = &[
    "web_fetch",
    "web_search",
    "read_file",
    "write_file",
    "list_dir",
    "grep",
    "shell",
    "calendar_create_event",
    "response_language_demonstration",
    "concept_measurement_lookup",
    "compose_from_sources",
    "explain_previous_turn",
    "report_issue",
    "ask_user",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

struct Case {
    id: String,
    language: String,
    intent: String,
    prompt: String,
    expected: String,
    prohibited: String,
}

fn field(record: &[&str], wanted: &str) -> String {
    let raw = record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .find_map(|(name, value)| (name == wanted).then(|| value.trim().to_owned()))
        .unwrap_or_default();
    raw.trim_matches('"').replace("\"\"", "\"")
}

fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for language in ["en", "ru", "hi", "zh", "es"] {
        let path = repo_root().join(format!(
            "data/benchmarks/capability-routing/{language}.lino"
        ));
        let text = fs::read_to_string(&path).expect("routing partition readable");
        let mut record: Vec<&str> = Vec::new();
        let mut records: Vec<Vec<&str>> = Vec::new();
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            if !line.starts_with(char::is_whitespace) && !record.is_empty() {
                records.push(std::mem::take(&mut record));
            }
            record.push(line);
        }
        if !record.is_empty() {
            records.push(record);
        }
        for record in &records {
            cases.push(Case {
                id: field(record, "id"),
                language: language.to_owned(),
                intent: field(record, "intent"),
                prompt: field(record, "prompt"),
                expected: field(record, "expected_capability"),
                prohibited: field(record, "prohibited_capability"),
            });
        }
    }
    cases
}

fn resolved(outcome: &RoutingOutcome) -> Option<String> {
    match outcome {
        RoutingOutcome::Routed { capability } | RoutingOutcome::Lowered { capability, .. } => {
            Some(capability.clone())
        }
        RoutingOutcome::HonestGap { .. } | RoutingOutcome::Ask { .. } => None,
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if let Some(position) = arguments.iter().position(|value| value == "--prompt") {
        for prompt in &arguments[position + 1..] {
            let objects = object_type(prompt);
            let highest = objects.first().copied().unwrap_or_default();
            println!(
                "{prompt:?}\n  objects={objects:?}\n  acts={:?}\n  locus={:?}\n  outcome={:?}",
                acts(prompt),
                locus_of(highest, prompt),
                route(prompt, ADVERTISED),
            );
        }
        return;
    }
    let verbose = std::env::args().any(|argument| argument == "--verbose");
    let cases = cases();
    let mut passing = 0usize;
    let mut misroutes = 0usize;
    let mut silent = 0usize;
    let mut per_cell: BTreeMap<(String, String), usize> = BTreeMap::new();
    for case in &cases {
        let outcome = route(&case.prompt, ADVERTISED);
        match resolved(&outcome) {
            Some(capability) if capability == case.expected => passing += 1,
            Some(capability) => {
                if capability == case.prohibited {
                    misroutes += 1;
                }
                *per_cell
                    .entry((case.intent.clone(), case.language.clone()))
                    .or_default() += 1;
                if verbose {
                    let objects = object_type(&case.prompt);
                    let highest = objects.first().copied().unwrap_or_default();
                    println!(
                        "{} [{} {}] -> {capability} (expected {}) objects={objects:?} \
                         acts={:?} locus={:?}",
                        case.id,
                        case.language,
                        case.intent,
                        case.expected,
                        acts(&case.prompt),
                        locus_of(highest, &case.prompt),
                    );
                }
            }
            None => {
                silent += 1;
                *per_cell
                    .entry((case.intent.clone(), case.language.clone()))
                    .or_default() += 1;
                if verbose {
                    let objects = object_type(&case.prompt);
                    let highest = objects.first().copied().unwrap_or_default();
                    println!(
                        "{} [{} {}] -> {outcome:?} (expected {}) objects={objects:?} \
                         acts={:?} locus={:?}",
                        case.id,
                        case.language,
                        case.intent,
                        case.expected,
                        acts(&case.prompt),
                        locus_of(highest, &case.prompt),
                    );
                }
            }
        }
    }
    println!(
        "capability_routing_cases_passing {passing} / {}",
        cases.len()
    );
    println!("cross_tool_misroutes {misroutes}");
    println!("silent_unknowns {silent}");
    for (cell, failures) in &per_cell {
        println!("  failing {}/{} {failures}", cell.0, cell.1);
    }
}
