//! Release-cycle gate for Formal AI's reviewed self-development loop.

use super::{
    commit_has_formal_ai_evidence, git, project_trailing_basis_points, read_release_rows,
    trailer_values, EvidencePolicy, ReleaseRow, METRIC_VERSION, PULL_REQUEST_TRAILER,
};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseEligibility {
    pub pull_requests: Vec<String>,
    pub target_percentage_basis_points: u64,
    pub projected_percentage_basis_points: u64,
}

/// Validate an optional PR trailer without requiring one on legacy commits.
///
/// The release gate proves the referenced PR actually introduced the commit;
/// this earlier check only keeps malformed claims from reaching `main`.
pub(super) fn validated_commit_pull_request(
    repo: &Path,
    commit: &str,
) -> Result<Option<String>, String> {
    let values = trailer_values(repo, commit, PULL_REQUEST_TRAILER)?;
    if values.len() > 1 {
        return Err(format!(
            "commit {commit} records more than one {PULL_REQUEST_TRAILER}"
        ));
    }
    let Some(reference) = values.into_iter().next() else {
        return Ok(None);
    };
    pull_request_number(&reference).ok_or_else(|| {
        format!("{PULL_REQUEST_TRAILER} must be a canonical GitHub pull-request URL: {reference}")
    })?;
    Ok(Some(reference))
}

fn pull_request_number(reference: &str) -> Option<u64> {
    let path = reference.strip_prefix("https://github.com/")?;
    let mut components = path.split('/');
    let owner = components.next()?;
    let repository = components.next()?;
    let pull = components.next()?;
    let number = components.next()?;
    if owner.is_empty() || repository.is_empty() || pull != "pull" || components.next().is_some() {
        return None;
    }
    number.parse::<u64>().ok().filter(|number| *number > 0)
}

pub(super) fn merged_self_authored_pull_requests(
    repo: &Path,
    since: &str,
    until: &str,
    policy: EvidencePolicy,
) -> Result<Vec<String>, String> {
    let range = format!("{since}..{until}");
    let commits = git(repo, &["rev-list", "--reverse", "--no-merges", &range])?;
    let mut attributed = BTreeMap::new();
    for commit in commits.lines().filter(|commit| !commit.is_empty()) {
        let is_attributed = match commit_has_formal_ai_evidence(repo, commit) {
            Ok(attributed) => attributed,
            Err(error) => match policy {
                EvidencePolicy::Strict => return Err(error),
                EvidencePolicy::Lenient => {
                    eprintln!("warning: not attributing {commit}: {error}");
                    false
                }
            },
        };
        if is_attributed {
            if let Some(reference) = validated_commit_pull_request(repo, commit)? {
                attributed.insert(commit.to_owned(), reference);
            }
        }
    }

    let merges = git(
        repo,
        &[
            "rev-list",
            "--reverse",
            "--first-parent",
            "--merges",
            &range,
        ],
    )?;
    let mut pull_requests = Vec::new();
    for merge in merges.lines().filter(|merge| !merge.is_empty()) {
        let subject = git(repo, &["show", "-s", "--format=%s", merge])?;
        let Some(number) = subject
            .strip_prefix("Merge pull request #")
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
        else {
            continue;
        };
        let parents = git(repo, &["rev-list", "--parents", "-n", "1", merge])?;
        let parents = parents.split_whitespace().collect::<Vec<_>>();
        if parents.len() < 3 {
            continue;
        }
        let exclude_first_parent = format!("^{}", parents[1]);
        let branch_commits = git(
            repo,
            &["rev-list", "--no-merges", parents[2], &exclude_first_parent],
        )?;
        for commit in branch_commits.lines() {
            let Some(reference) = attributed.get(commit) else {
                continue;
            };
            if pull_request_number(reference) == Some(number) && !pull_requests.contains(reference)
            {
                pull_requests.push(reference.clone());
            }
        }
    }
    Ok(pull_requests)
}

pub(super) fn target_from_rows(rows: &[ReleaseRow]) -> u64 {
    rows.iter()
        .filter(|row| row.metric_version == METRIC_VERSION)
        .next_back()
        .map(|row| {
            row.target_percentage_basis_points
                .unwrap_or(row.trailing_percentage_basis_points)
                .max(row.trailing_percentage_basis_points)
        })
        .unwrap_or(0)
}

pub fn ensure_self_development_release(
    repo: &Path,
    ledger: &Path,
    since: &str,
    until: &str,
    trailing_window: usize,
) -> Result<ReleaseEligibility, String> {
    let pull_requests =
        merged_self_authored_pull_requests(repo, since, until, EvidencePolicy::Lenient)?;
    if pull_requests.is_empty() {
        return Err(format!(
            "release cycle {since}..{until} has no merged Formal AI-authored pull request with \
             valid session evidence"
        ));
    }
    let rows = read_release_rows(ledger)?;
    let target = target_from_rows(&rows);
    let projected = project_trailing_basis_points(repo, ledger, since, until, trailing_window)?;
    if projected < target {
        return Err(format!(
            "self-hosting target would fall from {} to {} for {since}..{until}; merge additional \
             reviewed Formal AI-authored work before cutting the release",
            super::format_percentage(target),
            super::format_percentage(projected),
        ));
    }
    Ok(ReleaseEligibility {
        pull_requests,
        target_percentage_basis_points: target,
        projected_percentage_basis_points: projected,
    })
}
