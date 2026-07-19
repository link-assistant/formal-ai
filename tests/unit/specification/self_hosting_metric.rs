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
    assert!(version_script.contains("self-hosting-metric.rs"));
    assert!(release_script.contains("## Self-hosting"));
    assert!(ledger.contains("tag \"v0.296.0\""));
    assert!(ledger.contains("percentage_basis_points \"0\""));
}
