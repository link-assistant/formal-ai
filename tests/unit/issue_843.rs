use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::option_network::{Comparison, Constraint, OptionNetwork, Tier};
use formal_ai::relative_meta_logic::RelativeEvidence;
use formal_ai::{
    CachedSourceClient, EventLog, FetchError, SourceTier, SourceTransport, Stance, UniversalSolver,
    execute_duckduckgo_search, execute_option_research, execute_statement_research, sha256_hex,
    try_web_search_with_client,
};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        if url.starts_with("https://html.duckduckgo.com/html/") {
            return Ok(
                br#"<div class="result__body"><a class="result__a" href="https://result.invalid/a">A result</a><a class="result__snippet">A result</a></div><div class="result__body"><a class="result__a" href="https://result.invalid/b">B result</a><a class="result__snippet">B result</a></div>"#
                    .to_vec(),
            );
        }
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                br#"{"AbstractURL":"https://result.invalid/a","AbstractText":"A result","RelatedTopics":[{"FirstURL":"https://result.invalid/b","Text":"B result"}]}"#
                    .to_vec(),
            );
        }
        if url == "https://result.invalid/a" {
            return Ok(b"Official supply: 20 V and 3.25 A. Price: $49.\n".to_vec());
        }
        if url == "https://result.invalid/b" {
            return Ok(b"Compatible supply: 20 V and 4 A. Price: $39.\n".to_vec());
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
    let prompts = [
        ("english", "An unrecognized term requiring research"),
        ("ru", "Исследуй неизвестный термин"),
        ("hi", "किसी अज्ञात शब्द पर शोध करें"),
        ("zh", "研究一个未知术语"),
    ];
    for (locale, prompt) in prompts {
        let answer = UniversalSolver::default().solve(prompt);
        assert!(
            answer
                .evidence_links
                .iter()
                .all(|link| !link.contains("example.org")),
            "{locale}: {:?}",
            answer.evidence_links
        );
        assert!(
            answer
                .evidence_links
                .iter()
                .all(|link| !link.starts_with("source:http:") && !link.starts_with("cache_hit:")),
            "{locale}: {:?}",
            answer.evidence_links
        );
    }
}

#[test]
fn declared_seed_sources_do_not_masquerade_as_http_captures() {
    let answer = UniversalSolver::default().solve("Combine translated definitions for IIR");
    assert_eq!(answer.intent, "definition_merge");
    assert!(
        answer
            .evidence_links
            .iter()
            .any(|link| link.starts_with("definition_merge:source_declared:"))
    );
    assert!(
        answer
            .evidence_links
            .iter()
            .all(|link| !link.starts_with("source:http:"))
    );
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
    assert!(
        cache
            .join("source-cache")
            .join("objects")
            .join(format!("{}.body", first.sha256()))
            .is_file(),
        "captured bytes must be stored under their content digest"
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
fn cache_metadata_cannot_escape_the_content_object_directory() {
    let cache = temp_cache();
    let url = "https://fixture.invalid/article";
    let online = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let capture = online.fetch(url).expect("fixture fetch");
    let cache_root = cache.join("source-cache");
    let metadata_path = fs::read_dir(&cache_root)
        .expect("source cache")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "meta")
        })
        .expect("capture metadata");
    let metadata = fs::read_to_string(&metadata_path).expect("read capture metadata");
    fs::write(
        &metadata_path,
        metadata.replace(
            &format!("sha256={}", capture.sha256()),
            "sha256=../../outside",
        ),
    )
    .expect("tamper capture metadata");

    let offline = CachedSourceClient::new(&cache, FixtureTransport::default());
    let error = offline.fetch(url).expect_err("invalid digest must fail");
    assert!(
        error
            .to_string()
            .contains("source_cache_invalid_content_hash"),
        "{error}"
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
fn web_search_handler_reports_only_executed_provider_results() {
    let cache = temp_cache();
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let mut log = EventLog::new();
    let answer = try_web_search_with_client(
        "Search the web for formal AI",
        "search the web for formal ai",
        &mut log,
        &client,
    )
    .expect("web-search intent");

    assert!(answer.answer.contains("https://result.invalid/a"));
    assert!(answer.answer.contains("https://result.invalid/b"));
    assert!(
        answer
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source:http:"))
    );
    assert_eq!(
        log.events()
            .iter()
            .filter(|event| event.kind == "web_search:provider")
            .map(|event| event.payload.as_str())
            .collect::<Vec<_>>(),
        vec!["duckduckgo"]
    );
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "web_search:combined")
    );
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn source_research_auto_learns_options_from_captured_pages_and_replays() {
    let cache = temp_cache();
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);
    let mut network = OptionNetwork::new("subject");
    network.require(Constraint::quantity(
        "output_voltage",
        20_000,
        "V",
        Comparison::Equal,
    ));
    let first = execute_option_research(
        &mut network,
        &online,
        "20 V power supply",
        Tier::OfficialCompatible,
        "$",
        2,
    )
    .expect("option research");

    assert_eq!(first.research.search.rankings.len(), 2);
    assert_eq!(first.research.pages.len(), 2);
    assert_eq!(network.candidates().len(), 2);
    assert!(
        network
            .candidates()
            .iter()
            .all(|candidate| candidate.supplies.contains_key("output_voltage"))
    );
    assert!(network.links_notation().contains("official_compatible"));
    let proposal = first.learning_proposal(&network);
    assert!(proposal.contains(first.research.pages[0].capture.sha256()));
    assert!(proposal.contains("https://result.invalid/a"));

    let offline = CachedSourceClient::new(&cache, transport);
    let mut replay_network = OptionNetwork::new("subject");
    replay_network.require(Constraint::quantity(
        "output_voltage",
        20_000,
        "V",
        Comparison::Equal,
    ));
    let replay = execute_option_research(
        &mut replay_network,
        &offline,
        "20 V power supply",
        Tier::OfficialCompatible,
        "$",
        2,
    )
    .expect("offline option replay");
    assert_eq!(replay_network.links_notation(), network.links_notation());
    assert_eq!(
        replay.learning_proposal(&replay_network),
        first.learning_proposal(&network)
    );
    assert!(
        replay
            .research
            .pages
            .iter()
            .all(|page| page.capture.cached())
    );
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn statement_research_attaches_only_classified_captured_evidence() {
    let cache = temp_cache();
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let execution = execute_statement_research(
        "The captured option is available.",
        &client,
        2,
        |statement, capture| {
            capture.source_url().ends_with("/a").then(|| {
                RelativeEvidence::new(
                    format!("{statement}:fixture-a"),
                    SourceTier::IndependentCorroboration,
                    Stance::Supports,
                    0.7,
                )
            })
        },
    )
    .expect("statement research");
    assert_eq!(execution.verification.plan.len(), 1);
    assert_eq!(execution.verification.captures.len(), 2);
    assert_eq!(execution.verification.classified.len(), 1);
    assert!(
        execution.verification.plan.statements[0]
            .assessment
            .posterior
            .get()
            > 0.6,
        "captured corroboration should update the assumed-true prior"
    );
    let audit = execution.verification.audit_evidence();
    assert_eq!(audit.len(), 1);
    assert_eq!(
        audit[0].sha256,
        execution.verification.classified[0].capture.sha256()
    );
    assert_eq!(
        audit[0].source_url,
        execution.verification.classified[0].capture.source_url()
    );
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

#[test]
fn whole_source_research_task_executes_and_replays_without_inventing_evidence() {
    let cache = temp_cache();
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);
    let prompt = "Search the web for 20 v power supply";
    let mut live_log = EventLog::new();
    let live_answer =
        try_web_search_with_client(prompt, &prompt.to_lowercase(), &mut live_log, &online)
            .expect("production search handler");

    let mut live_network = OptionNetwork::new("power_supply");
    live_network.require(Constraint::quantity(
        "output_voltage",
        20_000,
        "V",
        Comparison::Equal,
    ));
    let live_options = execute_option_research(
        &mut live_network,
        &online,
        "20 v power supply",
        Tier::OfficialCompatible,
        "$",
        2,
    )
    .expect("option research");
    live_options.research.record(&mut live_log);
    let live_statements = execute_statement_research(
        "The captured option supplies 20 V.",
        &online,
        2,
        |statement, capture| {
            capture.source_url().ends_with("/a").then(|| {
                RelativeEvidence::new(
                    format!("{statement}:captured"),
                    SourceTier::OriginalFirstParty,
                    Stance::Supports,
                    0.8,
                )
            })
        },
    )
    .expect("statement research");
    for search in &live_statements.searches {
        search.record(&mut live_log);
    }
    let live_audit = live_statements.verification.audit_evidence();

    assert_eq!(live_network.candidates().len(), 2);
    assert_eq!(live_audit.len(), 1);
    assert!(
        live_answer
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source:http:"))
    );
    assert!(live_log.events().iter().all(|event| {
        event.kind != "source:http"
            || (event.payload.contains("fetched_at=1753444800")
                && event.payload.contains("sha256=")
                && !event.payload.contains("example.org"))
    }));

    let offline = CachedSourceClient::new(&cache, transport);
    let mut replay_log = EventLog::new();
    let replay_answer =
        try_web_search_with_client(prompt, &prompt.to_lowercase(), &mut replay_log, &offline)
            .expect("offline handler replay");
    let mut replay_network = OptionNetwork::new("power_supply");
    replay_network.require(Constraint::quantity(
        "output_voltage",
        20_000,
        "V",
        Comparison::Equal,
    ));
    let replay_options = execute_option_research(
        &mut replay_network,
        &offline,
        "20 v power supply",
        Tier::OfficialCompatible,
        "$",
        2,
    )
    .expect("offline option replay");
    let replay_statements = execute_statement_research(
        "The captured option supplies 20 V.",
        &offline,
        2,
        |statement, capture| {
            capture.source_url().ends_with("/a").then(|| {
                RelativeEvidence::new(
                    format!("{statement}:captured"),
                    SourceTier::OriginalFirstParty,
                    Stance::Supports,
                    0.8,
                )
            })
        },
    )
    .expect("offline statement replay");

    assert_eq!(replay_answer.answer, live_answer.answer);
    assert_eq!(
        replay_network.links_notation(),
        live_network.links_notation()
    );
    assert_eq!(
        replay_options.learning_proposal(&replay_network),
        live_options.learning_proposal(&live_network)
    );
    assert_eq!(replay_statements.verification.audit_evidence(), live_audit);
    assert_eq!(requests.load(Ordering::SeqCst), 4);
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn same_task_agent_cli_authorship_is_preserved() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };
    let session = "ses_052d57d6affe1f2GYFodXhrqPl";
    let generated = read(
        "docs/case-studies/issue-843/self-hosting-authorship/source-evidence-honesty-invariant.lino",
    );
    let canonical = read("data/meta/source-evidence-honesty-invariant.lino");
    assert_eq!(generated, canonical);

    let agent_log = read("docs/case-studies/issue-843/self-hosting-authorship/agent-cli.log");
    assert!(agent_log.contains(session));
    let formal_ai_log = read("docs/case-studies/issue-843/self-hosting-authorship/formal-ai.log");
    for transition in [
        "planned ToolCalls",
        "tool=write",
        "tool: \"bash\"",
        "planned Final",
        "source-evidence-honesty-invariant.lino",
    ] {
        assert!(
            formal_ai_log.contains(transition),
            "server trace is missing {transition}"
        );
    }

    let decomposition =
        read("docs/case-studies/issue-843/self-hosting-authorship/decomposition.lino");
    assert_eq!(decomposition.matches("issue_843_smallest_leaf_").count(), 5);
    assert_eq!(
        decomposition
            .matches("authorship formal_ai_agent_cli")
            .count(),
        1
    );
    assert!(decomposition.contains(&format!("session {session}")));
    assert!(decomposition.contains("formal_ai_authored_percent 20"));
}
