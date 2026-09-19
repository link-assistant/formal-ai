#!/usr/bin/env rust-script
//! Keep capability-routing coverage and debt tied to measurements.
//!
//! The ledger is deliberately two-sided: every recorded value must equal the
//! current tree, and a comparison with the target branch prevents a downward
//! measure from rising or an upward measure from falling.  Better measurements
//! therefore require the ledger to move in the same commit; placeholders and
//! stale "targets" cannot make the gate green.
//!
//! Usage:
//!   rust-script scripts/check-capability-routing.rs --base origin/main
//!   rust-script --test scripts/check-capability-routing.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LEDGER: &str = "data/meta/capability-routing-ratchet.lino";
const CORPUS: &str = "data/benchmarks/capability-routing";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Ratchet {
    values: BTreeMap<String, u64>,
    upward: BTreeSet<String>,
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

fn parse_ratchet(text: &str) -> Result<Ratchet, String> {
    let mut ratchet = Ratchet::default();
    let mut measure: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "ceiling" {
            measure = None;
        } else if let Some(value) = trimmed.strip_prefix("measure ") {
            measure = Some(unquote(value));
        } else if let Some(value) = trimmed.strip_prefix("direction ") {
            let name = measure
                .as_ref()
                .ok_or_else(|| "`direction` has no `measure` above it".to_owned())?;
            match unquote(value).as_str() {
                "up" => {
                    ratchet.upward.insert(name.clone());
                }
                "down" => {}
                other => return Err(format!("measure `{name}` has direction `{other}`")),
            }
        } else if let Some(value) = trimmed.strip_prefix("value ") {
            let name = measure
                .as_ref()
                .ok_or_else(|| "`value` has no `measure` above it".to_owned())?;
            let value = unquote(value)
                .parse::<u64>()
                .map_err(|error| format!("measure `{name}`: {error}"))?;
            if ratchet.values.insert(name.clone(), value).is_some() {
                return Err(format!("measure `{name}` is declared twice"));
            }
        }
    }
    if ratchet.values.is_empty() {
        return Err("the ratchet declares no measures".to_owned());
    }
    Ok(ratchet)
}

fn field<'a>(record: &'a [&str], wanted: &str) -> Option<&'a str> {
    record.iter().find_map(|line| {
        let (name, value) = line.trim().split_once(' ')?;
        (name == wanted).then(|| value.trim().trim_matches('"'))
    })
}

fn records(text: &str) -> Vec<Vec<&str>> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
    {
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(std::mem::take(&mut current));
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn corpus_measures(root: &Path) -> Result<BTreeMap<String, u64>, String> {
    let directory = root.join(CORPUS);
    let mut files: Vec<PathBuf> = fs::read_dir(&directory)
        .map_err(|error| format!("{CORPUS}: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("lino"))
        .collect();
    files.sort();
    let mut intents = BTreeSet::new();
    let mut languages = BTreeSet::new();
    let mut cells: BTreeMap<(String, String), u64> = BTreeMap::new();
    for path in files {
        let text =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        for record in records(&text) {
            if field(&record, "record_type") != Some("capability_routing_case") {
                continue;
            }
            let intent = field(&record, "intent")
                .ok_or_else(|| format!("{}: routing case has no intent", path.display()))?;
            let language = field(&record, "language")
                .ok_or_else(|| format!("{}: routing case has no language", path.display()))?;
            intents.insert(intent.to_owned());
            languages.insert(language.to_owned());
            *cells
                .entry((intent.to_owned(), language.to_owned()))
                .or_default() += 1;
        }
    }
    if cells.is_empty() {
        return Err(format!(
            "{CORPUS} contains no capability_routing_case records"
        ));
    }
    let mut measured = BTreeMap::new();
    measured.insert("intents_measured".to_owned(), intents.len() as u64);
    measured.insert("languages_measured".to_owned(), languages.len() as u64);
    measured.insert(
        "paraphrases_per_intent_per_language".to_owned(),
        cells.values().copied().min().unwrap_or(0),
    );
    Ok(measured)
}

fn quoted_values(line: &str) -> usize {
    let mut count = 0;
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '"' {
            continue;
        }
        if quoted && chars.peek() == Some(&'"') {
            chars.next();
            continue;
        }
        quoted = !quoted;
        if !quoted {
            count += 1;
        }
    }
    count
}

fn memorized_capability_cues(text: &str) -> u64 {
    let mut inside_cues = false;
    let mut count = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "cues" {
            inside_cues = true;
            continue;
        }
        if line.starts_with("  capability ") {
            inside_cues = false;
        } else if inside_cues && line.starts_with("      ") {
            count += quoted_values(line) as u64;
        }
    }
    count
}

fn phrase_rows(text: &str) -> u64 {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("phrase ") || line.starts_with("keyword "))
        .count() as u64
}

fn function_body<'a>(text: &'a str, signature: &str) -> &'a str {
    let Some(start) = text.find(signature) else {
        return "";
    };
    let tail = &text[start..];
    let Some(open) = tail.find('{') else {
        return "";
    };
    let body = &tail[open + 1..];
    let mut depth = 1_i64;
    for (index, character) in body.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &body[..index];
                }
            }
            _ => {}
        }
    }
    ""
}

/// Top-level conditional route arms in the two planner dispatch functions.
/// This counts the general source-order cascade, not names of individual
/// handlers, and therefore remains meaningful while handlers are migrated.
fn planner_route_arms(text: &str) -> u64 {
    [
        "fn plan_chat_step_routes(",
        "pub(super) fn plan_settled_routes(",
    ]
    .into_iter()
    .map(|signature| {
        function_body(text, signature)
            .lines()
            .filter(|line| line.starts_with("    if "))
            .count() as u64
    })
    .sum()
}

fn static_measures(root: &Path) -> Result<BTreeMap<String, u64>, String> {
    let mut measured = corpus_measures(root)?;
    let capabilities = fs::read_to_string(root.join("data/seed/agentic-tool-capabilities.lino"))
        .map_err(|error| format!("agentic-tool-capabilities.lino: {error}"))?;
    measured.insert(
        "memorized_capability_cues".to_owned(),
        memorized_capability_cues(&capabilities),
    );
    let intent_routing = fs::read_to_string(root.join("data/seed/intent-routing.lino"))
        .map_err(|error| format!("intent-routing.lino: {error}"))?;
    measured.insert(
        "intent_routing_phrase_rows".to_owned(),
        phrase_rows(&intent_routing),
    );
    let planner = fs::read_to_string(root.join("src/agentic_coding/planner.rs"))
        .map_err(|error| format!("planner.rs: {error}"))?;
    measured.insert(
        "planner_route_arms".to_owned(),
        planner_route_arms(&planner),
    );
    let frontier = fs::read_to_string(root.join("tests/unit/issue_1138_frontier_classes.rs"))
        .map_err(|error| format!("issue_1138_frontier_classes.rs: {error}"))?;
    let expected = [
        "news_class_routes_to_a_live_search",
        "non_understanding_class_re_renders_the_previous_turn",
        "compose_class_routes_to_the_composition_procedure",
        "demonstrate_class_forces_the_response_language",
        "schedule_class_routes_to_the_calendar",
        "measurement_class_routes_to_a_concept_measurement_lookup",
        "ui_complaint_class_routes_to_a_structured_report",
    ];
    measured.insert(
        "frontier_prompts_open".to_owned(),
        expected
            .iter()
            .filter(|name| !frontier.contains(*name))
            .count() as u64,
    );
    Ok(measured)
}

fn parse_runtime_measures(output: &str) -> Result<BTreeMap<String, u64>, String> {
    let mut measured = BTreeMap::new();
    for line in output.lines() {
        let mut words = line.split_whitespace();
        let Some(name) = words.next() else { continue };
        if ![
            "capability_routing_cases_passing",
            "cross_tool_misroutes",
            "silent_unknowns",
        ]
        .contains(&name)
        {
            continue;
        }
        let value = words
            .next()
            .ok_or_else(|| format!("runtime measure `{name}` has no value"))?
            .parse::<u64>()
            .map_err(|error| format!("runtime measure `{name}`: {error}"))?;
        measured.insert(name.to_owned(), value);
    }
    for name in [
        "capability_routing_cases_passing",
        "cross_tool_misroutes",
        "silent_unknowns",
    ] {
        if !measured.contains_key(name) {
            return Err(format!("measurement output omitted `{name}`"));
        }
    }
    Ok(measured)
}

fn command_output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|error| format!("could not run {program} {args:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {args:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn runtime_measures(root: &Path) -> Result<BTreeMap<String, u64>, String> {
    let output = command_output(
        root,
        "cargo",
        &[
            "+1.98.1",
            "run",
            "--quiet",
            "--example",
            "measure_capability_routing",
        ],
    )?;
    parse_runtime_measures(&output)
}

fn check_exact(ratchet: &Ratchet, measured: &BTreeMap<String, u64>) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, expected) in &ratchet.values {
        match measured.get(name) {
            Some(actual) if actual == expected => {}
            Some(actual) => failures.push(format!(
                "{name}: measured {actual}, ledger records {expected}; update the implementation or record the honest measurement in {LEDGER}"
            )),
            None => failures.push(format!("{name}: ledger measure has no implementation")),
        }
    }
    for name in measured.keys() {
        if !ratchet.values.contains_key(name) {
            failures.push(format!("{name}: measured but absent from {LEDGER}"));
        }
    }
    failures
}

fn check_previous(previous: &Ratchet, current: &Ratchet) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, before) in &previous.values {
        let Some(after) = current.values.get(name) else {
            failures.push(format!("{name}: ratchet measure was deleted"));
            continue;
        };
        if current.upward.contains(name) {
            if after < before {
                failures.push(format!(
                    "{name}: upward floor fell from {before} to {after}"
                ));
            }
        } else if after > before {
            failures.push(format!(
                "{name}: downward ceiling rose from {before} to {after}"
            ));
        }
    }
    failures
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    command_output(root, "git", args).map(|value| value.trim().to_owned())
}

fn ratchet_at(root: &Path, revision: &str) -> Result<Option<Ratchet>, String> {
    let spec = format!("{revision}:{LEDGER}");
    match command_output(root, "git", &["show", &spec]) {
        Ok(text) => parse_ratchet(&text).map(Some),
        Err(error) if error.contains("does not exist") || error.contains("exists on disk") => {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

struct Options {
    root: PathBuf,
    base: String,
    measurement_output: Option<PathBuf>,
}

fn options() -> Result<Options, String> {
    let mut root = PathBuf::from(".");
    let mut base = env::var("GITHUB_BASE_REF")
        .ok()
        .filter(|value| !value.is_empty())
        .map(|value| format!("origin/{value}"));
    let mut measurement_output = None;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--repo" => root = PathBuf::from(args.next().ok_or("--repo requires a value")?),
            "--base" => base = Some(args.next().ok_or("--base requires a value")?),
            "--measurement-output" => {
                measurement_output = Some(PathBuf::from(
                    args.next().ok_or("--measurement-output requires a value")?,
                ));
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    let base = base.ok_or(
        "--base <rev> is required; CI supplies GITHUB_BASE_REF and local runs should use origin/main",
    )?;
    if root == Path::new(".") {
        root = PathBuf::from(git(&root, &["rev-parse", "--show-toplevel"])?);
    }
    Ok(Options {
        root,
        base,
        measurement_output,
    })
}

fn run() -> Result<(), String> {
    let options = options()?;
    let ledger = fs::read_to_string(options.root.join(LEDGER))
        .map_err(|error| format!("{LEDGER}: {error}"))?;
    let ratchet = parse_ratchet(&ledger)?;
    let mut measured = static_measures(&options.root)?;
    let runtime = if let Some(path) = options.measurement_output {
        let output =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        parse_runtime_measures(&output)?
    } else {
        runtime_measures(&options.root)?
    };
    measured.extend(runtime);

    println!("capability-routing ratchet ({LEDGER}):");
    for (name, value) in &measured {
        println!("  {name}: measured {value}");
    }
    let mut failures = check_exact(&ratchet, &measured);
    match ratchet_at(&options.root, &options.base) {
        Ok(Some(previous)) => failures.extend(check_previous(&previous, &ratchet)),
        Ok(None) => println!(
            "  ({} has no {LEDGER}; no earlier ratchet to compare)",
            options.base
        ),
        Err(error) => return Err(format!("could not compare {}: {error}", options.base)),
    }
    if failures.is_empty() {
        println!("capability-routing ratchet holds");
        return Ok(());
    }
    for failure in &failures {
        println!("::error file={LEDGER}::{failure}");
    }
    Err(format!("{} capability-routing failure(s)", failures.len()))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("check-capability-routing: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_directions_and_values() {
        let parsed = parse_ratchet(
            "capability_routing_ratchet\n  ceiling\n    measure passing\n    value 4\n    direction up\n  ceiling\n    measure misses\n    value 0\n    direction down\n",
        )
        .unwrap();
        assert_eq!(parsed.values["passing"], 4);
        assert!(parsed.upward.contains("passing"));
        assert!(!parsed.upward.contains("misses"));
    }

    #[test]
    fn parses_runtime_output_without_trusting_the_denominator() {
        let parsed = parse_runtime_measures(
            "capability_routing_cases_passing 420 / 420\ncross_tool_misroutes 0\nsilent_unknowns 0\n",
        )
        .unwrap();
        assert_eq!(parsed["capability_routing_cases_passing"], 420);
        assert_eq!(parsed["cross_tool_misroutes"], 0);
        assert_eq!(parsed["silent_unknowns"], 0);
    }

    #[test]
    fn exact_check_rejects_better_as_well_as_worse_values() {
        let ratchet = Ratchet {
            values: BTreeMap::from([("passing".to_owned(), 4)]),
            upward: BTreeSet::from(["passing".to_owned()]),
        };
        assert!(check_exact(&ratchet, &BTreeMap::from([("passing".to_owned(), 4)])).is_empty());
        assert_eq!(
            check_exact(&ratchet, &BTreeMap::from([("passing".to_owned(), 5)])).len(),
            1
        );
        assert_eq!(
            check_exact(&ratchet, &BTreeMap::from([("passing".to_owned(), 3)])).len(),
            1
        );
    }

    #[test]
    fn previous_check_obeys_each_direction() {
        let previous = Ratchet {
            values: BTreeMap::from([("passing".to_owned(), 4), ("misses".to_owned(), 3)]),
            upward: BTreeSet::from(["passing".to_owned()]),
        };
        let improved = Ratchet {
            values: BTreeMap::from([("passing".to_owned(), 5), ("misses".to_owned(), 2)]),
            upward: BTreeSet::from(["passing".to_owned()]),
        };
        assert!(check_previous(&previous, &improved).is_empty());
        assert_eq!(check_previous(&improved, &previous).len(), 2);
    }

    #[test]
    fn cue_and_phrase_measurements_count_data_rows() {
        let cues = "  capability x\n    cues\n      en (\"one\" \"two\")\n      zh (\"三\")\n  capability y\n    aliases \"y\"\n";
        assert_eq!(memorized_capability_cues(cues), 3);
        assert_eq!(
            phrase_rows("    phrase one\n    keyword two\n    slug three\n"),
            2
        );
    }

    #[test]
    fn planner_measure_counts_only_top_level_dispatch_conditions() {
        let source = "fn plan_chat_step_routes() {\n    if one {\n        if nested {}\n    }\n}\npub(super) fn plan_settled_routes() {\n    if two {}\n}\n";
        assert_eq!(planner_route_arms(source), 2);
    }
}
