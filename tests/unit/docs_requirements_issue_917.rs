use std::fs;
use std::path::{Path, PathBuf};

const INVARIANT: &str = "Formal language projections must map one language-neutral semantic statement into seed-defined concrete syntaxes and preserve the same meaning on the return path.";

#[test]
fn issue_917_case_study_and_release_metadata_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_contains_all(
        "REQUIREMENTS.md",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #917 General Natural-Formal Translation",
            "| R917-1 ",
            "| R917-2 ",
            "| R917-3 ",
            "| R917-4 ",
            "| R917-5 ",
            "| R917-6 ",
            "| R917-7 ",
            "data/seed/formal-language-projections.lino",
            "Spanish",
            "every_seed_language_round_trips_through_a_seeded_formal_target",
        ],
    );
    assert_contains_all(
        "ARCHITECTURE.md",
        &read(root.join("ARCHITECTURE.md")),
        &[
            "Issue #917 makes formal languages first-class projections",
            "src/translation/formal_statement.rs",
            "data/seed/formal-language-projections.lino",
            "src/web/wasm-worker/src/formal_statement_worker.rs",
            "natural -> FOL -> natural",
        ],
    );
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "Issue #917 General Natural-Formal Translation (PR #984)",
            "project to seed-defined",
            "docs/case-studies/issue-917/",
            "delivered by #917 for the seeded FOL statement slice",
        ],
    );

    assert_contains_all(
        "issue 917 case study",
        &read(root.join("docs/case-studies/issue-917/README.md")),
        &[
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Reproduction And Root Cause",
            "## 4. Implemented Design",
            "## 5. Verification",
            "SemanticStatement",
            "tests/e2e/tests/issue-917.spec.js",
        ],
    );
    assert_contains_all(
        "issue 917 requirements",
        &read(root.join("docs/case-studies/issue-917/requirements.md")),
        &[
            "R917-1",
            "R917-7",
            "one of five",
            "whole_task_translation_uses_the_formal_projection_in_both_directions",
        ],
    );
    assert_contains_all(
        "issue 917 solution plan",
        &read(root.join("docs/case-studies/issue-917/solution-plan.md")),
        &[
            "Statement Meaning And Round Trips",
            "Seed-Defined Concrete Syntax",
            "Whole-Engine And Browser Parity",
            "Traceability And Self-Hosting",
        ],
    );
    assert_contains_all(
        "issue 917 online research",
        &read(root.join("docs/case-studies/issue-917/raw-data/online-research.md")),
        &[
            "https://www.grammaticalframework.org/",
            "https://github.com/Attempto/APE",
            "https://universaldependencies.org/",
            "Repository Prior Art",
        ],
    );

    for relative in [
        "docs/case-studies/issue-917/raw-data/github/issue.json",
        "docs/case-studies/issue-917/raw-data/github/issue-comments.json",
        "docs/case-studies/issue-917/raw-data/github/pull-request.json",
        "docs/case-studies/issue-917/raw-data/github/pull-conversation-comments.json",
        "docs/case-studies/issue-917/raw-data/github/pull-review-comments.json",
        "docs/case-studies/issue-917/raw-data/github/pull-reviews.json",
        "docs/case-studies/issue-917/raw-data/github/related-issue-526.json",
        "docs/case-studies/issue-917/raw-data/github/related-issue-890.json",
        "docs/case-studies/issue-917/raw-data/github/related-issue-914.json",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #917 traceability"
        );
    }

    let fragment = root.join("changelog.d/20260808_120000_issue_917_natural_formal_translation.md");
    let release_notes = if fragment.is_file() {
        read(fragment)
    } else {
        read(root.join("CHANGELOG.md"))
    };
    assert_contains_all(
        "issue 917 release metadata",
        &release_notes,
        &["first-order logic", "Wikidata-grounded", "#917"],
    );
}

#[test]
fn issue_917_agent_cli_authorship_leaf_is_byte_exact_and_reproducible() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-917/agent-cli-evidence");
    assert_eq!(
        read(evidence.join("formal-language-projection-invariant.md")),
        INVARIANT
    );
    let session_id = read(evidence.join("session-id.txt"));
    let session_id = session_id.trim();
    assert!(
        session_id.starts_with("ses_") && session_id.len() > 4,
        "Agent CLI evidence must preserve a real session id"
    );
    assert_eq!(
        read(evidence.join("task.txt")).trim(),
        format!("Create file formal-language-projection-invariant.md containing {INVARIANT}")
    );

    let stream = read(evidence.join("agent-stream.raw.log"));
    assert!(
        stream.contains(session_id),
        "Agent CLI raw stream must preserve its recorded session"
    );
    assert_contains_all("Agent CLI raw stream", &stream, &["formal-ai"]);
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

    let script = read(root.join("experiments/issue_917_agent_cli.sh"));
    assert_contains_all(
        "issue 917 Agent CLI replay",
        &script,
        &[
            "serve --host 127.0.0.1",
            "--output-format stream-json",
            "FORMAL_AI_MEMORY_PATH=\"$work/.git/formal-ai-memory/memory.lino\"",
            "formal-language-projection-invariant.md",
            INVARIANT,
            "cmp -s",
        ],
    );
    assert!(
        script
            .lines()
            .all(|line| !line.trim_start().starts_with("rg ")),
        "issue 917 Agent CLI replay must only require tools installed by its CI job"
    );
    let workflow = read(root.join(".github/workflows/release.yml"));
    assert_contains_all(
        "issue 917 Agent CLI CI gate",
        &workflow,
        &[
            "formal projection invariant (issue #917)",
            "experiments/issue_917_agent_cli.sh",
            "/tmp/formal-ai-issue-917-evidence",
            "/tmp/formal-ai-issue-*-evidence",
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
