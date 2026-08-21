//! Issue #709 learning regressions: fusion procedures are inferred from
//! successful executions, inert before review, durable, and reusable.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::{
    CachedSourceClient, FetchError, SEARCH_FUSION_TASK_FAMILY, SearchFusionLearningApproval,
    SearchFusionLearningFrontier, SearchFusionLearningGate, SearchFusionRecipeLedger,
    SearchSourceClassification, SourceTier, SourceTransport, execute_search_fusion,
    execute_search_fusion_with_recipe,
};

use formal_ai::agentic_coding::learning_report::search_fusion_learning;
use formal_ai::agentic_coding::{REPORTS, SEARCH_FUSION_LEARNING_PATH, run_agentic_task};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
struct LearningTransport;

impl SourceTransport for LearningTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                r#"{"AbstractURL":"https://learning.invalid/original","Heading":"Primary handbook","AbstractText":"Apple is a fruit.","RelatedTopics":[{"FirstURL":"https://learning.invalid/report","Text":"Independent report - Яблоко это фрукт."}]}"#
                    .as_bytes()
                    .to_vec(),
            );
        }
        match url {
            "https://learning.invalid/original" => Ok(b"Apple is a fruit.\n".to_vec()),
            "https://learning.invalid/report" => Ok("Яблоко это фрукт.\n".as_bytes().to_vec()),
            _ => Err(FetchError::Transport(format!("fixture_missing:{url}"))),
        }
    }
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-709-learning-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn classify(url: &str) -> SearchSourceClassification {
    if url.ends_with("/original") {
        SearchSourceClassification::auto(SourceTier::OriginalFirstParty)
    } else {
        SearchSourceClassification::auto(SourceTier::IndependentCorroboration)
    }
}

#[test]
fn execution_frontier_infers_only_after_two_independent_successes() {
    let cache = temp_cache("frontier");
    let client = CachedSourceClient::new(&cache, LearningTransport).with_online(true);
    let first = execute_search_fusion(&client, "apple taxonomy", "en", 2, classify)
        .expect("first accepted execution");
    let second = execute_search_fusion(&client, "parser speed", "en", 2, classify)
        .expect("second accepted execution");
    let mut frontier = SearchFusionLearningFrontier::new();

    assert!(
        frontier
            .record_execution("training/apple-taxonomy", &first)
            .expect("record first execution")
            .is_none()
    );
    let candidate = frontier
        .record_execution("training/parser-speed", &second)
        .expect("record second execution")
        .expect("two independent runs infer a candidate");

    assert_eq!(candidate.task_family, SEARCH_FUSION_TASK_FAMILY);
    assert_eq!(candidate.evidence_count, 2);
    assert_eq!(candidate.stages.len(), 7);
    assert_eq!(frontier.observation_count(), 2);
    assert!(frontier.links_notation().contains(&candidate.id));

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn one_execution_cannot_be_counted_twice_under_different_task_ids() {
    let cache = temp_cache("duplicate-observation");
    let client = CachedSourceClient::new(&cache, LearningTransport).with_online(true);
    let execution = execute_search_fusion(&client, "apple taxonomy", "en", 2, classify)
        .expect("accepted execution");
    let mut frontier = SearchFusionLearningFrontier::new();

    assert!(
        frontier
            .record_execution("training/one", &execution)
            .expect("first observation")
            .is_none()
    );
    let duplicate = frontier
        .record_execution("training/two", &execution)
        .expect_err("renaming one execution must not make its evidence independent");

    assert_eq!(duplicate.reason, "learning_execution_not_independent");
    assert_eq!(frontier.observation_count(), 1);

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn candidate_is_inert_until_both_green_gate_and_named_review() {
    let cache = temp_cache("gates");
    let client = CachedSourceClient::new(&cache, LearningTransport).with_online(true);
    let execution = execute_search_fusion(&client, "apple taxonomy", "en", 2, classify)
        .expect("accepted execution");
    let second = execute_search_fusion(&client, "parser speed", "en", 2, classify)
        .expect("second accepted execution");
    let mut frontier = SearchFusionLearningFrontier::new();
    frontier
        .record_execution("training/one", &execution)
        .expect("record first execution");
    let candidate = frontier
        .record_execution("training/two", &second)
        .expect("record second execution")
        .expect("candidate");
    let mut ledger = SearchFusionRecipeLedger::new();

    assert!(ledger.plan_for(SEARCH_FUSION_TASK_FAMILY).is_none());
    assert!(
        ledger
            .promote(
                &candidate,
                SearchFusionLearningGate::failed("issue_709_held_out", 8, 1),
                SearchFusionLearningApproval::granted("pull_request_review"),
            )
            .is_err()
    );
    assert!(
        ledger
            .promote(
                &candidate,
                SearchFusionLearningGate::passed("issue_709_held_out", 9),
                SearchFusionLearningApproval::declined("pull_request_review"),
            )
            .is_err()
    );
    assert!(ledger.plan_for(SEARCH_FUSION_TASK_FAMILY).is_none());

    ledger
        .promote(
            &candidate,
            SearchFusionLearningGate::passed("issue_709_held_out", 9),
            SearchFusionLearningApproval::granted("pull_request_review"),
        )
        .expect("green reviewed candidate is promoted");
    assert_eq!(
        ledger
            .plan_for(SEARCH_FUSION_TASK_FAMILY)
            .expect("approved plan")
            .len(),
        7
    );

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn approved_recipe_round_trips_and_executes_a_held_out_task() {
    let cache = temp_cache("generalization");
    let client = CachedSourceClient::new(&cache, LearningTransport).with_online(true);
    let training = execute_search_fusion(&client, "apple taxonomy", "en", 2, classify)
        .expect("accepted execution");
    let second = execute_search_fusion(&client, "parser speed", "en", 2, classify)
        .expect("second accepted execution");
    let mut frontier = SearchFusionLearningFrontier::new();
    frontier
        .record_execution("training/apple", &training)
        .expect("first observation");
    let candidate = frontier
        .record_execution("training/parser", &second)
        .expect("second observation")
        .expect("candidate");
    let mut ledger = SearchFusionRecipeLedger::new();
    ledger
        .promote(
            &candidate,
            SearchFusionLearningGate::passed("issue_709_held_out", 9),
            SearchFusionLearningApproval::granted("pull_request_review"),
        )
        .expect("promote candidate");

    let durable = ledger.links_notation();
    let restored = SearchFusionRecipeLedger::from_links_notation(&durable)
        .expect("restore content-addressed ledger");
    assert_eq!(restored.links_notation(), durable);
    let held_out =
        execute_search_fusion_with_recipe(&restored, &client, "tomato taxonomy", "en", 2, classify)
            .expect("approved recipe executes unseen equivalent task");
    assert!(!held_out.answer.statements.is_empty());
    assert!(held_out.trace().contains("search_fusion:rank"));
    assert!(
        fs::read_to_string("data/benchmarks/search-fusion-learning-generalization.lino")
            .expect("Agent-authored held-out fixture")
            .contains("held_out tomato_taxonomy")
    );

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn agent_authored_policy_and_contract_are_runtime_inputs() {
    let contract = fs::read_to_string("data/meta/search-fusion-learning-contract.lino")
        .expect("Agent-authored learning contract");
    let policy = fs::read_to_string("data/meta/search-fusion-source-policy.lino")
        .expect("Agent-authored source policy");

    assert!(contract.contains("minimum_independent_executions 2"));
    assert!(contract.contains("candidate_inert true"));
    assert_eq!(policy.matches("  stage ").count(), 7);
    assert!(policy.contains("language_scope statement"));
    assert!(policy.contains("duplicate_capture unoriginal"));
    assert_eq!(
        formal_ai::search_fusion_grammar::policy_document(),
        fs::read_to_string("data/seed/search-fusion-language-grammar.lino")
            .expect("Agent-authored language grammar")
    );
    assert_eq!(
        formal_ai::search_fusion_grammar::role_order("hi"),
        ["subject", "object", "predicate"]
    );
}

#[test]
fn associative_report_is_derived_from_agent_authored_observations() {
    let memory = fs::read_to_string("data/meta/issue-709-search-fusion-learning.lino")
        .expect("Agent-authored observation network");
    let changed = memory.replace("accessCount \"9\"", "accessCount \"19\"");
    let report = search_fusion_learning::render_document_from(&memory);

    assert_ne!(
        report,
        search_fusion_learning::render_document_from(&changed),
        "ranking must be derived from the persisted network"
    );
    assert!(report.starts_with("search_fusion_learning_report\n  issue \"709\"\n"));
    assert!(report.contains("lesson:gated-recipe-replay"));
    assert!(report.contains("decision \"awaiting_human_review\""));
    assert!(!report.contains("decision \"promoted\""));
    assert!(REPORTS.iter().any(|descriptor| descriptor.issue == "709"));
    assert_eq!(
        fs::read_to_string(
            "docs/case-studies/issue-709/agent-cli-evidence/learning-report-execution/search-fusion-learning-report.lino"
        )
        .expect("live Agent CLI report"),
        search_fusion_learning::render_document(),
        "the committed live Agent CLI output must match Formal AI's in-process recipe"
    );
}

#[test]
fn formal_ai_executes_the_issue_709_learning_report_recipe() {
    let task = "Rank the reusable issue 709 fusion lessons without adopting them and write search-fusion-learning-report.lino";
    let outcome = run_agentic_task(task).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.steps.len(), 2);
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], SEARCH_FUSION_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        search_fusion_learning::render_document()
    );
    assert!(outcome.final_answer.contains("human-review-gated report"));
}
