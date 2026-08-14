use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn issue_922_case_study_and_release_metadata_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_contains_all(
        "REQUIREMENTS.md",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #922 Method Learning From Experience",
            "| R922-1 ",
            "| R922-2 ",
            "| R922-3 ",
            "| R922-4 ",
            "| R922-5 ",
            "| R922-6 ",
            "promotion_run_21bc44690947f221",
        ],
    );
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "Issue #922 Method Learning From Experience (PR #1005)",
            "held-out-validated",
            "data/seed/learned-methods.lino",
            "docs/case-studies/issue-922/",
        ],
    );
    assert_contains_all(
        "meta-algorithm design",
        &read(root.join("docs/meta-algorithm.md")),
        &[
            "Learning reusable methods from experience (issue #922)",
            "Normalize stable control flow",
            "Remain proposal-only",
            "Load adopted link data",
            "examples/issue-922-method-learning/run.sh",
        ],
    );

    assert_contains_all(
        "issue 922 case study",
        &read(root.join("docs/case-studies/issue-922/README.md")),
        &[
            "## 1. Collected Data",
            "## 2. Reproduction And Root Cause",
            "## 3. Implemented Lifecycle",
            "## 4. Real Proposal And Promotion Evidence",
            "## 5. Verification",
            "promotion_agent_session_13a49fd18a6f7f54",
            "Agent CLI (version 0.26.0)",
        ],
    );
    assert_contains_all(
        "issue 922 requirements",
        &read(root.join("docs/case-studies/issue-922/requirements.md")),
        &["R922-1", "R922-6", "4/4", "13/13", "12/12"],
    );
    assert_contains_all(
        "issue 922 solution plan",
        &read(root.join("docs/case-studies/issue-922/solution-plan.md")),
        &[
            "Real Experience And Reproduction",
            "Proposal And Trust Boundaries",
            "Human-Confirmed Adoption",
            "Verification And Traceability",
        ],
    );
    assert_contains_all(
        "issue 922 online research",
        &read(root.join("docs/case-studies/issue-922/raw-data/online-research.md")),
        &[
            "https://arxiv.org/abs/2006.08381",
            "https://docs.rs/stitch_core/latest/stitch_core/",
            "Repository prior art",
        ],
    );
    assert_contains_all(
        "PR 1005 case study",
        &read(root.join("docs/case-studies/pull-request-1005/README.md")),
        &[
            "## Review Scope",
            "## Review Channels",
            "## CI History",
            "## Decisions",
            "## Verification",
            "no screenshot",
        ],
    );

    for relative in [
        "docs/case-studies/issue-922/raw-data/github/issue.json",
        "docs/case-studies/issue-922/raw-data/github/issue-comments.json",
        "docs/case-studies/issue-922/raw-data/github/pull-request.json",
        "docs/case-studies/issue-922/raw-data/github/pull-conversation-comments.json",
        "docs/case-studies/issue-922/raw-data/github/pull-review-comments.json",
        "docs/case-studies/issue-922/raw-data/github/pull-reviews.json",
        "docs/case-studies/issue-922/agent-cli-run/agent-stderr.log",
        "docs/case-studies/issue-922/agent-cli-run/agent-stream.jsonl",
        "docs/case-studies/issue-922/agent-cli-run/agent-stream.raw.log",
        "docs/case-studies/issue-922/agent-cli-run/formal-ai.log",
        "docs/case-studies/issue-922/agent-cli-run/general-change-plan.lino",
        "docs/case-studies/issue-922/agent-cli-run/promotion-result.diff",
        "docs/case-studies/issue-922/agent-cli-run/promotion-run.lino",
        "docs/case-studies/issue-922/agent-cli-run/session.json",
        "docs/case-studies/pull-request-1005/raw-data/github/pull-request.json",
        "docs/case-studies/pull-request-1005/raw-data/github/pull-conversation-comments.json",
        "docs/case-studies/pull-request-1005/raw-data/github/pull-review-comments.json",
        "docs/case-studies/pull-request-1005/raw-data/github/pull-reviews.json",
        "docs/case-studies/pull-request-1005/raw-data/github/issue-922.json",
        "docs/case-studies/pull-request-1005/raw-data/github/initial-ci-runs.json",
    ] {
        assert!(
            root.join(relative).is_file(),
            "missing issue #922 evidence: {relative}"
        );
    }

    let fragment = root.join("changelog.d/20260813_220000_issue_922_method_learning.md");
    let release_notes = if fragment.is_file() {
        read(fragment)
    } else {
        read(root.join("CHANGELOG.md"))
    };
    assert_contains_all(
        "issue 922 release metadata",
        &release_notes,
        &["event logs", "human-confirmed promotion", "#922"],
    );
}

#[test]
fn issue_922_promotion_and_agent_cli_evidence_are_reproducible() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-922/agent-cli-run");
    assert_contains_all(
        "promotion run",
        &read(evidence.join("promotion-run.lino")),
        &[
            "promotion_run_21bc44690947f221",
            "promoted \"1\"",
            "rejected \"0\"",
            "issue_362_multilingual_coding_modification:cleared:4/4@floor4",
            "issue_304_industry_permissive_slice:cleared:13/13@floor13",
            "formal_ai_unit_specifications:cleared:12/12@floor1",
        ],
    );
    assert_contains_all(
        "Formal AI promotion log",
        &read(evidence.join("promotion-run.log")),
        &[
            "1 considered, 1 promoted, 0 rejected",
            "promotion_agent_session_13a49fd18a6f7f54",
            "Created local review branch",
        ],
    );
    assert_eq!(read(evidence.join("agent-version.txt")).trim(), "0.26.0");
    assert_contains_all(
        "external Agent CLI stream",
        &read(evidence.join("agent-stream.raw.log")),
        &["ses_002c548f2ffeAK4C2qHqVRf8QS", "formal-ai", "success"],
    );
    assert_contains_all(
        "Formal AI session",
        &read(evidence.join("session.json")),
        &[
            "wrote 738 byte(s) to data/seed/learned-methods.lino",
            "\"hit_turn_cap\": false",
            "\"tool\": \"run_command\"",
        ],
    );
    assert_contains_all(
        "replay script",
        &read(root.join("examples/issue-922-method-learning/run.sh")),
        &[
            "improve --promote",
            "--apply --confirm",
            "--output-format stream-json",
            "cmp \"$promotion_work/$TARGET\" \"$ROOT/$TARGET\"",
            "cmp \"$promotion_work/$TARGET\" \"$external_work/$TARGET\"",
        ],
    );
    assert_contains_all(
        "Agent CLI CI gate",
        &read(root.join(".github/workflows/release.yml")),
        &[
            "promoted method learning (issue #922)",
            "examples/issue-922-method-learning/run.sh",
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
