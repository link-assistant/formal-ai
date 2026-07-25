use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use formal_ai::option_network::{Candidate, OptionNetwork, Tier};
use formal_ai::relative_meta_logic::RelativeEvidence;
use formal_ai::statement_audit::EvidenceCapture;
use formal_ai::{
    execute_duckduckgo_search, sha256_hex, CachedSourceClient, EventLog, FetchError, SourceTier,
    SourceTransport, Stance, StatementVerificationPlan, UniversalSolver,
};

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
                br#"{"AbstractURL":"https://result.invalid/a","AbstractText":"A result","RelatedTopics":[{"FirstURL":"https://result.invalid/b","Text":"B result"}]}"#
                    .to_vec(),
            );
        }
        Ok(format!("recorded source bytes for {url}\n").into_bytes())
    }
}

const fn fixed_time() -> u64 {
    1_753_444_800
}

fn temp_cache() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-843-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

#[test]
fn external_search_never_fabricates_source_provenance() {
    let answer = UniversalSolver::default().solve("An unrecognized term requiring research");
    assert!(
        answer
            .evidence_links
            .iter()
            .all(|link| !link.contains("example.org")),
        "{:?}",
        answer.evidence_links
    );
    assert!(answer
        .evidence_links
        .iter()
        .all(|link| !link.starts_with("source:http:") && !link.starts_with("cache_hit:")));
}

#[test]
fn successful_fetch_records_bytes_and_truthful_cache_state() {
    let cache = temp_cache();
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);

    let first = online
        .fetch("https://fixture.invalid/article")
        .expect("fixture fetch");
    assert!(!first.cached());
    assert_eq!(first.fetched_at(), fixed_time().to_string());
    assert_eq!(
        first.sha256(),
        sha256_hex(b"recorded source bytes for https://fixture.invalid/article\n")
    );

    let offline = CachedSourceClient::new(&cache, transport);
    let replay = offline
        .fetch("https://fixture.invalid/article")
        .expect("offline cache replay");
    let replay_again = offline
        .fetch("https://fixture.invalid/article")
        .expect("second offline cache replay");
    assert!(replay.cached());
    assert_eq!(replay.bytes(), first.bytes());
    assert_eq!(replay.fetched_at(), first.fetched_at());
    assert_eq!(replay.sha256(), first.sha256());
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(replay.trace_payload(), replay_again.trace_payload());
    assert_eq!(replay.bytes(), replay_again.bytes());

    let mut first_log = EventLog::new();
    first.record(&mut first_log);
    let mut replay_log = EventLog::new();
    replay.record(&mut replay_log);
    assert_eq!(
        first.trace_payload(),
        replay
            .trace_payload()
            .replace("cached=true", "cached=false")
    );
    assert_eq!(
        first_log
            .events()
            .iter()
            .find(|event| event.kind == "source:http")
            .map(|event| event.payload.as_str()),
        replay_log
            .events()
            .iter()
            .find(|event| event.kind == "source:http")
            .map(|event| event.payload.replace("cached=true", "cached=false"))
            .as_deref()
    );

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn fetched_rankings_reach_rrf_and_replay_offline() {
    let cache = temp_cache();
    let online = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let first = execute_duckduckgo_search(&online, "formal ai").expect("search fixture");
    assert_eq!(first.rankings.len(), 2);
    assert_eq!(first.fused.len(), 2);
    assert!(!first.captures[0].cached());

    let offline = CachedSourceClient::new(&cache, FixtureTransport::default());
    let replay = execute_duckduckgo_search(&offline, "formal ai").expect("search replay");
    assert!(replay.captures[0].cached());
    assert_eq!(replay.rankings, first.rankings);
    assert_eq!(replay.fused, first.fused);
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn fetched_content_drives_option_observation_and_statement_evidence() {
    let cache = temp_cache();
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let mut network = OptionNetwork::new("subject");
    let capture = network
        .fetch_and_observe(&client, "https://fixture.invalid/option", |capture| {
            assert!(!capture.bytes().is_empty());
            Ok(Candidate::new("captured-option", Tier::Authentic))
        })
        .expect("captured candidate");
    assert_eq!(network.candidates()[0].id, "captured-option");

    let execution = StatementVerificationPlan::execute(
        "The captured option is available.",
        &client,
        |_| vec![String::from("https://fixture.invalid/evidence")],
        |statement, capture| {
            assert!(!capture.bytes().is_empty());
            Some(RelativeEvidence::new(
                format!("{statement}:{}", capture.source_url()),
                SourceTier::IndependentCorroboration,
                Stance::Supports,
                0.7,
            ))
        },
    )
    .expect("statement execution");
    assert_eq!(execution.plan.len(), 1);
    assert_eq!(execution.captures.len(), 1);
    assert!(
        execution.plan.statements[0].assessment.posterior.get() > 0.6,
        "captured corroboration should update the assumed-true prior"
    );

    let audit_capture = EvidenceCapture::from_source_capture(
        "The captured option is available",
        "fixture",
        &capture,
        SourceTier::OriginalFirstParty,
        Stance::Supports,
        0.9,
    );
    assert_eq!(audit_capture.captured_at, capture.fetched_at());
    assert_eq!(audit_capture.sha256, capture.sha256());
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn offline_cache_miss_emits_no_evidence() {
    let cache = temp_cache();
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let client = CachedSourceClient::new(&cache, transport);
    assert!(matches!(
        client.fetch("https://fixture.invalid/missing"),
        Err(FetchError::OfflineCacheMiss(_))
    ));
    assert_eq!(requests.load(Ordering::SeqCst), 0);
}
