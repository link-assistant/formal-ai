#!/usr/bin/env rust-script
//! Publish coverage reports and enforce a non-decreasing coverage ratchet.
//!
//! Issue #895 (E68 / issue #710): the `coverage` job produced an LCOV file,
//! retained it as an artifact and stopped there. Nothing read the numbers, so
//! coverage could fall to zero with CI green, and the "double the tests toward
//! ~100%" ask of issues #449/#451 had no measurable meaning.
//!
//! This script closes that gap:
//!
//! * it parses LCOV into per-file and per-denominator totals;
//! * it publishes a **human-readable** `summary-<denominator>.md` and a
//!   **machine-readable** `summary-<denominator>.json` next to the LCOV file,
//!   and appends the table to `$GITHUB_STEP_SUMMARY` when running in Actions;
//! * it fails the build when a denominator drops below its committed baseline
//!   (minus a small tolerance for measurement noise);
//! * it refuses to *lower* a committed baseline without an explicit reviewed
//!   justification, which is recorded in `coverage/baseline.json` itself;
//! * it keeps the browser denominator honest: every browser production file
//!   must either be measured or be listed, with a reason, in the committed
//!   unmeasured inventory, so the uncovered surface can never grow silently.
//!
//! Usage:
//!   rust-script scripts/check-coverage-ratchet.rs --only rust --lcov rust=lcov.info
//!   rust-script scripts/check-coverage-ratchet.rs --only browser
//!   rust-script scripts/check-coverage-ratchet.rs --only rust --lcov rust=lcov.info \
//!     --update-baseline --justification "reviewed: dropped the vendored parser"
//!
//! Tests: rust-script --test scripts/check-coverage-ratchet.rs
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! walkdir = "2"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const DEFAULT_BASELINE: &str = "coverage/baseline.json";
const DEFAULT_OUT_DIR: &str = "coverage";

// ---------------------------------------------------------------------------
// Baseline file
// ---------------------------------------------------------------------------

/// The committed, reviewed coverage baseline. Every field is spelled out and
/// unknown fields are rejected so the file cannot quietly grow a setting that
/// nothing reads.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    /// Prose that travels with the numbers, so a reviewer reading the diff sees
    /// the rule without opening this script.
    policy: String,
    /// Denominator name -> its reviewed floor. A `BTreeMap` keeps the file
    /// ordering deterministic across rewrites.
    denominators: BTreeMap<String, Denominator>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Denominator {
    /// Human-readable description of *what* is being measured. Two suites with
    /// different denominators are never averaged into one dishonest number.
    label: String,
    /// Where the LCOV report for this denominator is read from by default.
    lcov: String,
    /// Path prefixes (repo-relative, `/`-separated) that belong to this
    /// denominator. Anything else in the LCOV file — test files, vendored
    /// sources — is ignored.
    include: Vec<String>,
    /// Path prefixes excluded from the denominator even when they match
    /// `include`, e.g. generated bundles and deployment mirrors.
    exclude: Vec<String>,
    /// Reviewed floor for line coverage, in percent.
    lines_percent: f64,
    /// Reviewed floor for function coverage, in percent.
    functions_percent: f64,
    /// Slack for measurement noise (parallel test ordering, inlining). A drop
    /// larger than this fails the build.
    tolerance_percent: f64,
    /// Date the floors above were last reviewed and updated.
    reviewed: String,
    /// Where the reviewed numbers came from (workflow run, command).
    evidence: String,
    /// Present only when the last reviewed update *lowered* a floor: the
    /// justification the author had to supply on the command line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lowered_reason: Option<String>,
    /// Optional honesty gate: every production file under `roots` must either
    /// appear in the LCOV report or be listed, with a reason, in
    /// `unmeasured_list`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inventory: Option<Inventory>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    /// Directories walked when inventorying production files.
    roots: Vec<String>,
    /// File extensions considered production code.
    extensions: Vec<String>,
    /// Committed list of `path<TAB>reason` rows for files this denominator does
    /// not measure yet.
    unmeasured_list: String,
}

fn load_baseline(path: &Path) -> Result<Baseline, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Could not read baseline {}: {error}", path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("Could not parse baseline {}: {error}", path.display()))
}

fn write_baseline(path: &Path, baseline: &Baseline) -> Result<(), String> {
    let mut serialized = serde_json::to_string_pretty(baseline)
        .map_err(|error| format!("Could not serialize baseline: {error}"))?;
    serialized.push('\n');
    fs::write(path, serialized)
        .map_err(|error| format!("Could not write baseline {}: {error}", path.display()))
}

// ---------------------------------------------------------------------------
// LCOV parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
struct FileCoverage {
    path: String,
    lines_found: usize,
    lines_hit: usize,
    functions_found: usize,
    functions_hit: usize,
}

impl FileCoverage {
    fn lines_missed(&self) -> usize {
        self.lines_found.saturating_sub(self.lines_hit)
    }
}

/// Percentage helper that treats an empty denominator as 0% rather than NaN,
/// and rounds to two decimals so a stored baseline compares exactly.
fn percent(hit: usize, found: usize) -> f64 {
    if found == 0 {
        return 0.0;
    }
    round2(hit as f64 * 100.0 / found as f64)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Normalize an LCOV `SF:` path to a repo-relative, `/`-separated path.
///
/// `cargo llvm-cov` writes absolute paths; node's LCOV reporter writes paths
/// relative to the working directory. Both must land on the same string so a
/// single `include` prefix matches either producer.
fn normalize_source_path(raw: &str, repo_root: &Path) -> String {
    let cleaned = raw.trim().replace('\\', "/");
    let root = repo_root.to_string_lossy().replace('\\', "/");
    let root = root.trim_end_matches('/');
    let relative = cleaned
        .strip_prefix(&format!("{root}/"))
        .unwrap_or(&cleaned)
        .to_string();
    relative.trim_start_matches("./").to_string()
}

/// Parse an LCOV report into per-file totals.
///
/// Line and function totals are computed from the per-line `DA:`/`FNDA:`
/// records rather than the `LF`/`LH` summary records: the summaries are
/// optional, and some producers emit them before the data they summarize.
/// A file that appears twice (one record per test binary, which
/// `cargo llvm-cov` can emit) is merged by taking the maximum hit count per
/// line so a line covered by any binary counts once.
fn parse_lcov(content: &str, repo_root: &Path) -> Vec<FileCoverage> {
    let mut files: BTreeMap<String, (BTreeMap<u64, u64>, BTreeMap<String, u64>)> = BTreeMap::new();
    let mut current: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SF:") {
            current = Some(normalize_source_path(rest, repo_root));
            files.entry(current.clone().unwrap()).or_default();
        } else if line == "end_of_record" {
            current = None;
        } else if let Some(name) = current.as_ref() {
            let entry = files.entry(name.clone()).or_default();
            if let Some(rest) = line.strip_prefix("DA:") {
                let mut parts = rest.split(',');
                let (Some(number), Some(count)) = (parts.next(), parts.next()) else {
                    continue;
                };
                let (Ok(number), Ok(count)) = (number.trim().parse::<u64>(), parse_count(count))
                else {
                    continue;
                };
                let slot = entry.0.entry(number).or_insert(0);
                *slot = (*slot).max(count);
            } else if let Some(rest) = line.strip_prefix("FN:") {
                // `FN:<line>,<name>` declares a function; `FNDA` reports its
                // hits. Declaring it here keeps never-executed functions in the
                // denominator even when the producer omits their `FNDA` row.
                if let Some((_, name)) = rest.split_once(',') {
                    entry.1.entry(name.trim().to_string()).or_insert(0);
                }
            } else if let Some(rest) = line.strip_prefix("FNDA:") {
                let Some((count, name)) = rest.split_once(',') else {
                    continue;
                };
                let Ok(count) = parse_count(count) else {
                    continue;
                };
                let slot = entry.1.entry(name.trim().to_string()).or_insert(0);
                *slot = (*slot).max(count);
            }
        }
    }

    files
        .into_iter()
        .map(|(path, (lines, functions))| FileCoverage {
            path,
            lines_found: lines.len(),
            lines_hit: lines.values().filter(|count| **count > 0).count(),
            functions_found: functions.len(),
            functions_hit: functions.values().filter(|count| **count > 0).count(),
        })
        .collect()
}

fn parse_count(raw: &str) -> Result<u64, ()> {
    let raw = raw.trim();
    // LCOV writes `-` for "not instrumented" and some producers emit floats.
    if raw == "-" {
        return Ok(0);
    }
    raw.parse::<u64>()
        .or_else(|_| raw.parse::<f64>().map(|value| value as u64))
        .map_err(|_| ())
}

fn path_matches(path: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| path.starts_with(prefix))
}

fn in_denominator(path: &str, denominator: &Denominator) -> bool {
    path_matches(path, &denominator.include) && !path_matches(path, &denominator.exclude)
}

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
struct Measurement {
    denominator: String,
    label: String,
    files: Vec<FileCoverage>,
    lines_found: usize,
    lines_hit: usize,
    lines_percent: f64,
    functions_found: usize,
    functions_hit: usize,
    functions_percent: f64,
}

fn measure(name: &str, denominator: &Denominator, files: Vec<FileCoverage>) -> Measurement {
    let mut kept: Vec<FileCoverage> = files
        .into_iter()
        .filter(|file| in_denominator(&file.path, denominator))
        .collect();
    kept.sort_by(|left, right| left.path.cmp(&right.path));

    let lines_found = kept.iter().map(|file| file.lines_found).sum();
    let lines_hit = kept.iter().map(|file| file.lines_hit).sum();
    let functions_found = kept.iter().map(|file| file.functions_found).sum();
    let functions_hit = kept.iter().map(|file| file.functions_hit).sum();

    Measurement {
        denominator: name.to_string(),
        label: denominator.label.clone(),
        files: kept,
        lines_found,
        lines_hit,
        lines_percent: percent(lines_hit, lines_found),
        functions_found,
        functions_hit,
        functions_percent: percent(functions_hit, functions_found),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RatchetStatus {
    /// At or above the reviewed floor, within the drift band.
    Held,
    /// Above the floor by more than the tolerance: the baseline should be
    /// raised in this pull request to lock the gain in.
    Improved,
    /// Below the floor by more than the tolerance: the build fails.
    Regressed,
}

#[derive(Debug, Clone, PartialEq)]
struct MetricVerdict {
    metric: &'static str,
    measured: f64,
    baseline: f64,
    tolerance: f64,
    status: RatchetStatus,
}

impl MetricVerdict {
    fn delta(&self) -> f64 {
        round2(self.measured - self.baseline)
    }
}

fn classify(metric: &'static str, measured: f64, baseline: f64, tolerance: f64) -> MetricVerdict {
    // Compare on two-decimal values so a stored baseline of 61.23 is never
    // "below itself" because of binary floating point.
    let measured = round2(measured);
    let baseline = round2(baseline);
    let status = if measured + 1e-9 < baseline - tolerance {
        RatchetStatus::Regressed
    } else if measured > baseline + tolerance + 1e-9 {
        RatchetStatus::Improved
    } else {
        RatchetStatus::Held
    };
    MetricVerdict {
        metric,
        measured,
        baseline,
        tolerance,
        status,
    }
}

fn verdicts(measurement: &Measurement, denominator: &Denominator) -> Vec<MetricVerdict> {
    vec![
        classify(
            "lines",
            measurement.lines_percent,
            denominator.lines_percent,
            denominator.tolerance_percent,
        ),
        classify(
            "functions",
            measurement.functions_percent,
            denominator.functions_percent,
            denominator.tolerance_percent,
        ),
    ]
}

// ---------------------------------------------------------------------------
// Unmeasured inventory (browser honesty gate)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct InventoryReport {
    /// Production files that are neither measured nor declared unmeasured.
    undeclared: Vec<String>,
    /// Declared rows whose file no longer exists.
    missing: Vec<String>,
    /// Declared rows for files that *are* measured now and must be pruned.
    stale: Vec<String>,
    /// Rows declared without a reason.
    unexplained: Vec<String>,
}

impl InventoryReport {
    fn is_clean(&self) -> bool {
        self.undeclared.is_empty()
            && self.missing.is_empty()
            && self.stale.is_empty()
            && self.unexplained.is_empty()
    }
}

fn parse_unmeasured_list(content: &str) -> Vec<(String, String)> {
    content
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| match line.split_once('\t') {
            Some((path, reason)) => (path.trim().to_string(), reason.trim().to_string()),
            None => (line.trim().to_string(), String::new()),
        })
        .collect()
}

fn collect_production_files(
    repo_root: &Path,
    inventory: &Inventory,
    exclude: &[String],
) -> Vec<String> {
    let mut found = BTreeSet::new();
    for root in &inventory.roots {
        for entry in WalkDir::new(repo_root.join(root))
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let matches_extension = entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    inventory
                        .extensions
                        .iter()
                        .any(|wanted| wanted.eq_ignore_ascii_case(extension))
                });
            if !matches_extension {
                continue;
            }
            let relative = normalize_source_path(&entry.path().to_string_lossy(), repo_root);
            if path_matches(&relative, exclude) {
                continue;
            }
            found.insert(relative);
        }
    }
    found.into_iter().collect()
}

fn check_inventory(
    repo_root: &Path,
    inventory: &Inventory,
    exclude: &[String],
    measured: &BTreeSet<String>,
    declared: &[(String, String)],
) -> InventoryReport {
    let production = collect_production_files(repo_root, inventory, exclude);
    let declared_paths: BTreeSet<String> = declared.iter().map(|(path, _)| path.clone()).collect();

    let undeclared = production
        .iter()
        .filter(|path| !measured.contains(*path) && !declared_paths.contains(*path))
        .cloned()
        .collect();
    let production_set: BTreeSet<String> = production.into_iter().collect();
    let missing = declared_paths
        .iter()
        .filter(|path| !production_set.contains(*path))
        .cloned()
        .collect();
    let stale = declared_paths
        .iter()
        .filter(|path| production_set.contains(*path) && measured.contains(*path))
        .cloned()
        .collect();
    let unexplained = declared
        .iter()
        .filter(|(_, reason)| reason.is_empty())
        .map(|(path, _)| path.clone())
        .collect();

    InventoryReport {
        undeclared,
        missing,
        stale,
        unexplained,
    }
}

// ---------------------------------------------------------------------------
// Reports
// ---------------------------------------------------------------------------

#[path = "check-coverage-ratchet-reports.rs"]
mod reports;
use reports::{render_summary_json, render_summary_markdown};

// ---------------------------------------------------------------------------
// Command line
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default)]
struct Options {
    baseline: Option<String>,
    out_dir: Option<String>,
    only: Vec<String>,
    lcov_overrides: BTreeMap<String, String>,
    update_baseline: bool,
    justification: Option<String>,
    /// Date recorded as the last review of the floors, written by
    /// `--update-baseline`.
    reviewed: Option<String>,
    /// Where the updated numbers came from, written by `--update-baseline`.
    evidence: Option<String>,
}

fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Options, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--baseline" => options.baseline = Some(require_value(&mut args, "--baseline")?),
            "--out-dir" => options.out_dir = Some(require_value(&mut args, "--out-dir")?),
            "--only" => options.only.push(require_value(&mut args, "--only")?),
            "--update-baseline" => options.update_baseline = true,
            "--justification" => {
                options.justification = Some(require_value(&mut args, "--justification")?);
            }
            "--reviewed" => options.reviewed = Some(require_value(&mut args, "--reviewed")?),
            "--evidence" => options.evidence = Some(require_value(&mut args, "--evidence")?),
            "--lcov" => {
                let value = require_value(&mut args, "--lcov")?;
                let (name, path) = value.split_once('=').ok_or_else(|| {
                    "--lcov expects <denominator>=<path>, e.g. --lcov rust=lcov.info".to_string()
                })?;
                options
                    .lcov_overrides
                    .insert(name.to_string(), path.to_string());
            }
            other => return Err(format!("Unknown argument: {other}")),
        }
    }
    Ok(options)
}

fn require_value<I: Iterator<Item = String>>(args: &mut I, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} expects a value"))
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct DenominatorOutcome {
    name: String,
    markdown: String,
    json: String,
    failed: bool,
    messages: Vec<String>,
    measurement: Measurement,
}

/// Everything the run produced, so `main` only prints and sets the exit code
/// and the tests can assert on the same values CI acts on.
#[derive(Debug, Clone, PartialEq)]
struct RunReport {
    outcomes: Vec<DenominatorOutcome>,
    baseline_updated: bool,
}

impl RunReport {
    fn failed(&self) -> bool {
        self.outcomes.iter().any(|outcome| outcome.failed)
    }
}

fn selected_denominators(baseline: &Baseline, only: &[String]) -> Result<Vec<String>, String> {
    if only.is_empty() {
        return Ok(baseline.denominators.keys().cloned().collect());
    }
    for name in only {
        if !baseline.denominators.contains_key(name) {
            return Err(format!(
                "Unknown denominator `{name}`. Known: {}",
                baseline
                    .denominators
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    Ok(only.to_vec())
}

fn run(repo_root: &Path, options: &Options) -> Result<RunReport, String> {
    let baseline_path = repo_root.join(options.baseline.as_deref().unwrap_or(DEFAULT_BASELINE));
    let mut baseline = load_baseline(&baseline_path)?;
    let out_dir = repo_root.join(options.out_dir.as_deref().unwrap_or(DEFAULT_OUT_DIR));
    let names = selected_denominators(&baseline, &options.only)?;

    let mut outcomes = Vec::new();
    let mut updates: BTreeMap<String, Measurement> = BTreeMap::new();

    for name in &names {
        let denominator = baseline.denominators[name].clone();
        let lcov_path = repo_root.join(
            options
                .lcov_overrides
                .get(name)
                .unwrap_or(&denominator.lcov),
        );
        let content = fs::read_to_string(&lcov_path).map_err(|error| {
            format!(
                "Could not read the LCOV report for `{name}` at {}: {error}\n\
                 Generate it first (see docs/design/coverage-ratchet.md) or pass \
                 --lcov {name}=<path>.",
                lcov_path.display()
            )
        })?;
        let measurement = measure(name, &denominator, parse_lcov(&content, repo_root));

        let mut messages = Vec::new();
        let mut failed = false;

        if measurement.files.is_empty() {
            failed = true;
            messages.push(format!(
                "::error::No file in {} matched the `{name}` denominator ({}). \
                 An empty denominator would report 0% and hide a real regression.",
                lcov_path.display(),
                denominator.include.join(", ")
            ));
        }

        let verdicts = verdicts(&measurement, &denominator);
        for verdict in &verdicts {
            match verdict.status {
                RatchetStatus::Regressed => {
                    failed = true;
                    messages.push(format!(
                        "::error::Coverage ratchet broken for `{name}`: {} coverage fell to {:.2}% \
                         from the reviewed baseline of {:.2}% (tolerance {:.2} pp). Add tests, or \
                         lower the baseline deliberately with `rust-script \
                         scripts/check-coverage-ratchet.rs --only {name} --update-baseline \
                         --justification \"<reviewed reason>\"` in this pull request.",
                        verdict.metric, verdict.measured, verdict.baseline, verdict.tolerance
                    ));
                }
                RatchetStatus::Improved => {
                    messages.push(format!(
                        "::notice::Coverage for `{name}` {} rose to {:.2}% (baseline {:.2}%). Run \
                         `--only {name} --update-baseline` to lock the gain in.",
                        verdict.metric, verdict.measured, verdict.baseline
                    ));
                }
                RatchetStatus::Held => {}
            }
        }

        let measured_paths: BTreeSet<String> = measurement
            .files
            .iter()
            .map(|file| file.path.clone())
            .collect();
        let inventory_report = denominator.inventory.as_ref().map(|inventory| {
            let list_path = repo_root.join(&inventory.unmeasured_list);
            let declared = fs::read_to_string(&list_path)
                .map(|content| parse_unmeasured_list(&content))
                .unwrap_or_default();
            check_inventory(
                repo_root,
                inventory,
                &denominator.exclude,
                &measured_paths,
                &declared,
            )
        });

        if let (Some(report), Some(inventory)) =
            (inventory_report.as_ref(), denominator.inventory.as_ref())
        {
            if !report.is_clean() {
                failed = true;
                messages.extend(inventory_messages(name, inventory, report));
            }
        }

        let markdown = render_summary_markdown(&measurement, &verdicts, inventory_report.as_ref());
        let json = render_summary_json(
            &measurement,
            &denominator,
            &verdicts,
            inventory_report.as_ref(),
        );

        if options.update_baseline {
            let lowers = verdicts
                .iter()
                .any(|verdict| verdict.measured + 1e-9 < verdict.baseline);
            if lowers && options.justification.is_none() {
                return Err(format!(
                    "Refusing to lower the reviewed baseline for `{name}` without \
                     --justification \"<reviewed reason>\". A decrease must be an explicit, \
                     reviewed decision recorded in the baseline file."
                ));
            }
            updates.insert(name.clone(), measurement.clone());
        }

        outcomes.push(DenominatorOutcome {
            name: name.clone(),
            markdown,
            json,
            failed: failed && !options.update_baseline,
            messages,
            measurement,
        });
    }

    write_reports(&out_dir, &outcomes)?;

    let mut baseline_updated = false;
    if options.update_baseline {
        for (name, measurement) in &updates {
            let entry = baseline
                .denominators
                .get_mut(name)
                .expect("denominator was selected from the baseline");
            let lowered = measurement.lines_percent + 1e-9 < entry.lines_percent
                || measurement.functions_percent + 1e-9 < entry.functions_percent;
            entry.lines_percent = measurement.lines_percent;
            entry.functions_percent = measurement.functions_percent;
            entry.lowered_reason = if lowered {
                options.justification.clone()
            } else {
                None
            };
            if let Some(reviewed) = options.reviewed.as_ref() {
                entry.reviewed = reviewed.clone();
            }
            if let Some(evidence) = options.evidence.as_ref() {
                entry.evidence = evidence.clone();
            }
        }
        write_baseline(&baseline_path, &baseline)?;
        baseline_updated = true;
    }

    Ok(RunReport {
        outcomes,
        baseline_updated,
    })
}

fn inventory_messages(name: &str, inventory: &Inventory, report: &InventoryReport) -> Vec<String> {
    let list = &inventory.unmeasured_list;
    let mut messages = Vec::new();
    if !report.undeclared.is_empty() {
        messages.push(format!(
            "::error::`{name}` production file(s) are neither measured nor declared unmeasured: {}. \
             Add a test that exercises them, or add a `<path>\t<reason>` row to {list} so the \
             unmeasured surface stays visible.",
            report.undeclared.join(", ")
        ));
    }
    if !report.missing.is_empty() {
        messages.push(format!(
            "::error::{list} lists file(s) that no longer exist: {}. Remove the stale row(s).",
            report.missing.join(", ")
        ));
    }
    if !report.stale.is_empty() {
        messages.push(format!(
            "::error::{list} still lists file(s) that are measured now: {}. Prune the row(s) so the \
             inventory only records real debt.",
            report.stale.join(", ")
        ));
    }
    if !report.unexplained.is_empty() {
        messages.push(format!(
            "::error::{list} row(s) have no reason: {}. Every unmeasured file must say why.",
            report.unexplained.join(", ")
        ));
    }
    messages
}

fn write_reports(out_dir: &Path, outcomes: &[DenominatorOutcome]) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|error| format!("Could not create {}: {error}", out_dir.display()))?;
    for outcome in outcomes {
        let markdown = out_dir.join(format!("summary-{}.md", outcome.name));
        let json = out_dir.join(format!("summary-{}.json", outcome.name));
        fs::write(&markdown, &outcome.markdown)
            .map_err(|error| format!("Could not write {}: {error}", markdown.display()))?;
        fs::write(&json, &outcome.json)
            .map_err(|error| format!("Could not write {}: {error}", json.display()))?;
    }
    Ok(())
}

#[cfg(not(test))]
fn append_step_summary(outcomes: &[DenominatorOutcome]) {
    let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY") else {
        return;
    };
    use std::io::Write as _;
    let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    for outcome in outcomes {
        let _ = writeln!(file, "{}\n", outcome.markdown);
    }
}

#[cfg(not(test))]
fn main() {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            println!("::error::{error}");
            std::process::exit(2);
        }
    };

    let repo_root = std::env::current_dir().expect("Failed to get current directory");
    let report = match run(&repo_root, &options) {
        Ok(report) => report,
        Err(error) => {
            println!("::error::{error}");
            std::process::exit(2);
        }
    };

    for outcome in &report.outcomes {
        println!("{}", outcome.markdown);
        for message in &outcome.messages {
            println!("{message}");
        }
    }
    append_step_summary(&report.outcomes);

    if report.baseline_updated {
        println!("Baseline updated. Commit coverage/baseline.json with this change.");
        std::process::exit(0);
    }

    if report.failed() {
        std::process::exit(1);
    }
    println!("Coverage ratchet held.");
}

#[cfg(test)]
#[path = "check-coverage-ratchet-tests.rs"]
mod tests;
