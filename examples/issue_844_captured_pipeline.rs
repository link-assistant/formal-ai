//! Issue #844's production path over exact captures and offline replay.
//!
//! Run with:
//! `cargo run --example issue_844_captured_pipeline`

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use formal_ai::{
    execute_multi_source_summary, CachedSourceClient, CapturedSourceMetadata, FactChecker,
    FetchError, FormalSystem, GatheringPlan, SolverConfig, SourceCapture, SourceTier,
    SourceTransport, SummarizationConfig, SummarizationMode,
};

#[derive(Clone, Default)]
struct CapturedThread {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for CapturedThread {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        match url {
            "https://fixture.invalid/question" => Ok(b"How fast is the parser?\n".to_vec()),
            "https://fixture.invalid/answer" => Ok(b"The parser is fast.\n".to_vec()),
            "https://fixture.invalid/denial" => Ok(b"The parser is not fast.\n".to_vec()),
            _ => Err(FetchError::Transport(format!("fixture_missing:{url}"))),
        }
    }
}

fn classify(capture: &SourceCapture) -> CapturedSourceMetadata {
    let text = String::from_utf8(capture.bytes().to_vec()).expect("fixture is UTF-8");
    match capture.source_url() {
        "https://fixture.invalid/question" => {
            CapturedSourceMetadata::new(SourceTier::OriginalJournalism, text)
                .supplying("question")
                .linking("https://fixture.invalid/answer")
                .linking("https://fixture.invalid/denial")
        }
        "https://fixture.invalid/answer" => {
            CapturedSourceMetadata::new(SourceTier::OriginalFirstParty, text).supplying("answer")
        }
        _ => CapturedSourceMetadata::new(SourceTier::Unoriginal, text),
    }
}

fn checker() -> FactChecker {
    FactChecker::from_solver_config(SolverConfig {
        max_decomposition_depth: 2,
        ..SolverConfig::default()
    })
}

const fn fixed_time() -> u64 {
    1_753_444_800
}

fn main() {
    let cache = std::env::temp_dir().join(format!(
        "formal-ai-issue-844-example-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&cache);
    let transport = CapturedThread::default();
    let requests = Arc::clone(&transport.requests);
    let live = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);
    let plan = GatheringPlan::new("parser speed", 1)
        .requiring("question")
        .requiring("answer")
        .seeded_with("https://fixture.invalid/question");
    let formal_system = FormalSystem::new("captured_parser_reports")
        .with_universe("captured source statements")
        .with_interpretation("relative source evidence");

    let execution = execute_multi_source_summary(
        "parser_context",
        formal_system.clone(),
        &plan,
        &live,
        checker(),
        classify,
    );
    println!("=== exact-capture gathering ===");
    println!("{}\n", execution.gathering.report.trace());
    println!("=== checked summary ===");
    println!(
        "{}\n",
        execution.checked_summary(&SummarizationConfig::default())
    );
    println!("=== identifier rung ===");
    println!(
        "{}\n",
        execution.checked_summary(
            &SummarizationConfig::default().with_mode(SummarizationMode::Identifier)
        )
    );
    println!("=== named-context fact check ===");
    println!("{}\n", execution.audit.links_notation());
    println!("=== review-gated learning proposal ===");
    println!("{}\n", execution.learning_proposal());

    let replay = execute_multi_source_summary(
        "parser_context",
        formal_system,
        &plan,
        &CachedSourceClient::new(&cache, transport),
        checker(),
        classify,
    );
    assert_eq!(
        replay.checked_summary(&SummarizationConfig::default()),
        execution.checked_summary(&SummarizationConfig::default())
    );
    assert_eq!(replay.learning_proposal(), execution.learning_proposal());
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    println!("offline replay: byte-identical derivation, zero new requests");

    fs::remove_dir_all(cache).expect("remove example cache");
}
