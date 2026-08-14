use std::fs;
use std::path::{Path, PathBuf};

const INVARIANT: &str = "Formal reasoning remains auditable when equality is discharged by bounded e-graph saturation and rule consequences by a bounded Datalog least fixed point.";
const SESSION_ID: &str = "ses_001f733ceffe5UboLW4JATfkoZ";

#[test]
fn issue_923_case_study_and_release_metadata_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_contains_all(
        "REQUIREMENTS.md",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #923 Symbolic-Kernel Coverage Growth",
            "| R923-1 ",
            "| R923-2 ",
            "| R923-3 ",
            "| R923-4 ",
            "| R923-5 ",
            "20/20 egg laws",
            "5/5 Ascent closure assertions",
        ],
    );
    assert_contains_all(
        "ARCHITECTURE.md",
        &read(root.join("ARCHITECTURE.md")),
        &[
            "Issue #923",
            "decision/equality.rs",
            "decision/rules.rs",
            "equality-saturation",
            "least-fixed-point",
        ],
    );
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "Issue #923 Symbolic-Kernel Coverage Growth (PR #1006)",
            "20/20 egg laws",
            "5/5 Ascent assertions",
        ],
    );

    assert_contains_all(
        "issue 923 case study",
        &read(root.join("docs/case-studies/issue-923/README.md")),
        &[
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Reproduction And Root Cause",
            "## 4. Implemented Design",
            "## 5. Honest External Scores",
            "## 6. Verification",
            "## 7. Self-Hosting Evidence",
            "20 | 20",
            "5 | 5",
            SESSION_ID,
        ],
    );
    assert_contains_all(
        "issue 923 requirements",
        &read(root.join("docs/case-studies/issue-923/requirements.md")),
        &["R923-1", "R923-5", "one-of-five", SESSION_ID],
    );
    assert_contains_all(
        "issue 923 solution plan",
        &read(root.join("docs/case-studies/issue-923/solution-plan.md")),
        &[
            "Bounded Equality Saturation",
            "Bounded Rule Inference",
            "Pinned Sources And Honest Scores",
            "Regression, Traceability, And Self-Hosting",
        ],
    );
    assert_contains_all(
        "issue 923 online research",
        &read(root.join("docs/case-studies/issue-923/raw-data/online-research.md")),
        &[
            "https://docs.rs/egg/0.11.0/egg/struct.Runner.html",
            "2f31b28e3f9d78e02273b6c6d4201b5b0720b343",
            "cf5e9a87525bb95268cf6680a59882264b0fe0de",
            "Repository Prior Art",
        ],
    );
    assert_contains_all(
        "PR 1006 case study",
        &read(root.join("docs/case-studies/pull-request-1006/README.md")),
        &["Issue #923", "Review Scope", "No screenshots"],
    );

    for relative in [
        "docs/case-studies/issue-923/raw-data/github/issue-923.json",
        "docs/case-studies/issue-923/raw-data/github/issue-923-comments.json",
        "docs/case-studies/issue-923/raw-data/github/pr-1006.json",
        "docs/case-studies/issue-923/raw-data/github/pr-1006-conversation-comments.json",
        "docs/case-studies/issue-923/raw-data/github/pr-1006-review-comments.json",
        "docs/case-studies/issue-923/raw-data/github/pr-1006-reviews.json",
        "docs/case-studies/pull-request-1006/raw-data/github/issue-923.json",
        "docs/case-studies/pull-request-1006/raw-data/github/issue-923-comments.json",
        "docs/case-studies/pull-request-1006/raw-data/github/pr-1006.json",
        "docs/case-studies/pull-request-1006/raw-data/github/pr-1006-conversation-comments.json",
        "docs/case-studies/pull-request-1006/raw-data/github/pr-1006-review-comments.json",
        "docs/case-studies/pull-request-1006/raw-data/github/pr-1006-reviews.json",
    ] {
        let path = root.join(relative);
        let contents = read(&path);
        serde_json::from_str::<serde_json::Value>(&contents)
            .unwrap_or_else(|error| panic!("{relative} should be valid JSON: {error}"));
    }

    let fragment = root.join("changelog.d/20260814_021500_issue_923_symbolic_kernel.md");
    let release_notes = if fragment.is_file() {
        read(fragment)
    } else {
        read(root.join("CHANGELOG.md"))
    };
    assert_contains_all(
        "issue 923 release metadata",
        &release_notes,
        &["equality", "Datalog", "#923"],
    );
}

#[test]
fn issue_923_agent_cli_authorship_leaf_is_byte_exact_and_reproducible() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-923/agent-cli-evidence");
    assert_eq!(
        read(evidence.join("symbolic-kernel-invariant.md")),
        INVARIANT
    );
    assert_eq!(read(evidence.join("session-id.txt")).trim(), SESSION_ID);
    assert_eq!(
        read(evidence.join("task.txt")).trim(),
        format!("Create file symbolic-kernel-invariant.md containing {INVARIANT}")
    );

    let stream = read(evidence.join("agent-stream.raw.log"));
    assert_contains_all("Agent CLI raw stream", &stream, &[SESSION_ID, "formal-ai"]);
    for file in [
        "agent-stderr.log",
        "agent-stream.jsonl",
        "formal-ai-server.log",
        "worktree-status.txt",
    ] {
        assert!(
            evidence.join(file).is_file(),
            "missing Agent evidence: {file}"
        );
    }

    let script = read(root.join("experiments/issue_923_agent_cli.sh"));
    assert_contains_all(
        "issue 923 Agent CLI replay",
        &script,
        &[
            "serve --host 127.0.0.1",
            "--output-format stream-json",
            "symbolic-kernel-invariant.md",
            INVARIANT,
            "cmp -s",
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
