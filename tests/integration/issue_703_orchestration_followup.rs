use formal_ai::orchestration::{
    apply_verified_translation, dispatch_agents, extract_agent_result,
    observe_orchestration_session, replay_session, resume_agent, run_agent, synthesize_sessions,
    write_session, AgentCommand, AgentRunError, AgentRunPermission, AgentStatus, ComparisonEntry,
    ComparisonLedger, DispatchConfig, DispatchMode, ReplayError, VerificationCommand,
};
use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

use super::issue_703_orchestration::{
    fixture_command, fixture_commands, fixture_config, fixture_config_with_output,
    grant_fixture_agent_command, TestWorkspace, FIXTURE_ENV, FIXTURE_RELEASE_ENV,
    FIXTURE_STARTED_ENV,
};

const AUTHORSHIP_SESSION: &str = "ses_050646852ffetdnQ73vR1yZ8la";
const CONTINUATION_SESSION: &str = "ses_04e25ba4cffeibfMekv188DNLX";

#[test]
fn formal_ai_authored_invariant_is_byte_pinned_to_its_session_evidence() {
    let repository_artifact = include_bytes!("../../data/meta/orchestration-safety-invariant.lino");
    let captured_artifact = include_bytes!(
        "../../docs/case-studies/issue-703/self-hosting-authorship/orchestration-safety-invariant.lino"
    );
    let evidence =
        include_str!("../../docs/case-studies/issue-703/self-hosting-authorship/agent-cli.log");

    assert_eq!(repository_artifact, captured_artifact);
    assert!(evidence.contains(AUTHORSHIP_SESSION));
    assert!(evidence.contains("\"providerID\": \"formal-ai\""));
    assert!(evidence.contains("\"tool\": \"write\""));
}

#[test]
fn documentation_traces_the_maintainer_followup_without_overclaiming_fact_or_learning_gates() {
    let requirements = include_str!("../../REQUIREMENTS.md");
    let guide = include_str!("../../docs/configuration/orchestration.md");
    let case_study = include_str!("../../docs/case-studies/issue-703/README.md");
    let evidence = include_str!("../../docs/case-studies/issue-703/followup-authorship/README.md");

    for requirement in 1..=12 {
        assert!(requirements.contains(&format!("R703-{requirement}")));
    }
    assert!(guide.contains("cross_agent_evidence_preflight"));
    assert!(guide.contains("proposal-only and human-gated"));
    assert!(guide.contains("--allow-agent-command"));
    assert!(guide.contains("--translation-session"));
    assert!(case_study.contains("ses_04e25ba4cffeibfMekv188DNLX"));
    assert!(evidence.contains("First correction"));
    assert!(evidence.contains("contaminated"));
}

#[test]
fn cross_agent_correction_evidence_is_seeded_in_every_supported_language() {
    for language in ["en", "ru", "hi", "zh"] {
        let template = formal_ai::seed::response_for("orchestration_cross_agent_denial", language)
            .unwrap_or_else(|| panic!("missing {language} correction-evidence template"));
        assert!(template.contains("{sources}"));
        assert!(template.contains("{probability}"));
    }
}

#[test]
fn real_formal_ai_controller_agent_run_replays_from_committed_bytes() {
    let bytes = include_bytes!(
        "../../docs/case-studies/issue-703/controller-agent-run/controller-session.json"
    );
    let artifact = include_bytes!(
        "../../docs/case-studies/issue-703/controller-agent-run/controller-proof.lino"
    );

    let session = replay_session(bytes).expect("committed controller session is canonical");

    assert_eq!(session.cli, "agent");
    assert!(session.passed());
    assert!(session.stdout.contains("ses_05046b1c9ffe59CvG0N3QrrsV4"));
    assert!(session.stdout.contains("--no-retry-on-rate-limits"));
    assert!(session.stdout.contains("\"name\":\"write\""));
    let change = session
        .changes
        .iter()
        .find(|change| change.path == "controller-proof.lino")
        .expect("the real Agent CLI run recorded its file effect");
    assert_eq!(
        change.after_sha256.as_deref(),
        Some("b70e7374ba7654d7d57a908ae0d1b7d40e7b1c1651d6bcc2dd80671d35777637")
    );
    assert_eq!(
        artifact,
        b"controller_proof and the phrase replayable workspace edit."
    );
}

#[test]
fn real_formal_ai_correction_chain_resumes_one_native_session_and_pins_the_artifact() {
    let parent = replay_session(include_bytes!(
        "../../docs/case-studies/issue-703/followup-authorship/controller-session.json"
    ))
    .expect("the committed parent session is canonical");
    let corrected = replay_session(include_bytes!(
        "../../docs/case-studies/issue-703/followup-authorship/corrected-session.json"
    ))
    .expect("the first correction session is canonical");
    let final_session = replay_session(include_bytes!(
        "../../docs/case-studies/issue-703/followup-authorship/final-session.json"
    ))
    .expect("the final correction session is canonical");

    assert!(parent.changes.is_empty(), "the first claim was disproved");
    for session in [&parent, &corrected, &final_session] {
        assert_eq!(
            session
                .native_session
                .as_ref()
                .map(|native| native.id.as_str()),
            Some(CONTINUATION_SESSION)
        );
    }
    let parent_sha256 = formal_ai::orchestration::session_sha256(&parent).unwrap();
    let corrected_sha256 = formal_ai::orchestration::session_sha256(&corrected).unwrap();
    assert_eq!(
        corrected
            .continuation
            .as_ref()
            .map(|continuation| continuation.parent_session_sha256.as_str()),
        Some(parent_sha256.as_str())
    );
    assert_eq!(
        final_session
            .continuation
            .as_ref()
            .map(|continuation| continuation.parent_session_sha256.as_str()),
        Some(corrected_sha256.as_str())
    );

    let resume_position = final_session
        .args
        .windows(2)
        .position(|args| args == ["--orchestration-resume", CONTINUATION_SESSION])
        .expect("the Formal AI wrapper receives the recorded Agent session");
    let prompt_position = final_session
        .args
        .iter()
        .position(|arg| arg == "agent")
        .expect("the controlled client is explicit");
    assert!(resume_position < prompt_position);
    assert!(final_session
        .stdout
        .contains(&format!("--resume {CONTINUATION_SESSION} --no-fork -p")));
    assert_eq!(
        final_session
            .native_session
            .as_ref()
            .map(|native| native.resume_command.as_str()),
        Some("agent --resume ses_04e25ba4cffeibfMekv188DNLX --no-fork")
    );

    let artifact = include_bytes!("../../data/meta/orchestration-continuation-invariant.lino");
    assert_eq!(
        artifact,
        b"orchestration_continuation resume_exact_native_id."
    );
    let change = final_session
        .changes
        .iter()
        .find(|change| change.path == "data/meta/orchestration-continuation-invariant.lino")
        .expect("the final correction records the self-authored artifact");
    assert_eq!(
        change.after_sha256.as_deref(),
        Some("1a94da829c1803f18ea2bf0b16f93ec30c5edc1fa1c02936afb2383b467ec439")
    );
}

#[test]
fn committed_parallel_comparison_ledger_records_the_selected_winner() {
    let ledger: ComparisonLedger = serde_json::from_slice(include_bytes!(
        "../../docs/case-studies/issue-703/comparison/comparison-ledger.json"
    ))
    .unwrap();

    assert_eq!(ledger.schema, "formal-ai-comparison-ledger-v1");
    assert_eq!(ledger.entries.len(), 2);
    assert!(ledger.entries.iter().all(|entry| entry.passed));
    assert_eq!(ledger.winner.as_deref(), Some("codex"));
    assert_eq!(
        ComparisonLedger::select_winner(&ledger.entries),
        ledger.winner
    );
    for bytes in [
        include_bytes!("../../docs/case-studies/issue-703/comparison/sessions/000-codex.json")
            .as_slice(),
        include_bytes!("../../docs/case-studies/issue-703/comparison/sessions/001-claude.json")
            .as_slice(),
    ] {
        let session = replay_session(bytes).unwrap();
        assert!(session.passed());
        assert_eq!(session.verification.len(), 1);
        assert_eq!(session.verification[0].program, "test");
    }
}

#[test]
fn timeout_is_recorded_without_an_implicit_retry() {
    let workspace = TestWorkspace::new("timeout");
    let mut config = fixture_config("codex", workspace.path(), "timeout");
    config.timeout = Duration::from_millis(20);

    let session = run_agent(&config).expect("timeout is a recorded agent result");

    assert_eq!(session.status, AgentStatus::TimedOut);
    assert_eq!(
        session
            .events
            .iter()
            .filter(|event| event.kind == "process_started")
            .count(),
        1
    );
}

#[cfg(unix)]
#[test]
fn timeout_terminates_descendant_processes() {
    let workspace = TestWorkspace::new("descendant-timeout");
    let mut config = fixture_config("codex", workspace.path(), "descendant_timeout");
    config.timeout = Duration::from_millis(20);

    let session = run_agent(&config).expect("timeout is recorded");
    std::thread::sleep(Duration::from_millis(250));

    assert_eq!(session.status, AgentStatus::TimedOut);
    assert!(!workspace.path().join("descendant-survived").exists());
}

#[test]
fn verification_commands_must_be_explicitly_allowlisted() {
    let workspace = TestWorkspace::new("allowlist");
    let mut config = fixture_config("codex", workspace.path(), "success");
    config.verification.push(VerificationCommand::new(
        "unreviewed-command",
        std::iter::empty::<String>(),
    ));

    let error = run_agent(&config).expect_err("unreviewed command must not execute");

    assert!(matches!(
        error,
        AgentRunError::CommandNotAllowlisted(command) if command == "unreviewed-command"
    ));
}

#[test]
fn verification_timeout_is_recorded_and_fails_the_session() {
    let workspace = TestWorkspace::new("verification-timeout");
    let mut config = fixture_config("codex", workspace.path(), "success");
    let test_binary = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    config.timeout = Duration::from_millis(20);
    config.allowlisted_commands.insert(test_binary.clone());
    config.verification.push(VerificationCommand::new(
        test_binary,
        [
            "--ignored",
            "--exact",
            "issue_703_orchestration::external_slow_verification_fixture",
        ],
    ));

    let session = run_agent(&config).expect("verification timeout is recorded");

    assert!(!session.passed());
    assert_eq!(session.verification.len(), 1);
    assert!(session.verification[0].timed_out);
    assert!(session
        .events
        .iter()
        .any(|event| event.kind == "verification_started"));
    assert!(session
        .events
        .iter()
        .any(|event| event.kind == "verification_timed_out"));
}

#[test]
fn failed_external_process_is_visible_in_the_session() {
    let workspace = TestWorkspace::new("failed");
    let config = fixture_config("agent", workspace.path(), "failed");

    let session = run_agent(&config).expect("non-zero status is evidence, not a controller error");

    assert_eq!(session.status, AgentStatus::Failed);
    assert_eq!(session.exit_code, Some(7));
    assert_eq!(
        session
            .events
            .iter()
            .filter(|event| event.kind == "process_started")
            .count(),
        1
    );
}

#[test]
fn recorded_session_replays_byte_for_byte() {
    let workspace = TestWorkspace::new("replay");
    let session = run_agent(&fixture_config("gemini", workspace.path(), "success")).unwrap();
    let path = workspace.path().join("session.json");
    write_session(&path, &session).unwrap();
    let bytes = fs::read(path).unwrap();

    assert_eq!(replay_session(&bytes).unwrap(), session);
}

#[test]
fn replay_rejects_valid_but_noncanonical_json() {
    let workspace = TestWorkspace::new("noncanonical-replay");
    let session = run_agent(&fixture_config("gemini", workspace.path(), "success")).unwrap();
    let bytes = serde_json::to_vec(&session).unwrap();

    assert!(matches!(
        replay_session(&bytes),
        Err(ReplayError::NonCanonical)
    ));
}

#[test]
fn parallel_comparison_records_a_ledger_and_composes_the_winner() {
    let workspace = TestWorkspace::new("compare");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string(), "claude".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides = fixture_commands(&["codex", "claude"], "success");
    grant_fixture_agent_command(&mut config);

    let report = dispatch_agents(&config).expect("parallel comparison");

    assert_eq!(report.sessions.len(), 2);
    assert_eq!(report.ledger.entries.len(), 2);
    assert!(report.ledger.winner.is_some());
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "fixture change\n"
    );
    assert!(config.output_dir.join("comparison-ledger.json").is_file());
}

#[test]
fn council_results_are_formalized_summarized_and_cross_checked() {
    let first_workspace = TestWorkspace::new("synthesis-first");
    let second_workspace = TestWorkspace::new("synthesis-second");
    let first = fixture_config_with_output(
        "codex",
        first_workspace.path(),
        "Rust is memory safe. The Moon is cheese.",
    );
    let second = fixture_config_with_output(
        "claude",
        second_workspace.path(),
        "Rust is memory safe. The Moon is not cheese.",
    );
    let sessions = [
        run_agent(&first).expect("first source"),
        run_agent(&second).expect("second source"),
    ];

    let mut report = synthesize_sessions(&sessions, "ru").expect("deterministic synthesis");

    assert_eq!(report.schema, "formal-ai-agent-synthesis-v1");
    assert_eq!(report.sources.len(), 2);
    assert!(report
        .sources
        .iter()
        .all(|source| source.meta_language.starts_with("formalization_candidate")));
    assert!(report
        .claims
        .iter()
        .any(|claim| claim.text.contains("Rust is memory safe") && claim.presented));
    assert!(!report.contradictions.is_empty());
    assert!(!report.corrections.is_empty());
    assert_eq!(report.fact_check_scope, "cross_agent_evidence_preflight");
    assert!(report.translation_required);

    let wrong_language = apply_verified_translation(
        &mut report,
        "Rust is memory safe.",
        "translator-session-sha256",
    )
    .expect_err("a translation in the wrong language must not be presented");
    assert_eq!(
        wrong_language.to_string(),
        "translation_language_mismatch:ru:en"
    );

    apply_verified_translation(
        &mut report,
        "Rust обеспечивает безопасность памяти.",
        "translator-session-sha256",
    )
    .expect("Russian translation");
    assert_eq!(report.final_language, "ru");
    assert!(!report.translation_required);
    assert!(report
        .translation
        .as_ref()
        .is_some_and(|translation| translation.session_sha256 == "translator-session-sha256"));
}

#[test]
fn synthesis_extracts_the_final_answer_without_promoting_agent_diagnostics() {
    let stream = concat!(
        "{\n",
        "  \"type\": \"log\",\n",
        "  \"message\": \"diagnostic should not become a claim\"\n",
        "}\n",
        "{\"type\":\"message\",\"role\":\"assistant\",\"content\":",
        "[{\"type\":\"text\",\"text\":\"First draft.\"}]}\n",
        "{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",",
        "\"text\":\"Final supported answer.\"}}\n",
        "{\"type\":\"result\",\"status\":\"success\"}\n",
    );

    assert_eq!(extract_agent_result(stream), "Final supported answer.");
    assert_eq!(
        extract_agent_result(&format!(
            "supervisor starting\n{stream}supervisor finished\n"
        )),
        "Final supported answer."
    );
    assert_eq!(
        extract_agent_result(concat!(
            "test external_agent_fixture_process ... ",
            "{\"type\":\"message\",\"role\":\"assistant\",\"content\":",
            "[{\"type\":\"text\",\"text\":\"Embedded answer.\"}]} ok\n",
        )),
        "Embedded answer."
    );
    assert_eq!(
        extract_agent_result("Plain agent answer.\n"),
        "Plain agent answer."
    );
}

#[test]
fn public_cli_synthesizes_translates_and_proposes_learned_adapter_updates() {
    let first_workspace = TestWorkspace::new("cli-synthesis-first");
    let second_workspace = TestWorkspace::new("cli-synthesis-second");
    let translator_workspace = TestWorkspace::new("cli-synthesis-translator");
    let mut first_config =
        fixture_config_with_output("codex", first_workspace.path(), "Rust is memory safe.");
    first_config.task = "inspect the repository".to_string();
    let first = run_agent(&first_config).unwrap();
    let mut second_config =
        fixture_config_with_output("codex", second_workspace.path(), "Rust is memory safe.");
    second_config.task = "review all project files".to_string();
    let second = run_agent(&second_config).unwrap();
    let translator = run_agent(&fixture_config_with_output(
        "qwen",
        translator_workspace.path(),
        "{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"Rust обеспечивает безопасность памяти.\"}]}",
    ))
    .unwrap();
    let extracted_translation = extract_agent_result(&translator.stdout);
    assert_eq!(
        extracted_translation, "Rust обеспечивает безопасность памяти.",
        "the recorded translator session must preserve its assistant payload"
    );
    assert_eq!(
        formal_ai::detect_language(&extracted_translation).slug(),
        "ru",
        "the translator payload must pass the same language gate as the CLI"
    );
    let first_path = first_workspace.path().join("first.json");
    let second_path = second_workspace.path().join("second.json");
    let translator_path = translator_workspace.path().join("translator.json");
    write_session(&first_path, &first).unwrap();
    write_session(&second_path, &second).unwrap();
    write_session(&translator_path, &translator).unwrap();

    let synthesis = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["agent", "synthesize"])
        .arg(&first_path)
        .arg(&second_path)
        .args(["--response-language", "ru", "--translation-session"])
        .arg(&translator_path)
        .output()
        .unwrap();
    assert!(
        synthesis.status.success(),
        "{}",
        String::from_utf8_lossy(&synthesis.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&synthesis.stdout).unwrap();
    assert_eq!(report["final_language"], "ru");
    assert_eq!(
        report["final_answer"],
        "Rust обеспечивает безопасность памяти."
    );
    assert_eq!(
        report["translation"]["session_sha256"]
            .as_str()
            .map(str::len),
        Some(64)
    );

    let learning = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["agent", "learn"])
        .arg(&first_path)
        .arg(&second_path)
        .output()
        .unwrap();
    assert!(
        learning.status.success(),
        "{}",
        String::from_utf8_lossy(&learning.stderr)
    );
    assert!(
        String::from_utf8_lossy(&learning.stdout).contains("decision \"awaiting_human_review\"")
    );
}

#[test]
fn repeated_orchestration_sessions_feed_human_gated_adapter_learning() {
    let first_workspace = TestWorkspace::new("learning-first");
    let second_workspace = TestWorkspace::new("learning-second");
    let mut first = fixture_config("codex", first_workspace.path(), "success");
    first.task = "inspect the repository".to_string();
    let mut second = fixture_config("codex", second_workspace.path(), "success");
    second.task = "review all project files".to_string();
    let first = run_agent(&first).unwrap();
    let second = run_agent(&second).unwrap();
    let observations = [
        observe_orchestration_session(&first, "sessions/first.json"),
        observe_orchestration_session(&second, "sessions/second.json"),
    ];

    let report =
        formal_ai::learn_client_contracts(&observations, &formal_ai::seed::client_integrations());

    assert!(report.awaiting_human_review);
    assert!(report
        .proposals
        .iter()
        .any(|proposal| proposal.field == "orchestration_target"));
    assert!(report
        .links_notation()
        .contains("decision \"awaiting_human_review\""));
}

#[test]
fn a_proven_false_claim_resumes_the_same_native_session_with_parent_evidence() {
    let workspace = TestWorkspace::new("native-resume");
    let mut initial =
        fixture_config_with_output("agent", workspace.path(), "The release date is 2027.");
    initial.command_override = Some(
        initial
            .command_override
            .take()
            .unwrap()
            .env(FIXTURE_ENV, "native_session"),
    );
    let parent = run_agent(&initial).expect("initial native session");
    assert_eq!(
        parent
            .native_session
            .as_ref()
            .map(|session| session.id.as_str()),
        Some("ses_issue_703")
    );

    let request = formal_ai::orchestration::CorrectionRequest {
        cli: "agent".to_string(),
        claim: "The release date is 2027.".to_string(),
        evidence: "The signed release record says 2026.".to_string(),
        source_session_sha256: formal_ai::orchestration::session_sha256(&parent)
            .expect("parent digest"),
    };
    let mut resumed = fixture_config_with_output(
        "agent",
        workspace.path(),
        "The corrected release date is 2026.",
    );
    resumed.task = "The corrected release date is 2026.".to_string();
    resumed.command_override = Some(
        resumed
            .command_override
            .take()
            .unwrap()
            .env(FIXTURE_ENV, "success"),
    );
    let corrected = resume_agent(&parent, &request, resumed).expect("resume correction");

    let continuation = corrected
        .continuation
        .as_ref()
        .expect("continuation provenance");
    assert_eq!(continuation.native_session_id, "ses_issue_703");
    assert_eq!(
        continuation.parent_session_sha256,
        request.source_session_sha256
    );
    assert_eq!(continuation.disproved_claim, request.claim);
    assert_eq!(continuation.evidence, request.evidence);
    assert!(
        corrected
            .task
            .contains("The corrected release date is 2026."),
        "{:?}",
        corrected.task
    );
    assert!(corrected.task.contains("The release date is 2027."));
    assert!(corrected
        .task
        .contains("The signed release record says 2026."));
    assert!(
        corrected
            .task
            .ends_with("Complete this correction goal: The corrected release date is 2026."),
        "the task must remain last so literal task parsers cannot consume correction evidence"
    );
    assert!(corrected
        .events
        .iter()
        .any(|event| event.kind == "native_session_resumed"));
}

#[test]
fn continuation_rejects_a_client_that_reports_a_different_native_session() {
    let workspace = TestWorkspace::new("native-resume-mismatch");
    let mut initial = fixture_config("agent", workspace.path(), "native_session");
    initial.task = "state the release date".to_string();
    let parent = run_agent(&initial).expect("initial native session");
    let request = formal_ai::orchestration::CorrectionRequest {
        cli: "agent".to_string(),
        claim: "The release date is 2027.".to_string(),
        evidence: "The signed release record says 2026.".to_string(),
        source_session_sha256: formal_ai::orchestration::session_sha256(&parent).unwrap(),
    };
    let resumed = fixture_config("agent", workspace.path(), "mismatched_native_session");

    let error =
        resume_agent(&parent, &request, resumed).expect_err("a fresh session must fail closed");

    assert!(matches!(error, AgentRunError::ContinuationMismatch));
}

#[test]
fn comparison_rejects_duplicate_cli_identities_before_creating_candidates() {
    let workspace = TestWorkspace::new("duplicate-cli");
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string(), "codex".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .command_overrides
        .insert("codex".to_string(), fixture_command("success"));
    grant_fixture_agent_command(&mut config);

    let error = dispatch_agents(&config).expect_err("duplicate identities are ambiguous");

    assert_eq!(error.to_string(), "duplicate_cli:codex");
    assert!(!config.output_dir.exists());
}

#[test]
fn comparison_refuses_to_overwrite_workspace_drift_after_candidates_fork() {
    let workspace = TestWorkspace::new("workspace-drift");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    let candidate_started = workspace.path().join("candidate-started");
    let release_candidate = workspace.path().join("release-candidate");
    let candidate_started_env = candidate_started.display().to_string();
    let release_candidate_env = release_candidate.display().to_string();
    config.command_overrides.insert(
        "codex".to_string(),
        fixture_command("coordinated_success")
            .env(FIXTURE_STARTED_ENV, candidate_started_env)
            .env(FIXTURE_RELEASE_ENV, release_candidate_env),
    );
    grant_fixture_agent_command(&mut config);
    let original = workspace.path().join("README.md");
    let concurrent_update = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !candidate_started.exists() {
            assert!(
                Instant::now() < deadline,
                "candidate fixture did not signal startup"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        fs::write(original, "external update\n").unwrap();
        fs::write(release_candidate, "release\n").unwrap();
    });

    let error = dispatch_agents(&config).expect_err("composition must detect workspace drift");
    concurrent_update.join().unwrap();

    assert!(error.to_string().contains("workspace_drift:README.md"));
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "external update\n"
    );
}

#[test]
fn custom_output_directory_inside_workspace_is_excluded_from_candidate_copies() {
    let workspace = TestWorkspace::new("nested-output");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.output_dir = workspace.path().join("agent-artifacts");
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides = fixture_commands(&["codex"], "success");
    grant_fixture_agent_command(&mut config);

    let report = dispatch_agents(&config).expect("nested output stays out of candidate snapshots");

    assert_eq!(report.sessions.len(), 1);
    assert!(config.output_dir.join("comparison-ledger.json").is_file());
    assert!(!config
        .output_dir
        .join("candidates/000-codex/agent-artifacts")
        .exists());
}

#[test]
fn dispatch_output_cannot_escape_the_granted_workspace() {
    let workspace = TestWorkspace::new("output-boundary");
    let outside = TestWorkspace::new("outside-output");
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string()],
    );
    config.output_dir = outside.path().join("agent-artifacts");
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .command_overrides
        .insert("codex".to_string(), fixture_command("success"));
    grant_fixture_agent_command(&mut config);

    let error = dispatch_agents(&config).unwrap_err();

    assert_eq!(error.to_string(), "output_outside_workspace");
    assert!(!config.output_dir.exists());
}

#[test]
fn dispatch_joins_every_worker_before_returning_an_error() {
    let workspace = TestWorkspace::new("join-on-error");
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string(), "claude".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides.insert(
        "codex".to_string(),
        AgentCommand::new(workspace.path().join("missing-program")),
    );
    config
        .command_overrides
        .insert("claude".to_string(), fixture_command("delayed_success"));
    grant_fixture_agent_command(&mut config);
    config.allowlisted_agent_commands.insert(
        workspace
            .path()
            .join("missing-program")
            .display()
            .to_string(),
    );
    let started = Instant::now();

    let error = dispatch_agents(&config).expect_err("one worker cannot be started");

    assert!(error.to_string().contains("process:"));
    assert!(started.elapsed() >= Duration::from_millis(100));
    assert_eq!(
        fs::read_to_string(config.output_dir.join("candidates/001-claude/README.md")).unwrap(),
        "fixture change\n"
    );
}

#[test]
fn universal_decomposition_dispatches_independent_leaves_in_parallel() {
    let workspace = TestWorkspace::new("decompose");
    let mut config = DispatchConfig::new(
        "Create a README badge and add a release note.",
        workspace.path(),
        vec!["codex".to_string(), "opencode".to_string()],
    );
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides = fixture_commands(&["codex", "opencode"], "success");
    grant_fixture_agent_command(&mut config);

    let report = dispatch_agents(&config).expect("decomposed dispatch");

    assert!(report.tasks.len() >= 2, "{:?}", report.tasks);
    assert_eq!(report.tasks.len(), report.sessions.len());
    assert!(!report.composed_changes.is_empty());
}

#[test]
fn decomposition_never_composes_changes_from_a_failed_agent() {
    let workspace = TestWorkspace::new("failed-decomposition");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "Create a README badge and add a release note.",
        workspace.path(),
        vec!["codex".to_string(), "opencode".to_string()],
    );
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .command_overrides
        .insert("codex".to_string(), fixture_command("success"));
    config
        .command_overrides
        .insert("opencode".to_string(), fixture_command("failed_change"));
    grant_fixture_agent_command(&mut config);

    let report = dispatch_agents(&config).expect("failed candidate stays isolated");

    assert!(report.sessions.iter().any(|session| !session.passed()));
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "fixture change\n"
    );
}

#[test]
fn winner_selection_is_deterministic_for_a_recorded_ledger() {
    let entries = vec![
        ComparisonEntry {
            cli: "codex".to_string(),
            task: "task".to_string(),
            passed: true,
            diff_size: 20,
            wall_time_ms: 10,
            session_file: "codex.json".to_string(),
        },
        ComparisonEntry {
            cli: "claude".to_string(),
            task: "task".to_string(),
            passed: true,
            diff_size: 10,
            wall_time_ms: 50,
            session_file: "claude.json".to_string(),
        },
        ComparisonEntry {
            cli: "agent".to_string(),
            task: "task".to_string(),
            passed: false,
            diff_size: 1,
            wall_time_ms: 1,
            session_file: "agent.json".to_string(),
        },
    ];

    assert_eq!(
        ComparisonLedger::select_winner(&entries).as_deref(),
        Some("claude")
    );
    assert_eq!(ComparisonLedger::select_winner(&entries[2..]), None);
}
