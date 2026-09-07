//! Withdrawing an attribution claim from a history that cannot be rewritten.
//!
//! This repository's ruleset forbids non-fast-forward pushes on every branch
//! with no bypass actors, so a mis-trailered commit message is permanent and
//! the strict gate it fails has no in-branch remedy. `Formal-AI-Retract` is
//! that remedy, and these tests pin the property that makes it admissible: it
//! only ever moves a commit *out* of the numerator (issue #1079).

use std::fs;

use super::{commit, fixture_repo, git, merge_formal_ai_pull_request, metric_script};

/// Issue #1079: the commit that introduced `Formal-AI-Retract` showed the
/// trailer's format in an indented example in its own message, and the gate
/// read the example as a declaration -- `must name a full 40-character sha,
/// found <full 40-character sha>`. `git interpret-trailers --parse` ignores an
/// indented line; so must this parser, or documenting a trailer becomes
/// indistinguishable from using one.
#[test]
fn an_indented_example_of_a_trailer_is_not_a_trailer() {
    let repo = fixture_repo();
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    fs::write(
        repo.join("docs/evidence/session.txt"),
        "formal-ai session documented-session\n",
    )
    .expect("session evidence must be written");
    fs::write(repo.join("formal-ai-code.txt"), "generated\n").expect("code must be written");
    commit(
        &repo,
        "document the retraction trailer\n\nWithdraw a claim with:\n\n             Formal-AI-Retract: <full 40-character sha>\n\nFormal-AI-Session:          documented-session\nFormal-AI-Evidence: docs/evidence/session.txt",
    );

    let measurement = metric_script::measure(&repo, "v1.0.0", "HEAD")
        .expect("an indented example must not be parsed as a retraction");
    assert_eq!(
        measurement.self_authored_commits, 1,
        "the commit's own real trailers must still be read"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// Issue #1079. The strict pull-request gate rejects a commit that records
/// `Formal-AI-Evidence` without `Formal-AI-Session`, and the remedy the design
/// assumes -- amend the commit -- does not exist in this repository: the
/// `protection` ruleset applies `non_fast_forward` to `~ALL` branches with an
/// empty `bypass_actors` list, so a pushed commit message is immutable. Without
/// a retraction the branch is permanently red and the only way out is to
/// abandon the pull request, which is the deadlock
/// `a_malformed_historical_evidence_record_cannot_deadlock_a_release` already
/// removed from the release path.
#[test]
fn a_retraction_unblocks_a_branch_whose_history_cannot_be_rewritten() {
    let repo = fixture_repo();
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    fs::write(
        repo.join("docs/evidence/sweep.md"),
        "formal-ai sweep without the session id\n",
    )
    .expect("evidence must be written");
    fs::write(repo.join("code.txt"), "base\nchanged\n").expect("code must be written");
    commit(
        &repo,
        "half-attributed change\n\nFormal-AI-Evidence: docs/evidence/sweep.md",
    );
    let broken = git(&repo, &["rev-parse", "HEAD"]);

    let error = metric_script::measure(&repo, "v1.0.0", "HEAD")
        .expect_err("a commit recording evidence without a session must fail the gate");
    assert!(error.contains("must record both"), "unexpected: {error}");

    fs::write(repo.join("later.txt"), "later work\n").expect("later work must be written");
    commit(
        &repo,
        &format!("withdraw the incomplete attribution\n\nFormal-AI-Retract: {broken}"),
    );

    let measurement = metric_script::measure(&repo, "v1.0.0", "HEAD")
        .expect("a retracted commit must not deadlock the branch");
    assert_eq!(
        measurement.self_authored_commits, 0,
        "a retracted commit is never self-authored"
    );
    assert_eq!(
        measurement.commits, 2,
        "retraction withdraws the claim, it does not hide the work"
    );
    assert_eq!(
        measurement.percentage_basis_points, 0,
        "nothing in this range is attributable"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// The safety property that makes a retraction admissible at all: it is
/// one-directional. It can only move a commit out of the numerator, so it can
/// never raise the measured share, which is the failure the strict gate exists
/// to prevent (issue #1079).
#[test]
fn a_retraction_can_only_lower_the_measured_share() {
    let repo = fixture_repo();
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    fs::write(
        repo.join("docs/evidence/session.txt"),
        "formal-ai session fixture-session\n",
    )
    .expect("session evidence must be written");
    fs::write(
        repo.join("formal-ai-code.txt"),
        "generated one\ngenerated two\n",
    )
    .expect("generated code must be written");
    commit(
        &repo,
        "formal ai change\n\nFormal-AI-Session: fixture-session\nFormal-AI-Evidence: \
         docs/evidence/session.txt",
    );
    let attributed = git(&repo, &["rev-parse", "HEAD"]);

    let before = metric_script::measure(&repo, "v1.0.0", "HEAD").expect("the fixture must measure");
    assert_eq!(before.self_authored_commits, 1);
    assert_eq!(before.percentage_basis_points, 10_000);

    fs::write(repo.join("later.txt"), "later work\n").expect("later work must be written");
    commit(
        &repo,
        &format!("withdraw a valid claim\n\nFormal-AI-Retract: {attributed}"),
    );

    let after = metric_script::measure(&repo, "v1.0.0", "HEAD")
        .expect("retracting a valid claim must still measure");
    assert_eq!(
        after.self_authored_commits, 0,
        "a retraction withdraws even a well-formed attribution"
    );
    assert!(
        after.percentage_basis_points <= before.percentage_basis_points,
        "a retraction must never raise the share: {} -> {}",
        before.percentage_basis_points,
        after.percentage_basis_points
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// A retraction withdraws the claim on both walks, not only on the metric.
/// `merged_self_authored_pull_requests` decides the release floor from the same
/// `commit_has_formal_ai_evidence` answer, so honouring retractions in one
/// place and not the other would leave a withdrawn claim standing exactly where
/// it is worth the most (issue #1079).
#[test]
fn a_retraction_also_withdraws_the_commit_from_the_release_floor() {
    let repo = fixture_repo();
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    let pull_request = merge_formal_ai_pull_request(&repo, 41);

    let eligibility = metric_script::ensure_self_development_release(
        &repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3,
    )
    .expect("the session-backed merged pull request must make the cycle releasable");
    assert_eq!(eligibility.pull_requests, vec![pull_request]);

    let attributed = git(&repo, &["rev-parse", "issue-41"]);
    fs::write(repo.join("later.txt"), "later work\n").expect("later work must be written");
    commit(
        &repo,
        &format!("withdraw the claim\n\nFormal-AI-Retract: {attributed}"),
    );

    let status = metric_script::self_development_release_status(
        &repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3,
    )
    .expect("policy ineligibility must not be confused with an operational error");
    assert!(
        matches!(
            status,
            metric_script::SelfDevelopmentReleaseStatus::Blocked(ref reason)
                if reason.contains("merged Formal AI-authored pull request")
        ),
        "a retracted commit must not keep backing a release cycle"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// A retraction reaches exactly as far as the work under review. Anything else
/// -- a short sha, a typo, a commit from before the range -- is a hard error,
/// so a retraction that has stopped matching becomes visible instead of
/// silently doing nothing (issue #1079).
#[test]
fn a_retraction_must_name_a_full_sha_inside_the_measured_range() {
    let repo = fixture_repo();
    let baseline = git(&repo, &["rev-parse", "v1.0.0"]);

    fs::write(repo.join("later.txt"), "later work\n").expect("later work must be written");
    commit(&repo, "retract a short sha\n\nFormal-AI-Retract: abc1234");
    let error = metric_script::measure(&repo, "v1.0.0", "HEAD")
        .expect_err("a short sha must not be accepted");
    assert!(
        error.contains("full 40-character sha"),
        "unexpected: {error}"
    );

    git(&repo, &["reset", "--hard", "v1.0.0"]);
    fs::write(repo.join("later.txt"), "later work\n").expect("later work must be written");
    commit(
        &repo,
        &format!("retract outside the range\n\nFormal-AI-Retract: {baseline}"),
    );
    let error = metric_script::measure(&repo, "v1.0.0", "HEAD")
        .expect_err("a commit outside the range must not be retractable");
    assert!(
        error.contains("not one of the commits being measured"),
        "unexpected: {error}"
    );

    // Lenient recording must not inherit the deadlock the strict gate rejects:
    // a malformed retraction in immutable history is reported, not fatal.
    let row = metric_script::record_release(
        &repo,
        &repo.join("data/meta/self-hosting-ledger.lino"),
        "v1.1.0",
        "v1.0.0",
        "HEAD",
        3,
    )
    .expect("a release must not be blocked by a malformed retraction");
    assert_eq!(row.self_authored_commits, 0);

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// A stale retraction is ordinary in immutable history: once a release range
/// begins after the commit a retraction names, that trailer stops matching for
/// good. The lenient reader must drop only the trailer it cannot resolve. If it
/// dropped the whole set, the claims the *other* retractions withdraw would
/// return to the numerator -- the one direction a retraction must never move in
/// (issue #1079).
#[test]
fn a_stale_retraction_does_not_restore_the_claims_the_others_withdraw() {
    let repo = fixture_repo();
    let outside_the_range = git(&repo, &["rev-parse", "v1.0.0"]);
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    fs::write(
        repo.join("docs/evidence/session.txt"),
        "formal-ai session fixture-session\n",
    )
    .expect("session evidence must be written");
    fs::write(
        repo.join("formal-ai-code.txt"),
        "generated one\ngenerated two\n",
    )
    .expect("generated code must be written");
    commit(
        &repo,
        "formal ai change\n\nFormal-AI-Session: fixture-session\nFormal-AI-Evidence: \
         docs/evidence/session.txt",
    );
    let attributed = git(&repo, &["rev-parse", "HEAD"]);

    fs::write(repo.join("later.txt"), "later work\n").expect("later work must be written");
    commit(
        &repo,
        &format!(
            "withdraw one claim and one that has gone stale\n\nFormal-AI-Retract: \
             {attributed}\nFormal-AI-Retract: {outside_the_range}"
        ),
    );

    let row = metric_script::record_release(
        &repo,
        &repo.join("data/meta/self-hosting-ledger.lino"),
        "v1.1.0",
        "v1.0.0",
        "HEAD",
        3,
    )
    .expect("a stale retraction must not block a release");
    assert_eq!(
        row.self_authored_commits, 0,
        "the resolvable retraction must survive the unresolvable one"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}
