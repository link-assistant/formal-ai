#!/usr/bin/env rust-script
//! Check that the tip of the default branch has actually been through the
//! pipeline, and name the commits that were not.
//!
//! ## Why this exists
//!
//! Issue #977 named the repository's most expensive class of defect: a result
//! nobody sees. It found `cancelled` runs, which are grey rather than red.
//! Issue #1081 found the same shape one step earlier -- a commit on `main` with
//! **no run at all**. Nothing is grey, nothing is red, and the branch summary
//! shows the last commit that did run, which looks exactly like success.
//!
//! The cause is documented behaviour, not a bug:
//!
//! > When you use the repository's `GITHUB_TOKEN` to perform tasks, events
//! > triggered by the `GITHUB_TOKEN` [...] will not create a new workflow run.
//! > This prevents you from accidentally creating recursive workflow runs.
//! > -- <https://docs.github.com/en/actions/security-for-github-actions/security-guides/automatic-token-authentication>
//!
//! `.github/workflows/external-benchmarks.yml` pushes the refreshed benchmark
//! ledger to the default branch on a weekly cron, authenticated with exactly
//! that token. Measured on 2026-09-07: commit `6039c4d9` received the full
//! push fan-out including `CI/CD Pipeline` (which concluded `failure`), and the
//! ledger commit `dda02efb` pushed on top of it received **zero** push-triggered
//! runs. So the tree at the tip of `main` had never been validated, and the
//! branch page showed the previous commit's verdict.
//!
//! This is not a defect that can be fixed by changing the token: a personal
//! access token would re-enable the runs and re-enable recursion with them, and
//! `workflow_dispatch` on `release.yml` means "publish a release", not
//! "validate this commit". What can be fixed is the invisibility. This audit
//! says out loud, once a week, that the tip of the default branch has not been
//! tested and how many commits back the last tested one is.
//!
//! ## Why "unknown" is a distinct answer
//!
//! A commit pushed a minute ago has a run that may not be listed yet, and a
//! run that has not appeared yet is not the same fact as a run that will never
//! appear. Reporting the first as a gap would make the audit cry wolf whenever
//! it happened to fire during a push. So a tip younger than
//! [`GRACE_MINUTES`] is reported as `unknown`, which is neither a pass nor a
//! failure -- the recommendation in the pipeline best-practices guide
//! (principle 16: "Report `unknown`, never a guess").
//!
//! ## Why only the tip, and not every commit
//!
//! Pushing five commits at once creates one workflow run, for the fifth. The
//! other four are legitimately unrun, and counting them as gaps would make the
//! audit's number meaningless: measured over 60 `main` commits, 23 had no
//! `CI/CD Pipeline` run and all but one of those were interior commits of a
//! batch push. The tip is the tree that ships, so the tip is what is audited;
//! the walk backwards exists only to say how stale the last real verdict is.
//!
//! ## Usage
//!
//!   rust-script scripts/check-head-pipeline-coverage.rs [--branch <name>]
//!                                                       [--depth <n>]
//!                                                       [--json <file>]
//!
//! Requires `gh`, authenticated. Set `GH_TOKEN` when running under Actions.
//! `--json` writes the collected evidence so a failing audit can be re-read
//! without re-querying the API.
//!
//! Exit status is 0 whether or not a gap is found: this is a visibility audit,
//! and the tip being untested is a property of how the ledger is written rather
//! than a regression any pull request introduced. It emits a `::warning`, and
//! `--fail-on-gap` turns that into an error for anyone who wants the stronger
//! contract.

use std::fmt::Write as _;
use std::process::Command;

/// How recent a tip has to be before "no run yet" stops meaning "no run".
const GRACE_MINUTES: i64 = 20;

/// The workflow whose absence is the thing worth reporting.
const PIPELINE: &str = "CI/CD Pipeline";

/// One commit and the pipeline runs that named it as their head.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CommitCoverage {
    sha: String,
    committed_at: String,
    author: String,
    subject: String,
    /// The `event` of every `CI/CD Pipeline` run whose head is this commit.
    pipeline_events: Vec<String>,
}

impl CommitCoverage {
    fn covered(&self) -> bool {
        !self.pipeline_events.is_empty()
    }

    fn short(&self) -> &str {
        &self.sha[..self.sha.len().min(9)]
    }
}

/// What the audit concluded about the tip.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Verdict {
    /// The tip has a pipeline run.
    Covered,
    /// The tip is younger than the grace period and has none yet, which is not
    /// yet an answer.
    Unknown { age_minutes: i64 },
    /// The tip has no pipeline run and is old enough that it never will.
    Gap {
        /// How many commits back the most recent covered commit is; `None`
        /// when none of the sampled commits is covered.
        stale_by: Option<usize>,
    },
}

/// The whole verdict, derived from the sample alone so it can be tested
/// without the network.
fn verdict(commits: &[CommitCoverage], age_minutes: i64) -> Verdict {
    let Some(tip) = commits.first() else {
        // No commits is a broken query, not a healthy branch. Treated as
        // unknown so a transient API answer cannot be read as a pass.
        return Verdict::Unknown { age_minutes };
    };
    if tip.covered() {
        return Verdict::Covered;
    }
    if age_minutes < GRACE_MINUTES {
        return Verdict::Unknown { age_minutes };
    }
    Verdict::Gap {
        stale_by: commits.iter().position(CommitCoverage::covered),
    }
}

/// The human-readable table, printed and appended to the job summary.
fn report(commits: &[CommitCoverage], verdict: &Verdict) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n## Default-branch pipeline coverage\n");
    let _ = writeln!(out, "| commit | committed | author | {PIPELINE} |");
    let _ = writeln!(out, "| --- | --- | --- | --- |");
    for commit in commits {
        let runs = if commit.covered() {
            commit.pipeline_events.join(", ")
        } else {
            "— none —".to_string()
        };
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} |",
            commit.short(),
            commit.committed_at,
            commit.author,
            runs
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{}",
        match verdict {
            Verdict::Covered => "The tip of the default branch has been through the pipeline.".to_string(),
            Verdict::Unknown { age_minutes } => format!(
                "Unknown: the tip is {age_minutes} minute(s) old and its runs may not be listed \
                 yet (grace: {GRACE_MINUTES} minutes). Not a pass and not a failure."
            ),
            Verdict::Gap { stale_by: Some(distance) } => format!(
                "The tip has no {PIPELINE} run. The last commit that does is {distance} commit(s) \
                 back, so that is the newest tree anything has validated."
            ),
            Verdict::Gap { stale_by: None } => format!(
                "The tip has no {PIPELINE} run, and neither does any commit in this sample."
            ),
        }
    );
    out
}

/// The lines a run of this audit adds to the log. Separated from `main` so the
/// exact wording is testable.
fn annotations(commits: &[CommitCoverage], verdict: &Verdict) -> Vec<String> {
    let tip = commits.first();
    match verdict {
        Verdict::Covered => Vec::new(),
        Verdict::Unknown { age_minutes } => vec![format!(
            "::notice title=Head pipeline coverage unknown::The tip is {age_minutes} minute(s) \
             old, younger than the {GRACE_MINUTES}-minute grace period, so \"no run\" does not \
             yet mean \"no run will appear\"."
        )],
        Verdict::Gap { stale_by } => {
            let tip = tip.expect("a gap verdict implies a sampled tip");
            let staleness = match stale_by {
                Some(distance) => format!(
                    "the last validated tree is {distance} commit(s) behind it"
                ),
                None => "no commit in the sample has been validated".to_string(),
            };
            let cause = if tip.author.contains("[bot]") {
                " The tip was pushed by a bot, which is the documented cause: a push \
                 authenticated with the repository's GITHUB_TOKEN creates no workflow run."
            } else {
                ""
            };
            vec![format!(
                "::warning title=Default-branch tip was never tested::{} (\"{}\") has no \
                 {PIPELINE} run, and {staleness}. The branch page shows the previous commit's \
                 verdict, which is why this is invisible without asking.{cause}",
                tip.short(),
                tip.subject
            )]
        }
    }
}

fn gh(arguments: &[&str]) -> String {
    let output = Command::new("gh")
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("gh {arguments:?}: {error}"));
    if !output.status.success() {
        eprintln!("gh {arguments:?} failed:\n{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(2);
    }
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Minutes between an RFC 3339 timestamp and now, without pulling in a date
/// crate: `date -d` is on every runner this audit runs on, and the arithmetic
/// is not worth a dependency for a script that already requires `gh`.
fn age_minutes(timestamp: &str) -> i64 {
    let parse = |argument: &str| -> Option<i64> {
        let output = Command::new("date").args(["-u", "-d", argument, "+%s"]).output().ok()?;
        String::from_utf8_lossy(&output.stdout).trim().parse().ok()
    };
    match (parse(timestamp), parse("now")) {
        (Some(then), Some(now)) => (now - then) / 60,
        // An unparsed timestamp must not become a confident "old enough".
        _ => 0,
    }
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let mut branch: Option<String> = None;
    let mut depth: usize = 10;
    let mut json_path: Option<String> = None;
    let mut fail_on_gap = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--branch" => branch = arguments.next(),
            "--depth" => {
                depth = arguments
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--depth takes a number of commits");
                        std::process::exit(2);
                    })
            }
            "--json" => json_path = arguments.next(),
            "--fail-on-gap" => fail_on_gap = true,
            other => {
                eprintln!("unknown argument {other:?}");
                std::process::exit(2);
            }
        }
    }

    let repository = std::env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| {
        gh(&["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"])
            .trim()
            .to_string()
    });
    let branch = branch.filter(|value| !value.is_empty()).unwrap_or_else(|| {
        gh(&["api", &format!("repos/{repository}"), "--jq", ".default_branch"])
            .trim()
            .to_string()
    });
    eprintln!("Auditing the last {depth} commit(s) of {repository} on {branch}.");

    let listing = gh(&[
        "api",
        &format!("repos/{repository}/commits?sha={branch}&per_page={depth}"),
        "--jq",
        r#".[] | [.sha, .commit.author.date, (.author.login // .commit.author.name // "?"), (.commit.message | split("\n")[0])] | @tsv"#,
    ]);

    let mut commits = Vec::new();
    for line in listing.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let [sha, committed_at, author, subject] = fields[..] else {
            continue;
        };
        let events = gh(&[
            "api",
            &format!("repos/{repository}/actions/runs?head_sha={sha}&per_page=100"),
            "--jq",
            &format!(r#"[.workflow_runs[] | select(.name == "{PIPELINE}") | .event] | join(",")"#),
        ]);
        commits.push(CommitCoverage {
            sha: sha.to_string(),
            committed_at: committed_at.to_string(),
            author: author.to_string(),
            subject: subject.to_string(),
            pipeline_events: events
                .trim()
                .split(',')
                .filter(|event| !event.is_empty())
                .map(str::to_string)
                .collect(),
        });
    }

    let tip_age = commits.first().map_or(0, |tip| age_minutes(&tip.committed_at));
    let verdict = verdict(&commits, tip_age);

    let summary = report(&commits, &verdict);
    print!("{summary}");
    if let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY") {
        use std::io::Write as _;
        if let Ok(mut file) = std::fs::OpenOptions::new().append(true).create(true).open(path) {
            let _ = file.write_all(summary.as_bytes());
        }
    }
    if let Some(path) = json_path {
        let rows: Vec<String> = commits
            .iter()
            .map(|commit| {
                format!(
                    r#"{{"sha":"{}","committed_at":"{}","author":"{}","pipeline_events":[{}]}}"#,
                    commit.sha,
                    commit.committed_at,
                    commit.author,
                    commit
                        .pipeline_events
                        .iter()
                        .map(|event| format!(r#""{event}""#))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect();
        let _ = std::fs::write(&path, format!("[{}]\n", rows.join(",")));
    }

    for annotation in annotations(&commits, &verdict) {
        println!("{annotation}");
    }

    if fail_on_gap && matches!(verdict, Verdict::Gap { .. }) {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(sha: &str, author: &str, events: &[&str]) -> CommitCoverage {
        CommitCoverage {
            sha: sha.to_string(),
            committed_at: "2026-09-07T09:31:42Z".to_string(),
            author: author.to_string(),
            subject: "chore(benchmarks): record scheduled upstream results (#698)".to_string(),
            pipeline_events: events.iter().map(|event| event.to_string()).collect(),
        }
    }

    /// The measured case. `dda02efb` (a GITHUB_TOKEN ledger push) had no run;
    /// `6039c4d9` beneath it had the full fan-out.
    #[test]
    fn the_measured_ledger_push_is_reported_as_a_gap_one_commit_deep() {
        let commits = vec![
            commit("dda02efb475c0042cdad3583ff74bdd3e90ee5df", "github-actions[bot]", &[]),
            commit("6039c4d94634beb62c9d853fcdbd59c1ece8d541", "konard", &["push"]),
        ];
        assert_eq!(
            verdict(&commits, 120),
            Verdict::Gap { stale_by: Some(1) }
        );
        let lines = annotations(&commits, &verdict(&commits, 120));
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("::warning title=Default-branch tip was never tested::"));
        assert!(
            lines[0].contains("GITHUB_TOKEN creates no workflow run"),
            "a bot-authored tip should name the documented cause: {}",
            lines[0]
        );
    }

    /// The false positive this audit must not produce: a tip pushed a moment
    /// ago has runs that may simply not be listed yet.
    #[test]
    fn a_tip_younger_than_the_grace_period_is_unknown_rather_than_a_gap() {
        let commits = vec![commit("aaaaaaaa", "konard", &[])];
        assert_eq!(
            verdict(&commits, GRACE_MINUTES - 1),
            Verdict::Unknown { age_minutes: GRACE_MINUTES - 1 }
        );
        let lines = annotations(&commits, &verdict(&commits, GRACE_MINUTES - 1));
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("::notice"), "unknown is not a warning: {}", lines[0]);
    }

    #[test]
    fn a_covered_tip_says_nothing_at_all() {
        let commits = vec![commit("aaaaaaaa", "konard", &["push"])];
        assert_eq!(verdict(&commits, 500), Verdict::Covered);
        assert!(annotations(&commits, &Verdict::Covered).is_empty());
    }

    /// An interior commit of a batch push is legitimately unrun, so a covered
    /// tip is a pass no matter how many uncovered commits sit beneath it.
    #[test]
    fn uncovered_commits_beneath_a_covered_tip_are_not_a_gap() {
        let commits = vec![
            commit("aaaaaaaa", "konard", &["push"]),
            commit("bbbbbbbb", "konard", &[]),
            commit("cccccccc", "konard", &[]),
        ];
        assert_eq!(verdict(&commits, 500), Verdict::Covered);
    }

    /// An empty listing is a failed query, and a failed query must never read
    /// as a healthy branch.
    #[test]
    fn an_empty_sample_is_unknown_and_never_a_pass() {
        assert_eq!(verdict(&[], 999), Verdict::Unknown { age_minutes: 999 });
    }

    #[test]
    fn a_sample_with_nothing_covered_says_so_instead_of_naming_a_distance() {
        let commits = vec![
            commit("aaaaaaaa", "konard", &[]),
            commit("bbbbbbbb", "konard", &[]),
        ];
        assert_eq!(verdict(&commits, 500), Verdict::Gap { stale_by: None });
        let text = report(&commits, &verdict(&commits, 500));
        assert!(text.contains("neither does any commit in this sample"), "{text}");
    }

    #[test]
    fn the_report_names_every_sampled_commit() {
        let commits = vec![
            commit("dda02efb475c0042cdad3583ff74bdd3e90ee5df", "github-actions[bot]", &[]),
            commit("6039c4d94634beb62c9d853fcdbd59c1ece8d541", "konard", &["push"]),
        ];
        let text = report(&commits, &verdict(&commits, 500));
        assert!(text.contains("`dda02efb4`"), "{text}");
        assert!(text.contains("`6039c4d94`"), "{text}");
        assert!(text.contains("— none —"), "{text}");
    }
}
