//! Deterministically measure release work backed by committed Formal AI sessions.
//!
//! Run with `rust-script scripts/self-hosting-metric.rs --since <tag>`. A commit
//! is attributed only when it records both of these Git trailers:
//!
//! - `Formal-AI-Session: <session-id>`
//! - `Formal-AI-Evidence: <repo-relative-path>`
//!
//! The evidence must exist in that commit and contain both `formal-ai` and the
//! recorded session id. Changed lines are additions plus deletions reported by
//! `git show --numstat`; merge commits and binary files do not contribute.

#![allow(dead_code)]

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const SESSION_TRAILER: &str = "Formal-AI-Session";
const EVIDENCE_TRAILER: &str = "Formal-AI-Evidence";
const DEFAULT_LEDGER: &str = "data/meta/self-hosting-ledger.lino";
const DEFAULT_TRAILING_WINDOW: usize = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Measurement {
    pub self_authored_lines: u64,
    pub changed_lines: u64,
    pub self_authored_commits: u64,
    pub commits: u64,
    pub percentage_basis_points: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseRow {
    pub tag: String,
    pub since: String,
    pub until: String,
    pub self_authored_lines: u64,
    pub changed_lines: u64,
    pub self_authored_commits: u64,
    pub commits: u64,
    pub percentage_basis_points: u64,
    pub trailing_window: usize,
    pub trailing_percentage_basis_points: u64,
}

pub fn format_percentage(basis_points: u64) -> String {
    format!("{}.{:02}%", basis_points / 100, basis_points % 100)
}

pub fn measure(repo: &Path, since: &str, until: &str) -> Result<Measurement, String> {
    let range = format!("{since}..{until}");
    let output = git(repo, &["rev-list", "--reverse", "--no-merges", &range])?;
    let commits = output
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let mut changed_lines = 0_u64;
    let mut self_authored_lines = 0_u64;
    let mut self_authored_commits = 0_u64;

    for commit in &commits {
        let lines = changed_lines_for_commit(repo, commit)?;
        changed_lines = changed_lines
            .checked_add(lines)
            .ok_or_else(|| "changed-line total overflowed u64".to_owned())?;
        if commit_has_formal_ai_evidence(repo, commit)? {
            self_authored_lines = self_authored_lines
                .checked_add(lines)
                .ok_or_else(|| "self-authored line total overflowed u64".to_owned())?;
            self_authored_commits += 1;
        }
    }

    Ok(Measurement {
        self_authored_lines,
        changed_lines,
        self_authored_commits,
        commits: commits.len() as u64,
        percentage_basis_points: percentage_basis_points(self_authored_lines, changed_lines),
    })
}

fn changed_lines_for_commit(repo: &Path, commit: &str) -> Result<u64, String> {
    let output = git(
        repo,
        &["show", "--format=", "--numstat", "--no-renames", commit],
    )?;
    output.lines().try_fold(0_u64, |total, line| {
        let mut fields = line.split('\t');
        let additions = fields.next().unwrap_or_default();
        let deletions = fields.next().unwrap_or_default();
        if additions == "-" || deletions == "-" {
            return Ok(total);
        }
        let additions = additions
            .parse::<u64>()
            .map_err(|error| format!("invalid numstat additions in {commit}: {error}"))?;
        let deletions = deletions
            .parse::<u64>()
            .map_err(|error| format!("invalid numstat deletions in {commit}: {error}"))?;
        total
            .checked_add(additions)
            .and_then(|sum| sum.checked_add(deletions))
            .ok_or_else(|| format!("changed-line count overflowed in {commit}"))
    })
}

fn commit_has_formal_ai_evidence(repo: &Path, commit: &str) -> Result<bool, String> {
    let sessions = trailer_values(repo, commit, SESSION_TRAILER)?;
    let evidence_paths = trailer_values(repo, commit, EVIDENCE_TRAILER)?;
    if sessions.is_empty() && evidence_paths.is_empty() {
        return Ok(false);
    }
    if sessions.is_empty() || evidence_paths.is_empty() {
        return Err(format!(
            "commit {commit} must record both {SESSION_TRAILER} and {EVIDENCE_TRAILER}"
        ));
    }

    let mut evidence = Vec::with_capacity(evidence_paths.len());
    for path in evidence_paths {
        validate_evidence_path(&path)?;
        let object = format!("{commit}:{path}");
        let content = git(repo, &["show", &object])?;
        if !content.to_ascii_lowercase().contains("formal-ai") {
            return Err(format!(
                "evidence {path} in commit {commit} does not identify formal-ai"
            ));
        }
        evidence.push((path, content));
    }

    for session in sessions {
        if !evidence
            .iter()
            .any(|(_, content)| content.contains(&session))
        {
            return Err(format!(
                "no committed {EVIDENCE_TRAILER} in {commit} records session {session}"
            ));
        }
    }
    Ok(true)
}

fn trailer_values(repo: &Path, commit: &str, key: &str) -> Result<Vec<String>, String> {
    let format = format!("%(trailers:key={key},valueonly,separator=%x1f)");
    let format_arg = format!("--format={format}");
    let output = git(repo, &["show", "-s", &format_arg, commit])?;
    Ok(output
        .trim()
        .split('\x1f')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect())
}

fn validate_evidence_path(path: &str) -> Result<(), String> {
    let candidate = Path::new(path);
    if path.contains(':')
        || path.contains('\n')
        || candidate.is_absolute()
        || candidate
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "{EVIDENCE_TRAILER} must be a normal repo-relative path: {path}"
        ));
    }
    Ok(())
}

fn percentage_basis_points(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        return 0;
    }
    let rounded =
        (u128::from(numerator) * 10_000 + u128::from(denominator) / 2) / u128::from(denominator);
    u64::try_from(rounded).expect("percentage basis points fit in u64")
}

pub fn record_release(
    repo: &Path,
    ledger: &Path,
    tag: &str,
    since: &str,
    until: &str,
    trailing_window: usize,
) -> Result<ReleaseRow, String> {
    if trailing_window == 0 {
        return Err("trailing window must be greater than zero".to_owned());
    }
    let rows = read_release_rows(ledger)?;
    let measurement = measure(repo, since, until)?;
    let until_commitish = format!("{until}^{{commit}}");
    let canonical_until = git(repo, &["rev-parse", &until_commitish])?
        .trim()
        .to_owned();
    let mut window_rows = rows
        .iter()
        .rev()
        .take(trailing_window.saturating_sub(1))
        .cloned()
        .collect::<Vec<_>>();
    window_rows.reverse();
    let mut row = ReleaseRow {
        tag: tag.to_owned(),
        since: since.to_owned(),
        until: canonical_until,
        self_authored_lines: measurement.self_authored_lines,
        changed_lines: measurement.changed_lines,
        self_authored_commits: measurement.self_authored_commits,
        commits: measurement.commits,
        percentage_basis_points: measurement.percentage_basis_points,
        trailing_window,
        trailing_percentage_basis_points: 0,
    };
    window_rows.push(row.clone());
    row.trailing_percentage_basis_points = weighted_percentage(&window_rows);

    if let Some(existing) = rows.iter().find(|existing| existing.tag == tag) {
        if existing == &row {
            return Ok(existing.clone());
        }
        return Err(format!(
            "release {tag} already has a different self-hosting ledger row"
        ));
    }
    if let Some(previous) = rows.last() {
        if row.trailing_percentage_basis_points < previous.trailing_percentage_basis_points {
            return Err(format!(
                "self-hosting ratchet would fall from {} to {} for {tag}",
                format_percentage(previous.trailing_percentage_basis_points),
                format_percentage(row.trailing_percentage_basis_points)
            ));
        }
    }
    append_release_row(ledger, &row)?;
    Ok(row)
}

fn weighted_percentage(rows: &[ReleaseRow]) -> u64 {
    let self_authored = rows.iter().map(|row| row.self_authored_lines).sum::<u64>();
    let changed = rows.iter().map(|row| row.changed_lines).sum::<u64>();
    percentage_basis_points(self_authored, changed)
}

pub fn read_release_rows(ledger: &Path) -> Result<Vec<ReleaseRow>, String> {
    let text = fs::read_to_string(ledger)
        .map_err(|error| format!("could not read {}: {error}", ledger.display()))?;
    let mut rows = Vec::new();
    let mut fields = Vec::new();
    for line in text.lines() {
        if line.trim() == "release" {
            if !fields.is_empty() {
                rows.push(parse_release_row(&fields)?);
                fields.clear();
            }
        } else if line.starts_with("    ") {
            fields.push(line.trim().to_owned());
        }
    }
    if !fields.is_empty() {
        rows.push(parse_release_row(&fields)?);
    }
    Ok(rows)
}

fn parse_release_row(fields: &[String]) -> Result<ReleaseRow, String> {
    let value = |key: &str| -> Result<String, String> {
        let prefix = format!("{key} \"");
        fields
            .iter()
            .find_map(|field| {
                field
                    .strip_prefix(&prefix)
                    .and_then(|value| value.strip_suffix('"'))
                    .map(str::to_owned)
            })
            .ok_or_else(|| format!("release row is missing {key}"))
    };
    let number = |key: &str| -> Result<u64, String> {
        value(key)?
            .parse::<u64>()
            .map_err(|error| format!("release row has invalid {key}: {error}"))
    };
    Ok(ReleaseRow {
        tag: value("tag")?,
        since: value("since")?,
        until: value("until")?,
        self_authored_lines: number("self_authored_lines")?,
        changed_lines: number("changed_lines")?,
        self_authored_commits: number("self_authored_commits")?,
        commits: number("commits")?,
        percentage_basis_points: number("percentage_basis_points")?,
        trailing_window: usize::try_from(number("trailing_window")?)
            .map_err(|_| "trailing_window does not fit usize".to_owned())?,
        trailing_percentage_basis_points: number("trailing_percentage_basis_points")?,
    })
}

fn append_release_row(ledger: &Path, row: &ReleaseRow) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(ledger)
        .map_err(|error| format!("could not append {}: {error}", ledger.display()))?;
    writeln!(
        file,
        "  release\n    tag \"{}\"\n    since \"{}\"\n    until \"{}\"\n    \
         self_authored_lines \"{}\"\n    changed_lines \"{}\"\n    self_authored_commits \
         \"{}\"\n    commits \"{}\"\n    percentage_basis_points \"{}\"\n    trailing_window \
         \"{}\"\n    trailing_percentage_basis_points \"{}\"",
        row.tag,
        row.since,
        row.until,
        row.self_authored_lines,
        row.changed_lines,
        row.self_authored_commits,
        row.commits,
        row.percentage_basis_points,
        row.trailing_window,
        row.trailing_percentage_basis_points,
    )
    .map_err(|error| format!("could not append {}: {error}", ledger.display()))
}

pub fn release_note_for_tag(ledger: &Path, tag: &str) -> Result<String, String> {
    let row = read_release_rows(ledger)?
        .into_iter()
        .find(|row| row.tag == tag)
        .ok_or_else(|| format!("self-hosting ledger has no release row for {tag}"))?;
    Ok(format!(
        "## Self-hosting\n\nFormal AI authored **{}** of this release ({} of {} changed lines). The \
         {}-release trailing share is **{}**.",
        format_percentage(row.percentage_basis_points),
        row.self_authored_lines,
        row.changed_lines,
        row.trailing_window,
        format_percentage(row.trailing_percentage_basis_points),
    ))
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("could not run git {args:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("git output was not UTF-8: {error}"))
}

#[derive(Debug)]
struct Options {
    repo: PathBuf,
    since: String,
    until: String,
    ledger: PathBuf,
    record_release: Option<String>,
    trailing_window: usize,
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut repo = PathBuf::from(".");
    let mut since = None;
    let mut until = "HEAD".to_owned();
    let mut ledger = PathBuf::from(DEFAULT_LEDGER);
    let mut record_release = None;
    let mut trailing_window = DEFAULT_TRAILING_WINDOW;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let value = |args: &mut dyn Iterator<Item = String>| {
            args.next().ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--repo" => repo = PathBuf::from(value(&mut args)?),
            "--since" => since = Some(value(&mut args)?),
            "--until" => until = value(&mut args)?,
            "--ledger" => ledger = PathBuf::from(value(&mut args)?),
            "--record-release" => record_release = Some(value(&mut args)?),
            "--trailing-window" => {
                trailing_window = value(&mut args)?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --trailing-window: {error}"))?;
            }
            "--help" | "-h" => {
                return Err(
                    "usage: self-hosting-metric.rs --since <tag> [--until <rev>] \
                     [--record-release <tag>] [--ledger <path>] [--repo <path>]"
                        .to_owned(),
                );
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    let since = since.ok_or_else(|| "--since <tag> is required".to_owned())?;
    let ledger = if ledger.is_absolute() {
        ledger
    } else {
        repo.join(ledger)
    };
    Ok(Options {
        repo,
        since,
        until,
        ledger,
        record_release,
        trailing_window,
    })
}

fn run() -> Result<(), String> {
    let options = parse_options(env::args().skip(1))?;
    let measurement = if let Some(tag) = options.record_release {
        let row = record_release(
            &options.repo,
            &options.ledger,
            &tag,
            &options.since,
            &options.until,
            options.trailing_window,
        )?;
        Measurement {
            self_authored_lines: row.self_authored_lines,
            changed_lines: row.changed_lines,
            self_authored_commits: row.self_authored_commits,
            commits: row.commits,
            percentage_basis_points: row.percentage_basis_points,
        }
    } else {
        measure(&options.repo, &options.since, &options.until)?
    };
    println!(
        "{} ({}/{} changed lines; {}/{} commits)",
        format_percentage(measurement.percentage_basis_points),
        measurement.self_authored_lines,
        measurement.changed_lines,
        measurement.self_authored_commits,
        measurement.commits,
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("self-hosting metric error: {error}");
        std::process::exit(1);
    }
}
