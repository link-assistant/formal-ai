#[allow(clippy::duplicate_mod)]
#[path = "../../../scripts/self-hosting-metric.rs"]
mod metric_script;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use links_notation::parse_lino;

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git must run in the metric fixture");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output must be UTF-8")
        .trim()
        .to_owned()
}

fn commit(repo: &Path, message: &str) {
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", message]);
}

fn fixture_repo() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    let repo = std::env::temp_dir().join(format!(
        "formal-ai-self-hosting-metric-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(repo.join("data/meta")).expect("fixture directory must be created");
    git(&repo, &["init", "--quiet"]);
    git(&repo, &["config", "user.name", "Metric Fixture"]);
    git(&repo, &["config", "user.email", "metric@example.invalid"]);
    fs::write(
        repo.join("data/meta/self-hosting-ledger.lino"),
        "self_hosting_ledger\n  attribution_policy \"commit_trailers\"\n  session_trailer \
         \"Formal-AI-Session\"\n  evidence_trailer \"Formal-AI-Evidence\"\n  line_unit \
         \"added_and_deleted\"\n  default_trailing_window \"3\"\n",
    )
    .expect("fixture ledger must be written");
    fs::write(repo.join("code.txt"), "base\n").expect("base file must be written");
    commit(&repo, "fixture baseline");
    git(&repo, &["tag", "v1.0.0"]);
    repo
}

#[test]
fn recorded_formal_ai_evidence_drives_the_release_metric_and_ratchet() {
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
    fs::write(repo.join("human-code.txt"), "human change\n").expect("human code must be written");
    commit(&repo, "human change");

    let measurement =
        metric_script::measure(&repo, "v1.0.0", "HEAD").expect("fixture measurement must succeed");
    assert_eq!(measurement.self_authored_lines, 3);
    assert_eq!(measurement.changed_lines, 4);
    assert_eq!(measurement.self_authored_commits, 1);
    assert_eq!(measurement.commits, 2);
    assert_eq!(measurement.percentage_basis_points, 7_500);
    assert_eq!(
        metric_script::format_percentage(measurement.percentage_basis_points),
        "75.00%"
    );

    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    let row = metric_script::record_release(&repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3)
        .expect("the first release row must be recorded");
    assert_eq!(row.percentage_basis_points, 7_500);
    assert_eq!(row.trailing_percentage_basis_points, 7_500);
    assert_eq!(
        metric_script::record_release(&repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3,)
            .expect("recording the identical row must be idempotent"),
        row
    );
    assert_eq!(
        metric_script::release_note_for_tag(&ledger, "v1.1.0")
            .expect("release note must be rendered"),
        "## Self-hosting\n\nFormal AI authored **75.00%** of this release \
         (3 of 4 changed lines). The 3-release trailing share is **75.00%**."
    );
    let ledger_text = fs::read_to_string(&ledger).expect("fixture ledger must be readable");
    parse_lino(ledger_text.trim()).expect("recorded ledger must be canonical Links Notation");

    fs::write(repo.join("second-human-change.txt"), "regression\n")
        .expect("second human change must be written");
    commit(&repo, "second human change");
    let error = metric_script::record_release(&repo, &ledger, "v1.2.0", "HEAD^", "HEAD", 3)
        .expect_err("a falling trailing share must be rejected");
    assert!(error.contains("ratchet"), "unexpected error: {error}");

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// Regression for issue #796: release commit 59650f2b separated its two
/// trailers with a blank line. Git only treats the last paragraph of a message
/// as the trailer block, so `%(trailers:key=Formal-AI-Session)` returned
/// nothing, the metric reported "must record both", and Auto Release failed.
#[test]
fn trailers_are_recognized_even_when_separated_by_blank_lines() {
    let repo = fixture_repo();
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    fs::write(
        repo.join("docs/evidence/session.txt"),
        "formal-ai session spaced-session\n",
    )
    .expect("session evidence must be written");
    fs::write(repo.join("formal-ai-code.txt"), "generated\n").expect("code must be written");
    commit(
        &repo,
        "formal ai change\n\nFormal-AI-Session: spaced-session\n\nFormal-AI-Evidence: \
         docs/evidence/session.txt",
    );

    let measurement =
        metric_script::measure(&repo, "v1.0.0", "HEAD").expect("measurement must succeed");
    assert_eq!(
        measurement.self_authored_commits, 1,
        "a blank line between trailers must not hide the session trailer"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// Issue #810: `Auto Release` (run 29737218421) died with "no committed
/// Formal-AI-Evidence in 10e65ae2 records session issue-804-claude-20260720".
/// The pull-request evidence gate only landed in 39fdef91, *after* that commit
/// merged, so the malformed record sat permanently inside every
/// `<last tag>..HEAD` release range: the release could not run, so no tag could
/// be cut, so the range could never move past it. Recording a release must
/// therefore degrade to "not self-authored" instead of aborting, while the
/// pull-request gate stays strict.
#[test]
fn a_malformed_historical_evidence_record_cannot_deadlock_a_release() {
    let repo = fixture_repo();
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    // Mentions formal-ai, but never names the session it claims to document --
    // exactly the shape of commit 10e65ae2.
    fs::write(
        repo.join("docs/evidence/analysis.md"),
        "formal-ai analysis without the session id\n",
    )
    .expect("evidence must be written");
    fs::write(repo.join("code.txt"), "base\nchanged\n").expect("code must be written");
    commit(
        &repo,
        "formal ai change\n\nFormal-AI-Session: orphan-session\nFormal-AI-Evidence: \
         docs/evidence/analysis.md",
    );

    let strict = metric_script::measure(&repo, "v1.0.0", "HEAD");
    assert!(
        strict.is_err(),
        "the pull-request gate must still reject a malformed evidence record"
    );

    let row = metric_script::record_release(
        &repo,
        &repo.join("data/meta/self-hosting-ledger.lino"),
        "v1.1.0",
        "v1.0.0",
        "HEAD",
        3,
    )
    .expect("a release must not be blocked by an immutable malformed commit");
    assert_eq!(
        row.self_authored_commits, 0,
        "an unverifiable commit must not be counted as self-authored"
    );
    assert_eq!(
        row.commits, 1,
        "the commit must still count toward total changed work"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// Issue #812: `Auto Release` (run 29751001867, log line 23918) died with
/// "self-hosting ratchet would fall from 32.68% to 18.24% for v0.301.0". In the
/// measured `v0.300.0..HEAD` range, 601 765 of 636 240 changed lines -- 94.58%
/// -- were the captured CI logs under `dev/log/issues/<n>/pulls/<n>/` that this
/// repository requires every iteration to commit. The published share therefore
/// described log volume, not authored work, and moved in whichever direction the
/// attached transcripts happened to land. Captured output must not count, on
/// either side of the ratio.
#[test]
fn captured_artifacts_and_lockfiles_do_not_move_the_metric() {
    for captured in [
        "dev/log/issues/812/pulls/813/ci-logs/run-29751001867.log",
        "dev/log/issues/812/pulls/813/session.jsonl",
        "dev/log/issues/812/pulls/813/change.diff",
        "dev/log/issues/812/pulls/813/build.STDERR",
        "Cargo.lock",
        "desktop/bun.lock",
        "\"dev/log/issues/812/nasty\\346\\227\\245.log\"",
    ] {
        assert!(
            metric_script::is_non_authored_path(captured),
            "{captured} is captured output, not authored work"
        );
    }
    for authored in [
        "scripts/self-hosting-metric.rs",
        "dev/log/issues/812/pulls/813/analysis.md",
        ".github/workflows/release.yml",
        "logger.rs",
    ] {
        assert!(
            !metric_script::is_non_authored_path(authored),
            "{authored} is authored work"
        );
    }

    let repo = fixture_repo();
    fs::create_dir_all(repo.join("dev/log")).expect("log directory must be created");
    let bulk = "captured line\n".repeat(5_000);
    fs::write(repo.join("dev/log/run.log"), &bulk).expect("captured log must be written");
    fs::write(repo.join("human-code.txt"), "one authored line\n").expect("code must be written");
    commit(&repo, "human change carrying a captured log");

    let measurement =
        metric_script::measure(&repo, "v1.0.0", "HEAD").expect("measurement must succeed");
    assert_eq!(
        measurement.changed_lines, 1,
        "5000 lines of captured log must not enter the denominator"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// The exclusion above redefines what the ratio measures, so rows written before
/// it are not comparable with rows written after it. The ratchet and the
/// trailing window must therefore only ever look at rows of the same
/// `metric_version`; averaging two definitions would silently compare two
/// different quantities and re-create the #812 outage from the other direction.
#[test]
fn rows_from_an_older_measurement_epoch_are_never_compared() {
    let repo = fixture_repo();
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    // A row in the shape written before issue #812: no `metric_version` field,
    // and a share no epoch-2 row could plausibly reach.
    let mut legacy = fs::read_to_string(&ledger).expect("fixture ledger must be readable");
    legacy.push_str(
        "  release\n    tag \"v0.9.0\"\n    since \"v0.8.0\"\n    until \"v0.9.0\"\n    \
         self_authored_lines \"9900\"\n    changed_lines \"10000\"\n    self_authored_commits \
         \"9\"\n    commits \"10\"\n    percentage_basis_points \"9900\"\n    trailing_window \
         \"3\"\n    trailing_percentage_basis_points \"9900\"\n",
    );
    fs::write(&ledger, legacy).expect("legacy ledger must be written");
    assert_eq!(
        metric_script::read_release_rows(&ledger).expect("ledger must parse")[0].metric_version,
        1,
        "a row without the field predates issue #812 and is, by definition, epoch 1"
    );

    fs::write(repo.join("human-code.txt"), "human change\n").expect("code must be written");
    commit(&repo, "human change");
    let row = metric_script::record_release(&repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3)
        .expect("a 99% epoch-1 row must not ratchet against an epoch-2 measurement");
    assert_eq!(row.metric_version, 2);
    assert_eq!(
        row.trailing_percentage_basis_points, 0,
        "the trailing window must average epoch-2 rows only"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// Issue #812, second root cause: the ratchet ran on `main`, after every
/// contributing commit was immutable, and the only way to move the measured
/// range forward -- cutting a tag -- was exactly what it blocked. Third outage of
/// that shape in a row (#796, #810, #812). Recording a release must report the
/// fall and ship; enforcement belongs to the pull-request gate.
#[test]
fn a_falling_ratchet_reports_at_release_time_instead_of_blocking_it() {
    let repo = fixture_repo();
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    fs::write(
        repo.join("docs/evidence/session.txt"),
        "formal-ai session fixture-session\n",
    )
    .expect("session evidence must be written");
    commit(
        &repo,
        "formal ai change\n\nFormal-AI-Session: fixture-session\nFormal-AI-Evidence: \
         docs/evidence/session.txt",
    );
    let first = metric_script::record_release(&repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3)
        .expect("the first release row must be recorded");
    assert_eq!(first.trailing_percentage_basis_points, 10_000);
    git(&repo, &["tag", "v1.1.0"]);

    fs::write(repo.join("human-code.txt"), "unattributed\n").expect("code must be written");
    commit(&repo, "human change");

    assert!(
        metric_script::record_release(&repo, &ledger, "v1.2.0", "v1.1.0", "HEAD", 3)
            .expect_err("the enforcing policy must still reject a fall")
            .contains("ratchet"),
    );
    let reported = metric_script::record_release_with_policy(
        &repo,
        &ledger,
        "v1.2.0",
        "v1.1.0",
        "HEAD",
        3,
        metric_script::RatchetPolicy::Report,
    )
    .expect("a release must not be blocked by a fall it cannot fix");
    assert!(reported.trailing_percentage_basis_points < first.trailing_percentage_basis_points);
    assert_eq!(
        metric_script::read_release_rows(&ledger)
            .expect("ledger must parse")
            .last()
            .map(|row| row.tag.clone()),
        Some("v1.2.0".to_owned()),
        "the regression must be recorded honestly, not hidden behind a failing job"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

/// The pull-request gate is where a fall is still actionable. It must be
/// *differential* -- this branch against its base -- so that a regression already
/// merged into `main` cannot fail every unrelated pull request, which would just
/// move the #812 deadlock to a new job.
#[test]
fn the_pull_request_gate_only_judges_the_branchs_own_delta() {
    let repo = fixture_repo();
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    fs::create_dir_all(repo.join("docs/evidence")).expect("evidence directory must be created");
    fs::write(
        repo.join("docs/evidence/session.txt"),
        "formal-ai session fixture-session\n",
    )
    .expect("session evidence must be written");
    commit(
        &repo,
        "formal ai change\n\nFormal-AI-Session: fixture-session\nFormal-AI-Evidence: \
         docs/evidence/session.txt",
    );
    metric_script::record_release(&repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3)
        .expect("baseline release row must be recorded");
    git(&repo, &["tag", "v1.1.0"]);

    // A branch of attributed work: the projected share cannot fall.
    fs::write(repo.join("formal-ai-code.txt"), "generated\n").expect("code must be written");
    commit(
        &repo,
        "formal ai follow-up\n\nFormal-AI-Session: fixture-session\nFormal-AI-Evidence: \
         docs/evidence/session.txt",
    );
    assert_eq!(
        metric_script::ratchet_check(&repo, &ledger, "v1.1.0", "HEAD", 3)
            .expect("the ratchet check must run"),
        None,
        "attributed work must never trip the gate"
    );

    // A branch of unattributed work lowers the projection and is rejected while
    // its commits can still be amended.
    fs::write(repo.join("human-code.txt"), "unattributed\n").expect("code must be written");
    commit(&repo, "human change");
    let regression = metric_script::ratchet_check(&repo, &ledger, "HEAD^", "HEAD", 3)
        .expect("the ratchet check must run")
        .expect("unattributed work must be reported against its own base");
    assert!(
        regression.contains("Formal-AI-Session"),
        "the message must say how to fix it: {regression}"
    );

    // An empty branch is always neutral, whatever main's own trend is.
    assert_eq!(
        metric_script::ratchet_check(&repo, &ledger, "HEAD", "HEAD", 3)
            .expect("the ratchet check must run"),
        None,
        "a no-op branch must never be blocked by a pre-existing regression"
    );

    fs::remove_dir_all(repo).expect("fixture directory must be removed");
}

#[test]
fn release_pipeline_and_ledger_remain_pinned_to_the_metric() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow = fs::read_to_string(root.join(".github/workflows/release.yml"))
        .expect("release workflow must be readable");
    let version_script = fs::read_to_string(root.join("scripts/version-and-commit.rs"))
        .expect("version script must be readable");
    let release_script = fs::read_to_string(root.join("scripts/create-github-release.rs"))
        .expect("release script must be readable");
    let ledger = fs::read_to_string(root.join("data/meta/self-hosting-ledger.lino"))
        .expect("self-hosting ledger must be readable");

    assert!(workflow.contains("--self-hosting-ledger"));
    assert!(
        workflow.contains("--check-ratchet"),
        "issue #812: the ratchet must be enforced at the pull-request gate"
    );
    assert!(version_script.contains("self-hosting-metric.rs"));
    assert!(
        version_script.contains("RatchetPolicy::Report"),
        "issue #812: a release must never be blocked by immutable history"
    );
    assert!(
        ledger.contains("current_metric_version \"2\""),
        "issue #812: the ledger must name the definition its newest rows use"
    );
    assert!(release_script.contains("## Self-hosting"));
    assert!(ledger.contains("tag \"v0.296.0\""));
    assert!(ledger.contains("percentage_basis_points \"0\""));
}
