//! Release-cycle gate for Formal AI's reviewed self-development loop.

use super::{
    EvidencePolicy, METRIC_VERSION, PULL_REQUEST_TRAILER, ReleaseRow,
    commit_has_formal_ai_evidence, git, read_release_rows, trailer_values,
};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseEligibility {
    pub pull_requests: Vec<String>,
    pub target_percentage_basis_points: u64,
    pub projected_percentage_basis_points: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelfDevelopmentReleaseStatus {
    Eligible(ReleaseEligibility),
    Deferred(String),
    /// A deferral that has outlived its budget.
    ///
    /// Deferring is the right answer for a cycle that is merely young: `main`
    /// is immutable, so a policy-ineligible push must not go red. But the
    /// deferral is invisible — the job stays green and publishes nothing — and
    /// an invisible stop has no natural end. Issue #1064 measured what that
    /// costs: 268 commits and 45 changelog fragments waited 14 days behind a
    /// green pipeline, and one of them was the fix a downstream consumer was
    /// blocked on. Past the budget the same deferral is reported as a failure,
    /// so the pipeline says out loud what it has been doing silently.
    Overdue(String),
}

impl SelfDevelopmentReleaseStatus {
    /// The reason a non-eligible status carries, whatever its severity.
    pub fn deferral_reason(&self) -> Option<&str> {
        match self {
            Self::Eligible(_) => None,
            Self::Deferred(reason) | Self::Overdue(reason) => Some(reason),
        }
    }
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
        if is_attributed && let Some(reference) = validated_commit_pull_request(repo, commit)? {
            attributed.insert(commit.to_owned(), reference);
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
        let introduced = branch_commits
            .lines()
            .filter(|commit| !commit.is_empty())
            .collect::<Vec<_>>();
        let end_to_end = !introduced.is_empty()
            && introduced.iter().all(|commit| {
                attributed
                    .get(*commit)
                    .is_some_and(|reference| pull_request_number(reference) == Some(number))
            });
        if end_to_end {
            let reference = attributed
                .get(introduced[0])
                .expect("end-to-end attribution checked every introduced commit");
            if !pull_requests.contains(reference) {
                pull_requests.push(reference.clone());
            }
        }
    }
    Ok(pull_requests)
}

pub(super) fn target_from_rows(rows: &[ReleaseRow]) -> u64 {
    rows.iter()
        .rfind(|row| row.metric_version == METRIC_VERSION)
        .map_or(0, |row| {
            row.target_percentage_basis_points
                .unwrap_or(row.trailing_percentage_basis_points)
                .max(row.trailing_percentage_basis_points)
        })
}

/// How long a release may stay deferred before the deferral itself is a defect.
///
/// Seven days is the shortest window that cannot be hit by ordinary weekend
/// quiet: a cycle that has gone a full week without a single reviewed Formal
/// AI-authored pull request is not waiting for one, it is stuck.
pub const DEFERRAL_BUDGET_DAYS: u64 = 7;

/// How many pending changelog fragments a deferral may hold back.
///
/// Every fragment is a user-visible change already merged to `main` and
/// promised to the next release. Issue #1064 hit 45. Twenty is well above a
/// normal cycle and well below a backlog nobody can review as one release.
pub const DEFERRAL_BUDGET_FRAGMENTS: usize = 20;

/// Age in whole days of the commit a range starts from.
///
/// `since` is the last released tag, so this is how long the unreleased cycle
/// has been accumulating. A tag git cannot date is not evidence of an overdue
/// release, so it reports no age rather than guessing one.
fn cycle_age_days(repo: &Path, since: &str) -> Option<u64> {
    let timestamp = git(repo, &["log", "-1", "--format=%ct", since])
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(now.saturating_sub(timestamp) / 86_400)
}

/// Changelog fragments waiting for the release this cycle has not cut.
///
/// `changelog.d/` holds one file per user-visible change; the insert marker and
/// any dotfile are bookkeeping, not fragments.
fn pending_fragment_count(repo: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(repo.join("changelog.d")) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.') && name.ends_with(".md"))
        .count()
}

/// Escalate a deferral that has outlived its budget.
///
/// The reason text is preserved exactly and the budget breach is appended, so
/// the operator reads why the cycle is ineligible *and* why that stopped being
/// acceptable in the same sentence.
fn classify_deferral(repo: &Path, since: &str, reason: String) -> SelfDevelopmentReleaseStatus {
    let age = cycle_age_days(repo, since);
    let fragments = pending_fragment_count(repo);
    let mut breaches = Vec::new();
    if let Some(days) = age
        && days >= DEFERRAL_BUDGET_DAYS
    {
        breaches.push(format!(
            "the cycle has been deferred for {days} days (budget {DEFERRAL_BUDGET_DAYS})"
        ));
    }
    if fragments >= DEFERRAL_BUDGET_FRAGMENTS {
        breaches.push(format!(
            "{fragments} changelog fragments are waiting (budget {DEFERRAL_BUDGET_FRAGMENTS})"
        ));
    }
    if breaches.is_empty() {
        return SelfDevelopmentReleaseStatus::Deferred(reason);
    }
    SelfDevelopmentReleaseStatus::Overdue(format!(
        "{reason}. This deferral has outlived its budget: {}. A release blocked this long is not \
         a quiet cycle, it is an outage: cut the release through the manual path, or merge the \
         reviewed Formal AI-authored work the policy is waiting for",
        breaches.join(", ")
    ))
}

pub fn self_development_release_status(
    repo: &Path,
    ledger: &Path,
    tag: &str,
    since: &str,
    until: &str,
    trailing_window: usize,
) -> Result<SelfDevelopmentReleaseStatus, String> {
    let pull_requests =
        merged_self_authored_pull_requests(repo, since, until, EvidencePolicy::Lenient)?;
    if pull_requests.is_empty() {
        return Ok(classify_deferral(
            repo,
            since,
            format!(
                "release cycle {since}..{until} has no merged Formal AI-authored pull request; an \
                 end-to-end Formal AI-authored pull request requires valid session evidence and \
                 the same canonical PR trailer on every introduced non-merge commit"
            ),
        ));
    }
    let mut rows = read_release_rows(ledger)?;
    rows.retain(|row| row.tag != tag);
    let target = target_from_rows(&rows);
    let projected =
        super::project_trailing_share(repo, ledger, since, until, trailing_window, Some(tag))?;
    if projected < target {
        return Ok(classify_deferral(
            repo,
            since,
            format!(
                "self-hosting target would fall from {} to {} for {since}..{until}; merge \
                 additional reviewed Formal AI-authored work before cutting the release",
                super::format_percentage(target),
                super::format_percentage(projected),
            ),
        ));
    }
    Ok(SelfDevelopmentReleaseStatus::Eligible(ReleaseEligibility {
        pull_requests,
        target_percentage_basis_points: target,
        projected_percentage_basis_points: projected,
    }))
}

pub fn ensure_self_development_release(
    repo: &Path,
    ledger: &Path,
    tag: &str,
    since: &str,
    until: &str,
    trailing_window: usize,
) -> Result<ReleaseEligibility, String> {
    match self_development_release_status(repo, ledger, tag, since, until, trailing_window)? {
        SelfDevelopmentReleaseStatus::Eligible(eligibility) => Ok(eligibility),
        SelfDevelopmentReleaseStatus::Deferred(reason)
        | SelfDevelopmentReleaseStatus::Overdue(reason) => Err(reason),
    }
}
