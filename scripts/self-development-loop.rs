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
    /// The cycle cannot be released, and that is a failure from the first push.
    ///
    /// There is no deferral. Issue #1065 introduced a seven-day, twenty-fragment
    /// window in which an ineligible cycle still reported success, on the theory
    /// that a young cycle is merely waiting. That theory is what issue #1064
    /// measured the cost of: 275 commits and 48 fragments — one of them the fix a
    /// downstream consumer was blocked on — sat behind a green checkmark for two
    /// weeks, because a silent stop is indistinguishable from a healthy pipeline.
    ///
    /// Work in this repository is not deferred, however hard it is, so a cycle
    /// that cannot be cut is reported as blocked immediately and stays blocked
    /// until the work that unblocks it is done (issue #1066).
    Blocked(String),
}

impl SelfDevelopmentReleaseStatus {
    /// Why the cycle cannot be released, if it cannot.
    pub fn blocked_reason(&self) -> Option<&str> {
        match self {
            Self::Eligible(_) => None,
            Self::Blocked(reason) => Some(reason),
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
        // A pull request counts for the work Formal AI did in it, not for what
        // else happened to be in it. Requiring *every* introduced commit to be
        // attributed measured the composition of a pull request rather than the
        // authorship of the work, and it had one practical consequence: a
        // self-authored change could never ride along inside ordinary review,
        // because a single human commit beside it erased it. Every contribution
        // therefore needed a pull request containing nothing else, which is the
        // separate pull request the maintainer asked to stop needing on
        // [PR #1070][decision].
        //
        // The measurement is unaffected. `measure` counts lines per commit: an
        // unattributed commit contributes to the denominator and not the
        // numerator whether or not a sibling commit is attributed, so this
        // cannot move the share by a basis point. What is still enforced is
        // every claim the trailers make -- valid session evidence, an evidence
        // path present in that commit, and no attributed commit pointing at a
        // pull request other than the one that introduced it.
        //
        // [decision]: https://github.com/link-assistant/formal-ai/pull/1070#issuecomment-5539328163
        let attributed_here = introduced
            .iter()
            .filter_map(|commit| attributed.get(*commit))
            .collect::<Vec<&String>>();
        let claims_this_pull_request = attributed_here
            .iter()
            .all(|reference| pull_request_number(reference) == Some(number));
        if let Some(reference) = attributed_here.first().filter(|_| claims_this_pull_request)
            && !pull_requests.contains(*reference)
        {
            pull_requests.push((*reference).clone());
        }
    }
    Ok(pull_requests)
}

/// The share the next release must reach, read off the newest comparable row.
///
/// A row that records no target derives one from its own measured trailing
/// share, and because every recorded release carries the previous target
/// forward (see `record_release_with_policy`), the sequence ratchets upward on
/// its own: a dip in one cycle does not lower the bar for the next.
///
/// The ratchet can only ever climb, which is why it needs a way back down that
/// is not a bypass. Once a cycle measured high the level became unreachable
/// except by out-measuring it, and no review could lower it -- issue #1069 hit
/// exactly that wall. `target_override_basis_points` is that way back down: a
/// number written into the ledger by a reviewed commit, replacing the ratchet
/// for as long as it stays there. The maintainer's decision on
/// [PR #1070][decision] is that the level is theirs to set: *"It is ok to
/// contradict the issue #1069, I asked to reduce % to pass faster and fail
/// faster in production we need release with actual docker image to test it and
/// continue to iterate, we will increase % later."*
///
/// The lever is deliberately the ledger and nothing else. There is no flag, no
/// environment variable, and no workflow input that changes this number: moving
/// it means committing a reviewed change to
/// `data/meta/self-hosting-ledger.lino`, where the value is visible in the diff
/// and named in the release notes. Lowering the bar is allowed; lowering it
/// quietly is not.
///
/// [decision]: https://github.com/link-assistant/formal-ai/pull/1070#issuecomment-5535449300
pub(super) fn target_from_rows(rows: &[ReleaseRow]) -> u64 {
    rows.iter()
        .rfind(|row| row.metric_version == METRIC_VERSION)
        .map_or(0, |row| {
            row.target_override_basis_points.unwrap_or_else(|| {
                row.target_percentage_basis_points
                    .unwrap_or(0)
                    .max(row.trailing_percentage_basis_points)
            })
        })
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
        return Ok(SelfDevelopmentReleaseStatus::Blocked(format!(
            "release cycle {since}..{until} has no merged Formal AI-authored pull request; a \
             merged pull request counts once it introduced at least one commit carrying valid \
             session evidence, and every attributed commit it introduced names that same pull \
             request"
        )));
    }
    let mut rows = read_release_rows(ledger)?;
    rows.retain(|row| row.tag != tag);
    let target = target_from_rows(&rows);
    let projected =
        super::project_trailing_share(repo, ledger, since, until, trailing_window, Some(tag))?;
    if projected < target {
        return Ok(SelfDevelopmentReleaseStatus::Blocked(format!(
            "self-hosting target would fall from {} to {} for {since}..{until}; merge additional \
             reviewed Formal AI-authored work before cutting the release",
            super::format_percentage(target),
            super::format_percentage(projected),
        )));
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
        SelfDevelopmentReleaseStatus::Blocked(reason) => Err(reason),
    }
}
