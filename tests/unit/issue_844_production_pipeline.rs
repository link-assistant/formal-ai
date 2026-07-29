//! Production-boundary regressions for issue #844.
//!
//! The original issue branch tested recursive gathering through a mock-only
//! `SourceProvider` and called a local probability reassessment a "recheck".
//! Issues #843 and #845 have since supplied the exact-capture and named
//! fact-checking boundaries. These tests require the multi-source pipeline to
//! compose those real boundaries and to derive replayable learning proposals
//! from the captures it actually read.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use formal_ai::{
    execute_captured_gathering, execute_multi_source_summary, sha256_hex, CachedSourceClient,
    CapturedSourceMetadata, EventLog, FactChecker, FetchError, FormalSystem, GatheringPlan,
    NamingConvention, SolverConfig, SourceTier, SourceTransport, SummarizationConfig,
    SummarizationMode,
};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        match url {
            "https://fixture.invalid/first" => Ok(b"The parser is fast.\n".to_vec()),
            "https://fixture.invalid/second" => Ok(b"Parser is fast.\n".to_vec()),
            "https://fixture.invalid/denial" => Ok(b"The parser is not fast.\n".to_vec()),
            _ => Err(FetchError::Transport(format!("fixture_missing:{url}"))),
        }
    }
}

const fn fixed_time() -> u64 {
    1_753_444_800
}

fn temp_cache() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-844-production-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn captured_text(capture: &formal_ai::SourceCapture) -> String {
    String::from_utf8(capture.bytes().to_vec()).expect("fixture capture is UTF-8")
}

fn recursive_metadata(capture: &formal_ai::SourceCapture) -> CapturedSourceMetadata {
    match capture.source_url() {
        "https://fixture.invalid/first" => {
            CapturedSourceMetadata::new(SourceTier::OriginalFirstParty, captured_text(capture))
                .supplying("speed")
                .linking("https://fixture.invalid/second")
        }
        "https://fixture.invalid/second" => {
            CapturedSourceMetadata::new(SourceTier::OriginalJournalism, captured_text(capture))
                .supplying("documentation")
        }
        _ => CapturedSourceMetadata::new(SourceTier::Unoriginal, captured_text(capture)),
    }
}

fn merge_metadata(capture: &formal_ai::SourceCapture) -> CapturedSourceMetadata {
    match capture.source_url() {
        "https://fixture.invalid/first" => {
            CapturedSourceMetadata::new(SourceTier::OriginalFirstParty, captured_text(capture))
        }
        "https://fixture.invalid/second" => {
            CapturedSourceMetadata::new(SourceTier::OriginalJournalism, captured_text(capture))
        }
        _ => CapturedSourceMetadata::new(SourceTier::Unoriginal, captured_text(capture)),
    }
}

#[test]
fn recursive_gathering_uses_exact_captures_and_replays_its_learning_proposal() {
    let cache = temp_cache();
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let live = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);
    let plan = GatheringPlan::new("parser", 1)
        .requiring("speed")
        .requiring("documentation")
        .seeded_with("https://fixture.invalid/first");

    let first = execute_captured_gathering(&plan, &live, recursive_metadata);

    assert!(first.report.is_closed());
    assert_eq!(first.sources.len(), 2);
    assert!(first.failures.is_empty());
    assert_eq!(
        first.report.fetches[0].digest,
        sha256_hex(b"The parser is fast.\n"),
        "the traversal receipt must use the digest of the exact captured bytes"
    );
    assert_eq!(
        first.sources[0].capture.sha256(),
        first.report.fetches[0].digest
    );
    let proposal = first.learning_proposal();
    assert!(proposal.contains("multi_source_gathering"));
    assert!(proposal.contains(first.sources[0].capture.sha256()));
    assert!(proposal.contains(first.sources[1].capture.sha256()));

    let offline = CachedSourceClient::new(&cache, transport);
    let replay = execute_captured_gathering(&plan, &offline, recursive_metadata);

    assert!(replay.sources.iter().all(|source| source.capture.cached()));
    assert_eq!(replay.report.trace(), first.report.trace());
    assert_eq!(replay.learning_proposal(), proposal);
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn the_whole_pipeline_merges_into_a_named_context_fact_checks_and_learns() {
    let cache = temp_cache();
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let client = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);
    let plan = GatheringPlan::new("parser", 0)
        .seeded_with("https://fixture.invalid/first")
        .seeded_with("https://fixture.invalid/second")
        .seeded_with("https://fixture.invalid/denial");
    let formal_system = FormalSystem::new("captured_parser_reports")
        .with_universe("captured source statements")
        .with_interpretation("relative source evidence");
    let checker = FactChecker::from_solver_config(SolverConfig {
        max_decomposition_depth: 2,
        ..SolverConfig::default()
    });

    let execution = execute_multi_source_summary(
        "parser_context",
        formal_system.clone(),
        &plan,
        &client,
        checker,
        merge_metadata,
    );

    assert_eq!(execution.gathering.sources.len(), 3);
    assert_eq!(
        execution.audit.formal_system_name,
        "captured_parser_reports"
    );
    assert_eq!(
        execution.merged.context.formal_system().id(),
        execution.audit.formal_system_id
    );
    assert_eq!(execution.audit.statements.len(), 2);
    assert_eq!(execution.presentable_statement_ids.len(), 1);
    assert_eq!(execution.withheld_statement_ids.len(), 1);
    for ranked in &execution.merged.ranked {
        let statement_id =
            formal_ai::world_model::Statement::new(&ranked.statement.representative.text).id;
        assert_eq!(
            ranked.probability,
            execution
                .audit
                .statement(&statement_id)
                .expect("every ranked statement was audited")
                .probability,
            "the returned ranking must expose the post-audit probability"
        );
    }

    let presented = execution.checked_summary(&SummarizationConfig::default());
    assert!(presented.to_lowercase().contains("parser"));
    assert!(!presented.to_lowercase().contains("not fast"));
    let identifier = execution
        .checked_summary(&SummarizationConfig::default().with_mode(SummarizationMode::Identifier));
    assert!(formal_ai::is_valid_identifier(
        &identifier,
        NamingConvention::SnakeCase
    ));
    assert!(identifier
        .chars()
        .all(|character| !character.is_whitespace()));

    let learning = execution.learning_proposal();
    assert!(learning.contains("multi_source_statement_merge"));
    assert!(learning.contains("captured_parser_reports"));
    assert!(learning.contains("contradiction"));
    for source in &execution.gathering.sources {
        assert!(learning.contains(source.capture.sha256()));
    }
    let mut live_log = EventLog::new();
    execution.record(&mut live_log);
    assert_eq!(
        live_log
            .events()
            .iter()
            .filter(|event| event.kind == "source:http")
            .count(),
        3
    );
    assert!(live_log.first_of("fact_check:context").is_some());

    let replay = execute_multi_source_summary(
        "parser_context",
        formal_system,
        &plan,
        &CachedSourceClient::new(&cache, transport),
        FactChecker::from_solver_config(SolverConfig {
            max_decomposition_depth: 2,
            ..SolverConfig::default()
        }),
        merge_metadata,
    );
    assert!(
        replay
            .gathering
            .sources
            .iter()
            .all(|source| source.capture.cached()),
        "the complete operation must replay without enabling the network"
    );
    assert_eq!(
        replay.checked_summary(&SummarizationConfig::default()),
        presented
    );
    assert_eq!(
        replay.audit.links_notation(),
        execution.audit.links_notation()
    );
    assert_eq!(replay.learning_proposal(), learning);
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    let mut replay_log = EventLog::new();
    replay.record(&mut replay_log);
    assert_eq!(
        replay_log
            .events()
            .iter()
            .filter(|event| event.kind == "cache_hit")
            .count(),
        3
    );
    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn a_capture_failure_is_diagnostic_and_never_becomes_evidence() {
    let cache = temp_cache();
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let plan = GatheringPlan::new("missing", 0).seeded_with("https://fixture.invalid/missing");

    let report = execute_captured_gathering(&plan, &client, merge_metadata);

    assert!(report.sources.is_empty());
    assert!(report.report.fetches.is_empty());
    assert!(report.report.observations.is_empty());
    assert_eq!(report.failures.len(), 1);
    assert!(report.learning_proposal().contains("source_failure"));
    assert!(!report.learning_proposal().contains("source_observation"));
    assert!(
        !cache.exists(),
        "a transport failure must not be persisted as a capture"
    );
}

#[test]
fn same_task_agent_cli_authorship_is_preserved_for_issue_844() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };
    let session = "ses_050f9a572ffefpehWRjysug6cv";
    let generated = read(
        "docs/case-studies/issue-844/self-hosting-authorship/multi-source-summary-honesty-invariant.lino",
    );
    let canonical = read("data/meta/multi-source-summary-honesty-invariant.lino");
    assert_eq!(generated, canonical);

    let agent_log = read("docs/case-studies/issue-844/self-hosting-authorship/agent-cli.log");
    assert!(agent_log.contains(session));
    let formal_ai_log = read("docs/case-studies/issue-844/self-hosting-authorship/formal-ai.log");
    for transition in [
        "planned ToolCalls",
        "tool=write",
        "tool: \"bash\"",
        "planned Final",
        "multi-source-summary-honesty-invariant.lino",
    ] {
        assert!(
            formal_ai_log.contains(transition),
            "server trace is missing {transition}"
        );
    }

    let decomposition =
        read("docs/case-studies/issue-844/self-hosting-authorship/decomposition.lino");
    assert_eq!(decomposition.matches("issue_844_smallest_leaf_").count(), 5);
    assert_eq!(
        decomposition
            .matches("authorship formal_ai_agent_cli")
            .count(),
        1
    );
    assert!(decomposition.contains(&format!("session {session}")));
    assert!(decomposition.contains("formal_ai_authored_percent 20"));
}
