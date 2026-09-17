//! Issue #1138 B7 (plan 03, L12–L13): `solve` refuses to commit by default.
//!
//! Mutation is opt-in. Without `--commit` the tree is untouched and the diff is
//! printed; with it the commit carries the four self-hosting trailers
//! `scripts/self-hosting-metric.rs` already reads, and the evidence bundle names
//! the model verbatim. Only `formal-ai` is an authoring path — a hosted model
//! may not claim the attribution.

use std::path::{Path, PathBuf};
use std::process::Command;

use formal_ai::cli_solve::{
    SolveArgs, github_issue_api_url, parse_github_issue_requirement, run_solve,
};

const TRAILERS: &[&str] = &[
    "Formal-AI-Session:",
    "Formal-AI-Model:",
    "Formal-AI-Evidence:",
    "Formal-AI-Pull-Request:",
];

const REQUIREMENT: &str =
    "Add \"wikiquote\" to the list of trusted search providers, and keep the file valid.";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn head_commit() -> String {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root().display().to_string(),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("git rev-parse should run");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn args(tag: &str) -> SolveArgs {
    let evidence = std::env::temp_dir().join(format!("formal-ai-issue-1138-solve-{tag}"));
    let _ = std::fs::remove_dir_all(&evidence);
    SolveArgs {
        issue: None,
        task: Some(String::from(REQUIREMENT)),
        repository: repo_root().display().to_string(),
        base_commit: Some(head_commit()),
        model: String::from("formal-ai"),
        evidence,
        pull_request: Some(String::from(
            "https://github.com/link-assistant/formal-ai/pull/1138",
        )),
        commit: false,
    }
}

/// Default-deny for mutation: the diff is printed, nothing is committed, and the
/// working tree is exactly as it was.
#[test]
fn solve_refuses_to_commit_by_default() {
    let before = Command::new("git")
        .args([
            "-C",
            &repo_root().display().to_string(),
            "status",
            "--porcelain",
        ])
        .output()
        .expect("git status should run");

    let outcome = run_solve(&args("default")).expect("a solve run reports whatever it observed");
    assert!(
        !outcome.committed,
        "without --commit nothing may be committed"
    );
    assert!(
        outcome.commit_message.is_empty(),
        "no commit message is written when nothing is committed"
    );

    let after = Command::new("git")
        .args([
            "-C",
            &repo_root().display().to_string(),
            "status",
            "--porcelain",
        ])
        .output()
        .expect("git status should run");
    assert_eq!(
        String::from_utf8_lossy(&before.stdout),
        String::from_utf8_lossy(&after.stdout),
        "the ambient working tree must be unchanged by a default solve run"
    );
}

/// The commit is attributable: four trailers, and an evidence bundle that names
/// the model so the self-hosting metric can read it back.
#[test]
fn solve_writes_all_four_trailers() {
    let mut args = args("trailers");
    args.commit = true;
    let outcome = run_solve(&args).expect("a committing solve run reports its commit");

    for trailer in TRAILERS {
        assert!(
            outcome.commit_message.contains(trailer),
            "the commit message must carry {trailer}:\n{}",
            outcome.commit_message
        );
    }
    assert!(
        !outcome.evidence_files.is_empty(),
        "the raw protocol trace is committed under --evidence"
    );
    assert!(
        outcome.committed,
        "an observed source edit is committed in the isolated clone"
    );
    assert!(
        outcome.diff.contains(".formal-ai/evidence/"),
        "the attributable evidence must be part of the same diff as the source edit: {}",
        outcome.diff
    );
    let bundle: String = outcome
        .evidence_files
        .iter()
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .collect();
    assert!(
        bundle.contains("formal-ai/"),
        "the evidence bundle names the model verbatim so the metric can read it back"
    );
}

/// GitHub's mutable HTML is not scraped. A canonical issue URL is projected to
/// the documented API resource, whose exact captured bytes become evidence.
#[test]
fn canonical_issue_url_and_payload_are_retrievable_inputs() {
    let issue = "https://github.com/link-assistant/formal-ai/issues/1138";
    assert_eq!(
        github_issue_api_url(issue).as_deref(),
        Some("https://api.github.com/repos/link-assistant/formal-ai/issues/1138")
    );
    assert!(github_issue_api_url("https://example.com/issues/1138").is_none());
    assert!(github_issue_api_url("https://github.com/a/b/pull/1").is_none());

    let payload = br#"{"title":"Add a provider","body":"Include the missing source.","html_url":"https://github.com/link-assistant/formal-ai/issues/1138"}"#;
    assert_eq!(
        parse_github_issue_requirement(issue, payload).expect("the captured payload should parse"),
        "Add a provider\n\nInclude the missing source."
    );
    let pull = br#"{"title":"not an issue","body":"","pull_request":{}}"#;
    assert!(
        parse_github_issue_requirement(issue, pull).is_err(),
        "an issues API response describing a pull request is refused"
    );
}

/// A hosted model did not author anything here, so it may not be recorded as the
/// author.
#[test]
fn solve_with_a_hosted_model_is_not_an_authoring_path() {
    let mut args = args("hosted");
    args.commit = true;
    args.model = String::from("claude-opus-4");

    match run_solve(&args) {
        Err(error) => assert!(
            !error.to_string().trim().is_empty(),
            "the refusal must say which model was refused"
        ),
        Ok(outcome) => {
            assert!(
                !outcome.commit_message.contains("Formal-AI-Model:"),
                "a hosted model may never be written into the model trailer:\n{}",
                outcome.commit_message
            );
            assert!(
                !outcome.committed,
                "a hosted model is not an authoring path, so nothing is landed under it"
            );
        }
    }
}
