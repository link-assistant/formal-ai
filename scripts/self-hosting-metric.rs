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
//! `git show --numstat`; merge commits, binary files and captured artifacts
//! (see [`is_non_authored_path`]) do not contribute.

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

/// Measurement-definition epoch of the rows this build writes.
///
/// Issue #812: version 1 counted every changed line, including the captured CI
/// logs that `dev/log/issues/<n>/pulls/<n>/` bundles hold. In the `v0.300.0..HEAD`
/// range that was **94.58% of the whole denominator** (601 765 of 636 240 lines,
/// measured by `experiments/self_hosting_ratchet_replay/replay.py`), so the
/// published share described log volume rather than authored work — in both
/// directions. Version 2 excludes captured artifacts.
///
/// Rows measured under different definitions are not comparable, so the ratchet
/// and the trailing window only ever look at rows of the *same* version. A
/// definition change therefore starts a new epoch instead of silently averaging
/// two different quantities.
const METRIC_VERSION: u64 = 2;

/// File extensions whose content is captured, not authored: CI run logs, agent
/// transcripts, saved diffs and process output. Committing them is deliberate
/// (they are the evidence bundles this repository requires), but they are not
/// release work and must not move the metric.
const CAPTURED_ARTIFACT_EXTENSIONS: &[&str] =
    &["log", "jsonl", "diff", "patch", "stderr", "stdout"];

/// Dependency lockfiles: written by a package manager, never hand-authored.
const LOCKFILE_NAMES: &[&str] = &[
    "Cargo.lock",
    "bun.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "poetry.lock",
    "uv.lock",
];

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
    /// Measurement epoch this row was produced under; see [`METRIC_VERSION`].
    /// Rows written before issue #812 carry no field and read back as `1`.
    pub metric_version: u64,
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

/// How a malformed evidence record is treated.
///
/// Issue #810: the `Formal-AI-Evidence` gate (`evidence-check` in
/// `.github/workflows/release.yml`) was introduced in 39fdef91, *after* commit
/// 10e65ae2 had already merged with an evidence document that never spells out
/// its own `Formal-AI-Session` id. Because `record_release` measures
/// `<last tag>..HEAD`, that historical commit stayed inside the release range
/// forever, `Auto Release` aborted on it every single run (29737218421), and no
/// new tag could ever be cut to move the range past it — a permanent deadlock
/// that history cannot be edited out of.
///
/// So the two callers get different policies:
///
/// - `Strict` — the pull-request gate. New commits must be well formed, and a
///   malformed record is a hard error *before* it can reach the default branch.
/// - `Lenient` — release recording. A malformed record is reported on stderr and
///   the commit simply does not count as self-authored. The metric can only
///   under-report, never over-report, so the ratchet stays honest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidencePolicy {
    Strict,
    Lenient,
}

pub fn measure(repo: &Path, since: &str, until: &str) -> Result<Measurement, String> {
    measure_with_policy(repo, since, until, EvidencePolicy::Strict)
}

pub fn measure_with_policy(
    repo: &Path,
    since: &str,
    until: &str,
    policy: EvidencePolicy,
) -> Result<Measurement, String> {
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
        let attributed = match commit_has_formal_ai_evidence(repo, commit) {
            Ok(attributed) => attributed,
            Err(error) => match policy {
                EvidencePolicy::Strict => return Err(error),
                EvidencePolicy::Lenient => {
                    eprintln!("warning: not attributing {commit}: {error}");
                    false
                }
            },
        };
        if attributed {
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

/// Whether a path holds captured output rather than authored work.
///
/// Applied symmetrically to the numerator and the denominator, so a commit that
/// only files evidence contributes nothing either way and the share it reports
/// is unchanged by how much log volume happened to be attached to it.
pub fn is_non_authored_path(path: &str) -> bool {
    // `git show --numstat` C-quotes any path with non-printable or non-ASCII
    // bytes, so the raw field can arrive wrapped in double quotes.
    let path = path.trim().trim_matches('"');
    let name = path.rsplit('/').next().unwrap_or(path);
    if LOCKFILE_NAMES.contains(&name) {
        return true;
    }
    name.rsplit_once('.').is_some_and(|(_, extension)| {
        CAPTURED_ARTIFACT_EXTENSIONS
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
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
        let path = fields.next().unwrap_or_default();
        if additions == "-" || deletions == "-" {
            return Ok(total);
        }
        if is_non_authored_path(path) {
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

/// Collects `key: value` trailers from a commit message.
///
/// This deliberately scans the whole commit body instead of using git's
/// `%(trailers:key=...)` placeholder. Git only recognises the *last* paragraph
/// of a message as the trailer block, so a blank line between two trailers
/// makes every trailer above it invisible. Release commit
/// 59650f2b45ebebf29ed67d33c9a19c1e82e7c003 separated its `Formal-AI-Session`
/// and `Formal-AI-Evidence` trailers with a blank line, git reported only the
/// evidence trailer, and the resulting "must record both" error failed the
/// whole Auto Release job (issue #796).
fn trailer_values(repo: &Path, commit: &str, key: &str) -> Result<Vec<String>, String> {
    let body = git(repo, &["show", "-s", "--format=%B", commit])?;
    let prefix = format!("{key}:");
    Ok(body
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Trailer keys are case-insensitive per `git interpret-trailers`.
            line.get(..prefix.len())
                .filter(|candidate| candidate.eq_ignore_ascii_case(&prefix))
                .map(|_| line[prefix.len()..].trim())
        })
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

/// What a falling trailing share does to the caller.
///
/// Issue #812: `Auto Release` on `main` died with "self-hosting ratchet would
/// fall from 32.68% to 18.24% for v0.301.0" (run 29751001867, log line 23918).
/// That check runs *after* every contributing commit is already immutable
/// history on the default branch, so nothing anybody can do makes it pass, and
/// the only way to move the measured range forward — cutting a tag — is exactly
/// what it blocks. It is the same shape of permanent outage as issue #810's
/// evidence deadlock, and the third release outage in a row (#796, #810, #812)
/// caused by enforcing a policy where the policy is not actionable.
///
/// So enforcement moves to where a contributor can still act on it:
///
/// - `Enforce` — the pull-request gate (`Self-Hosting Evidence Check` in
///   `.github/workflows/release.yml`, via `--check-ratchet`). A fall is a hard
///   error while the commits that cause it can still be amended.
/// - `Report` — release recording. The row is appended exactly as measured, the
///   fall is announced on stderr and as a GitHub `::warning` annotation, and the
///   release ships. The ledger records the regression honestly instead of
///   hiding it behind a job that never completes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RatchetPolicy {
    Enforce,
    Report,
}

pub fn record_release(
    repo: &Path,
    ledger: &Path,
    tag: &str,
    since: &str,
    until: &str,
    trailing_window: usize,
) -> Result<ReleaseRow, String> {
    record_release_with_policy(
        repo,
        ledger,
        tag,
        since,
        until,
        trailing_window,
        RatchetPolicy::Enforce,
    )
}

pub fn record_release_with_policy(
    repo: &Path,
    ledger: &Path,
    tag: &str,
    since: &str,
    until: &str,
    trailing_window: usize,
    ratchet: RatchetPolicy,
) -> Result<ReleaseRow, String> {
    if trailing_window == 0 {
        return Err("trailing window must be greater than zero".to_owned());
    }
    let all_rows = read_release_rows(ledger)?;
    // Only rows measured under the current definition are comparable; see
    // `METRIC_VERSION`. Mixing epochs would average two different quantities.
    let rows = all_rows
        .iter()
        .filter(|row| row.metric_version == METRIC_VERSION)
        .cloned()
        .collect::<Vec<_>>();
    // Lenient on purpose: see `EvidencePolicy`. The pull-request gate is what
    // keeps new commits honest; a release must never be blocked by a commit
    // that is already immutable history.
    let measurement = measure_with_policy(repo, since, until, EvidencePolicy::Lenient)?;
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
        metric_version: METRIC_VERSION,
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

    // Idempotency is checked against the whole ledger, not just this epoch: a
    // tag must never be recorded twice, whatever definition produced it.
    if let Some(existing) = all_rows.iter().find(|existing| existing.tag == tag) {
        if existing == &row {
            return Ok(existing.clone());
        }
        return Err(format!(
            "release {tag} already has a different self-hosting ledger row"
        ));
    }
    if let Some(regression) = ratchet_regression(rows.last(), &row) {
        match ratchet {
            RatchetPolicy::Enforce => return Err(regression),
            RatchetPolicy::Report => {
                eprintln!("warning: {regression}");
                println!("::warning title=Self-hosting ratchet fell::{regression}");
            }
        }
    }
    append_release_row(ledger, &row)?;
    Ok(row)
}

/// The ratchet message when `row` regresses against `previous`, else `None`.
fn ratchet_regression(previous: Option<&ReleaseRow>, row: &ReleaseRow) -> Option<String> {
    let previous = previous?;
    if row.trailing_percentage_basis_points >= previous.trailing_percentage_basis_points {
        return None;
    }
    Some(format!(
        "self-hosting ratchet would fall from {} to {} for {}",
        format_percentage(previous.trailing_percentage_basis_points),
        format_percentage(row.trailing_percentage_basis_points),
        row.tag
    ))
}

/// The trailing share a release cut at `until` would record.
///
/// Same arithmetic `record_release_with_policy` performs, without writing
/// anything, so a pull request can be told what its merge would do to the
/// ratchet while its commits can still be amended.
pub fn project_trailing_basis_points(
    repo: &Path,
    ledger: &Path,
    since: &str,
    until: &str,
    trailing_window: usize,
) -> Result<u64, String> {
    if trailing_window == 0 {
        return Err("trailing window must be greater than zero".to_owned());
    }
    let measurement = measure_with_policy(repo, since, until, EvidencePolicy::Lenient)?;
    let rows = read_release_rows(ledger)?;
    let mut window_rows = rows
        .iter()
        .filter(|row| row.metric_version == METRIC_VERSION)
        .rev()
        .take(trailing_window.saturating_sub(1))
        .cloned()
        .collect::<Vec<_>>();
    window_rows.reverse();
    window_rows.push(ReleaseRow {
        metric_version: METRIC_VERSION,
        tag: String::new(),
        since: since.to_owned(),
        until: until.to_owned(),
        self_authored_lines: measurement.self_authored_lines,
        changed_lines: measurement.changed_lines,
        self_authored_commits: measurement.self_authored_commits,
        commits: measurement.commits,
        percentage_basis_points: measurement.percentage_basis_points,
        trailing_window,
        trailing_percentage_basis_points: 0,
    });
    Ok(weighted_percentage(&window_rows))
}

/// Whether merging `head` would lower the share the next release records.
///
/// The comparison is *differential*: the projection including `head` against the
/// projection for `base` alone. An absolute threshold would repeat the issue
/// #812 outage in a new place — a regression already merged into `main` would
/// then fail every subsequent pull request, none of which could fix it. A
/// pull request is only answerable for its own delta, and a no-op pull request
/// is always neutral.
///
/// `since` is the last tag in the ledger, so the projected range is the one the
/// release will actually measure. Returns the message when the share falls.
pub fn ratchet_check(
    repo: &Path,
    ledger: &Path,
    base: &str,
    head: &str,
    trailing_window: usize,
) -> Result<Option<String>, String> {
    let Some(since) = read_release_rows(ledger)?.last().map(|row| row.tag.clone()) else {
        // No release has ever been recorded: there is no ratchet to fall from.
        return Ok(None);
    };
    // A checkout without tags (shallow clone, fork without `fetch-tags`) cannot
    // measure the release range. Skip loudly rather than fail: this gate exists
    // to catch regressions, and turning a missing tag into a red check would be
    // exactly the kind of false positive issue #812 is about.
    if git(
        repo,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{since}^{{commit}}"),
        ],
    )
    .is_err()
    {
        eprintln!("warning: skipping ratchet check: {since} is not present in this checkout");
        return Ok(None);
    }
    let baseline = project_trailing_basis_points(repo, ledger, &since, base, trailing_window)?;
    let candidate = project_trailing_basis_points(repo, ledger, &since, head, trailing_window)?;
    if candidate >= baseline {
        return Ok(None);
    }
    Ok(Some(format!(
        "merging this branch would lower the projected self-hosting share of the next release from \
         {} to {}; record Formal-AI-Session and Formal-AI-Evidence trailers on the commits that \
         Formal AI authored, or split unattributed work out of this branch",
        format_percentage(baseline),
        format_percentage(candidate),
    )))
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
    let optional_number = |key: &str| -> Result<Option<u64>, String> {
        let Ok(raw) = value(key) else {
            return Ok(None);
        };
        raw.parse::<u64>()
            .map(Some)
            .map_err(|error| format!("release row has invalid {key}: {error}"))
    };
    Ok(ReleaseRow {
        // Rows recorded before issue #812 carry no epoch marker; they are, by
        // definition, epoch 1.
        metric_version: optional_number("metric_version")?.unwrap_or(1),
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
    let fields = [
        ("metric_version", row.metric_version.to_string()),
        ("tag", row.tag.clone()),
        ("since", row.since.clone()),
        ("until", row.until.clone()),
        ("self_authored_lines", row.self_authored_lines.to_string()),
        ("changed_lines", row.changed_lines.to_string()),
        (
            "self_authored_commits",
            row.self_authored_commits.to_string(),
        ),
        ("commits", row.commits.to_string()),
        (
            "percentage_basis_points",
            row.percentage_basis_points.to_string(),
        ),
        ("trailing_window", row.trailing_window.to_string()),
        (
            "trailing_percentage_basis_points",
            row.trailing_percentage_basis_points.to_string(),
        ),
    ];
    let mut record = String::from("  release\n");
    for (key, value) in &fields {
        record.push_str("    ");
        record.push_str(key);
        record.push_str(" \"");
        record.push_str(value);
        record.push_str("\"\n");
    }
    write!(file, "{record}")
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
    since: Option<String>,
    until: String,
    ledger: PathBuf,
    record_release: Option<String>,
    /// Base revision to compare against; see [`ratchet_check`].
    check_ratchet: Option<String>,
    trailing_window: usize,
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut repo = PathBuf::from(".");
    let mut since = None;
    let mut until = "HEAD".to_owned();
    let mut ledger = PathBuf::from(DEFAULT_LEDGER);
    let mut record_release = None;
    let mut check_ratchet = None;
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
            "--check-ratchet" => check_ratchet = Some(value(&mut args)?),
            "--trailing-window" => {
                trailing_window = value(&mut args)?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --trailing-window: {error}"))?;
            }
            "--help" | "-h" => {
                return Err(
                    "usage: self-hosting-metric.rs --since <tag> [--until <rev>] \
                     [--record-release <tag>] [--check-ratchet <base-rev>] [--ledger <path>] \
                     [--repo <path>]"
                        .to_owned(),
                );
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    if since.is_none() && check_ratchet.is_none() {
        return Err("--since <tag> is required".to_owned());
    }
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
        check_ratchet,
        trailing_window,
    })
}

fn run() -> Result<(), String> {
    let options = parse_options(env::args().skip(1))?;
    if let Some(base) = &options.check_ratchet {
        if let Some(regression) = ratchet_check(
            &options.repo,
            &options.ledger,
            base,
            &options.until,
            options.trailing_window,
        )? {
            return Err(regression);
        }
        println!("self-hosting ratchet holds against {base}");
        return Ok(());
    }
    let since = options
        .since
        .ok_or_else(|| "--since <tag> is required".to_owned())?;
    let measurement = if let Some(tag) = options.record_release {
        let row = record_release_with_policy(
            &options.repo,
            &options.ledger,
            &tag,
            &since,
            &options.until,
            options.trailing_window,
            // The pull-request gate (`--check-ratchet`) is where a fall is still
            // actionable; on `main` the commits are immutable. See `RatchetPolicy`.
            RatchetPolicy::Report,
        )?;
        Measurement {
            self_authored_lines: row.self_authored_lines,
            changed_lines: row.changed_lines,
            self_authored_commits: row.self_authored_commits,
            commits: row.commits,
            percentage_basis_points: row.percentage_basis_points,
        }
    } else {
        measure(&options.repo, &since, &options.until)?
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
