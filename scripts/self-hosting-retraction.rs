//! Withdrawing an attribution claim that cannot be amended.
//!
//! Split out of `self-hosting-metric.rs` to keep that file under the per-file
//! ceiling `scripts/check-file-size.rs` enforces (issue #1079).

use super::{EvidencePolicy, trailer_values};
use std::path::Path;

const RETRACT_TRAILER: &str = "Formal-AI-Retract";

/// Commits whose attribution a later commit in the same range withdraws.
///
/// Issue #1079. The strict pull-request gate is written on the assumption that
/// "a fall is a hard error while the commits that cause it can still be
/// amended" -- the sentence directly above `EvidencePolicy`. In this repository
/// that assumption does not hold: the `protection` ruleset applies
/// `non_fast_forward` to `~ALL` branches with an empty `bypass_actors` list, so
/// no push can rewrite a commit message once it has left a workstation. A
/// commit that records `Formal-AI-Evidence` and forgets `Formal-AI-Session` is
/// therefore a permanent hard error on its branch, and the only remedy left is
/// to abandon the pull request and open another one from a fresh branch. That
/// is the same deadlock shape as issues #796, #810 and #812, moved off the
/// release path and onto the pull-request gate;
/// `a_malformed_historical_evidence_record_cannot_deadlock_a_release` exists
/// because the release path already had it.
///
/// A retraction is the in-branch remedy, and it is safe because it is
/// one-directional: it can only ever move a commit *out* of the numerator.
/// Nothing about it can raise the measured share, so it cannot be used to
/// inflate the metric -- the failure mode the strict gate protects against. It
/// withdraws a claim; it never makes one.
///
///     Formal-AI-Retract: <40-character commit sha in this range>
///
/// The sha must be full and must name a commit inside the measured range, so a
/// retraction cannot reach past the work under review, and a stale one becomes
/// a hard error under the strict gate rather than silently matching nothing.
///
/// A lenient reader warns about the offending trailer and keeps the rest.
/// Discarding every retraction because one of them went stale would put the
/// commits the others withdraw back into the numerator, which is the one
/// direction this function must never move in; a release range that starts
/// after a retraction's target makes that stale trailer the normal case rather
/// than the exceptional one.
pub fn retracted_commits(
    repo: &Path,
    commits: &[&str],
    policy: EvidencePolicy,
) -> Result<Vec<String>, String> {
    let mut retracted = Vec::new();
    for commit in commits {
        for target in trailer_values(repo, commit, RETRACT_TRAILER)? {
            let rejection = if target.len() != 40 || !target.chars().all(|c| c.is_ascii_hexdigit())
            {
                Some(format!(
                    "{RETRACT_TRAILER} in commit {commit} must name a full 40-character sha, \
                     found {target}"
                ))
            } else if target == *commit {
                Some(format!(
                    "commit {commit} retracts itself; write the trailers correctly instead"
                ))
            } else if !commits.iter().any(|candidate| *candidate == target) {
                Some(format!(
                    "{RETRACT_TRAILER} in commit {commit} names {target}, which is not one of the \
                     commits being measured; a retraction only withdraws a claim made in the \
                     same range"
                ))
            } else {
                None
            };
            match rejection {
                None => retracted.push(target),
                Some(error) => match policy {
                    EvidencePolicy::Strict => return Err(error),
                    EvidencePolicy::Lenient => {
                        eprintln!("warning: ignoring a malformed retraction: {error}");
                    }
                },
            }
        }
    }
    Ok(retracted)
}
