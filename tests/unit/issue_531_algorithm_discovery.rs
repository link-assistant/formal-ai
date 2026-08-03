//! Regression coverage for the issue-531 review follow-up: repeated sequences
//! of events must become reviewable, validated algorithm candidates rather than
//! stopping at a "this sequence repeats" classification.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::process::Command;

use formal_ai::agentic_coding::{algorithm_learning::trace_from_driver_outcome, run_agentic_task};
use formal_ai::algorithm_discovery::{
    discover_algorithms, trace_from_compiled_procedure, trace_from_event_log, AlgorithmApproval,
    AlgorithmGate, AlgorithmHost, ArgumentPattern, ExecutionTrace, TraceStep,
    MAX_DISCOVERED_ALGORITHM_STEPS, MAX_DISCOVERY_INPUT_STEPS,
};
use formal_ai::{
    apply_dreaming_plan, compile_procedure, plan_memory_dreaming, DreamingConfig, EventLog,
    MemoryEvent, MemoryStore,
};

fn step(operation: &str, arguments: &[(&str, &str)]) -> TraceStep {
    TraceStep::new(operation).with_arguments(arguments.iter().copied())
}

fn trace(id: &str, subject: &str, format: &str) -> ExecutionTrace {
    ExecutionTrace::new(
        id,
        vec![
            step("fetch", &[("subject", subject)]),
            step("normalize", &[("subject", subject), ("format", format)]),
            step("persist", &[("subject", subject)]),
        ],
    )
}

#[test]
fn repeated_event_sequences_become_a_validated_parameterized_algorithm() {
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", "json"),
        trace("run-beta", "beta", "json"),
        trace("run-gamma", "gamma", "json"),
    ]);

    assert!(run.associative_compression_lossless);
    assert!(run.associative_compression_steps > 0);
    let candidates = run.validated_candidates();
    assert_eq!(candidates.len(), 1, "{}", run.links_notation());
    let candidate = candidates[0];
    assert_eq!(candidate.steps.len(), 3);
    assert_eq!(candidate.support_trace_ids, ["run-alpha", "run-beta"]);
    assert_eq!(candidate.held_out.len(), 1);
    assert!(candidate.held_out[0].passed);

    let fetch_subject = &candidate.steps[0].arguments["subject"];
    let normalize_subject = &candidate.steps[1].arguments["subject"];
    assert_eq!(fetch_subject, normalize_subject);
    assert!(matches!(fetch_subject, ArgumentPattern::Parameter(_)));
    assert_eq!(
        candidate.steps[1].arguments["format"],
        ArgumentPattern::Constant(String::from("json"))
    );

    let document = run.links_notation();
    assert!(document.contains("mode \"proposal_only\""));
    assert!(document.contains("human_gated \"true\""));
    assert!(document.contains("associative_compression_lossless \"true\""));
}

#[test]
fn a_repeated_episode_inside_one_log_is_discovered_without_trace_boundaries() {
    let trace = ExecutionTrace::new(
        "single-log",
        vec![
            step("read", &[]),
            step("verify", &[]),
            step("read", &[]),
            step("verify", &[]),
            step("read", &[]),
            step("verify", &[]),
        ],
    );

    let run = discover_algorithms(&[trace]);
    let candidate = run
        .validated_candidates()
        .into_iter()
        .find(|candidate| {
            candidate
                .steps
                .iter()
                .map(|step| step.operation.as_str())
                .eq(["read", "verify"])
        })
        .expect("the repeated read/verify episode should be inferred");
    assert_eq!(candidate.support_trace_ids.len(), 2);
    assert_eq!(candidate.held_out.len(), 1);
}

#[test]
fn held_out_constant_drift_is_preserved_as_a_failed_candidate() {
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", "json"),
        trace("run-beta", "beta", "json"),
        trace("run-gamma", "gamma", "xml"),
    ]);

    assert!(run.validated_candidates().is_empty());
    let candidate = run
        .candidates
        .iter()
        .find(|candidate| candidate.steps.len() == 3)
        .expect("the rejected candidate must remain inspectable");
    assert_eq!(candidate.held_out.len(), 1);
    assert!(!candidate.held_out[0].passed);
    assert!(candidate.held_out[0]
        .failures
        .iter()
        .any(|failure| failure.contains("constant_mismatch")));
    assert!(run
        .links_notation()
        .contains("status \"held_out_validation_failed\""));
    assert!(candidate
        .conformance_links_notation("must-not-pass", &BTreeMap::new())
        .is_err());
}

#[test]
fn held_out_missing_or_changed_steps_reject_an_incomplete_algorithm() {
    let truncated = ExecutionTrace::new(
        "run-gamma",
        vec![
            step("fetch", &[("subject", "gamma")]),
            step("persist", &[("subject", "gamma")]),
        ],
    );
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", "json"),
        trace("run-beta", "beta", "json"),
        truncated,
    ]);

    let candidate = run
        .candidates
        .iter()
        .find(|candidate| candidate.steps.len() == 3)
        .expect("two support traces should retain an inspectable proposal");
    assert!(!candidate.validated());
    assert!(candidate.held_out[0]
        .failures
        .iter()
        .any(|failure| failure.contains("operation_mismatch")));
    assert!(candidate.held_out[0]
        .failures
        .iter()
        .any(|failure| failure.contains("missing_step")));
}

#[test]
fn failed_longer_episode_does_not_hide_a_validated_subroutine() {
    let changed_tail = ExecutionTrace::new(
        "run-gamma",
        vec![
            step("fetch", &[("subject", "gamma")]),
            step("normalize", &[("subject", "gamma"), ("format", "json")]),
            step("publish", &[("subject", "gamma")]),
        ],
    );
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", "json"),
        trace("run-beta", "beta", "json"),
        changed_tail,
    ]);

    assert!(run
        .candidates
        .iter()
        .any(|candidate| { candidate.steps.len() == 3 && !candidate.validated() }));
    assert!(run.validated_candidates().iter().any(|candidate| {
        candidate
            .steps
            .iter()
            .map(|step| step.operation.as_str())
            .eq(["fetch", "normalize"])
    }));
}

#[test]
fn empty_constant_arguments_round_trip_without_becoming_missing_fields() {
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", ""),
        trace("run-beta", "beta", ""),
        trace("run-gamma", "gamma", ""),
    ]);
    let expected = run.validated_candidates()[0];
    assert_eq!(
        expected.steps[1].arguments["format"],
        ArgumentPattern::Constant(String::new())
    );

    let restored = formal_ai::algorithm_discovery::AlgorithmCandidate::from_links_notation(
        &run.links_notation(),
    )
    .expect("an explicitly empty constant is still a present value");
    assert_eq!(&restored, expected);
}

#[test]
fn held_out_argument_shape_drift_is_a_counterexample() {
    let mut changed = trace("run-gamma", "gamma", "json");
    changed.steps[1]
        .arguments
        .insert(String::from("privileged"), String::from("true"));
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", "json"),
        trace("run-beta", "beta", "json"),
        changed,
    ]);
    let candidate = run
        .candidates
        .iter()
        .find(|candidate| candidate.steps.len() == 3)
        .expect("support retains a reviewable failed proposal");

    assert!(!candidate.validated());
    assert!(candidate.held_out[0]
        .failures
        .iter()
        .any(|failure| failure.contains("unexpected_argument")));
}

#[test]
fn oversized_observation_sets_fail_closed_without_partial_candidates() {
    let trace = ExecutionTrace::new(
        "oversized",
        vec![step("read", &[]); MAX_DISCOVERY_INPUT_STEPS + 1],
    );
    let run = discover_algorithms(&[trace]);

    assert!(run.observation_limit_exceeded);
    assert!(run.candidates.is_empty());
    assert!(!run.associative_compression_lossless);
    assert!(run
        .links_notation()
        .contains("observation_limit_exceeded \"true\""));
}

#[test]
fn imported_artifacts_cannot_bypass_the_reviewable_step_limit() {
    let mut artifact = String::from(
        "algorithm_candidate \"oversized\"\n  evidence_id \"oversized\"\n  mode \"proposal_only\"\n  human_gated \"true\"\n  status \"held_out_validated\"\n  associative_root \"0\"\n  steps\n",
    );
    for index in 0..=MAX_DISCOVERED_ALGORITHM_STEPS {
        let _ = write!(artifact, "    step \"{index}\"\n      operation \"read\"\n");
    }
    artifact.push_str(
        "  support\n    trace \"alpha\"\n    trace \"beta\"\n  held_out\n    test \"gamma\"\n      start_step \"0\"\n      passed \"true\"\n",
    );

    let error = formal_ai::algorithm_discovery::AlgorithmCandidate::from_links_notation(&artifact)
        .expect_err("import must reapply the same bound as discovery");
    assert!(
        error.to_string().contains("reviewable step limit"),
        "{error}"
    );
}

#[test]
fn runtime_event_logs_and_compiled_guides_share_the_trace_model() {
    let event_traces = ["alpha", "beta", "gamma"].map(|subject| {
        let mut log = EventLog::new();
        log.append("fetch", subject);
        log.append("normalize", subject);
        log.append("persist", subject);
        trace_from_event_log(format!("event-log-{subject}"), &log)
    });
    let event_candidate = discover_algorithms(&event_traces)
        .validated_candidates()
        .into_iter()
        .next()
        .expect("runtime logs should use the common held-out miner")
        .clone();
    assert_eq!(event_candidate.steps.len(), 3);

    let guide = compile_procedure(
        "When I paste a link, fetch its title, translate it to Russian, save both, and reply with the translation.",
    )
    .expect("seeded multi-step guide should compile");
    let guide_trace = trace_from_compiled_procedure(&guide);
    assert_eq!(guide_trace.id, guide.id);
    assert_eq!(
        guide_trace
            .steps
            .iter()
            .map(|step| step.operation.as_str())
            .collect::<Vec<_>>(),
        [
            "skill_procedure_fetch",
            "skill_procedure_translate",
            "skill_procedure_store",
            "skill_procedure_reply",
        ]
    );
    assert_eq!(
        guide_trace.steps[1].arguments["target_language"],
        "language_russian"
    );
}

#[derive(Default)]
struct RecordingHost {
    operations: Vec<(String, BTreeMap<String, String>)>,
}

impl AlgorithmHost for RecordingHost {
    fn perform(
        &mut self,
        operation: &str,
        arguments: &BTreeMap<String, String>,
        input: &str,
    ) -> Result<String, String> {
        self.operations
            .push((operation.to_owned(), arguments.clone()));
        Ok(format!("{operation}({input})"))
    }
}

#[test]
fn learned_algorithm_is_inert_until_green_gate_and_named_review() {
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", "json"),
        trace("run-beta", "beta", "json"),
        trace("run-gamma", "gamma", "json"),
    ]);
    let candidate = run.validated_candidates()[0];

    assert!(candidate
        .promote(
            AlgorithmGate::passed("issue_531_algorithm_discovery", 1),
            AlgorithmApproval::declined("reviewer"),
        )
        .is_err());
    assert!(candidate
        .promote(
            AlgorithmGate::failed("issue_531_algorithm_discovery", 0, 1),
            AlgorithmApproval::granted("reviewer"),
        )
        .is_err());
    assert!(candidate
        .promote(
            AlgorithmGate::passed("", 1),
            AlgorithmApproval::granted("reviewer"),
        )
        .is_err());

    let approved = candidate
        .promote(
            AlgorithmGate::passed("issue_531_algorithm_discovery", 1),
            AlgorithmApproval::granted("reviewer"),
        )
        .expect("validated candidates can be explicitly promoted");
    assert_eq!(approved.candidate(), candidate);
    assert_eq!(approved.gate().suite, "issue_531_algorithm_discovery");
    assert_eq!(approved.approval().reviewer, "reviewer");
    let parameter = candidate.steps[0].arguments["subject"]
        .parameter_name()
        .expect("subject varies across support");
    let bindings = BTreeMap::from([(parameter.to_owned(), String::from("delta"))]);
    let mut host = RecordingHost::default();
    let executed = approved
        .execute("trigger", &bindings, &mut host)
        .expect("an approved algorithm should execute through the generic host");

    assert_eq!(executed.outcomes.len(), 3);
    assert_eq!(host.operations[0].1["subject"], "delta");
    assert_eq!(host.operations[1].1["format"], "json");
}

#[test]
fn idle_learning_proposes_algorithms_from_persisted_conversation_events() {
    let events = memory_observations();

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());
    assert_eq!(plan.algorithm_candidates.len(), 1);
    assert!(plan.algorithm_candidates[0].validated());

    let mut store = MemoryStore::from_events(events);
    let outcome = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(outcome.learned_algorithm_candidates, 1);
    let learned = store
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("algorithm_learning_candidate"))
        .expect("idle learning should retain the proposal");
    assert_eq!(learned.intent.as_deref(), Some("generalize"));
    assert!(learned
        .content
        .as_deref()
        .is_some_and(|content| content.contains("human_gated \"true\"")));
}

fn memory_observations() -> Vec<MemoryEvent> {
    let mut store = MemoryStore::new();
    store.replace_from_links_notation(include_str!(
        "../../data/benchmarks/issue-531-algorithm-traces.lino"
    ));
    store.events().to_vec()
}

#[test]
fn artifact_round_trip_and_conformance_do_not_implicitly_approve_execution() {
    let run = discover_algorithms(&[
        trace("run-alpha", "alpha", "json"),
        trace("run-beta", "beta", "json"),
        trace("run-gamma", "gamma", "json"),
    ]);
    let candidate = run.validated_candidates()[0];
    let restored = formal_ai::algorithm_discovery::AlgorithmCandidate::from_links_notation(
        &run.links_notation(),
    )
    .expect("generated discovery artifact should round-trip");
    assert_eq!(&restored, candidate);

    let parameter = candidate.steps[0].arguments["subject"]
        .parameter_name()
        .expect("subject is parameterized");
    let conformance = restored
        .conformance_links_notation(
            "dry-trigger",
            &BTreeMap::from([(parameter.to_owned(), String::from("delta"))]),
        )
        .expect("all parameters are bound");
    assert!(conformance.contains("side_effects \"false\""));
    assert!(conformance.contains("result \"passed\""));
    assert!(!conformance.contains("reviewer"));

    let tampered = run
        .links_notation()
        .replacen(&candidate.id, "algorithm_tampered", 1);
    assert!(
        formal_ai::algorithm_discovery::AlgorithmCandidate::from_links_notation(&tampered).is_err()
    );
    let tampered_evidence =
        run.links_notation()
            .replacen("test \"run-gamma\"", "test \"forged-run\"", 1);
    assert!(
        formal_ai::algorithm_discovery::AlgorithmCandidate::from_links_notation(&tampered_evidence)
            .is_err(),
        "held-out evidence cannot be rewritten into a promotion-eligible artifact"
    );
}

#[test]
fn public_cli_mines_and_conformance_checks_a_portable_memory_file() {
    let workspace = std::env::temp_dir().join(format!(
        "formal-ai-issue-531-algorithm-cli-{}",
        std::process::id()
    ));
    fs::create_dir_all(&workspace).expect("create isolated CLI fixture");
    let observations = workspace.join("observations.lino");
    let artifact = workspace.join("algorithms.lino");
    fs::write(
        &observations,
        MemoryStore::from_events(memory_observations()).export_links_notation(),
    )
    .expect("write portable observations");

    let learned = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "learn",
            "algorithms",
            "--from",
            observations.to_str().expect("UTF-8 fixture path"),
            "--output",
            artifact.to_str().expect("UTF-8 artifact path"),
        ])
        .output()
        .expect("run public learning CLI");
    assert!(learned.status.success(), "{learned:?}");
    assert!(String::from_utf8_lossy(&learned.stderr).contains("held-out validated"));
    let document = fs::read_to_string(&artifact).expect("read learned artifact");
    assert!(document.contains("algorithm_candidate"));

    let conformance = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "algorithm",
            "conformance",
            "--artifact",
            artifact.to_str().expect("UTF-8 artifact path"),
            "--trigger",
            "cli-regression",
            "--binding",
            "parameter_1=delta",
        ])
        .output()
        .expect("run public conformance CLI");
    assert!(conformance.status.success(), "{conformance:?}");
    let output = String::from_utf8_lossy(&conformance.stdout);
    assert!(output.contains("side_effects \"false\""));
    assert!(output.contains("result \"passed\""));

    fs::remove_dir_all(workspace).expect("remove isolated CLI fixture");
}

#[test]
fn formal_ai_agent_cli_discovers_reads_back_and_conformance_checks_the_same_task() {
    let memory = MemoryStore::from_events(memory_observations()).export_links_notation();
    let task = format!(
        "Derive any reusable execution algorithm from these recorded events and verify it.\n{memory}"
    );
    let run = run_agentic_task(&task).expect("in-repo Agent CLI replay");

    assert!(!run.hit_turn_cap);
    assert_eq!(
        run.steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>(),
        ["write_file", "run_command", "run_command", "run_command"]
    );
    assert!(run.steps[1]
        .arguments
        .contains("formal-ai learn algorithms"));
    assert!(run.steps[2].result.contains("algorithm_candidate"));
    assert!(run.steps[3]
        .arguments
        .contains("formal-ai algorithm conformance"));
    assert!(run.steps[3].result.contains("side_effects \"false\""));
    assert!(run.final_answer.contains("status \"conformance_passed\""));
    assert!(run.final_answer.contains("human_gated \"true\""));

    let replayable_trace = trace_from_driver_outcome("agent-session", &run);
    assert_eq!(
        replayable_trace
            .steps
            .iter()
            .map(|step| step.operation.as_str())
            .collect::<Vec<_>>(),
        ["write_file", "run_command", "run_command", "run_command"]
    );
}
