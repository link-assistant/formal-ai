//! Issue #873 — an unresolved request is a research trigger, not a terminal
//! `unknown` answer, whenever external research is permitted.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::language::registered_languages;
use formal_ai::orchestration::AgentRunConfig;
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::research_learning::{
    AutonomyMode, CycleConfig, CycleState, DEFAULT_RESEARCH_TIME_LIMIT_SECONDS, KnowledgeKind,
    RESEARCH_LEARNING_RECIPE, RecoveryDecision, RecoveryOption, ResearchLearningCycle,
    VerificationGate, VersionStatus, recipe_steps,
};
use formal_ai::{SolverConfig, UniversalSolver};
use std::time::Duration;

fn cycle(autonomy: AutonomyMode) -> ResearchLearningCycle {
    ResearchLearningCycle::new(
        RESEARCH_LEARNING_RECIPE,
        ["baseline"],
        CycleConfig {
            autonomy,
            ..Default::default()
        },
    )
}

#[test]
fn online_solver_researches_unknown_in_every_registered_language() {
    let cases = [
        ("en", "Calibrate the snorflax against silent teal weather"),
        ("ru", "Откалибруй снорфлакс для тихой бирюзовой погоды"),
        ("hi", "शांत नीले मौसम के लिए स्नोरफ्लैक्स को कैलिब्रेट करो"),
        ("zh", "请 校准 斯诺弗拉克斯 适应 安静 青色 天气"),
        (
            "es",
            "Calibra el snorflax para el clima turquesa silencioso",
        ),
    ];
    assert_eq!(cases.len(), registered_languages().len());

    for (language, prompt) in cases {
        let response = UniversalSolver::new(SolverConfig::default()).solve(prompt);

        assert_eq!(
            response.intent, "web_search",
            "[{language}] {}",
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:unknown_reasoning_fallback"),
            "[{language}] {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn agentic_client_searches_an_unknown_instruction_without_question_punctuation() {
    let messages = vec![ChatMessage::user(
        "Calibrate the snorflax against silent teal weather",
    )];
    let plan = plan_chat_step(&messages, &["websearch", "webfetch"])
        .expect("an online unknown should produce a research step");

    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("online unknown must not end in a local final answer");
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "websearch");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&calls[0].arguments).unwrap()["query"],
        "Calibrate the snorflax against silent teal weather"
    );
}

#[test]
fn offline_mode_preserves_the_explicit_no_network_boundary() {
    let response = UniversalSolver::new(SolverConfig {
        offline: true,
        ..Default::default()
    })
    .solve("Calibrate the snorflax against silent teal weather");

    assert_eq!(response.intent, "unknown");
    assert!(
        response
            .links_notation
            .contains("reasoning:candidate_source allowed_external_api:skipped_offline"),
        "{}",
        response.links_notation
    );
}

#[test]
fn external_payloads_are_disposable_but_receipts_are_versioned() {
    let mut cycle = cycle(AutonomyMode::AskOnAmbiguity);
    cycle.begin_unknown("snorflax calibration");
    let first = cycle.record_source("https://example.invalid/v1", "observation-v1", true);

    assert!(cycle.evict_source(&first));
    let receipt = cycle
        .sources()
        .iter()
        .find(|source| source.id == first)
        .unwrap();
    assert!(receipt.cached_payload.is_none());
    assert!(!receipt.locator.is_empty());
    assert!(!receipt.content_id.is_empty());

    assert!(!cycle.recollect_source(&first, "changed observation"));
    assert!(cycle.recollect_source(&first, "observation-v1"));
    assert_eq!(
        cycle
            .sources()
            .iter()
            .find(|source| source.id == first)
            .unwrap()
            .cached_payload
            .as_deref(),
        Some("observation-v1")
    );

    let second = cycle.record_source("https://example.invalid/v1", "observation-v2", true);
    assert_ne!(first, second);
    assert_eq!(cycle.sources().len(), 2);
}

#[test]
fn researched_unknown_returns_a_grounded_answer_after_search_and_fetch() {
    let prompt = "Calibrate the snorflax against silent teal weather";
    let mut messages = vec![ChatMessage::user(prompt)];
    let AgenticPlan::ToolCalls(searches) =
        plan_chat_step(&messages, &["websearch", "webfetch"]).unwrap()
    else {
        panic!("the unresolved instruction must start research");
    };
    let search = &searches[0];
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "search-1",
        search.tool.clone(),
        search.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "search-1",
        search.tool.clone(),
        "Calibration reference https://standards.example/snorflax",
    ));

    let AgenticPlan::ToolCalls(fetches) =
        plan_chat_step(&messages, &["websearch", "webfetch"]).unwrap()
    else {
        panic!("search discovery must be followed by source capture");
    };
    let fetch = &fetches[0];
    assert_eq!(fetch.tool, "webfetch");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "fetch-1",
        fetch.tool.clone(),
        fetch.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "fetch-1",
        fetch.tool.clone(),
        "Snorflax calibration uses a teal reference and verifies silent tolerance.",
    ));

    let AgenticPlan::Final(answer) = plan_chat_step(&messages, &["websearch", "webfetch"]).unwrap()
    else {
        panic!("captured evidence must produce an answer");
    };
    assert!(answer.contains("teal reference"), "{answer}");
    assert!(
        answer.contains("https://standards.example/snorflax"),
        "{answer}"
    );
}

#[test]
fn failing_candidate_never_replaces_the_previous_stable_version() {
    let mut cycle = cycle(AutonomyMode::AskOnAmbiguity);
    let root = cycle.active_version().id.clone();
    let candidate = cycle.propose_version(KnowledgeKind::Procedure, "candidate procedure");

    assert!(!cycle.verify_candidate(
        &candidate,
        vec![
            VerificationGate::immutable("baseline", true),
            VerificationGate::immutable("compile", false),
            VerificationGate::adaptive("candidate-example", true),
        ],
    ));
    assert_eq!(cycle.active_version().id, root);
    assert_eq!(cycle.state(), CycleState::Recovering);
    assert_eq!(
        cycle
            .versions()
            .iter()
            .find(|version| version.id == candidate)
            .unwrap()
            .status,
        VersionStatus::Rejected
    );
}

#[test]
fn tested_version_promotes_and_any_prior_stable_version_can_be_recovered() {
    let mut cycle = cycle(AutonomyMode::AskOnAmbiguity);
    let root = cycle.active_version().id.clone();
    let candidate = cycle.propose_version(KnowledgeKind::Fact, "grounded lesson");

    assert!(cycle.verify_candidate(
        &candidate,
        vec![
            VerificationGate::immutable("baseline", true),
            VerificationGate::immutable("regression", true),
            VerificationGate::adaptive("new-case", true),
        ],
    ));
    assert_eq!(cycle.active_version().id, candidate);
    assert!(cycle.recover_stable(&root));
    assert_eq!(cycle.active_version().id, root);
}

#[test]
fn mutable_or_incomplete_test_suites_cannot_promote_memory() {
    let mut cycle = cycle(AutonomyMode::AskOnAmbiguity);
    let candidate = cycle.propose_version(KnowledgeKind::Fact, "under-tested lesson");

    assert!(!cycle.verify_candidate(
        &candidate,
        vec![
            VerificationGate::adaptive("baseline", true),
            VerificationGate::adaptive("new-case", true),
        ],
    ));

    let mut no_baseline = ResearchLearningCycle::new(
        "stable",
        std::iter::empty::<String>(),
        CycleConfig::default(),
    );
    let candidate = no_baseline.propose_version(KnowledgeKind::Fact, "unanchored lesson");
    assert!(!no_baseline.verify_candidate(
        &candidate,
        vec![VerificationGate::immutable("unconfigured", true)],
    ));
}

fn recovery_options() -> Vec<RecoveryOption> {
    vec![
        RecoveryOption {
            id: "retry_same_source".to_owned(),
            prior_successes: 1,
            prior_failures: 3,
            advantages: 1,
            disadvantages: 4,
        },
        RecoveryOption {
            id: "widen_sources".to_owned(),
            prior_successes: 4,
            prior_failures: 1,
            advantages: 3,
            disadvantages: 1,
        },
    ]
}

#[test]
fn ambiguous_recovery_asks_the_user_and_full_trust_uses_outcome_history() {
    let mut ask = cycle(AutonomyMode::AskOnAmbiguity);
    assert_eq!(
        ask.recover_from_error("provider_empty", recovery_options()),
        RecoveryDecision::AskUser {
            option_ids: vec!["widen_sources".to_owned(), "retry_same_source".to_owned()]
        }
    );
    assert!(ask.select_recovery("retry_same_source"));

    let mut trusted = cycle(AutonomyMode::FullTrust);
    assert_eq!(
        trusted.recover_from_error("provider_empty", recovery_options()),
        RecoveryDecision::Selected {
            option_id: "widen_sources".to_owned()
        }
    );
}

#[test]
fn per_command_mode_requires_permission_and_every_error_has_a_recovery() {
    let mut cycle = cycle(AutonomyMode::PerCommand);
    assert_eq!(
        cycle.recover_from_error("unexpected_error", Vec::new()),
        RecoveryDecision::PermissionRequired {
            option_id: "restore_stable_and_research".to_owned()
        }
    );
    assert_eq!(cycle.state(), CycleState::AwaitingPermission);
}

#[test]
fn default_one_hour_limit_returns_the_current_plan_for_continuation() {
    let mut cycle = cycle(AutonomyMode::AskOnAmbiguity);
    assert_eq!(
        cycle.config().time_limit_seconds,
        DEFAULT_RESEARCH_TIME_LIMIT_SECONDS
    );
    assert!(
        cycle
            .check_time_limit(DEFAULT_RESEARCH_TIME_LIMIT_SECONDS - 1, "fetch next source")
            .is_none()
    );
    assert_eq!(
        cycle.check_time_limit(DEFAULT_RESEARCH_TIME_LIMIT_SECONDS, "fetch next source"),
        Some(RecoveryDecision::AwaitingContinuation {
            current_plan: "fetch next source".to_owned()
        })
    );
    cycle.continue_with_permission(300);
    assert_eq!(cycle.state(), CycleState::Researching);
    assert_eq!(
        cycle.config().time_limit_seconds,
        DEFAULT_RESEARCH_TIME_LIMIT_SECONDS + 300
    );
}

#[test]
fn orchestration_uses_the_same_configurable_one_hour_default() {
    let config = AgentRunConfig::new("agent", "research task", ".");
    assert_eq!(
        config.timeout,
        Duration::from_secs(DEFAULT_RESEARCH_TIME_LIMIT_SECONDS)
    );
}

#[test]
fn one_data_recipe_drives_the_cycle_and_can_itself_be_versioned() {
    assert_eq!(
        recipe_steps(),
        [
            "inspect_local_state",
            "research_external_sources",
            "capture_recomputable_evidence",
            "propose_version",
            "verify_immutable_baseline",
            "promote_or_restore_stable",
            "recover_and_continue",
        ]
    );

    let mut cycle = cycle(AutonomyMode::FullTrust);
    let candidate = cycle.propose_version(KnowledgeKind::MetaAlgorithm, "appended recipe step");
    assert!(cycle.verify_candidate(
        &candidate,
        vec![
            VerificationGate::immutable("baseline", true),
            VerificationGate::immutable("recipe-replay", true),
            VerificationGate::adaptive("new-step", true),
        ]
    ));
    assert_eq!(cycle.active_version().kind, KnowledgeKind::MetaAlgorithm);
}

#[test]
fn cycle_history_is_hash_linked_and_rendered_for_replay() {
    let mut cycle = cycle(AutonomyMode::FullTrust);
    cycle.begin_unknown("unresolved task");
    cycle.record_source("local://workspace", "evidence", false);

    for pair in cycle.events().windows(2) {
        assert_eq!(pair[1].previous_id, pair[0].id);
    }
    let links = cycle.links_notation();
    assert!(links.contains("research_learning_cycle"));
    assert!(links.contains("source_receipt"));
    assert!(links.contains("unknown_frontier"));
}

#[test]
fn browser_worker_already_runs_the_same_unknown_research_before_fallback() {
    const WORKER: &str = include_str!("../../src/web/worker/formal_ai_worker_20.js");
    let research = WORKER.find("unknown_intent_research").unwrap();
    let fallback = WORKER.find("fallback:unknown").unwrap();
    assert!(research < fallback);
}

#[test]
fn formal_ai_and_the_real_agent_cli_authored_one_of_five_smallest_leaves() {
    const FORMAL_AI_AUTHORED_LEAVES: usize = 1;
    const SMALLEST_REQUIREMENT_LEAVES: usize = 5;
    const GENERATED: &[u8] = include_bytes!(
        "../../docs/case-studies/issue-873/self-hosting-authorship/research-learning-recovery-invariant.lino"
    );
    const CANONICAL: &[u8] =
        include_bytes!("../../data/meta/research-learning-recovery-invariant.lino");

    assert_eq!(GENERATED, CANONICAL);
    assert_eq!(
        FORMAL_AI_AUTHORED_LEAVES * 100 / SMALLEST_REQUIREMENT_LEAVES,
        20
    );
    assert!(
        include_str!("../../docs/case-studies/issue-873/self-hosting-authorship/agent-cli.log")
            .contains("ses_01c561b3bffeG2Bl3eBvtDHYzq")
    );
}
