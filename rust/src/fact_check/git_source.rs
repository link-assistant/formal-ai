//! Git-history evidence for fact checking (issue #1179 R5).
//!
//! Answers "when, and by which commit, was claim subject `X` last true?"
//! against the repository's own history, shelling out to `git` exactly the
//! way `statement_audit::repository` already does for `git ls-files`: plain
//! `Command::new("git")` invocations with `-C <root>`, no reimplementation of
//! any Git plumbing. History is bounded (the newest 50 revisions of a path)
//! so a long-lived file cannot make one query unbounded.

use std::path::Path;
use std::process::Command;

use crate::relative_meta_logic::{RelativeEvidence, Stance, TruthValue};

use super::tier_for_context;

/// How many revisions of one path a bounded query may walk.
const HISTORY_LIMIT: usize = 50;

/// One revision from `git log`, in repository order (newest first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHistoryEntry {
    /// The full commit hash.
    pub commit: String,
    /// The author date, `--date=short` (`YYYY-MM-DD`).
    pub date: String,
    /// The commit subject line.
    pub subject: String,
}

/// A bounded Git-history query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitQuery {
    /// "When was `path` last in a state containing `needle`?"
    ValueLastTrue { path: String, needle: String },
    /// "Which commit last touched `path`?"
    PathLastTouched { path: String },
}

/// The newest `HISTORY_LIMIT` revisions touching `path`, newest first.
///
/// Empty when `root` is not a Git repository or `path` has no history —
/// absence of history is the absence of evidence, never evidence itself.
#[must_use]
pub fn git_log_for_path(root: &Path, path: &str) -> Vec<GitHistoryEntry> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("log")
        .arg("-n")
        .arg(HISTORY_LIMIT.to_string())
        .arg("--follow")
        .arg("--format=%H%x1f%ad%x1f%s")
        .arg("--date=short")
        .arg("--")
        .arg(path)
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\u{1f}');
            let commit = fields.next()?.to_owned();
            let date = fields.next()?.to_owned();
            let subject = fields.next().unwrap_or_default().to_owned();
            (!commit.is_empty()).then_some(GitHistoryEntry {
                commit,
                date,
                subject,
            })
        })
        .collect()
}

/// The revision in which `path` last contained `needle`, newest first.
///
/// Walks the bounded log and shows each historical revision of the file;
/// binary-sized or unreadable revisions are skipped rather than guessed at.
#[must_use]
pub fn last_revision_containing(root: &Path, path: &str, needle: &str) -> Option<GitHistoryEntry> {
    git_log_for_path(root, path)
        .into_iter()
        .find(|entry| revision_contains(root, &entry.commit, path, needle))
}

/// The commit that last touched `path:path` at `line` (1-based), from
/// `git blame --porcelain -L`.
#[must_use]
pub fn blame_commit_for_line(root: &Path, path: &str, line: usize) -> Option<String> {
    let range = format!("{line},{line}");
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["blame", "--porcelain", "-L"])
        .arg(&range)
        .arg("--")
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .and_then(|header| header.split_whitespace().next())
        .map(str::to_owned)
}

/// Answer one bounded Git-history query against the repository at `root`.
///
/// A newest revision still containing the claimed value supports the claim
/// with that commit as the label; a path with history but no revision ever
/// containing the value contradicts it (the claim was never true in the
/// bounded window); no history at all yields no evidence.
#[must_use]
pub fn git_history_evidence_for(query: &GitQuery, root: &Path) -> Vec<RelativeEvidence> {
    let tier = tier_for_context("git");
    match query {
        GitQuery::ValueLastTrue { path, needle } => {
            let history = git_log_for_path(root, path);
            if history.is_empty() {
                return Vec::new();
            }
            match last_revision_containing(root, path, needle) {
                Some(entry) => vec![RelativeEvidence::new(
                    format!("git:log:{path}@{}", entry.commit),
                    tier,
                    Stance::Supports,
                    TruthValue::new(0.8),
                )],
                None => vec![RelativeEvidence::new(
                    crate::seed::report_text(
                        "fact_check_git_value_never_present",
                        &[("path", path), ("revisions", &history.len().to_string())],
                    ),
                    tier,
                    Stance::Contradicts,
                    TruthValue::new(0.8),
                )],
            }
        }
        GitQuery::PathLastTouched { path } => {
            let entry = git_log_for_path(root, path).into_iter().next();
            entry
                .map(|entry| {
                    RelativeEvidence::new(
                        crate::seed::report_text(
                            "fact_check_git_last_touched",
                            &[
                                ("path", path),
                                ("commit", &entry.commit),
                                ("date", &entry.date),
                            ],
                        ),
                        tier,
                        Stance::Supports,
                        TruthValue::new(0.7),
                    )
                })
                .into_iter()
                .collect()
        }
    }
}

/// Whether the historical `revision` of `path` contains `needle`, read
/// through `git show` with a bounded read (revisions over 2 MiB are treated
/// as unreadable rather than streamed).
fn revision_contains(root: &Path, revision: &str, path: &str, needle: &str) -> bool {
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("show")
        .arg(format!("{revision}:{path}"))
        .output()
    else {
        return false;
    };
    if !output.status.success() || output.stdout.len() > 2 * 1024 * 1024 {
        return false;
    }
    let content = String::from_utf8_lossy(&output.stdout);
    content.contains(needle)
}
