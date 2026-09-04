use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

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
            "| R924-6 ",
            "| R924-7 ",
            "| R924-8 ",
            "| R924-9 ",
            "| R924-10 ",
            "| R924-11 ",
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
            "## Incremental self-development execution",
            "E69",
            "E74",
            PR_URL,
        ],
    );
    assert_contains_all(
        "issue 924 requirement map",
        &read(root.join("docs/case-studies/issue-924/requirements.md")),
        &[
            "R924-1", "R924-2", "R924-3", "R924-4", "R924-5", "R924-6", "R924-7", "R924-8",
            "R924-9", "R924-10", "R924-11",
        ],
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
            "Every non-merge commit introduced by that pull request",
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
            "target_policy \"non-decreasing",
            // Issue #1069: the ratchet only climbs, so it needs a way back down
            // that is not a bypass. The schema has to say where that lever is,
            // and that it is nowhere else, so a reader does not go looking for a
            // flag.
            "target_override_procedure \"",
            "there is no flag, environment variable or workflow input",
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
fn issue_924_incremental_agent_task_is_replayable_and_learns_from_the_same_sessions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-924/incremental-self-authorship");
    let report: Value = serde_json::from_str(&read(evidence.join("dispatch-report.json")))
        .expect("captured incremental dispatch report");
    assert_eq!(report["mode"], "incremental");
    let trace = &report["incremental"];
    assert_eq!(trace["solved"], true);
    assert!(trace["split_depth_reached"].as_u64().unwrap_or(0) >= 1);

    let steps = trace["steps"].as_array().expect("incremental steps");
    assert!(steps.len() >= 4, "{steps:#?}");
    assert_eq!(steps.first().unwrap()["passed"], false);
    assert_eq!(steps.first().unwrap()["cli"], "agent");
    assert_eq!(steps.last().unwrap()["passed"], true);
    assert_eq!(steps.last().unwrap()["cli"], "composed-verifier");
    assert_eq!(
        steps.first().unwrap()["task"],
        steps.last().unwrap()["task"]
    );

    let splits = trace["splits"].as_array().expect("failure-driven splits");
    assert!(
        splits.iter().any(|split| split["children"]
            .as_array()
            .is_some_and(|children| children.len() >= 2)),
        "no productive split: {splits:#?}"
    );

    let agent_steps = steps.iter().filter(|step| step["cli"] == "agent").count();
    assert!(agent_steps >= 4, "{steps:#?}");
    for step in steps {
        let relative = step["session_file"].as_str().expect("session path");
        let session: Value =
            serde_json::from_str(&read(evidence.join(relative))).expect("replayable Agent session");
        if step["cli"] == "agent" {
            assert!(
                session["native_session"]["id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("ses_")),
                "missing native session id: {session:#?}"
            );
            assert!(
                session["native_session"]["resume_command"]
                    .as_str()
                    .is_some_and(|command| command.contains("agent --resume ses_")),
                "missing native resume command: {session:#?}"
            );
        } else {
            assert_eq!(step["cli"], "composed-verifier");
            assert_eq!(session["program"], "verification-only");
            assert!(session["native_session"].is_null(), "{session:#?}");
            assert_eq!(session["changes"].as_array().unwrap().len(), 0);
        }
    }

    let learning = read(evidence.join("learning.lino"));
    assert_contains_all(
        "proposal-only learning",
        &learning,
        &[
            "human_gated \"true\"",
            &format!("observation_count \"{agent_steps}\""),
        ],
    );
    assert!(!learning.contains("decision \"approved\""), "{learning}");
    assert!(evidence.join("proposals.lino").is_file());
}

#[test]
fn issue_924_agent_authored_contracts_are_canonical_and_cover_twenty_percent_of_leaves() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join("docs/case-studies/issue-924/incremental-self-authorship");
    for contract in [
        "self-development-execution-contract.lino",
        "self-development-pull-request-contract.lino",
    ] {
        assert_eq!(
            read(root.join("data/meta").join(contract)).as_bytes(),
            read(evidence.join(contract)).as_bytes(),
            "Formal AI-authored {contract} must be preserved byte for byte"
        );
    }

    let execution = read(root.join("data/meta/self-development-execution-contract.lino"));
    assert_contains_all(
        "self-development execution contract",
        &execution,
        &[
            "task_execution \"formal_ai_via_agent_cli\"",
            "strategy \"attempt_whole_then_split_only_after_failure\"",
            "recursion \"split_until_solvable_or_bounded_irreducible\"",
            "learning \"same_sessions_to_proposal_only_learning\"",
            "promotion \"human_review_required\"",
        ],
    );
    let pull_request = read(root.join("data/meta/self-development-pull-request-contract.lino"));
    assert_contains_all(
        "end-to-end pull request contract",
        &pull_request,
        &[
            "authorship \"end_to_end\"",
            "commit_coverage \"every_non_merge_commit_introduced_by_pull_request\"",
            "review_ci_promotion \"unchanged\"",
        ],
    );

    let decomposition = read(evidence.join("decomposition.lino"));
    assert_contains_all(
        "issue 924 decomposition",
        &decomposition,
        &[
            "leaf_count \"6\"",
            "formal_ai_authored_leaf_count \"2\"",
            "formal_ai_authored_percent \"33\"",
        ],
    );
    assert_eq!(
        decomposition
            .matches("owner \"formal_ai_agent_cli\"")
            .count(),
        2
    );
    assert_eq!(
        decomposition
            .matches("record_type \"smallest_leaf\"")
            .count(),
        6
    );

    let recipe = read(root.join("experiments/issue_924_self_authoring/run.sh"));
    assert_contains_all(
        "issue 924 incremental replay",
        &recipe,
        &[
            "agent dispatch",
            "--incremental",
            "--pull-request",
            "--cli agent",
            "--verify",
            "learning.lino",
        ],
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
