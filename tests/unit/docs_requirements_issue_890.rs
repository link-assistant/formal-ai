use std::fs;
use std::path::{Path, PathBuf};

const SESSION_ID: &str = "ses_03b44e557ffeSQeAuCYzxfc3BR";
const INVARIANT: &str = "Formal proofs must keep semantic bounds and witnesses independent from natural-language prose and programming-language syntax, then project the same meaning into every requested target.";

#[test]
fn issue_890_case_study_and_release_metadata_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_contains_all(
        "REQUIREMENTS.md",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #890 Formal Proof Program Translation",
            "| R890-1 ",
            "| R890-2 ",
            "| R890-3 ",
            "| R890-4 ",
            "| R890-5 ",
            "| R890-6 ",
            "| R890-7 ",
            "CodeMeaning::FormalProof",
            "every_registered_natural_language_can_request_proof_translation",
        ],
    );
    assert_contains_all(
        "ARCHITECTURE.md",
        &read(root.join("ARCHITECTURE.md")),
        &[
            "Issue #890 extends that meta-language path",
            "src/proof_program.rs",
            "data/seed/proof-program-templates.lino",
            "one proof formalizer plus one data-defined projection per target",
            "proof_translation_worker.rs",
        ],
    );
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "Issue #890 Formal Proof Program Translation (PR #911)",
            "complete Rust",
            "and Python programs",
            "docs/case-studies/issue-890/",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-890/README.md"));
    assert_contains_all(
        "issue 890 case study",
        &case_study,
        &[
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Reproduction And Root Cause",
            "## 4. Implemented Design",
            "## 5. Verification",
            "translate_proof_to_rust",
            "data/seed/proof-program-templates.lino",
            "ses_03b44e557ffeSQeAuCYzxfc3BR",
        ],
    );
    assert_contains_all(
        "issue 890 requirements",
        &read(root.join("docs/case-studies/issue-890/requirements.md")),
        &[
            "R890-1",
            "R890-7",
            "one of five leaves",
            "whole_issue_890_workflow_solves_translates_and_executes",
        ],
    );
    assert_contains_all(
        "issue 890 solution plan",
        &read(root.join("docs/case-studies/issue-890/solution-plan.md")),
        &[
            "Separate Meaning From Presentation",
            "General Translation And Execution",
            "Language Matrix And Browser Parity",
            "Traceability And Self-Hosting",
        ],
    );
    assert_contains_all(
        "issue 890 online research",
        &read(root.join("docs/case-studies/issue-890/raw-data/online-research.md")),
        &[
            "https://llvm.org/docs/LangRef.html",
            "https://doc.rust-lang.org/stable/core/macro.assert.html",
            "https://docs.python.org/3/reference/simple_stmts.html#the-assert-statement",
            "Repository Prior Art",
        ],
    );

    for relative in [
        "docs/case-studies/issue-890/raw-data/github/issue.json",
        "docs/case-studies/issue-890/raw-data/github/issue-comments.json",
        "docs/case-studies/issue-890/raw-data/github/pull-request.json",
        "docs/case-studies/issue-890/raw-data/github/pull-conversation-comments.json",
        "docs/case-studies/issue-890/raw-data/github/pull-review-comments.json",
        "docs/case-studies/issue-890/raw-data/github/pull-reviews.json",
        "docs/case-studies/issue-890/raw-data/github/related-issue-403.json",
        "docs/case-studies/issue-890/raw-data/github/related-issue-526.json",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #890 traceability"
        );
    }

    let issue_tests = read(root.join("tests/unit/issue_890.rs"));
    for test in [
        "proof_meaning_is_independent_from_its_programming_language_presentations",
        "same_solved_proof_uses_general_translation_path_for_two_targets",
        "generated_proof_programs_compile_and_execute",
        "every_registered_natural_language_can_request_proof_translation",
        "whole_issue_890_workflow_solves_translates_and_executes",
    ] {
        assert!(
            issue_tests.contains(test),
            "missing issue #890 test: {test}"
        );
    }

    let fragment = root.join("changelog.d/20260802_230000_issue_890_proof_program_translation.md");
    let release_notes = if fragment.is_file() {
        read(fragment)
    } else {
        read(root.join("CHANGELOG.md"))
    };
    assert_contains_all(
        "issue 890 release metadata",
        &release_notes,
        &["proof", "Rust", "Python", "#890"],
    );
}

#[test]
fn agent_cli_authorship_leaf_is_byte_exact_and_reproducible() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-890/agent-cli-evidence");
    assert_eq!(
        read(evidence.join("proof-translation-invariant.md")),
        INVARIANT
    );
    assert_eq!(read(evidence.join("session-id.txt")).trim(), SESSION_ID);
    assert_eq!(
        read(evidence.join("task.txt")).trim(),
        format!("Create file proof-translation-invariant.md containing {INVARIANT}")
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

    let script = read(root.join("experiments/issue_890_agent_cli.sh"));
    assert_contains_all(
        "issue 890 Agent CLI replay",
        &script,
        &[
            "serve --host 127.0.0.1",
            "--output-format stream-json",
            "FORMAL_AI_MEMORY_PATH=\"$work/.git/formal-ai-memory/memory.lino\"",
            "proof-translation-invariant.md",
            INVARIANT,
            "cmp -s",
        ],
    );
    assert!(
        script
            .lines()
            .all(|line| !line.trim_start().starts_with("rg ")),
        "issue 890 Agent CLI replay must only require tools installed by its CI job"
    );
    let workflow = read(root.join(".github/workflows/release.yml"));
    assert_contains_all(
        "issue 890 Agent CLI CI gate",
        &workflow,
        &[
            "proof translation invariant (issue #890)",
            "experiments/issue_890_agent_cli.sh",
            "/tmp/formal-ai-issue-890-evidence",
        ],
    );
    assert!(
        workflow
            .matches("/tmp/formal-ai-issue-890-evidence")
            .count()
            >= 2,
        "issue 890 evidence must be uploaded when its CI replay fails"
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
