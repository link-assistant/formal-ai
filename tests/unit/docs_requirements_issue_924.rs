use std::fs;
use std::path::{Path, PathBuf};

const SESSION_ID: &str = "ses_0020cec63ffe7RIFkQ1qH9YZcY";
const PR_URL: &str = "https://github.com/link-assistant/formal-ai/pull/1007";
const INVARIANT: &str = "Every Formal AI release cycle includes a reviewed pull request whose session-backed workspace effect passes unchanged review, CI, promotion, and a non-decreasing self-hosting target.";

#[test]
fn issue_924_requirements_and_release_contract_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_contains_all(
        "issue 924 root requirements",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #924 Formal AI Self-Development Loop",
            "| R924-1 ",
            "| R924-2 ",
            "| R924-3 ",
            "| R924-4 ",
            "| R924-5 ",
        ],
    );
    assert_contains_all(
        "issue 924 case study",
        &read(root.join("docs/case-studies/issue-924/README.md")),
        &[
            "## Root cause",
            "## Release-cycle contract",
            "## Unchanged gates",
            "## Replayable self-authorship",
            "E69",
            "E74",
            PR_URL,
        ],
    );
    assert_contains_all(
        "issue 924 requirement map",
        &read(root.join("docs/case-studies/issue-924/requirements.md")),
        &["R924-1", "R924-2", "R924-3", "R924-4", "R924-5"],
    );
    assert_contains_all(
        "self-development roadmap",
        &read(root.join("ROADMAP.md")),
        &[
            "self-development loop (delivered by #924)",
            "one merged, session-backed Formal AI pull request per release cycle",
        ],
    );

    let contributing = read(root.join("CONTRIBUTING.md"));
    assert_contains_all(
        "self-authorship contribution protocol",
        &contributing,
        &[
            "Formal-AI-Session: <session-id>",
            "Formal-AI-Evidence: <repo-relative committed evidence path>",
            "Formal-AI-Pull-Request: https://github.com/<owner>/<repo>/pull/<number>",
            "same commit object",
            "must not decrease",
            "git fetch origin --tags",
            "without the latest tag, the check reports a skip",
        ],
    );

    let ledger = read(root.join("data/meta/self-hosting-ledger.lino"));
    assert_contains_all(
        "release ledger schema",
        &ledger,
        &[
            "pull_request_trailer \"Formal-AI-Pull-Request\"",
            "release_cycle_floor \"1\"",
            "target_policy \"non-decreasing\"",
        ],
    );

    let fragment = root.join("changelog.d/20260814_020000_issue_924_self_development_loop.md");
    let release_notes = if fragment.is_file() {
        read(fragment)
    } else {
        read(root.join("CHANGELOG.md"))
    };
    assert_contains_all(
        "issue 924 release metadata",
        &release_notes,
        &["self-development", "pull request", "#924"],
    );
}

#[test]
fn issue_924_agent_cli_leaf_is_exact_and_replayable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-924/self-hosting-authorship");
    assert_eq!(read(evidence.join("release-invariant.txt")), INVARIANT);
    assert_eq!(read(evidence.join("session-id.txt")).trim(), SESSION_ID);
    assert_eq!(
        read(evidence.join("task.txt")).trim(),
        format!("Write release-invariant.txt containing exactly: {INVARIANT}")
    );
    assert_contains_all(
        "raw Agent CLI stream",
        &read(evidence.join("agent-stream.raw.log")),
        &[SESSION_ID, "formal-ai", INVARIANT],
    );
    assert_contains_all(
        "Formal AI server trace",
        &read(evidence.join("formal-ai-server.log")),
        &["formal-ai", INVARIANT],
    );

    let replay = read(root.join("experiments/issue_924_agent_cli.sh"));
    assert_contains_all(
        "issue 924 Agent CLI replay",
        &replay,
        &[
            "FORMAL_AI_AGENT_MODE=1",
            "serve --host 127.0.0.1",
            "--model formalai/formal-ai",
            "--permission-mode auto",
            "--output-format stream-json",
            "scripts/classify-agent-cli-stderr.sh",
            "cmp -s",
        ],
    );
}

#[test]
fn issue_924_release_path_enforces_the_loop_before_versioning() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = read(root.join("scripts/self-development-loop.rs"));
    assert_contains_all(
        "self-development release gate",
        &gate,
        &[
            "Merge pull request #",
            "valid session evidence",
            "self-hosting target would fall",
            "reviewed Formal AI-authored work before cutting the release",
        ],
    );
    assert_contains_all(
        "version release integration",
        &read(root.join("scripts/version-and-commit.rs")),
        &[
            "ensure_self_development_release",
            "Self-development release gate passed",
        ],
    );
}

fn read(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
