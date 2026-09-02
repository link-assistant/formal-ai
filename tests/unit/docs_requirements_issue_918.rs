use std::fs;
use std::path::{Path, PathBuf};

const INVARIANT: &str = "Compiled code belongs in Formal AI's minimal core only when it implements the meta algorithm, link-store substrate, a generic interpreter, or a host surface; domain knowledge and policy belong in data.";

#[test]
fn issue_918_case_study_and_release_metadata_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_contains_all(
        "REQUIREMENTS.md",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #918 Minimal-Core Boundary And Seed-Metadata Audit",
            "| R918-1 ",
            "| R918-2 ",
            "| R918-3 ",
            "| R918-4 ",
            "| R918-5 ",
            "| R918-6 ",
            "docs/design/minimal-core-boundary.md",
            "data/meta/seed-metadata-schema.lino",
            "all 46 files as migration candidates",
        ],
    );
    assert_contains_all(
        "ARCHITECTURE.md",
        &read(root.join("ARCHITECTURE.md")),
        &[
            "Minimal Compiled Core (Issue #918)",
            "Meta algorithm",
            "Link store",
            "Generic interpreters",
            "Host surfaces",
            "scripts/check-minimal-core-boundary.rs",
        ],
    );
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "Issue #918 Minimal-Core Boundary And Seed-Metadata Audit (PR #986)",
            "46 recursive handler sources",
            "19,543 outside-core lines",
            "3,447 remaining metadata-gap records",
        ],
    );

    assert_contains_all(
        "issue 918 case study",
        &read(root.join("docs/case-studies/issue-918/README.md")),
        &[
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Reproduction And Root Cause",
            "## 4. Implemented Design",
            "## 5. Verification",
            "46 recursive handler sources",
            "37 coding-path concepts",
            "3,447 other concepts",
        ],
    );
    assert_contains_all(
        "issue 918 requirements",
        &read(root.join("docs/case-studies/issue-918/requirements.md")),
        &["R918-1", "R918-6", "one of five", "FrameNet", "Wikidata"],
    );
    assert_contains_all(
        "issue 918 solution plan",
        &read(root.join("docs/case-studies/issue-918/solution-plan.md")),
        &[
            "Recursive Boundary Census",
            "Metadata Contract And Coding Floor",
            "Gap Data",
            "Traceability And Self-Hosting",
        ],
    );
    assert_contains_all(
        "issue 918 online research",
        &read(root.join("docs/case-studies/issue-918/raw-data/online-research.md")),
        &[
            "https://framenet.icsi.berkeley.edu/fndrupal/frameIndex",
            "https://aclanthology.org/P98-1013/",
            "https://www.wikidata.org/wiki/Wikidata:Data_model/en",
            "Repository Prior Art",
        ],
    );
    assert_contains_all(
        "PR 986 case study",
        &read(root.join("docs/case-studies/pull-request-986/README.md")),
        &["Issue #918", "Review Scope", "No screenshots"],
    );

    for relative in [
        "docs/case-studies/issue-918/raw-data/github/issue-918.json",
        "docs/case-studies/issue-918/raw-data/github/issue-918-comments.json",
        "docs/case-studies/issue-918/raw-data/github/pr-986.json",
        "docs/case-studies/issue-918/raw-data/github/pr-986-conversation-comments.json",
        "docs/case-studies/issue-918/raw-data/github/pr-986-review-comments.json",
        "docs/case-studies/issue-918/raw-data/github/pr-986-reviews.json",
        "docs/case-studies/issue-918/raw-data/github/issue-914.json",
        "docs/case-studies/issue-918/raw-data/github/pr-877.json",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #918 traceability"
        );
    }

    let fragment = root.join("changelog.d/20260810_120000_issue_918_minimal_core_metadata.md");
    let release_notes = if fragment.is_file() {
        read(fragment)
    } else {
        read(root.join("CHANGELOG.md"))
    };
    assert_contains_all(
        "issue 918 release metadata",
        &release_notes,
        &["minimal-core", "metadata", "#918"],
    );
}

#[test]
fn issue_918_agent_cli_authorship_leaf_is_byte_exact_and_reproducible() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-918/agent-cli-evidence");
    assert_eq!(read(evidence.join("minimal-core-invariant.md")), INVARIANT);

    let session_id = read(evidence.join("session-id.txt"));
    let session_id = session_id.trim();
    assert!(
        session_id.starts_with("ses_") && session_id.len() > 4,
        "Agent CLI evidence must preserve a real session id"
    );
    assert_eq!(
        read(evidence.join("task.txt")).trim(),
        format!("Create file minimal-core-invariant.md containing {INVARIANT}")
    );

    let stream = read(evidence.join("agent-stream.raw.log"));
    assert_contains_all("Agent CLI raw stream", &stream, &[session_id, "formal-ai"]);
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

    let script = read(root.join("experiments/issue_918_agent_cli.sh"));
    assert_contains_all(
        "issue 918 Agent CLI replay",
        &script,
        &[
            "serve --host 127.0.0.1",
            "--output-format stream-json",
            "FORMAL_AI_MEMORY_PATH=\"$work/.git/formal-ai-memory/memory.lino\"",
            "minimal-core-invariant.md",
            INVARIANT,
            "cmp -s",
        ],
    );
    let workflow = read(root.join(".github/workflows/release.yml"));
    assert_contains_all(
        "issue 918 Agent CLI CI gate",
        &workflow,
        &[
            "minimal-core invariant (issue #918)",
            "experiments/issue_918_agent_cli.sh",
            "/tmp/formal-ai-issue-918-evidence",
        ],
    );
    assert!(
        workflow
            .matches("/tmp/formal-ai-issue-918-evidence")
            .count()
            >= 2,
        "issue 918 evidence must be uploaded when its CI replay fails"
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
