//! The multi-CLI end-to-end matrix has to cover *every* client we ship, not the
//! handful somebody remembered (issue #671).
//!
//! PR #648 closed #647 with `claude` "intentionally not run" and `grok`/`aider`
//! "inferred from the shared adapters"; hands-on testing then produced issue
//! #650 with four defects. These guards make that failure mode impossible to
//! repeat: adding a client to `data/seed/client-integrations.lino` without a
//! pinned version, a CI leg and a documented row fails the build.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use formal_ai::client_contract_learning::{
    ClientContractObservation, DeliveryMode, learn_client_contracts, load_observations,
};
use formal_ai::seed::client_integrations;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn seeded_ids() -> Vec<String> {
    client_integrations()
        .iter()
        .map(|integration| integration.id.clone())
        .collect()
}

/// Client ids pinned in the lockfile, in file order.
fn locked_order() -> Vec<String> {
    read("experiments/agentic_cli_matrix/clients.lock")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let id = fields.next()?;
            // A row is only a pin if it also names an installer and a spec.
            (fields.next().is_some() && fields.next().is_some()).then(|| id.to_owned())
        })
        .collect()
}

fn locked_ids() -> BTreeSet<String> {
    locked_order().into_iter().collect()
}

#[test]
fn agentic_cli_matrix_covers_every_seeded_client() {
    let locked = locked_ids();
    let missing: Vec<_> = seeded_ids()
        .into_iter()
        .filter(|id| !locked.contains(id))
        .collect();

    assert!(
        missing.is_empty(),
        "clients missing a pinned version in experiments/agentic_cli_matrix/clients.lock: {missing:?}"
    );
}

#[test]
fn every_pinned_client_is_a_real_seeded_client() {
    let seeded: BTreeSet<String> = seeded_ids().into_iter().collect();
    let stale: Vec<_> = locked_ids()
        .into_iter()
        .filter(|id| !seeded.contains(id))
        .collect();

    // A pin for a client we no longer ship would install a CLI nothing drives.
    assert!(
        stale.is_empty(),
        "clients.lock pins ids that are not in the seed registry: {stale:?}"
    );
}

#[test]
fn ci_matrix_is_generated_instead_of_repeating_client_ids() {
    let workflow = read(".github/workflows/agentic-cli-matrix.yml");
    assert!(
        workflow.contains("fromJSON(needs.build.outputs.matrix)"),
        "CI must consume the registry-derived matrix plan"
    );
    for id in seeded_ids() {
        assert!(
            !workflow.contains(&format!("matrix.client == '{id}'")),
            "CI behavior for {id} is hardcoded instead of coming from the client contract"
        );
        assert!(
            !workflow.contains(&format!("- client: {id}\n")),
            "CI repeats {id} instead of deriving the leg from the registry"
        );
    }
}

#[test]
fn every_seeded_client_has_a_documented_matrix_row() {
    let guide = read("docs/testing/agentic-cli-tools.md");
    let missing: Vec<_> = seeded_ids()
        .into_iter()
        .filter(|id| !guide.contains(&format!("| `{id}` |")))
        .collect();

    assert!(
        missing.is_empty(),
        "clients missing a row in the docs/testing/agentic-cli-tools.md matrix table: {missing:?}"
    );
}

#[test]
fn every_ci_leg_gets_its_own_port_range() {
    let ports: Vec<usize> = locked_order()
        .iter()
        .enumerate()
        .map(|(index, _)| 8900 + index * 60)
        .collect();

    assert_eq!(
        ports.len(),
        seeded_ids().len(),
        "every leg needs a base_port: {ports:?}"
    );
    let unique: BTreeSet<_> = ports.iter().copied().collect();
    assert_eq!(unique.len(), ports.len(), "duplicate base_port: {ports:?}");

    // Each leg starts a server and a proxy per case, and `run_leg.sh` spaces its
    // cases 10 ports apart, so neighbouring legs must not overlap.
    let ordered: Vec<_> = unique.into_iter().collect();
    for pair in ordered.windows(2) {
        assert!(
            pair[1] - pair[0] >= 50,
            "leg port ranges overlap: {} and {}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn ci_and_local_runners_share_the_registry_plan() {
    let workflow = read(".github/workflows/agentic-cli-matrix.yml");
    let planner = read("experiments/agentic_cli_matrix/plan_matrix.sh");
    assert!(workflow.contains("experiments/agentic_cli_matrix/plan_matrix.sh"));
    assert!(read("experiments/agentic_cli_matrix/run_matrix.sh").contains("matrix_plan"));
    assert!(
        planner.contains("set -euo pipefail"),
        "a failed registry lookup must abort matrix generation"
    );
}

#[test]
fn every_client_behavior_is_a_seeded_verification_contract() {
    for client in client_integrations() {
        let contract = &client.verification;
        assert!(
            !contract.surface.is_empty(),
            "{} has no verification surface",
            client.id
        );
        if contract.surface == "cli" {
            assert!(
                matches!(contract.file_delivery.as_str(), "tool_call" | "in_band"),
                "{} has no testable file-delivery contract",
                client.id
            );
        }
        if contract.surface == "server" {
            assert!(
                !contract.launch_args.is_empty()
                    && !contract.launch_ready.is_empty()
                    && !contract.launch_http_path.is_empty(),
                "{} has no complete server launch contract",
                client.id
            );
        }
        if contract.surface == "mcp" {
            assert!(
                !contract.vendor_auth_error.is_empty(),
                "{} does not seed its upstream credential boundary",
                client.id
            );
        }
    }
}

#[test]
fn t3code_0_0_37_launch_contract_covers_its_complete_surface() {
    let clients = client_integrations();
    let t3code = clients
        .iter()
        .find(|client| client.id == "t3code")
        .expect("seeded t3code client");
    for subcommand in ["theme", "triage"] {
        assert!(
            t3code
                .verification
                .launch_subcommands
                .iter()
                .any(|actual| actual == subcommand),
            "t3code 0.0.37 exposes {subcommand}; classify it before accepting the upgraded client"
        );
    }
}

#[test]
fn server_launch_waits_for_every_required_output_line() {
    let leg = read("experiments/agentic_cli_matrix/run_leg.sh");
    assert!(
        leg.contains("matrix_await_log launch \"$required\" 120"),
        "a server can announce readiness before its remaining launch contract; \
         required output must use the bounded log wait"
    );
    assert!(
        !leg.contains("matrix_log_matches \"$MATRIX_CLIENT_LOG\" \"$required\""),
        "a one-shot required-output check races asynchronous server startup"
    );
}

#[test]
fn matrix_scripts_do_not_branch_on_client_identity() {
    for script in [
        "experiments/agentic_cli_matrix/lib.sh",
        "experiments/agentic_cli_matrix/run_leg.sh",
    ] {
        let contents = read(script);
        for forbidden in [
            r#"[ "$CLIENT" ="#,
            r#"case "$CLIENT" in"#,
            r#"case "$1" in"#,
        ] {
            assert!(
                !contents.contains(forbidden),
                "{script} still selects behavior with `{forbidden}`"
            );
        }
    }
}

#[test]
fn repeated_independent_observations_propose_a_human_gated_reusable_contract() {
    let observations = vec![
        ClientContractObservation::new(
            "future-client",
            "read_file",
            "read the fixture and return its bytes",
            DeliveryMode::ToolCall,
            ["workspace_read"],
            "headless.jsonl",
        ),
        ClientContractObservation::new(
            "future-client",
            "read_file",
            "show the exact text stored in the fixture",
            DeliveryMode::ToolCall,
            ["workspace_read"],
            "interactive.jsonl",
        ),
    ];

    let report = learn_client_contracts(&observations, &[]);
    assert!(report.awaiting_human_review);
    assert_eq!(report.proposals.len(), 1);
    assert_eq!(report.proposals[0].client_id, "future-client");
    assert_eq!(report.proposals[0].value, "workspace_read");
    assert!(
        report
            .links_notation()
            .contains("decision \"awaiting_human_review\"")
    );
}

#[test]
fn successful_matrix_runs_feed_the_human_gated_learner() {
    let leg = read("experiments/agentic_cli_matrix/run_leg.sh");
    let workflow = read(".github/workflows/agentic-cli-matrix.yml");
    assert!(
        leg.contains("matrix_observe_case read_file"),
        "real read-file cases are not normalized into learning observations"
    );
    assert!(
        leg.contains("verification.required_response_tools"),
        "human-approved response-tool requirements are not enforced by the live leg"
    );
    assert!(
        workflow.contains("formal-ai clients learn"),
        "CI does not aggregate successful real-client observations"
    );
    assert!(
        workflow.contains("human_gated \"true\""),
        "CI does not assert that inferred amendments remain human-gated"
    );
}

#[test]
fn committed_real_sessions_produce_a_deterministic_review_artifact() {
    let observations_path =
        root().join("docs/case-studies/issue-671/agent-cli-contract-learning/observations.jsonl");
    let observations = load_observations(&[&observations_path]).expect("load observations");
    let report = learn_client_contracts(&observations, &client_integrations());

    assert_eq!(report.observation_count, 16);
    assert_eq!(report.independently_worded_groups, 8);
    assert_eq!(report.findings.len(), 8);
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.status == "confirmed"),
        "the recorded delivery behavior drifted from the seed: {:?}",
        report.findings
    );
    assert_eq!(
        report.proposals.len(),
        7,
        "seven tool-bearing clients should propose their repeatedly observed read tool"
    );
    assert!(report.awaiting_human_review);
    for observation in &observations {
        assert!(
            root().join(&observation.evidence).is_file(),
            "observation evidence is missing: {}",
            observation.evidence
        );
    }

    let expected = read(
        "docs/case-studies/issue-671/agent-cli-contract-learning/client-contract-learning-report.lino",
    );
    assert_eq!(expected, format!("{}\n", report.links_notation()));
}

#[test]
fn formal_ai_executes_contract_learning_through_the_real_agent_cli() {
    assert!(
        read(".github/workflows/release.yml").contains("run_issue_671_contract_learning.sh"),
        "the required real Agent CLI execution must run in CI"
    );
    let expected = read(
        "docs/case-studies/issue-671/agent-cli-contract-learning/client-contract-learning-report.lino",
    );
    let agent_authored = read(
        "docs/case-studies/issue-671/agent-cli-contract-learning/agent-authored-client-contract-learning-report.lino",
    );
    assert_eq!(
        agent_authored, expected,
        "the report written by Agent CLI must match Formal AI's deterministic output"
    );

    let plan =
        read("docs/case-studies/issue-671/agent-cli-contract-learning/general-change-plan.lino");
    assert!(plan.contains("capability \"Run\""));
    assert!(plan.contains("formal-ai clients learn"));
    assert!(plan.contains("> 'client-contract-learning-report.lino'"));
    assert!(plan.contains("command \"cat client-contract-learning-report.lino\""));
    assert!(
        !plan.contains("expected_evidence \"its exact stdout\""),
        "the command-output reference must not be treated as literal file content"
    );

    let stream = read("docs/case-studies/issue-671/agent-cli-contract-learning/agent-stream.jsonl");
    assert!(stream.contains("\"status\":\"success\""));
    assert!(stream.contains("Completed the general change request"));
}

#[test]
fn shell_registry_reads_keep_false_and_empty_arrays_distinct_from_missing_data() {
    let library = read("experiments/agentic_cli_matrix/lib.sh");
    assert!(
        !library.contains("jq -er"),
        "jq -e reports a valid boolean false or empty stream as lookup failure"
    );
    assert!(
        library.contains("if $value == null then error(\"field is missing\")"),
        "scalar lookups must reject only missing data"
    );
    assert!(
        library.contains("then $value[]"),
        "array lookups must allow a valid empty contract array"
    );
}

/// Recorded transcripts committed under `recorded/`.
fn recorded_transcripts() -> Vec<(String, PathBuf)> {
    let dir = root().join("experiments/agentic_cli_matrix/recorded");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for client_dir in entries.flatten() {
        let client = client_dir.file_name().to_string_lossy().into_owned();
        let Ok(files) = std::fs::read_dir(client_dir.path()) else {
            continue;
        };
        for file in files.flatten() {
            if file.path().extension().is_some_and(|ext| ext == "jsonl") {
                found.push((client.clone(), file.path()));
            }
        }
    }
    found
}

/// Issue #671's acceptance criteria name these three by hand: PR #648 shipped
/// `claude` "intentionally not run" and `grok`/`aider` "inferred from the shared
/// adapters". A committed transcript is the evidence that each was really run.
#[test]
fn the_never_run_integrations_have_recorded_sessions() {
    let recorded = recorded_transcripts();
    for id in ["claude", "grok", "aider"] {
        assert!(
            recorded.iter().any(|(client, _)| client == id),
            "no recorded session under experiments/agentic_cli_matrix/recorded/{id}/ \
             — PR #648 shipped this integration without ever running it"
        );
    }
}

#[test]
fn every_recorded_transcript_is_replayable() {
    let locked = locked_ids();
    let recorded = recorded_transcripts();
    assert!(!recorded.is_empty(), "no recorded transcripts committed");

    for (client, path) in recorded {
        assert!(
            locked.contains(&client),
            "recorded/{client} is not a pinned client"
        );
        let text = std::fs::read_to_string(&path).expect("read transcript");
        let rows: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert!(!rows.is_empty(), "{} is empty", path.display());
        for row in rows {
            let value: serde_json::Value = serde_json::from_str(row)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            // Bodies carry the run's temp paths and session ids, so a transcript
            // that kept them would differ from the next re-record for reasons
            // that mean nothing — see `matrix_record_case` in lib.sh.
            for field in ["request_body", "response_body"] {
                assert!(
                    value.get(field).is_none(),
                    "{} still carries {field}",
                    path.display()
                );
            }
            let status = value.get("status").and_then(serde_json::Value::as_u64);
            assert!(
                status.is_none_or(|code| code < 400),
                "{} records a failing exchange: {status:?}",
                path.display()
            );
        }
    }
}

#[test]
fn matrix_scripts_are_executable() {
    for script in [
        "experiments/agentic_cli_matrix/install_client.sh",
        "experiments/agentic_cli_matrix/run_leg.sh",
        "experiments/agentic_cli_matrix/run_matrix.sh",
        "experiments/agentic_cli_matrix/plan_matrix.sh",
        "experiments/agentic_cli_matrix/replay.sh",
    ] {
        let path: &Path = &root().join(script);
        assert!(path.exists(), "{script} is missing");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(path)
                .expect("metadata")
                .permissions()
                .mode();
            assert!(
                mode & 0o111 != 0,
                "{script} is not executable (mode {mode:o})"
            );
        }
    }
}
