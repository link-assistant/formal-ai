//! Issue #1138 B6 (plan 06, L10): the whole family-1 recovery, live, five languages.
//!
//! This is the test a manual install must fail. It drives the held-out program
//! through the actual recovery path — probe, need, publisher lookup,
//! workspace-scoped install under a grant, re-probe, retry — and records what
//! happened. A live network/toolchain failure is evidence, not a reason to
//! rewrite this test until it asserts success.
//!
//! It reaches trusted publishers over the network, so it is ignored by default
//! and run deliberately.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use formal_ai::concept_lookup::RegistrySourceLookup;
use formal_ai::execution_box::BACKEND_ENV;
use formal_ai::execution_box::container::{
    ConversationContainer, SnapshotPolicy, conversation_snapshot_image,
};
use formal_ai::execution_box::conversation_container_name;
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::prerequisite::install::InstallGrant;
use formal_ai::prerequisite::ledger::ToolchainLedger;
use formal_ai::prerequisite::probe::{ProbeVerdict, ToolchainProbe, probe_command};
use formal_ai::prerequisite::{Platform, PrerequisiteNeed, RecoveryOutcome, recover};
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::source_walk::LookupBounds;

const CORPUS: &str = include_str!("../../../data/benchmarks/self-use-prerequisite.lino");

/// The held-out program family 1 is written around.
const PROGRAM: &str = "zig";

fn workspace_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("formal-ai-issue-1138-recovery-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the workspace root should be creatable");
    root
}

fn need_for(prompt: &str) -> PrerequisiteNeed {
    PrerequisiteNeed {
        program: String::from(PROGRAM),
        source_span: prompt.to_owned(),
        observed: ProbeVerdict::Missing {
            exit_code: Some(127),
            stderr: format!("{PROGRAM}: command not found"),
        },
        platform: Platform::observed(),
        requires: Vec::new(),
    }
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
        .replace("\"\"", "\"")
}

fn family_prompts(family: &str) -> Vec<(String, String)> {
    let mut current = String::new();
    let mut language = String::new();
    let mut prompts = Vec::new();
    for line in CORPUS.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("id ") {
            current = unquote(value);
        } else if let Some(value) = line.strip_prefix("language ") {
            language = unquote(value);
        } else if current == family
            && let Some(value) = line.strip_prefix("prompt ")
        {
            prompts.push((language.clone(), unquote(value)));
        }
    }
    prompts
}

const fn outcome_slug(outcome: &RecoveryOutcome) -> &'static str {
    match outcome {
        RecoveryOutcome::Recovered { .. } => "recovered",
        RecoveryOutcome::NotPermitted { .. } => "not_permitted",
        RecoveryOutcome::NotFound { .. } => "not_found",
        RecoveryOutcome::StillMissing { .. } => "still_missing",
    }
}

fn lino_value(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\"").replace('\n', "\\n"))
}

fn record_evidence(path: &Path, evidence: &str) {
    std::fs::write(path, evidence).expect("live evidence should be writable");
    let recorded = std::fs::read_to_string(path).expect("live evidence should be readable");
    assert_eq!(
        recorded, evidence,
        "the observation record must survive byte-for-byte"
    );
    eprintln!("{}\n{}", path.display(), evidence);
}

#[test]
#[ignore = "network: plan 06 L10"]
fn a_missing_held_out_compiler_is_discovered_installed_and_retried() {
    let bounds = LookupBounds::default();
    let cache = std::env::temp_dir().join("formal-ai-issue-1138-recovery-cache");
    let client = CachedSourceClient::new(&cache, CurlSourceTransport);
    let preferences = ServicePreferences::default();
    let mut availability = ServiceAccessibilityCache::new(&cache);

    let mut observations = String::from("live_prerequisite_recovery\n");
    for (language, prompt) in family_prompts("named_toolchain_is_not_seen") {
        // Both branches are observations. Unit tests own the deterministic
        // permission assertions; this ignored live run preserves the result it
        // actually obtained from today's publishers and host.
        let refused_root = workspace_root(&format!("refused-{language}"));
        let mut ledger = ToolchainLedger::new(refused_root.join("toolchain-ledger.lino"));
        let mut lookup = RegistrySourceLookup::new(
            &client,
            &preferences,
            &mut availability,
            bounds,
            &language,
            0,
        );
        let refused = recover(
            &need_for(&prompt),
            &InstallGrant::Refused,
            &mut lookup,
            &bounds,
            &mut ledger,
        );
        let _ = writeln!(observations, "  language_outcome {language}");
        let _ = writeln!(observations, "    refused {}", outcome_slug(&refused));
        let _ = writeln!(
            observations,
            "    refused_detail {}",
            lino_value(&format!("{refused:?}"))
        );
        let _ = writeln!(
            observations,
            "    refused_toolchain_tree_present {}",
            refused_root.join(".formal-ai/toolchains").exists()
        );

        // Granted: the toolchain lands inside the workspace, the postcondition
        // probe passes, and the version is the one that was observed.
        let granted_root = workspace_root(&format!("granted-{language}"));
        let mut ledger = ToolchainLedger::new(granted_root.join("toolchain-ledger.lino"));
        let mut lookup = RegistrySourceLookup::new(
            &client,
            &preferences,
            &mut availability,
            bounds,
            &language,
            0,
        );
        let granted = recover(
            &need_for(&prompt),
            &InstallGrant::Allowed {
                programs: vec![String::from(PROGRAM)],
                root: granted_root.clone(),
            },
            &mut lookup,
            &bounds,
            &mut ledger,
        );
        let reprobe = match &granted {
            RecoveryOutcome::Recovered { .. } => Some(probe_command(
                &ToolchainProbe {
                    program: String::from(PROGRAM),
                    argv: vec![String::from("version")],
                    expect: None,
                    requires: Vec::new(),
                },
                &granted_root,
            )),
            _ => None,
        };
        let _ = writeln!(observations, "    granted {}", outcome_slug(&granted));
        let _ = writeln!(
            observations,
            "    granted_detail {}",
            lino_value(&format!("{granted:?}"))
        );
        let _ = writeln!(
            observations,
            "    reprobe {}",
            lino_value(&format!("{reprobe:?}"))
        );
    }
    let evidence_root = workspace_root("observations");
    record_evidence(&evidence_root.join("recovery.lino"), &observations);
}

/// Exercise the exact harness-recovery/evaluator boundary used by the external
/// benchmark runner. The live result may be recovery failure, missing Docker,
/// evaluator failure, or an outcome; all are retained as evidence. The test
/// asserts only that the record was durably written, never that the benchmark
/// passed or even that today's infrastructure reached the evaluator.
#[test]
#[ignore = "network + Docker: plan 06 L17 records recovery/evaluator reachability"]
fn swebench_harness_recovery_records_whether_the_evaluator_was_reached() {
    use formal_ai::external_benchmarks::{BenchmarkCase, Expectation};

    let root = workspace_root("swebench-evaluator");
    let record = r#"{"instance_id":"astropy__astropy-12907","repo":"astropy/astropy","base_commit":"d16bfe05a744909de4b27f5875fe0d4ed41ce607","problem_statement":"Nested compound-model separability is incorrect.","FAIL_TO_PASS":"[]","PASS_TO_PASS":"[]"}"#;
    let case = BenchmarkCase {
        id: String::from("astropy__astropy-12907"),
        prompt: String::from("Repair the pinned upstream instance."),
        expectation: Expectation::SweBench {
            record: record.to_owned(),
        },
        repository: None,
        tests: None,
    };
    let patch = String::from(
        "```diff\ndiff --git a/README.rst b/README.rst\n--- a/README.rst\n+++ b/README.rst\n@@ -1 +1 @@\n-Astropy\n+Astropy\n```",
    );
    let result = formal_ai::external_benchmarks::grade::grade_swebench_with_install(
        &[case],
        &[patch],
        &root,
        true,
    );
    let evaluator_reached =
        root.join("dataset.json").is_file() && root.join("predictions.jsonl").is_file();
    let ledger_recorded = root
        .join(".formal-ai/prerequisite/toolchain-ledger.lino")
        .is_file();
    let mut evidence = String::from("live_swebench_recovery\n");
    let _ = writeln!(evidence, "  harness_ledger_recorded {ledger_recorded}");
    let _ = writeln!(evidence, "  evaluator_reached {evaluator_reached}");
    let _ = writeln!(
        evidence,
        "  result {}",
        lino_value(&match result {
            Ok(outcomes) => format!("outcomes:{outcomes:?}"),
            Err(reason) => format!("infrastructure:{reason}"),
        })
    );
    record_evidence(&root.join("attempt.lino"), &evidence);
}

fn live_conversation(id: &str, restore: SnapshotPolicy) -> ConversationContainer {
    ConversationContainer {
        conversation_id: id.to_owned(),
        image: String::from("konard/box-python:2.4.0"),
        handle: None,
        idle_after: Duration::from_millis(1),
        restore,
    }
}

fn remove_live_conversation(id: &str) {
    let _ = Command::new("docker")
        .args(["rm", "--force", &conversation_container_name(id)])
        .output();
    let _ = Command::new("docker")
        .args(["image", "rm", &conversation_snapshot_image(id)])
        .output();
}

#[test]
#[ignore = "Docker: plan 06 L14 snapshot/reattach evidence"]
fn an_idle_container_stops_and_restores_its_state() {
    let id = format!("issue-1138-snapshot-live-{}", std::process::id());
    temp_env::with_var(BACKEND_ENV, Some("docker"), || {
        let mut conversation = live_conversation(&id, SnapshotPolicy::Snapshot);
        let boxed = conversation.attach().expect("Docker conversation attaches");
        let written = boxed
            .run("open('state.txt', 'w').write('kept')\n", &[])
            .expect("state write is observed");
        assert_eq!(written.exit_code, Some(0));
        conversation
            .stop_if_idle(Instant::now() + Duration::from_secs(1))
            .expect("idle stop snapshots the container");
        let restored = conversation.attach().expect("snapshot reattaches");
        let observed = restored
            .run("print(open('state.txt').read())\n", &[])
            .expect("restored state is observed");
        assert_eq!(observed.exit_code, Some(0));
        assert!(observed.partial_output.contains("kept"));
    });
    remove_live_conversation(&id);
}

#[test]
#[ignore = "Docker: plan 06 L14 snapshot/replay comparison evidence"]
fn replay_and_snapshot_are_compared_when_both_exist() {
    let id = format!("issue-1138-replay-live-{}", std::process::id());
    temp_env::with_var(BACKEND_ENV, Some("docker"), || {
        let mut conversation = live_conversation(&id, SnapshotPolicy::Replay);
        let boxed = conversation.attach().expect("Docker conversation attaches");
        let observed = boxed
            .run("open('state.txt', 'w').write('replayed')\n", &[])
            .expect("state write is observed");
        assert_eq!(observed.exit_code, Some(0));
        let comparison = conversation
            .compare_restores()
            .expect("Docker export compares snapshot and replay");
        assert!(comparison.snapshot_digest.is_some());
        assert!(comparison.replay_digest.is_some());
        assert_eq!(
            comparison.diverged,
            comparison.snapshot_digest != comparison.replay_digest
        );
    });
    remove_live_conversation(&id);
}
