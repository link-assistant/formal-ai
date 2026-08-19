//! Regression coverage for the upstream-version pin that issue #1021 needed.
//!
//! The `E2E Tests (agent CLI <-> formal-ai)` job went red on a branch that
//! changed nothing in the Codex startup path: `@openai/codex@0.148.0` was
//! published overnight and drops the ENTER that answers its first-run trust
//! dialog (<https://github.com/openai/codex/issues/39487>), so the TUI leg hung
//! before a single request reached the server under test.
//!
//! `experiments/agentic_cli_matrix/clients.lock` already states the rule that
//! would have prevented it -- "a matrix leg fails because our server changed,
//! not because an upstream CLI shipped overnight" -- but `release.yml` was
//! only following it for the one package `--trust` forced someone to name.
//! These tests hold the rule for every third-party CLI CI installs, so the next
//! floating install is caught at review time rather than by a red job.

use super::workflow_fixtures::release_workflow;
use std::fs;
use std::path::Path;

/// Packages the project publishes itself. Tracking these at latest is the
/// point -- an E2E leg that pinned our own client would stop reporting whether
/// today's client still works against today's server.
const OWN_SCOPE: &str = "@link-assistant/";

fn workflow_file(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github/workflows")
        .join(name);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .replace("\r\n", "\n")
}

/// Split a `bun add -g` command into the package specs it installs, dropping
/// the flags. `bun add -g --ignore-scripts a@1 b@2` yields `["a@1", "b@2"]`.
fn installed_specs(line: &str) -> Vec<&str> {
    line.trim()
        .trim_start_matches("run:")
        .trim()
        .strip_prefix("bun add -g")
        .expect("caller filtered for `bun add -g`")
        .split_whitespace()
        .filter(|argument| !argument.starts_with('-'))
        .collect()
}

/// A version is explicit when the spec carries an `@<version>` after the
/// package name: `@openai/codex@0.147.0` and `opencode-ai@1.18.4` are pinned,
/// `@openai/codex` and `opencode-ai` are not.
fn is_pinned(spec: &str) -> bool {
    let name = spec.strip_prefix('@').unwrap_or(spec);
    name.contains('@')
}

#[test]
fn every_third_party_cli_a_workflow_installs_globally_carries_an_explicit_version() {
    let mut checked = Vec::new();
    for name in [
        "release.yml",
        "proactive-failure-report-e2e.yml",
        "agentic-cli-matrix.yml",
    ] {
        let workflow = workflow_file(name);
        for line in workflow.lines().filter(|line| {
            line.trim()
                .trim_start_matches("run:")
                .trim()
                .starts_with("bun add -g")
        }) {
            for spec in installed_specs(line) {
                if spec.starts_with(OWN_SCOPE) {
                    continue;
                }
                assert!(
                    is_pinned(spec),
                    "{name} installs {spec} without a version; \
                     pin it the way experiments/agentic_cli_matrix/clients.lock requires"
                );
                checked.push(spec.to_string());
            }
        }
    }
    checked.sort();
    assert_eq!(
        checked,
        vec![
            "@anthropic-ai/claude-code@2.1.234".to_string(),
            "@google/gemini-cli@0.55.1".to_string(),
            "@openai/codex@0.147.0".to_string(),
            "opencode-ai@1.18.4".to_string(),
        ],
        "the set of third-party CLIs CI installs changed; \
         re-run experiments/issue_1021_codex_tui_version/run.sh for the new pin"
    );
}

#[test]
fn the_codex_pin_names_the_upstream_defect_and_the_bisect_that_would_lift_it() {
    let workflow = release_workflow();
    let step = workflow
        .split("- name: Install external agent CLIs")
        .nth(1)
        .and_then(|tail| tail.split("- name:").next())
        .expect("the external agent CLI install step");

    assert!(step.contains("https://github.com/openai/codex/issues/39487"));
    assert!(step.contains("experiments/issue_1021_codex_tui_version"));
    assert!(step.contains("@openai/codex@0.147.0"));
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/issue_1021_codex_tui_version/run.sh")
            .is_file(),
        "the bisect the comment points at must exist"
    );
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/issue_1021_codex_tui_version/codex_trust_dialog_probe.py")
            .is_file(),
        "the wrapper-free reproduction the upstream report cites must exist"
    );
}
