//! Issue #919: a recorded coding gap can research, verify, retain, and replay
//! a source-derived procedure without treating search prose as the answer.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use formal_ai::coding_research_learning::{
    execute_researched_coding_procedure, research_coding_skill_gap, CodingResearchApproval,
    CodingResearchGap, ResearchedCodingProcedureLedger,
};
use formal_ai::{CachedSourceClient, FetchError, SourceTransport};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
struct ProcedureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for ProcedureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                br#"{"AbstractURL":"https://research.invalid/rust-trim","Heading":"Rust migration guide","AbstractText":"Use trim_end for the deprecated right-trim operation."}"#
                    .to_vec(),
            );
        }
        if url == "https://research.invalid/rust-trim" {
            return Ok(
                b"Formal AI coding procedure\nSPDX-License-Identifier: CC0-1.0\nTask: modernize_trim_right\nLanguage: rust\nOperation: verified_workspace_rewrite\nPattern: .trim_right()\nReplacement: .trim_end()\n"
                    .to_vec(),
            );
        }
        Err(FetchError::Transport(format!("fixture_missing:{url}")))
    }
}

const fn fixed_time() -> u64 {
    1_786_320_000
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-919-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

#[test]
fn coding_gap_is_solved_by_a_verified_researched_procedure_and_replays_offline() {
    let cache = temp_cache("end-to-end");
    let transport = ProcedureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);
    let source = "fn clean(value: &str) -> &str { value.trim_right() }\n";
    let expected = "fn clean(value: &str) -> &str { value.trim_end() }\n";
    let mut gap = CodingResearchGap::for_program_task("modernize_trim_right", "rust");
    let mut ledger = ResearchedCodingProcedureLedger::new();

    let learned = research_coding_skill_gap(
        &mut gap,
        &mut ledger,
        &online,
        source,
        expected,
        CodingResearchApproval::granted("pull_request_review"),
    )
    .expect("captured procedure solves the recorded skill gap");

    assert_eq!(learned.output, expected);
    assert_eq!(learned.gap_name, gap.name());
    assert!(learned.research_proposal.contains("source_research"));
    assert!(learned.formalization.contains("coding_procedure"));
    assert!(learned.cycle.contains("kind \"procedure\""));
    assert!(learned.cycle.contains("status \"stable\""));
    assert_eq!(gap.failed_rounds(), 0);

    let durable = ledger.links_notation();
    for field in [
        "origin \"research\"",
        "status \"execution_verified\"",
        "source_url \"https://research.invalid/rust-trim\"",
        "source_license \"CC0-1.0\"",
        "fetched_at \"1786320000\"",
        "source_sha256",
        "query",
        "formalization",
        "executor \"verified_workspace_rewrite\"",
        "reviewer \"pull_request_review\"",
    ] {
        assert!(durable.contains(field), "missing provenance field: {field}");
    }

    let requests_after_live = requests.load(Ordering::SeqCst);
    let offline = CachedSourceClient::new(&cache, transport);
    let mut replay_gap = CodingResearchGap::for_program_task("modernize_trim_right", "rust");
    let mut replay_ledger = ResearchedCodingProcedureLedger::new();
    let replay = research_coding_skill_gap(
        &mut replay_gap,
        &mut replay_ledger,
        &offline,
        source,
        expected,
        CodingResearchApproval::granted("pull_request_review"),
    )
    .expect("CI can replay the complete research loop from captured bytes");

    assert_eq!(requests.load(Ordering::SeqCst), requests_after_live);
    assert_eq!(replay.procedure_id, learned.procedure_id);
    assert_eq!(replay.research_proposal, learned.research_proposal);
    assert_eq!(replay_ledger.links_notation(), durable);

    let restored = ResearchedCodingProcedureLedger::from_links_notation(&durable)
        .expect("content-addressed researched procedure ledger restores");
    let held_out = execute_researched_coding_procedure(
        &restored,
        gap.name(),
        "let name = input.trim_right();\n",
    )
    .expect("approved researched procedure uses the same bounded executor");
    assert_eq!(held_out.output, "let name = input.trim_end();\n");

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn failed_execution_is_not_kept_and_schedules_the_next_research_round() {
    let cache = temp_cache("failed-verification");
    let client = CachedSourceClient::new(&cache, ProcedureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let mut gap = CodingResearchGap::for_program_task("modernize_trim_right", "rust");
    let first_query = gap.next_query().to_owned();
    let mut ledger = ResearchedCodingProcedureLedger::new();

    let error = research_coding_skill_gap(
        &mut gap,
        &mut ledger,
        &client,
        "value.trim_right()",
        "value.strip_suffix()",
        CodingResearchApproval::granted("pull_request_review"),
    )
    .expect_err("an execution that misses the task oracle cannot become a skill");

    assert_eq!(error.reason, "coding_research_execution_verification_failed");
    assert_eq!(gap.failed_rounds(), 1);
    assert_ne!(gap.next_query(), first_query);
    assert!(gap.next_query().contains("alternative evidence round 2"));
    assert!(gap.links_notation().contains(&first_query));
    assert!(gap
        .links_notation()
        .contains("coding_research_execution_verification_failed"));
    assert!(ledger.is_empty());

    fs::remove_dir_all(cache).expect("remove fixture cache");
}
