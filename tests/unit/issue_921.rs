use formal_ai::orchestration::{replay_session, AgentStatus, AgentTarget};
use std::fs;
use std::path::{Path, PathBuf};

const ISSUE_URL: &str = "https://github.com/link-assistant/formal-ai/issues/921";
const HIVE_RESULT: &str = "hive mind drove agent cli through formal ai";
const ORCHESTRATOR_RESULT: &str = "formal ai dispatched a hive-mind-shaped issue";

#[test]
fn issue_921_both_directions_are_committed_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_contains_all(
        "issue 921 requirements",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #921 Hive-Mind Full-Circle Integration Gate",
            "| R921-1 ",
            "| R921-2 ",
            "| R921-3 ",
            "| R921-4 ",
            "| R921-5 ",
            // The two defects the gate proved nothing about: a working
            // transport carrying no work (hive-mind#2158).
            "| R921-6 ",
            "| R921-7 ",
        ],
    );
    assert_contains_all(
        "issue 921 case study",
        &read(root.join("docs/case-studies/issue-921/README.md")),
        &[
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Reproduction And Root Cause",
            "## 4. Implemented Gate",
            "## 5. Verification",
            "## 6. The Two Defects The Gate Was Not Enough To Catch",
            "Hive Mind -> Agent CLI -> Formal AI",
            "Formal AI -> external Agent CLI",
            "E69",
            "hive-mind/issues/2158",
        ],
    );
    assert_contains_all(
        "pull request 1004 case study",
        &read(root.join("docs/case-studies/pull-request-1004/README.md")),
        &["Issue #921", "Review Scope", "No screenshots"],
    );

    for relative in [
        "docs/case-studies/issue-921/raw-data/github/issue-921.json",
        "docs/case-studies/issue-921/raw-data/github/issue-921-comments.json",
        "docs/case-studies/issue-921/raw-data/github/pr-1004.json",
        "docs/case-studies/issue-921/raw-data/github/pr-1004-conversation-comments.json",
        "docs/case-studies/issue-921/raw-data/github/pr-1004-review-comments.json",
        "docs/case-studies/issue-921/raw-data/github/pr-1004-reviews.json",
        "docs/case-studies/issue-921/raw-data/github/related-issue-655.json",
        "docs/case-studies/issue-921/raw-data/github/related-issue-703.json",
        "docs/case-studies/issue-921/raw-data/github/related-issue-916.json",
        "docs/case-studies/issue-921/raw-data/github/hive-mind-issue-2059.json",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #921 traceability"
        );
    }

    let fragment = root.join("changelog.d/20260813_210000_issue_921_hive_mind_gate.md");
    let release_notes = if fragment.is_file() {
        read(fragment)
    } else {
        read(root.join("CHANGELOG.md"))
    };
    assert_contains_all(
        "issue 921 release metadata",
        &release_notes,
        &["Hive Mind", "full-circle", "#921"],
    );
}

#[test]
fn issue_921_hive_mind_direction_uses_the_real_public_and_executor_boundaries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = read(root.join("experiments/issue_921_hive_mind_full_circle/run.sh"));
    let github_wrapper = read(
        root.join("experiments/issue_921_hive_mind_full_circle/github-readonly-prepare-wrapper.sh"),
    );
    assert_contains_all(
        "Hive Mind full-circle runner",
        &script,
        &[
            "solve \"${ISSUE_URL}\" --tool agent --model formal-ai",
            "--attach-logs --verbose",
            "--only-prepare-command",
            "NPM_CONFIG_PREFIX",
            "${SCRATCH}/npm-global",
            "git -C \"${SCRATCH}/prepare-repository\" config user.name",
            "git -C \"${SCRATCH}/prepare-repository\" config user.email",
            "git -C \"${SCRATCH}/prepare-repository\" checkout -B main HEAD",
            "git -C \"${SCRATCH}/prepare-repository\" update-ref refs/remotes/origin/main HEAD",
            "GIT_CONFIG_COUNT=2",
            "GIT_CONFIG_KEY_0=user.name",
            "GIT_CONFIG_KEY_1=user.email",
            "run-hive-executor.mjs",
            "formal-ai-to-hive-mind.txt",
            "agent replay",
            "exit 23",
        ],
    );
    let executor =
        read(root.join("experiments/issue_921_hive_mind_full_circle/run-hive-executor.mjs"));
    assert_contains_all(
        "Hive Mind production executor driver",
        &executor,
        &[
            "src/agent.lib.mjs",
            "executeAgentCommand",
            "model: 'formal-ai'",
            "agentPath",
        ],
    );
    assert!(
        !executor.contains("fake solve") && !executor.contains("mock solve"),
        "the passing direction must use Hive Mind's production executor"
    );
    assert_contains_all(
        "read-only GitHub prepare wrapper",
        &github_wrapper,
        &[
            "refused GitHub mutation",
            "--method=POST",
            "--field",
            "issue\" | \"pr",
        ],
    );

    let evidence = root.join("docs/case-studies/issue-921/hive-mind-to-formal-ai");
    for relative in [
        "executor.log.gz",
        "public-prepare.log.gz",
        "failure-executor.log",
    ] {
        assert!(
            evidence.join(relative).is_file(),
            "the real Hive Mind {relative} evidence should be committed"
        );
    }
    assert_eq!(read(evidence.join("result.txt")), HIVE_RESULT);
    assert_eq!(
        read(evidence.join("invocation.txt")).trim(),
        format!("solve {ISSUE_URL} --tool agent --model formal-ai --attach-logs --verbose")
    );
    assert_contains_all(
        "Hive Mind prepared command",
        &read(evidence.join("prepared-command.txt")),
        &[
            "agent --model formalai/formal-ai --verbose",
            "execution=skipped",
        ],
    );
    assert_contains_all(
        "Hive Mind workspace commit",
        &read(evidence.join("workspace-effect.patch")),
        &["hive-mind-to-formal-ai.txt", HIVE_RESULT.trim()],
    );
    assert_commit_id(&read(evidence.join("commit.txt")));
    let session_id = read(evidence.join("agent-session-id.txt"));
    assert!(session_id.trim().starts_with("ses_"));
    assert_contains_all(
        "Hive Mind failure probe",
        &read(evidence.join("failure-propagation.txt")),
        &["agent_exit=23", "hive_exit=23", "workspace_commit=absent"],
    );
    assert_contains_all(
        "Hive Mind production failure log",
        &read(evidence.join("failure-executor.log")),
        &[
            "hive-executor-result",
            "\"exitCode\":23",
            "issue 921 injected agent failure",
        ],
    );
}

#[test]
fn issue_921_formal_ai_direction_has_a_canonical_hash_chained_replay() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-921/formal-ai-to-hive-mind");
    assert_eq!(read(evidence.join("result.txt")), ORCHESTRATOR_RESULT);
    assert_contains_all(
        "hive-mind-shaped task",
        &read(evidence.join("task.md")),
        &[
            ISSUE_URL,
            "Acceptance criteria",
            "formal-ai-to-hive-mind.txt",
        ],
    );
    assert_contains_all(
        "Formal AI workspace commit",
        &read(evidence.join("workspace-effect.patch")),
        &["formal-ai-to-hive-mind.txt", ORCHESTRATOR_RESULT.trim()],
    );
    assert_commit_id(&read(evidence.join("commit.txt")));

    let bytes = fs::read(evidence.join("orchestration-session.json"))
        .expect("the committed orchestration session should be readable");
    let session = replay_session(&bytes).expect("the committed session should replay byte-exact");
    assert_eq!(session.status, AgentStatus::Succeeded);
    assert_eq!(session.target, AgentTarget::FormalAi);
    assert_eq!(session.cli, "agent");
    assert_eq!(
        session.task,
        "Create file formal-ai-to-hive-mind.txt containing formal ai dispatched a hive-mind-shaped issue"
    );
    assert_eq!(session.exit_code, Some(0));
    assert!(session.passed());
    assert!(
        session
            .changes
            .iter()
            .any(|effect| effect.path == "formal-ai-to-hive-mind.txt"),
        "the hash-chained session must record the requested workspace effect"
    );
    assert!(
        session
            .events
            .iter()
            .any(|event| event.kind == "workspace_effect"),
        "the event chain must include the observed effect"
    );
    assert_contains_all(
        "Formal AI failure probe",
        &read(evidence.join("failure-propagation.txt")),
        &[
            "agent_exit=23",
            "orchestrator_exit=1",
            "session_status=failed",
        ],
    );
    let failed_bytes = fs::read(evidence.join("failure-session.json"))
        .expect("the committed failed orchestration session should be readable");
    let failed_session = replay_session(&failed_bytes)
        .expect("the committed failed session should retain a valid hash chain");
    assert_eq!(failed_session.status, AgentStatus::Failed);
    assert_eq!(failed_session.exit_code, Some(23));
    assert!(!failed_session.passed());

    assert_contains_all(
        "issue 921 release gate",
        &read(root.join(".github/workflows/release.yml")),
        &[
            "Install @link-assistant/hive-mind CLI",
            "Hive Mind full-circle integration gate (issue #921)",
            "experiments/issue_921_hive_mind_full_circle/run.sh",
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

fn assert_commit_id(raw: &str) {
    let value = raw.trim();
    assert_eq!(value.len(), 40, "expected a full SHA-1 commit id: {value}");
    assert!(
        value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "commit id should be hexadecimal: {value}"
    );
}
