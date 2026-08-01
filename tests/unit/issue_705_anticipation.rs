//! Issue #705 (E63): deterministic anticipatory dreaming.
//!
//! These tests begin with the reported gap: append-only request history had no
//! next-request model, probe frontier, prelearned source cache, or prediction-hit
//! ledger. Each test pins one numbered requirement and the final test exercises
//! the whole held-out/offline capability delta through both production surfaces.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use formal_ai::anticipation::{
    answer_from_prelearned_cache_at, apply_anticipation, plan_anticipation, prediction_hit_event,
    prelearn_predictions, AnticipationConfig, AnticipationConsent, AnticipationLedger,
    PrelearningRun, PrelearningStatus, ProbeStatus, ANTICIPATION_FRONTIER,
};
use formal_ai::probability::ProbabilityModel;
use formal_ai::{
    create_chat_completion_with_solver_and_memory, create_response_with_solver_and_memory,
    run_core_dreaming_once, CachedSourceClient, ChatCompletionRequest, ChatMessage, FetchError,
    MemoryEvent, MemoryStore, ResponsesRequest, SolverConfig, SourceTransport, SyncStore,
    UniversalSolver,
};
use lino_objects_codec::format::parse_indented;

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                br#"{"AbstractURL":"https://result.invalid/frobulator","AbstractText":"A frobulator705 is a deterministic anticipation fixture.","RelatedTopics":[]}"#
                    .to_vec(),
            );
        }
        if url == "https://result.invalid/frobulator" {
            return Ok(b"A frobulator705 is a deterministic anticipation fixture.\n".to_vec());
        }
        Err(FetchError::Transport(format!("fixture_missing:{url}")))
    }
}

const fn fixed_time() -> u64 {
    2_000_000_000
}

fn temp_path(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-705-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn request(id: &str, prompt: &str, intent: &str) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        kind: Some(String::from("message")),
        role: Some(String::from("user")),
        intent: Some(intent.to_owned()),
        content: Some(prompt.to_owned()),
        sent_at: Some(format!("2026-08-01T00:00:{id}Z")),
        write_count: 1,
        ..MemoryEvent::default()
    }
}

/// The last class is `greeting`; its historical successors are three distinct
/// formalized classes. Repeated raw prompts are never used as Markov states.
fn scripted_requests() -> Vec<MemoryEvent> {
    vec![
        request("01", "hello", "greeting"),
        request("02", "2 + 2", "calculation"),
        request("03", "hello again", "greeting"),
        request("04", "reverse the words alpha beta", "text_transformation"),
        request("05", "hello once more", "greeting"),
        request("06", "describe frobulator705 resonance", "unknown"),
        request("07", "hello finally", "greeting"),
    ]
}

fn plan() -> formal_ai::anticipation::AnticipationPlan {
    plan_anticipation(&scripted_requests(), &AnticipationConfig::default())
}

#[test]
fn append_only_intent_transitions_predict_three_next_request_classes() {
    let plan = plan();
    assert_eq!(
        plan.current_class.as_ref().map(|class| class.id.as_str()),
        Some("intent:greeting")
    );
    assert_eq!(
        plan.predictions.len(),
        3,
        "the scripted history has three successors"
    );
    assert_eq!(
        plan.predictions
            .iter()
            .map(|prediction| prediction.class.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "intent:calculation",
            "intent:text_transformation",
            "intent:unknown"
        ],
        "equal transition counts use stable class-id ordering",
    );

    for transition in &plan.transitions {
        assert_eq!(
            transition.evidence.model,
            ProbabilityModel::MarkovTransition
        );
        assert_eq!(
            transition.evidence.transition_from.as_deref(),
            Some(transition.from.id.as_str())
        );
        assert!(transition
            .evidence_links
            .iter()
            .all(|link| link.starts_with("memory:")));
        assert!(!transition.to.id.contains(' '), "a state is not raw prose");
    }
    let why = plan
        .why_prediction(&plan.predictions[0].id)
        .expect("why answer");
    assert!(why.contains(&plan.predictions[0].transition_evidence_id));
}

#[test]
fn class_expansion_uses_meaning_operation_and_parameter_evidence() {
    let plan = plan();
    let sources = plan
        .predictions
        .iter()
        .flat_map(|prediction| prediction.variants.iter())
        .map(|variant| variant.source.as_str())
        .collect::<Vec<_>>();
    assert!(
        sources.iter().any(|source| source.starts_with("meaning:")),
        "a seeded lexical paraphrase should be generated: {sources:?}"
    );
    assert!(
        sources
            .iter()
            .any(|source| source.starts_with("operation:")),
        "a seeded operation paraphrase should be generated: {sources:?}"
    );
    assert!(
        sources
            .iter()
            .any(|source| source.starts_with("parameter:")),
        "observed members of one intent class are parameter variants: {sources:?}"
    );
    assert!(
        plan.predictions
            .iter()
            .flat_map(|prediction| prediction.variants.iter())
            .any(|variant| variant.source == "parameter:02" && variant.prompt == "2 + 2"),
        "deduplication keys must not replace punctuation-bearing probe text",
    );
    assert!(
        plan.probes
            .iter()
            .any(|probe| probe.prompt == "2 + 2" && probe.status == ProbeStatus::Passed),
        "the known-good observed member should remain a passing class probe",
    );
}

#[test]
fn every_unknown_or_failed_offline_probe_reaches_the_adoption_frontier() {
    let plan = plan();
    let failures = plan
        .probes
        .iter()
        .filter(|probe| probe.status != ProbeStatus::Passed)
        .collect::<Vec<_>>();
    assert!(
        !failures.is_empty(),
        "the unknown prediction must reproduce the gap"
    );
    assert_eq!(plan.frontier.len(), failures.len());
    assert_eq!(plan.learning_cycle.frontier, ANTICIPATION_FRONTIER);
    assert_eq!(plan.learning_cycle.frontier_items, failures.len());
    for failure in failures {
        assert!(plan
            .frontier
            .iter()
            .any(|item| item.prompt == failure.prompt));
    }
    let cycle = plan.learning_cycle.links_notation();
    assert!(cycle.contains("mode \"proposal_only\""));
    assert!(cycle.contains("human_gated \"true\""));
    assert!(plan.learning_cycle.proposals.iter().all(|proposal| proposal
        .source
        .starts_with("learning_frontier:anticipation:")));
}

#[test]
fn source_prelearning_is_consent_gated_and_uses_cache_provenance_and_ttl() {
    let cache = temp_path("consent-cache");
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let client = CachedSourceClient::new(&cache, transport)
        .with_online(true)
        .with_clock(fixed_time)
        .with_ttl_seconds(3_600);
    let plan = plan();

    let denied = prelearn_predictions(
        &plan,
        &client,
        AnticipationConsent::Denied,
        &AnticipationConfig::default(),
    );
    assert_eq!(
        requests.load(Ordering::SeqCst),
        0,
        "denial must perform no fetch"
    );
    assert!(denied
        .attempts
        .iter()
        .all(|attempt| attempt.status == PrelearningStatus::ConsentRequired));

    let granted = prelearn_predictions(
        &plan,
        &client,
        AnticipationConsent::Granted,
        &AnticipationConfig::default(),
    );
    assert!(!granted.sources.is_empty());
    assert!(requests.load(Ordering::SeqCst) > 0);
    for source in &granted.sources {
        assert_eq!(source.fetched_at, fixed_time().to_string());
        assert_eq!(source.expires_at, fixed_time() + 3_600);
        assert_eq!(source.sha256.len(), 64);
        assert!(source.result_url.starts_with("https://result.invalid/"));
    }
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn prediction_hits_link_later_actual_requests_and_zero_percent_is_honest() {
    let plan = plan();
    let denied = formal_ai::anticipation::PrelearningRun::default();
    let mut store = MemoryStore::new();
    apply_anticipation(&mut store, &plan, &denied);

    let before = AnticipationLedger::new(&plan, &denied, store.events()).links_notation();
    assert!(before.contains("prediction_hits \"0\""));
    assert!(before.contains("hit_rate_basis_points \"0\""));

    let hit = prediction_hit_event(store.events(), "2 + 2", "actual-request")
        .expect("arithmetic was predicted after greeting");
    assert_eq!(hit.kind.as_deref(), Some("prediction_hit"));
    assert!(hit.evidence.iter().any(|link| link == "actual-request"));
    assert!(hit
        .evidence
        .iter()
        .any(|link| link.starts_with("anticipation_prediction:")));
    store.append(hit);

    let after = AnticipationLedger::new(&plan, &denied, store.events()).links_notation();
    assert!(after.contains("prediction_hits \"1\""));
    assert!(!after.contains("hit_rate_basis_points \"0\""));
    parse_indented(&after).expect("the anticipation ledger is Links Notation");
}

#[test]
fn the_same_history_produces_byte_identical_predictions_and_ledger() {
    let left = plan_anticipation(&scripted_requests(), &AnticipationConfig::default());
    let right = plan_anticipation(&scripted_requests(), &AnticipationConfig::default());
    assert_eq!(left.links_notation(), right.links_notation());
    assert_eq!(
        AnticipationLedger::new(&left, &PrelearningRun::default(), &[]).links_notation(),
        AnticipationLedger::new(&right, &PrelearningRun::default(), &[]).links_notation(),
    );
}

#[test]
fn changed_transition_evidence_appends_a_new_prediction_revision() {
    let history = scripted_requests();
    let first = plan_anticipation(&history, &AnticipationConfig::default());
    let first_calculation = first
        .predictions
        .iter()
        .find(|prediction| prediction.class.id == "intent:calculation")
        .expect("initial calculation prediction");
    assert_eq!(first_calculation.count, 1);

    let mut store = MemoryStore::from_events(history);
    apply_anticipation(&mut store, &first, &PrelearningRun::default());
    store.append(request("08", "10 + 1", "calculation"));
    store.append(request("09", "hello after arithmetic", "greeting"));

    let updated = plan_anticipation(store.events(), &AnticipationConfig::default());
    let updated_calculation = updated
        .predictions
        .iter()
        .find(|prediction| prediction.class.id == "intent:calculation")
        .expect("updated calculation prediction");
    assert_eq!(updated_calculation.count, 2);
    assert_ne!(
        updated_calculation.id, first_calculation.id,
        "append-only predictions must version changed transition evidence",
    );

    let outcome = apply_anticipation(&mut store, &updated, &PrelearningRun::default());
    assert!(outcome.prediction_records > 0);
    assert!(store
        .events()
        .iter()
        .any(|event| event.id == first_calculation.id));
    assert!(store
        .events()
        .iter()
        .any(|event| event.id == updated_calculation.id));
}

#[test]
fn repeated_actual_requests_do_not_inflate_the_class_hit_rate() {
    let history = vec![
        request("01", "hello", "greeting"),
        request("02", "2 + 2", "calculation"),
        request("03", "hello again", "greeting"),
    ];
    let plan = plan_anticipation(&history, &AnticipationConfig::default());
    assert_eq!(plan.predictions.len(), 1);
    let mut store = MemoryStore::from_events(history);
    apply_anticipation(&mut store, &plan, &PrelearningRun::default());
    for actual in ["actual-one", "actual-two"] {
        let hit = prediction_hit_event(store.events(), "3 + 3", actual)
            .expect("the calculation class was predicted");
        store.append(hit);
    }

    let ledger =
        AnticipationLedger::new(&plan, &PrelearningRun::default(), store.events()).links_notation();
    assert!(ledger.contains("prediction_hits \"2\""));
    assert!(ledger.contains("predicted_classes_hit \"1\""));
    assert!(ledger.contains("hit_rate_basis_points \"10000\""));
}

#[test]
fn idle_dreaming_persists_the_ledger_and_later_live_usage_records_a_hit() {
    let dir = temp_path("idle-runtime");
    fs::create_dir_all(&dir).expect("create runtime fixture directory");
    let memory_path = dir.join("memory.lino");
    MemoryStore::from_events(scripted_requests())
        .save_to_file(&memory_path)
        .expect("seed append-only history");

    run_core_dreaming_once(&memory_path).expect("run one idle anticipation cycle");

    let ledger_path = formal_ai::anticipation::anticipation_ledger_path(&memory_path);
    let ledger = fs::read_to_string(&ledger_path).expect("idle run writes its ledger");
    parse_indented(&ledger).expect("idle ledger is Links Notation");
    assert!(ledger.contains("predictions \"3\""), "{ledger}");
    assert!(ledger.contains("prediction_hits \"0\""), "{ledger}");
    assert!(ledger.contains("mode \"proposal_only\""), "{ledger}");

    let mut sync = SyncStore::open_at(&memory_path);
    sync.record_chat_exchange("2 + 2", "4")
        .expect("record the later actual request");
    let hit = sync
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("prediction_hit"))
        .expect("the live memory path links the request to its prediction");
    assert!(hit
        .evidence
        .iter()
        .any(|link| link.starts_with("anticipation_prediction:")));
    assert!(hit
        .evidence
        .iter()
        .any(|link| link.starts_with("chat_user_")));

    fs::remove_dir_all(dir).expect("remove runtime fixture");
}

#[test]
fn a_held_out_predicted_prompt_becomes_answerable_offline_after_prelearning() {
    let cache = temp_path("held-out-cache");
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let client = CachedSourceClient::new(&cache, transport)
        .with_online(true)
        .with_clock(fixed_time)
        .with_ttl_seconds(3_600);
    let plan = plan();
    let offline_solver = UniversalSolver::new(SolverConfig {
        offline: true,
        compute_budget: 0,
        ..SolverConfig::default()
    });
    let held_out = plan
        .predictions
        .iter()
        .flat_map(|prediction| prediction.variants.iter())
        .filter(|variant| variant.source.starts_with("meaning:"))
        .filter(|variant| variant.prompt.contains("frobulator705"))
        .find(|variant| offline_solver.solve(&variant.prompt).intent == "unknown")
        .expect("the lexicon expansion should contain a distinct unresolved paraphrase")
        .prompt
        .clone();
    assert_eq!(offline_solver.solve(&held_out).intent, "unknown");

    let prelearning = prelearn_predictions(
        &plan,
        &client,
        AnticipationConsent::Granted,
        &AnticipationConfig::default(),
    );
    let requests_after_learning = requests.load(Ordering::SeqCst);
    let mut store = MemoryStore::new();
    let outcome = apply_anticipation(&mut store, &plan, &prelearning);
    assert!(outcome.prelearned_aliases > 0);

    let recalled = answer_from_prelearned_cache_at(&held_out, store.events(), fixed_time() + 1)
        .expect("the held-out paraphrase is answered from the prelearned class");
    assert_eq!(recalled.intent, "anticipation_cache");
    assert!(recalled
        .answer
        .contains("deterministic anticipation fixture"));
    assert!(recalled
        .evidence_links
        .iter()
        .any(|link| link.starts_with("source:http:")));
    assert!(
        answer_from_prelearned_cache_at(&held_out, store.events(), fixed_time() + 3_601,).is_none(),
        "expired prelearning must not be recalled"
    );

    let chat = chat_answer(&held_out, store.events(), &offline_solver);
    let responses = responses_answer(&held_out, store.events(), &offline_solver);
    assert_eq!(chat, recalled.answer);
    assert_eq!(responses, recalled.answer);
    assert_eq!(
        requests.load(Ordering::SeqCst),
        requests_after_learning,
        "future offline answers perform no transport request",
    );
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

fn chat_answer(prompt: &str, events: &[MemoryEvent], solver: &UniversalSolver) -> String {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage::user(prompt)],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };
    create_chat_completion_with_solver_and_memory(&request, solver, events).choices[0]
        .message
        .content
        .plain_text()
}

fn responses_answer(prompt: &str, events: &[MemoryEvent], solver: &UniversalSolver) -> String {
    let request = ResponsesRequest {
        input: serde_json::Value::String(prompt.to_owned()),
        ..ResponsesRequest::default()
    };
    create_response_with_solver_and_memory(&request, solver, events).output_messages()[0].content[0]
        .text
        .clone()
}
