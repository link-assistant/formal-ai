//! Issue #919: a recorded coding gap can research, verify, retain, and replay
//! a source-derived procedure without treating search prose as the answer.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use formal_ai::coding_research_learning::{
    execute_researched_coding_procedure, research_coding_skill_gap, CodingResearchApproval,
    CodingResearchGap, ResearchedCodingProcedureLedger, CODING_RESEARCH_LEARNING_CONTRACT,
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
                br#"{"AbstractURL":"https://research.invalid/ruby-count","Heading":"Ruby iteration guide","AbstractText":"Use upto to emit each integer in an inclusive range."}"#
                    .to_vec(),
            );
        }
        if url == "https://research.invalid/ruby-count" {
            return Ok(
                b"Formal AI coding procedure\nSPDX-License-Identifier: CC0-1.0\nTask: count_to_three\nLanguage: ruby\nOperation: verified_workspace_rewrite\nPattern: __COUNT_TO_THREE__\nReplacement: 1.upto(3) { |number| puts number }\n"
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
    let failed_task = formal_ai::FormalAiEngine.answer("Write a Ruby program that counts to three");
    assert_eq!(failed_task.intent, "write_program_skill_gap");
    assert_eq!(
        failed_task.answer,
        "I cannot write this program: no synthesis route reaches task \"count_to_three\" in language \"ruby\".\n\nI decomposed the request and tried every synthesis route I have, in order — catalog, blueprint_recipes, coding_oracle, seed_idiom_composer — and none of them derives it.\n\nNothing was guessed: I do not return a program I cannot derive, and I do not recite the templates I happen to hold. Teach me the missing idiom for `ruby`, or restate the task in steps I can already compile."
    );
    let source = "def main\n  __COUNT_TO_THREE__\nend\n";
    let expected = "def main\n  1.upto(3) { |number| puts number }\nend\n";
    let mut gap = CodingResearchGap::for_program_task("count_to_three", "ruby");
    assert_eq!(
        gap.next_query(),
        "ruby count_to_three verified coding procedure SPDX license"
    );
    let mut ledger = ResearchedCodingProcedureLedger::new();

    let learned = research_coding_skill_gap(
        &mut gap,
        &mut ledger,
        &online,
        source,
        expected,
        &CodingResearchApproval::granted("pull_request_review"),
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
        "source_url \"https://research.invalid/ruby-count\"",
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
    let mut replay_gap = CodingResearchGap::for_program_task("count_to_three", "ruby");
    let mut replay_ledger = ResearchedCodingProcedureLedger::new();
    let replay = research_coding_skill_gap(
        &mut replay_gap,
        &mut replay_ledger,
        &offline,
        source,
        expected,
        &CodingResearchApproval::granted("pull_request_review"),
    )
    .expect("CI can replay the complete research loop from captured bytes");

    assert_eq!(requests.load(Ordering::SeqCst), requests_after_live);
    assert_eq!(replay.procedure_id, learned.procedure_id);
    assert_eq!(replay.research_proposal, learned.research_proposal);
    assert_eq!(replay_ledger.links_notation(), durable);

    let restored = ResearchedCodingProcedureLedger::from_links_notation(&durable)
        .expect("content-addressed researched procedure ledger restores");
    let tampered = durable.replace("source_license \"CC0-1.0\"", "source_license \"NONE\"");
    assert!(
        ResearchedCodingProcedureLedger::from_links_notation(&tampered).is_err(),
        "a rewritten source license must invalidate durable capability"
    );
    let held_out =
        execute_researched_coding_procedure(&restored, gap.name(), "__COUNT_TO_THREE__\n")
            .expect("approved researched procedure uses the same bounded executor");
    assert_eq!(held_out.output, "1.upto(3) { |number| puts number }\n");

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn failed_execution_is_not_kept_and_schedules_the_next_research_round() {
    let cache = temp_cache("failed-verification");
    let client = CachedSourceClient::new(&cache, ProcedureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let mut gap = CodingResearchGap::for_program_task("count_to_three", "ruby");
    let first_query = gap.next_query().to_owned();
    let mut ledger = ResearchedCodingProcedureLedger::new();

    let error = research_coding_skill_gap(
        &mut gap,
        &mut ledger,
        &client,
        "__COUNT_TO_THREE__",
        "puts 3.downto(1)",
        &CodingResearchApproval::granted("pull_request_review"),
    )
    .expect_err("an execution that misses the task oracle cannot become a skill");

    assert_eq!(
        error.reason,
        "coding_research_execution_verification_failed"
    );
    assert_eq!(gap.failed_rounds(), 1);
    assert_ne!(gap.next_query(), first_query);
    assert_eq!(
        gap.next_query(),
        "ruby count_to_three verified coding procedure SPDX license alternative evidence round 2"
    );
    assert!(gap.links_notation().contains(&first_query));
    assert!(gap
        .links_notation()
        .contains("coding_research_execution_verification_failed"));
    assert!(ledger.is_empty());

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn coding_research_policy_is_data_authored_and_pins_the_safety_boundaries() {
    for invariant in [
        "gap_source program_skill_gap",
        "procedure_origin research",
        "candidate_effect inert_until_verified",
        "executor verified_workspace_rewrite",
        "live_fetch opt_in",
        "offline_replay source_cache",
        "failure_effect schedule_next_query",
        "base_query_template \"{language} {task} verified coding procedure SPDX license\"",
        "retry_query_template \"{base_query} alternative evidence round {round}\"",
        "human_review required",
    ] {
        assert!(
            CODING_RESEARCH_LEARNING_CONTRACT.contains(invariant),
            "missing contract invariant: {invariant}"
        );
    }
    assert_eq!(
        CODING_RESEARCH_LEARNING_CONTRACT
            .matches("provenance_field ")
            .count(),
        4
    );
}
