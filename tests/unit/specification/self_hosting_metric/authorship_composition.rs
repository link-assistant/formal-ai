//! Which merged pull requests count as Formal AI-authored.
//!
//! The release gate reads authorship off commit trailers, and these tests pin
//! what those trailers are allowed to claim: a pull request counts for the work
//! Formal AI did in it, and only for a pull request its own trailers name.

use std::fs;

use super::{commit, fixture_repo, git, metric_script};

/// A pull request is judged by the work Formal AI did in it, not by what else
/// rode along. The gate once demanded that *every* introduced commit carry the
/// trailers, which measured the composition of a pull request rather than the
/// authorship of the work, and forced every self-authored change into a pull
/// request of its own. The maintainer asked for that requirement to go
/// ([PR #1070][decision]), so a mixed pull request now counts for its
/// attributed part.
///
/// [decision]: https://github.com/link-assistant/formal-ai/pull/1070#issuecomment-5539328163
#[test]
fn release_cycle_counts_the_attributed_part_of_a_mixed_pull_request() {
    let repo = fixture_repo();
    let branch = "issue-42";
    let session = "fixture-session-42";
    let evidence = "docs/evidence/42/session.txt";
    let pull_request = "https://github.com/example/formal-ai/pull/42";

    git(&repo, &["switch", "-c", branch]);
    fs::write(repo.join("human-first.txt"), "unattributed change\n")
        .expect("human fixture must be written");
    commit(&repo, "human-authored part of the pull request");

    fs::create_dir_all(repo.join("docs/evidence/42")).expect("evidence directory must be created");
    fs::write(
        repo.join(evidence),
        format!("formal-ai session {session}\n"),
    )
    .expect("session evidence must be written");
    fs::write(repo.join("formal-ai-42.txt"), "session-backed change\n")
        .expect("generated fixture must be written");
    commit(
        &repo,
        &format!(
            "formal ai change\n\nFormal-AI-Session: {session}\nFormal-AI-Evidence: {evidence}\nFormal-AI-Pull-Request: {pull_request}"
        ),
    );
    git(&repo, &["switch", "main"]);
    git(
        &repo,
        &[
            "merge",
            "--no-ff",
            branch,
            "-m",
            "Merge pull request #42 from example/issue-42",
        ],
    );

    metric_script::ensure_self_development_release(
        &repo,
        &repo.join("data/meta/self-hosting-ledger.lino"),
        "v1.1.0",
        "v1.0.0",
        "HEAD",
        3,
    )
    .expect("a pull request carrying one session-backed commit must satisfy the structural gate");

    // The human commit is still nobody's but the human's: it lands in the
    // denominator and not the numerator, so counting the pull request cannot
    // inflate the share.
    let measurement =
        metric_script::measure(&repo, "v1.0.0", "HEAD").expect("the range must be measurable");
    assert_eq!(measurement.self_authored_commits, 1);
    assert_eq!(measurement.commits, 2);
    assert!(
        measurement.self_authored_lines < measurement.changed_lines,
        "the unattributed commit must stay outside the numerator: {measurement:?}"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// The relaxation stops at the trailers' own claims. An attributed commit that
/// names a *different* pull request is not evidence about the one that
/// introduced it, and it still disqualifies that pull request outright.
#[test]
fn release_cycle_rejects_a_commit_claiming_another_pull_request() {
    let repo = fixture_repo();
    let branch = "issue-43";
    let session = "fixture-session-43";
    let evidence = "docs/evidence/43/session.txt";
    let elsewhere = "https://github.com/example/formal-ai/pull/44";

    git(&repo, &["switch", "-c", branch]);
    fs::create_dir_all(repo.join("docs/evidence/43")).expect("evidence directory must be created");
    fs::write(
        repo.join(evidence),
        format!("formal-ai session {session}\n"),
    )
    .expect("session evidence must be written");
    fs::write(repo.join("formal-ai-43.txt"), "session-backed change\n")
        .expect("generated fixture must be written");
    commit(
        &repo,
        &format!(
            "formal ai change\n\nFormal-AI-Session: {session}\nFormal-AI-Evidence: {evidence}\nFormal-AI-Pull-Request: {elsewhere}"
        ),
    );
    git(&repo, &["switch", "main"]);
    git(
        &repo,
        &[
            "merge",
            "--no-ff",
            branch,
            "-m",
            "Merge pull request #43 from example/issue-43",
        ],
    );

    let error = metric_script::ensure_self_development_release(
        &repo,
        &repo.join("data/meta/self-hosting-ledger.lino"),
        "v1.1.0",
        "v1.0.0",
        "HEAD",
        3,
    )
    .expect_err("a trailer naming another pull request must not credit this one");
    assert!(
        error.contains("merged Formal AI-authored pull request"),
        "unexpected error: {error}"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}
