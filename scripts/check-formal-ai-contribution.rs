#!/usr/bin/env rust-script
//! Each pull request carries Formal AI-authored work, and the requirement
//! relaxes once the pull request has been open for more than a day.
//!
//! The architect stated both halves in one breath
//! (`docs/architect-notes/2026-09-11-do-not-obstruct-the-vision.md`):
//!
//! > we should force each pull request to use Formal AI to code part of it (as
//! > big as it can be, but as small as it actually can, if that will take more
//! > than 1 actual time of coding). So if pull request exists for for more than
//! > a day we can relax our requirements, and still be able to produce the
//! > release.
//!
//! Both halves are the same instruction. Forcing the work without the relief
//! valve is what obstructs progression to the vision, and the practical aim is
//! stated in the same note: *"it is critical to be able to produce formal-ai
//! releases for testing in Hive Mind on real GitHub issues."*
//!
//! This is **not** the thing issue #1066 forbids. That issue removed a budget
//! that let an unreleasable cycle report success while a downstream-critical fix
//! sat behind a green checkmark. Nothing here gates a release: the release path
//! cuts on CI correctness alone (issue #1085 D3.5), and a relaxed result is
//! reported as relaxed, never as satisfied. What relaxes is the per-pull-request
//! authoring requirement, which is the requirement the architect attached the
//! day to.
//!
//! Usage:
//!   rust-script scripts/check-formal-ai-contribution.rs --since <rev> --until <rev> \
//!     [--opened <RFC3339>] [--now <RFC3339>] [--repo <path>]
//!   rust-script --test scripts/check-formal-ai-contribution.rs
//!
//! ```cargo
//! [package]
//! edition = "2021"
//! ```

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(not(test))]
use std::process::exit;

/// The window the architect named. After this, the requirement relaxes.
const RELAXATION_HOURS: i64 = 24;

/// The trailer that marks a commit as authored by the formal-ai model.
const MODEL_TRAILER: &str = "Formal-AI-Model:";

/// What this run decided.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// The pull request carries Formal AI-authored work.
    Satisfied { commits: usize },
    /// It carries none, but has been open longer than the window.
    Relaxed { hours: i64 },
    /// It carries none and is younger than the window.
    Unsatisfied { hours: i64 },
}

impl Verdict {
    /// Only an outright unsatisfied verdict fails. A relaxed one reports.
    fn is_failure(&self) -> bool {
        matches!(self, Verdict::Unsatisfied { .. })
    }

    fn describe(&self) -> String {
        match self {
            Verdict::Satisfied { commits } => format!(
                "satisfied: {commits} commit(s) in this range carry `{MODEL_TRAILER} formal-ai`"
            ),
            Verdict::Relaxed { hours } => format!(
                "relaxed: no Formal AI-authored commit, but this pull request has been open \
                 {hours}h (> {RELAXATION_HOURS}h). The architect's note relaxes the requirement \
                 rather than blocking the work; nothing here blocks a release."
            ),
            Verdict::Unsatisfied { hours } => format!(
                "unsatisfied: no commit in this range carries `{MODEL_TRAILER} formal-ai`, and \
                 this pull request is {hours}h old (< {RELAXATION_HOURS}h). Let Formal AI author \
                 part of the change -- as big as it can be, as small as it actually can -- with \
                 `scripts/author-change-with-formal-ai.sh`. After {RELAXATION_HOURS}h this \
                 relaxes on its own."
            ),
        }
    }
}

/// Decide, given what was measured. Pure, so the window is testable without a
/// repository or a clock.
fn decide(formal_ai_commits: usize, age_hours: i64) -> Verdict {
    if formal_ai_commits > 0 {
        return Verdict::Satisfied {
            commits: formal_ai_commits,
        };
    }
    if age_hours > RELAXATION_HOURS {
        return Verdict::Relaxed { hours: age_hours };
    }
    Verdict::Unsatisfied { hours: age_hours }
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
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("git output was not UTF-8: {error}"))
}

/// Count commits in `since..until` whose `Formal-AI-Model` trailer names
/// formal-ai. A trailer naming a hosted model is not self-authored (R1085-4).
fn count_formal_ai_commits(repo: &Path, since: &str, until: &str) -> Result<usize, String> {
    let log = git(repo, &["log", "--format=%B%x00", &format!("{since}..{until}")])?;
    Ok(log
        .split('\0')
        .filter(|message| {
            message.lines().any(|line| {
                let line = line.trim();
                line.strip_prefix(MODEL_TRAILER)
                    .is_some_and(|value| value.trim().starts_with("formal-ai"))
            })
        })
        .count())
}

/// Hours between two RFC3339 timestamps, without a date dependency: both are
/// converted to a day count plus seconds and subtracted.
fn hours_between(opened: &str, now: &str) -> Result<i64, String> {
    Ok((epoch_seconds(now)? - epoch_seconds(opened)?) / 3600)
}

/// Seconds since the epoch for an RFC3339 timestamp such as
/// `2026-09-12T06:22:31Z`. Only the UTC form GitHub emits is accepted.
fn epoch_seconds(stamp: &str) -> Result<i64, String> {
    let stamp = stamp.trim();
    let (date, rest) = stamp
        .split_once('T')
        .ok_or_else(|| format!("`{stamp}` is not an RFC3339 timestamp"))?;
    let time = rest.trim_end_matches('Z');
    let part = |value: &str| -> Result<i64, String> {
        value
            .parse::<i64>()
            .map_err(|error| format!("`{stamp}`: {error}"))
    };
    let date: Vec<&str> = date.split('-').collect();
    let time: Vec<&str> = time.split(':').collect();
    if date.len() != 3 || time.len() < 3 {
        return Err(format!("`{stamp}` is not an RFC3339 timestamp"));
    }
    let (year, month, day) = (part(date[0])?, part(date[1])?, part(date[2])?);
    let seconds = part(time[2].split('.').next().unwrap_or("0"))?;
    // Days from civil, Howard Hinnant's algorithm: exact for the proleptic
    // Gregorian calendar and free of leap-year special cases at call sites.
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_shift = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_shift + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Ok(days * 86_400 + part(time[0])? * 3600 + part(time[1])? * 60 + seconds)
}

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut repo = PathBuf::from(".");
    let (mut since, mut until) = (String::new(), "HEAD".to_owned());
    let (mut opened, mut now) = (String::new(), String::new());

    let mut index = 0;
    while index < args.len() {
        let value = args.get(index + 1).cloned().unwrap_or_default();
        match args[index].as_str() {
            "--since" => since = value,
            "--until" => until = value,
            "--opened" => opened = value,
            "--now" => now = value,
            "--repo" => repo = PathBuf::from(value),
            other => {
                eprintln!("check-formal-ai-contribution: unknown option: {other}");
                exit(2);
            }
        }
        index += 2;
    }

    if since.is_empty() {
        eprintln!("check-formal-ai-contribution: --since is required");
        exit(2);
    }
    // No `--opened` means no pull request to age, so the requirement stands.
    let opened = if opened.is_empty() {
        env::var("PULL_REQUEST_CREATED_AT").unwrap_or_default()
    } else {
        opened
    };

    let commits = match count_formal_ai_commits(&repo, &since, &until) {
        Ok(commits) => commits,
        Err(error) => {
            eprintln!("check-formal-ai-contribution: {error}");
            exit(1);
        }
    };

    let age_hours = if opened.is_empty() {
        0
    } else {
        let now = if now.is_empty() {
            match git(&repo, &["log", "-1", "--format=%cI", "HEAD"]) {
                Ok(stamp) => stamp,
                Err(error) => {
                    eprintln!("check-formal-ai-contribution: {error}");
                    exit(1);
                }
            }
        } else {
            now
        };
        match hours_between(&opened, &now) {
            Ok(hours) => hours,
            Err(error) => {
                eprintln!("check-formal-ai-contribution: {error}");
                exit(1);
            }
        }
    };

    let verdict = decide(commits, age_hours);
    println!("Formal AI contribution for {since}..{until}");
    println!("  {}", verdict.describe());
    if verdict.is_failure() {
        exit(1);
    }
}

#[cfg(test)]
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pull request carrying Formal AI work passes, however old it is.
    #[test]
    fn formal_ai_authored_work_satisfies_the_requirement() {
        assert_eq!(decide(1, 0), Verdict::Satisfied { commits: 1 });
        assert_eq!(decide(3, 999), Verdict::Satisfied { commits: 3 });
        assert!(!decide(1, 0).is_failure());
    }

    /// Inside the window, the requirement stands: the pull request is asked to
    /// let Formal AI author part of the change.
    #[test]
    fn a_fresh_pull_request_without_formal_ai_work_fails() {
        let verdict = decide(0, 1);
        assert_eq!(verdict, Verdict::Unsatisfied { hours: 1 });
        assert!(verdict.is_failure());
        assert!(verdict.describe().contains("author-change-with-formal-ai.sh"));
    }

    /// Past the window it relaxes, exactly as the architect stated. This is the
    /// half that keeps the requirement from obstructing the vision.
    #[test]
    fn after_more_than_a_day_the_requirement_relaxes() {
        let verdict = decide(0, RELAXATION_HOURS + 1);
        assert_eq!(
            verdict,
            Verdict::Relaxed {
                hours: RELAXATION_HOURS + 1
            }
        );
        assert!(!verdict.is_failure(), "a relaxed requirement does not fail");
    }

    /// The boundary is "more than a day", so exactly a day is not yet relaxed.
    #[test]
    fn the_boundary_is_more_than_a_day_not_at_least_a_day() {
        assert!(decide(0, RELAXATION_HOURS).is_failure());
        assert!(!decide(0, RELAXATION_HOURS + 1).is_failure());
    }

    /// A relaxed verdict says it was relaxed. Issue #1066's complaint was that a
    /// deferral reported as success and so was invisible; this never does.
    #[test]
    fn a_relaxed_verdict_never_reports_itself_as_satisfied() {
        let described = decide(0, RELAXATION_HOURS + 5).describe();
        assert!(described.starts_with("relaxed:"), "{described}");
        assert!(
            !described.contains("satisfied:"),
            "a relaxed result must not read as a satisfied one: {described}"
        );
        assert!(
            described.contains("nothing here blocks a release"),
            "the report must say it blocks nothing: {described}"
        );
    }

    #[test]
    fn hours_are_counted_across_days_and_months() {
        assert_eq!(
            hours_between("2026-09-11T00:00:00Z", "2026-09-12T01:00:00Z").unwrap(),
            25
        );
        assert_eq!(
            hours_between("2026-09-12T06:00:00Z", "2026-09-12T09:30:00Z").unwrap(),
            3
        );
        // Across a month boundary, and across a leap day.
        assert_eq!(
            hours_between("2026-08-31T12:00:00Z", "2026-09-01T12:00:00Z").unwrap(),
            24
        );
        assert_eq!(
            hours_between("2024-02-28T00:00:00Z", "2024-03-01T00:00:00Z").unwrap(),
            48
        );
    }

    #[test]
    fn a_malformed_timestamp_is_an_error_not_a_silent_zero() {
        assert!(hours_between("not-a-date", "2026-09-12T00:00:00Z").is_err());
        assert!(epoch_seconds("2026-09-12").is_err());
    }
}
