use super::*;

pub(super) fn council_results_are_formalized_summarized_and_cross_checked() {
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
    assert!(
        report
            .sources
            .iter()
            .all(|source| source.meta_language.starts_with("formalization_candidate"))
    );
    assert!(
        report
            .claims
            .iter()
            .any(|claim| claim.text.contains("Rust is memory safe") && claim.presented)
    );
    // R703-9: a claim another agent denied is withheld, not presented as fact.
    assert!(
        report
            .claims
            .iter()
            .any(|claim| !claim.presented && !claim.denied_by.is_empty()),
        "{:?}",
        report.claims
    );
    assert_ne!(
        report.contradictions,
        [] as [formal_ai::orchestration::AgentSynthesisContradiction; 0]
    );
    assert_ne!(
        report.corrections,
        [] as [formal_ai::orchestration::CorrectionRequest; 0]
    );
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
    assert!(
        report
            .translation
            .as_ref()
            .is_some_and(|translation| translation.session_sha256 == "translator-session-sha256")
    );
}

pub(super) fn a_proven_false_claim_resumes_the_same_native_session_with_parent_evidence() {
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
    assert!(
        corrected
            .task
            .contains("The signed release record says 2026.")
    );
    assert!(
        corrected
            .task
            .ends_with("Complete this correction goal: The corrected release date is 2026."),
        "the task must remain last so literal task parsers cannot consume correction evidence"
    );
    assert!(
        corrected
            .events
            .iter()
            .any(|event| event.kind == "native_session_resumed")
    );
}
