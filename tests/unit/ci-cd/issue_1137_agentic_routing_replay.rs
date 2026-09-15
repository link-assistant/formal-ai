//! Pre-merge four-client coverage for agentic routing changes (issue #1137).

use std::fs;

#[test]
fn agentic_routing_changes_enable_full_four_client_replay_on_pull_requests() {
    let root = env!("CARGO_MANIFEST_DIR");
    let release = fs::read_to_string(format!("{root}/.github/workflows/release.yml"))
        .expect("release workflow");
    let detector = fs::read_to_string(format!("{root}/scripts/detect-code-changes.rs"))
        .expect("change detector");
    let reusable = fs::read_to_string(format!("{root}/.github/workflows/agent-cli-e2e.yml"))
        .expect("Agent CLI reusable workflow");

    assert!(
        release.contains(
            "agentic-routing-changed: ${{ steps.changes.outputs.agentic-routing-changed }}"
        )
    );
    assert!(release.contains(
        "github.event_name == 'pull_request' && needs.detect-changes.outputs.agentic-routing-changed == 'true'"
    ));
    assert!(detector.contains("file.starts_with(\"src/agentic_coding/\")"));
    assert!(
        reusable.contains("pull requests that change agentic routing pass true"),
        "the reusable workflow input must document the pre-merge replay contract"
    );
}

#[test]
fn the_full_replay_still_exercises_each_supported_client() {
    let root = env!("CARGO_MANIFEST_DIR");
    let workflow = fs::read_to_string(format!("{root}/.github/workflows/agent-cli-e2e.yml"))
        .expect("Agent CLI reusable workflow");
    let harness = fs::read_to_string(format!("{root}/experiments/agent_cli_e2e/run_issue_781.sh"))
        .expect("four-client harness");

    for client in ["agent", "opencode", "claude", "codex"] {
        assert!(
            harness.contains(&format!("    {client})")),
            "full replay must retain the {client} leg"
        );
    }
    assert!(harness.contains("CLIENTS=\"${CLIENTS:-agent opencode claude codex}\""));
    assert!(workflow.contains("experiments/agent_cli_e2e/run_issue_781.sh"));
}
