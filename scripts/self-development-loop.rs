//! Release-cycle gate for Formal AI's reviewed self-development loop.

use super::{
    EvidencePolicy, METRIC_VERSION, PULL_REQUEST_TRAILER, ReleaseRow,
    commit_has_formal_ai_evidence, git, read_release_rows, retracted_commits, trailer_values,
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

/// Commits in `since..until` whose Formal AI attribution survives validation.
///
/// The keys are commits; the values are the pull request each one names. Split
/// out of [`merged_self_authored_pull_requests`] so the pull-request reading of
/// the self-development floor validates a commit exactly as the merged reading
/// does -- same retractions, same evidence check, same pull-request agreement.
/// A second, looser walk would be a bypass wearing the same name.
pub fn attributed_commits(
    repo: &Path,
    since: &str,
    until: &str,
    policy: EvidencePolicy,
) -> Result<BTreeMap<String, String>, String> {
    let range = format!("{since}..{until}");
    let commits = git(repo, &["rev-list", "--reverse", "--no-merges", &range])?;
    let commits = commits
        .lines()
        .filter(|commit| !commit.is_empty())
        .collect::<Vec<_>>();
    // The same retractions the metric honours (issue #1079). Applying them on
    // only one of the two walks would let a commit stay out of the measured
    // share and still count toward the release floor, which is the direction
    // that matters -- a retraction must never leave a claim standing.
    let retracted = retracted_commits(repo, &commits, policy)?;
    let mut attributed = BTreeMap::new();
    for commit in commits {
        let is_attributed = if retracted.iter().any(|sha| sha == commit) {
            false
        } else {
            match commit_has_formal_ai_evidence(repo, commit) {
                Ok(attributed) => attributed,
                Err(error) => match policy {
                    EvidencePolicy::Strict => return Err(error),
                    EvidencePolicy::Lenient => {
                        eprintln!("warning: not attributing {commit}: {error}");
                        false
                    }
                },
            }
        };
        if is_attributed && let Some(reference) = validated_commit_pull_request(repo, commit)? {
            attributed.insert(commit.to_owned(), reference);
        }
    }
    Ok(attributed)
}

pub(super) fn merged_self_authored_pull_requests(
    repo: &Path,
    since: &str,
    until: &str,
    policy: EvidencePolicy,
) -> Result<Vec<String>, String> {
    let range = format!("{since}..{until}");
    let attributed = attributed_commits(repo, since, until, policy)?;

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

/// How many changed lines a cycle needs before its share may raise the ratchet.
///
/// Below this the percentage is dominated by whichever handful of lines the
/// cycle happened to contain: `v0.348.1` changed one line and measured 100%.
/// Such a row still records and reports its share -- it simply may not set the
/// floor every later cycle has to clear.
pub const RATCHET_EVIDENCE_FLOOR: u64 = 100;

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
///
/// One measured share is refused as a source for the ratchet: the one a cycle
/// too small to mean anything produced. `v0.348.1` changed a single line, that
/// line happened to be Formal AI's, and the cycle therefore measured 100%.
/// Weighted into the trailing window that one line carried the bar to 2.91%,
/// and the next cycle -- PR #1125, thousands of reviewed lines with a document
/// Formal AI authored inside it -- projected 0.05% and was blocked. The bar had
/// been set by a release too small to measure, and it punished the next cycle
/// for containing real work.
///
/// That inverts the rule the architect actually stated: each pull request
/// carries Formal AI-authored work, "as big as it can be, but as small as it
/// actually can", and a release must still be producible
/// (`docs/architect-notes/2026-09-11-do-not-obstruct-the-vision.md`). A share
/// measured over fewer than [`RATCHET_EVIDENCE_FLOOR`] changed lines is noise,
/// so it is not allowed to *raise* the bar. It is ignored only as a source for
/// the ratchet; it is still recorded, still reported, and still counts in the
/// trailing share the release notes carry.
pub fn target_from_rows(rows: &[ReleaseRow]) -> u64 {
    let Some(newest) = rows
        .iter()
        .rfind(|row| row.metric_version == METRIC_VERSION)
    else {
        return 0;
    };
    // A reviewed override replaces the ratchet outright, and it is read from the
    // newest row alone: a decision is about the level from here on, not about
    // how the level was reached.
    if let Some(override_target) = newest.target_override_basis_points {
        return override_target;
    }
    // The bar is recomputed from the rows that were *entitled* to set it rather
    // than read off the newest row's carried value. `target_percentage_basis_points`
    // is a cache of this same walk, and a bar a degenerate cycle manufactured
    // once would otherwise be carried forward verbatim by every row after it --
    // v0.349.0 is a legitimate 4050-line cycle and still carried the 291 that the
    // one-line v0.348.1 created two releases earlier. Refusing the source while
    // honouring its cached result would fix nothing.
    ratcheted_target(rows)
}

/// The bar implied by every comparable row that was entitled to raise it.
///
/// A reviewed override anywhere in the history replaces everything before it,
/// because that is what a decision about the level means; after it, the ordinary
/// ratchet resumes from the level it set.
fn ratcheted_target(rows: &[ReleaseRow]) -> u64 {
    rows.iter()
        .filter(|row| row.metric_version == METRIC_VERSION)
        .fold(0, |target, row| match row.target_override_basis_points {
            Some(override_target) => override_target,
            None if row.changed_lines < RATCHET_EVIDENCE_FLOOR => target,
            None => target.max(row.trailing_percentage_basis_points),
        })
}

/// How many commits in `since..until` carry validated Formal AI attribution.
///
/// The pull-request reading of the self-development floor: on a `pull_request`
/// event the cycle cannot contain a *merged* Formal AI pull request, because
/// this branch is the pull request and it merges after the check runs. The
/// commits counted here are the ones that become that merged pull request.
pub fn attributed_commits_in_range(repo: &Path, since: &str, until: &str) -> Result<usize, String> {
    Ok(attributed_commits(repo, since, until, super::EvidencePolicy::Lenient)?.len())
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
