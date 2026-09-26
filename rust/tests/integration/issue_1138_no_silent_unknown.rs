//! Issue #1138 B10, plan 10 leaf 10: no benchmark prompt reaches the unknown
//! opener.
//!
//! Every routing path ends in one of four observable outcomes — routed,
//! lowered, honest gap, ask — so "silent UNKNOWN" is unreachable by
//! construction and #745 clause 3 is structural rather than aspirational. This
//! integration test scans *every* committed suite under `data/benchmarks/`,
//! not just the routing corpus, because a prompt the system cannot place is a
//! defect wherever it was written down.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::FormalAiEngine;
use formal_ai::capability_routing::{RoutingOutcome, route};

/// Everything the decision table's rows can name, so a failure here is a
/// routing failure rather than an unadvertised tool.
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

const DOCUMENTED_PROMPT: &str = "Find y: 7 * y = 84";
const DOCUMENTED_ANSWER: &str = "12";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

/// Every `prompt` field in every `.lino` file under `data/benchmarks/`, with the
/// file it came from.
fn benchmark_prompts() -> Vec<(String, String)> {
    let mut prompts = Vec::new();
    for entry in walkdir::WalkDir::new(repo_root().join("data/benchmarks"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("lino") {
            continue;
        }
        let origin = entry
            .path()
            .strip_prefix(repo_root())
            .unwrap_or_else(|_| entry.path())
            .display()
            .to_string();
        let text = fs::read_to_string(entry.path()).unwrap_or_default();
        for line in text.lines() {
            let trimmed = line.trim();
            let Some(raw) = trimmed.strip_prefix("prompt ") else {
                continue;
            };
            let raw = raw.trim();
            let value = if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
                raw[1..raw.len() - 1].replace("\"\"", "\"")
            } else if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
                raw[1..raw.len() - 1].replace("''", "'")
            } else {
                raw.to_owned()
            };
            if !value.trim().is_empty() {
                prompts.push((origin.clone(), value));
            }
        }
    }
    prompts
}

/// The seeded unknown openers, long enough that a substring match is meaningful.
fn unknown_openers() -> Vec<String> {
    let text = fs::read_to_string(repo_root().join("data/seed/unknown-openers.lino"))
        .expect("data/seed/unknown-openers.lino readable");
    text.lines()
        .filter_map(|line| line.trim().split_once(' '))
        .map(|(_, value)| value.trim().trim_matches('"').to_owned())
        .filter(|value| value.chars().count() >= 12)
        .collect()
}

#[test]
fn no_benchmark_prompt_reaches_the_unknown_opener() {
    let documented = FormalAiEngine.answer(DOCUMENTED_PROMPT).answer;
    assert_eq!(documented, DOCUMENTED_ANSWER);
    let prompts = benchmark_prompts();
    assert!(
        prompts.len() >= 700,
        "the benchmark corpora should carry the committed suites, got {}",
        prompts.len()
    );
    let openers = unknown_openers();
    assert!(
        !openers.is_empty(),
        "the unknown openers must be readable for this gate to mean anything"
    );

    let mut silent: Vec<String> = Vec::new();
    for (origin, prompt) in &prompts {
        let answer = FormalAiEngine.answer(prompt).answer;
        if openers.iter().any(|opener| answer.contains(opener)) {
            silent.push(format!("{origin}: {prompt:?}"));
        }
    }
    assert!(
        silent.is_empty(),
        "{} of {} benchmark prompts reach the unknown opener. Each must instead be \
         routed, lowered to a named fallback, reported as an honest gap that names the \
         capability it needed, or turned into one question naming both readings. First \
         offenders: {:?}",
        silent.len(),
        prompts.len(),
        silent.iter().take(12).collect::<Vec<_>>()
    );

    // The opener scan alone proves only that a particular sentence was not
    // emitted. What makes the silent UNKNOWN unreachable *by construction* is
    // that every prompt ends in one of the four declared outcomes, each with a
    // distinct observable (plan 10 Architecture 5). Assert that too, so the gate
    // cannot pass because the fallback was reworded.
    let mut unplaced: Vec<String> = Vec::new();
    for (origin, prompt) in &prompts {
        let placed = match route(prompt, ADVERTISED) {
            RoutingOutcome::Routed { capability } => !capability.trim().is_empty(),
            RoutingOutcome::Lowered {
                preferred,
                capability,
            } => !preferred.trim().is_empty() && !capability.trim().is_empty(),
            RoutingOutcome::HonestGap { needed, missing } => {
                !needed.trim().is_empty() && !missing.trim().is_empty()
            }
            RoutingOutcome::Ask { readings } => readings.len() >= 2,
        };
        if !placed {
            unplaced.push(format!("{origin}: {prompt:?}"));
        }
    }
    assert!(
        unplaced.is_empty(),
        "{} of {} benchmark prompts ended in an outcome that names nothing, so the \
         four-outcome exhaustiveness #745 clause 3 asks for is not structural yet: {:?}",
        unplaced.len(),
        prompts.len(),
        unplaced.iter().take(12).collect::<Vec<_>>()
    );
}
