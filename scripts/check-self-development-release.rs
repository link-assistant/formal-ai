#!/usr/bin/env rust-script
//! Non-mutating self-development preflight for automatic releases.
//!
//! A policy-ineligible cycle fails this command. Work in this repository is not
//! deferred, however hard it is, so there is no budget: a release cycle that
//! cannot be cut is a failure from the first push, and stays one until the work
//! that unblocks it is done (issue #1066).
//!
//! One relaxation applies, and only on a pull request. The architect stated both
//! halves of one rule: force each pull request to carry Formal AI-authored work,
//! *and* relax that after a day so a release can still be produced
//! (`docs/architect-notes/2026-09-11-do-not-obstruct-the-vision.md`). A relaxed
//! cycle prints as relaxed, never as satisfied, which is the distinction issue
//! #1066 was actually about: the budget it removed let an unreleasable cycle
//! report success. On a push to `main` there is no pull request to age and the
//! floor stays a standing, unrelaxed report.
//!
//! This command never publishes; it only decides. `version-and-commit.rs` holds
//! the same gate for the manual path, so neither route can be used to escape the
//! other.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::env;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::Command;

#[path = "self-hosting-metric.rs"]
mod self_hosting_metric;

fn git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
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

fn set_output(key: &str, value: &str) -> Result<(), String> {
    let Some(path) = env::var_os("GITHUB_OUTPUT") else {
        println!("Output: {key}={value}");
        return Ok(());
    };
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{key}={value}"))
        .map_err(|error| format!("could not write {key} to GITHUB_OUTPUT: {error}"))
}

/// The window the architect named: after a day, the requirement relaxes.
///
/// The same number `check-formal-ai-contribution.rs` uses, because it is the
/// same rule -- "if pull request exists for for more than a day we can relax
/// our requirements, and still be able to produce the release".
const RELAXATION_HOURS: i64 = 24;

/// How long the pull request under evaluation has been open, in hours.
///
/// `None` outside a pull request. On `push` to `main` there is nothing to age:
/// the cycle either carries authored work or it does not, and the floor stays a
/// standing report of that, unrelaxed.
fn relaxation_age_hours() -> Option<i64> {
    let opened = env::var("PULL_REQUEST_CREATED_AT").ok()?;
    let opened = opened.trim();
    if opened.is_empty() {
        return None;
    }
    let now = git(&["log", "-1", "--format=%cI", "HEAD"]).ok()?;
    hours_between(opened, &now).ok()
}

/// Hours between two RFC3339 timestamps, without a date dependency.
fn hours_between(opened: &str, now: &str) -> Result<i64, String> {
    Ok((epoch_seconds(now)? - epoch_seconds(opened)?) / 3600)
}

/// Seconds since the epoch for an RFC3339 timestamp.
///
/// Both forms in play are accepted: GitHub emits UTC
/// (`2026-09-12T06:22:31Z`), while `git log %cI` emits the committer's offset
/// (`2026-09-12T07:35:30+07:00`). Ignoring an offset would make a commit look
/// up to a day older or younger than it is, so it is subtracted.
fn epoch_seconds(stamp: &str) -> Result<i64, String> {
    let stamp = stamp.trim();
    let (date, rest) = stamp
        .split_once('T')
        .ok_or_else(|| format!("`{stamp}` is not an RFC3339 timestamp"))?;
    let (clock, offset_seconds) = split_offset(rest)?;

    let mut date_parts = date.split('-');
    let mut next_number = |what: &str| -> Result<i64, String> {
        date_parts
            .next()
            .ok_or_else(|| format!("`{stamp}` has no {what}"))?
            .parse::<i64>()
            .map_err(|error| format!("`{stamp}` has an unreadable {what}: {error}"))
    };
    let year = next_number("year")?;
    let month = next_number("month")?;
    let day = next_number("day")?;

    let mut clock_parts = clock.split(':');
    let mut next_clock = |what: &str| -> Result<i64, String> {
        clock_parts
            .next()
            .ok_or_else(|| format!("`{stamp}` has no {what}"))?
            .parse::<i64>()
            .map_err(|error| format!("`{stamp}` has an unreadable {what}: {error}"))
    };
    let hour = next_clock("hour")?;
    let minute = next_clock("minute")?;
    let second = next_clock("second")?;

    Ok(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second
        - offset_seconds)
}

/// Split a time-of-day from its zone offset, returning the offset in seconds.
fn split_offset(rest: &str) -> Result<(&str, i64), String> {
    if let Some(clock) = rest.strip_suffix('Z') {
        return Ok((clock, 0));
    }
    let index = rest
        .rfind(['+', '-'])
        .ok_or_else(|| format!("`{rest}` has no zone offset"))?;
    let (clock, zone) = rest.split_at(index);
    let negative = zone.starts_with('-');
    let zone = &zone[1..];
    let (hours, minutes) = zone
        .split_once(':')
        .ok_or_else(|| format!("`{zone}` is not a `hh:mm` offset"))?;
    let hours = hours
        .parse::<i64>()
        .map_err(|error| format!("`{zone}` has unreadable hours: {error}"))?;
    let minutes = minutes
        .parse::<i64>()
        .map_err(|error| format!("`{zone}` has unreadable minutes: {error}"))?;
    let seconds = hours * 3600 + minutes * 60;
    Ok((clock, if negative { -seconds } else { seconds }))
}

/// Days from 1970-01-01 to a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Whether this run is evaluating a pull request.
///
/// GitHub substitutes an empty string for
/// `github.event.pull_request.created_at` on a push, so a blank value means
/// "no pull request" just as an absent one does.
fn is_pull_request_event(created_at: Option<&str>) -> bool {
    created_at.is_some_and(|value| !value.trim().is_empty())
}

/// Attributed commits on the pull request under evaluation, if this is one.
///
/// `None` outside a pull request, so a push to `main` keeps the literal reading
/// and this path cannot be reached there.
///
/// The commits are validated by the same
/// `merged_self_authored_pull_requests` walk the merged reading uses, against a
/// range that ends at the branch tip instead of requiring a merge commit that
/// does not exist yet. That keeps every claim the trailers make under the same
/// scrutiny -- valid session evidence, an evidence path present in the commit,
/// and no attributed commit naming a different pull request -- rather than
/// trusting a bare `Formal-AI-Model:` line, which is exactly the shortcut that
/// would turn this into a bypass.
fn pull_request_attributed_commits(repo: &PathBuf, since: &str) -> Result<Option<usize>, String> {
    if !is_pull_request_event(env::var("PULL_REQUEST_CREATED_AT").ok().as_deref()) {
        return Ok(None);
    }
    let attributed = self_hosting_metric::attributed_commits_in_range(repo, since, "HEAD")?;
    Ok(Some(attributed))
}

fn run() -> Result<(), String> {
    if env::var("SKIP_BUMP").as_deref() == Ok("true") {
        println!("Existing release artifacts are incomplete; preserving the recovery path.");
        set_output("should_release", "true")?;
        return Ok(());
    }

    let repo = PathBuf::from(git(&["rev-parse", "--show-toplevel"])?);
    let since = git(&[
        "describe",
        "--tags",
        "--match",
        "v[0-9]*",
        "--abbrev=0",
        "HEAD",
    ])?;
    // On a pull request the cycle cannot yet contain a *merged* Formal AI pull
    // request: this branch is the pull request, and it merges after this check
    // runs, not before. Measured literally the floor was therefore unsatisfiable
    // on every `pull_request` event, which is not a floor being enforced -- it
    // is a question asked of the wrong commit.
    //
    // What the branch *can* answer is whether the work Formal AI authored is
    // here, which is exactly what will be in the pull request once it merges.
    // So a pull request carrying attributed commits satisfies the floor
    // prospectively, and says so. Nothing is waived: the same commits are
    // re-measured as a merged pull request on the push to `main`, where the
    // literal reading applies and this branch is gone.
    if let Some(commits) = pull_request_attributed_commits(&repo, &since)?
        && commits > 0
    {
        println!(
            "Self-development floor satisfied prospectively: {commits} commit(s) on this pull \
             request carry Formal AI session evidence, and become a merged Formal AI pull request \
             when it lands. Re-measured literally on the push to `main`."
        );
        set_output("should_release", "true")?;
        return Ok(());
    }

    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    match self_hosting_metric::self_development_release_status(
        &repo,
        &ledger,
        "prospective-auto-release",
        &since,
        "HEAD",
        3,
    )? {
        self_hosting_metric::SelfDevelopmentReleaseStatus::Eligible(eligibility) => {
            println!(
                "Self-development release preflight passed with {} reviewed Formal AI pull request(s).",
                eligibility.pull_requests.len()
            );
            set_output("should_release", "true")?;
        }
        // There is no quiet outcome. `should_release` is still written so a
        // caller reading the output sees a definite answer, and then the command
        // fails: a cycle that cannot be released is a defect being reported, not
        // a state being tolerated.
        //
        // Unless the architect's relaxation applies. His note of 2026-09-11
        // states both halves of one rule: force each pull request to carry
        // Formal AI-authored work, *and* relax that once the pull request has
        // been open for more than a day, "and still be able to produce the
        // release". `check-formal-ai-contribution.rs` implements the relaxation
        // for the per-pull-request half; this is the same window applied to the
        // cycle half, so a release is not held by a requirement the architect
        // already said should yield after a day
        // (docs/architect-notes/2026-09-11-do-not-obstruct-the-vision.md).
        //
        // This is not the budget issue #1066 removed. That budget let an
        // unreleasable cycle report *success*; a relaxed cycle here prints as
        // relaxed, never as satisfied, and only after the window has actually
        // elapsed.
        self_hosting_metric::SelfDevelopmentReleaseStatus::Blocked(reason) => {
            match relaxation_age_hours() {
                Some(hours) if hours > RELAXATION_HOURS => {
                    println!(
                        "relaxed: {reason}.\nThe pull request has been open {hours}h \
                         (> {RELAXATION_HOURS}h), so the requirement relaxes rather than blocking \
                         the release (docs/architect-notes/\
                         2026-09-11-do-not-obstruct-the-vision.md). This is reported as relaxed, \
                         never as satisfied."
                    );
                    set_output("should_release", "true")?;
                }
                _ => {
                    set_output("should_release", "false")?;
                    return Err(reason);
                }
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Self-development release preflight failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pull_request_reading_is_skipped_without_a_pull_request() {
        // GitHub substitutes an empty string for
        // `github.event.pull_request.created_at` on a push, so "absent" and
        // "blank" must both mean "not a pull request". Treating a blank value
        // as a zero-hour-old pull request would apply the prospective reading
        // to `main`, where the literal merged reading has to stand.
        //
        // `std::env::set_var` is unsafe in edition 2024 and this file has no
        // dev-dependency to scope it, so the decision is exercised through the
        // same predicate the guard uses rather than by mutating the process.
        for absent in [None, Some(""), Some("   ")] {
            assert!(
                !is_pull_request_event(absent),
                "{absent:?} must not be read as a pull request"
            );
        }
        assert!(is_pull_request_event(Some("2026-09-11T22:08:24Z")));
    }

    #[test]
    fn the_window_matches_the_contribution_gate() {
        // One rule, one number. If these drift, a pull request can be relaxed
        // by one gate and blocked by the other, which is the confusing state
        // the architect's note exists to prevent.
        assert_eq!(RELAXATION_HOURS, 24);
    }

    #[test]
    fn a_day_is_not_yet_more_than_a_day() {
        // The note says "more than a day", so the boundary is exclusive.
        let opened = "2026-09-11T00:00:00Z";
        assert_eq!(hours_between(opened, "2026-09-12T00:00:00Z").unwrap(), 24);
        assert!(hours_between(opened, "2026-09-12T00:00:00Z").unwrap() <= RELAXATION_HOURS);
        assert!(hours_between(opened, "2026-09-12T01:00:00Z").unwrap() > RELAXATION_HOURS);
    }

    #[test]
    fn a_zone_offset_is_subtracted_rather_than_ignored() {
        // `git log %cI` emits the committer's offset. Ignoring it would make a
        // commit look up to a day older or younger than it is, which would
        // relax a pull request that is not yet old enough.
        let utc = epoch_seconds("2026-09-12T00:00:00Z").unwrap();
        assert_eq!(epoch_seconds("2026-09-12T07:00:00+07:00").unwrap(), utc);
        assert_eq!(epoch_seconds("2026-09-11T19:00:00-05:00").unwrap(), utc);
    }

    #[test]
    fn hours_are_counted_across_days_and_months() {
        assert_eq!(
            hours_between("2026-08-31T23:00:00Z", "2026-09-01T01:00:00Z").unwrap(),
            2
        );
        assert_eq!(
            hours_between("2026-02-28T00:00:00Z", "2026-03-01T00:00:00Z").unwrap(),
            24
        );
    }

    #[test]
    fn a_malformed_timestamp_is_an_error_not_a_silent_zero() {
        // A silent zero would read as "opened just now" and block forever.
        assert!(epoch_seconds("not-a-timestamp").is_err());
        assert!(epoch_seconds("2026-09-12").is_err());
    }
}
