//! Issue #701 (E59): a learning cycle must demonstrably change answers.
//!
//! Each test below pins one requirement of the issue: the adoption contract
//! (#701-1), the recorded capability delta (#701-2), the closed Google Trends
//! gap (#701-3), and the durability of every failure to adopt (#701-6).

use formal_ai::learning_cycle::{
    GOOGLE_TRENDS_FRONTIER, GOOGLE_TRENDS_FRONTIER_RECORD, LEARNED_REQUEST_OPENERS_SEED_FILE,
};
use formal_ai::promotion::{parse_promotion_proposals, render_promotion_proposals};
use formal_ai::{
    google_trends_adoption_ledger, google_trends_learning_cycle, parse_frontier_record,
};
use lino_objects_codec::format::parse_indented;

/// Every language the engine supports; the issue requires the delta in all of
/// them, not just English.
const SUPPORTED_LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

#[test]
fn committed_adoption_ledger_matches_a_fresh_run() {
    let committed = include_str!("../../data/meta/learning-adoption-ledger.lino");
    let fresh = format!("{}\n", google_trends_adoption_ledger().links_notation());
    assert_eq!(
        committed, fresh,
        "the committed adoption ledger is stale; regenerate it with \
         `cargo run --example issue_701_adoption_ledger > data/meta/learning-adoption-ledger.lino`",
    );
    parse_indented(committed).expect("the adoption ledger should parse as Links Notation");
}

#[test]
fn the_adoption_ledger_records_a_real_capability_delta_in_every_language() {
    // Issue #701 requirement 2 and its acceptance criterion: at least 20
    // before/after pairs, spanning at least 3 topics and all 4 languages, each
    // one unknown before and answered after.
    let ledger = google_trends_adoption_ledger();
    let adopted = ledger.adopted();
    assert!(
        adopted.len() >= 20,
        "expected >= 20 adopted before/after pairs, got {}",
        adopted.len(),
    );
    assert!(
        ledger.adopted_topics().len() >= 3,
        "expected >= 3 topics, got {}",
        ledger.adopted_topics().len(),
    );
    for language in SUPPORTED_LANGUAGES {
        assert!(
            ledger.adopted_languages().contains(language),
            "language {language} must contribute an adopted pair",
        );
    }

    // The delta is a delta: unknown before, a routed intent after, and the
    // topic the prompt was generated from recovered from the answer.
    for pair in adopted {
        assert_eq!(pair.before_intent, "unknown", "{}", pair.prompt);
        assert_ne!(pair.after_intent, "unknown", "{}", pair.prompt);
        assert!(pair.topic_recovered(), "{}", pair.prompt);
        assert_eq!(pair.capability_delta(), "unknown_to_web_search");
    }
}

#[test]
fn the_trends_corpus_unknown_rate_is_ratcheted_to_zero() {
    // Issue #701 requirement 3: drive the #498/#499 frontier through the loop
    // until the trends prompts answer (target 80/80). The ratchet may fall,
    // never rise — a regression that reintroduces an unknown fails here.
    let ledger = google_trends_adoption_ledger();
    assert_eq!(ledger.corpus_prompts, 80, "10 topics x 4 languages x 2 variations");
    assert!(
        ledger.unknown_rate_after_basis_points() < ledger.unknown_rate_before_basis_points(),
        "the unknown rate must fall: {} -> {}",
        ledger.unknown_rate_before_basis_points(),
        ledger.unknown_rate_after_basis_points(),
    );
    assert_eq!(
        ledger.corpus_unknown_after, 0,
        "80/80 trends prompts must answer; still unknown: {}",
        ledger.corpus_unknown_after,
    );
}

#[test]
fn the_learning_cycle_emits_promotion_proposals_in_the_issue_656_shape() {
    // Issue #701 requirement 1 and acceptance criterion 1: the cycle produces at
    // least one valid promotion proposal, with tests, that #656 consumes.
    let run = google_trends_learning_cycle();
    assert_eq!(run.frontier, GOOGLE_TRENDS_FRONTIER);
    assert!(!run.proposals.is_empty(), "the cycle must propose something");
    assert!(
        run.held_out_count() > 0,
        "every proposal must carry generated held-out tests",
    );
    for candidate in run.validated_candidates() {
        assert!(candidate.failed_count() == 0 && candidate.passed_count() > 0);
    }

    // The rendered document round-trips through the promotion parser, which is
    // exactly what `formal-ai improve --promote --proposals -` reads.
    let rendered = render_promotion_proposals(&run.proposals);
    let parsed = parse_promotion_proposals(&rendered).expect("proposals must round-trip");
    assert_eq!(parsed.len(), run.proposals.len());
    for proposal in &parsed {
        assert!(proposal.source.starts_with("learning_frontier:google-trends:"));
        assert_eq!(proposal.edit.seed_file, LEARNED_REQUEST_OPENERS_SEED_FILE);
        assert!(!proposal.edit.lino.is_empty());
    }
}

#[test]
fn the_cycle_is_deterministic_and_reproducible_offline() {
    // Acceptance criterion 1: deterministic and reproducible from cached data.
    // The frozen frontier record is the cache; two runs over it must agree, and
    // the run must never depend on the live catalog.
    let first = google_trends_learning_cycle();
    let second = google_trends_learning_cycle();
    assert_eq!(first.links_notation(), second.links_notation());
    assert_eq!(
        render_promotion_proposals(&first.proposals),
        render_promotion_proposals(&second.proposals),
    );
    assert!(first.links_notation().contains("mode \"proposal_only\""));
    assert!(first.links_notation().contains("human_gated \"true\""));
}

#[test]
fn the_frozen_frontier_record_spans_every_supported_language() {
    // The record is the durable "what we could not answer" artifact (#701-6):
    // it must keep the pre-adoption verdict for every language, so the delta
    // stays checkable after the gap is closed.
    let items = parse_frontier_record(GOOGLE_TRENDS_FRONTIER_RECORD);
    assert!(!items.is_empty(), "the frontier record must not be empty");
    for language in SUPPORTED_LANGUAGES {
        assert!(
            items.iter().any(|item| item.language == language),
            "language {language} must appear on the recorded frontier",
        );
    }
    assert!(
        items.iter().all(|item| item.engine_intent == "unknown"),
        "a frontier entry is by definition an unrouted prompt",
    );
}

#[test]
fn every_failure_to_adopt_is_preserved_as_a_durable_record() {
    // Issue #701 requirement 6 / R425: nothing is silently dropped. Whatever the
    // cycle does not adopt appears as a blocked class with a named reason, and
    // whatever the ledger does not adopt stays as an unadopted pair.
    let run = google_trends_learning_cycle();
    let record = run.links_notation();
    let rejected_after_validation = run
        .blocked
        .iter()
        .filter(|blocked| blocked.reason == "held_out_validation_failed")
        .count();
    assert_eq!(
        rejected_after_validation,
        run.candidates.len() - run.validated_candidates().len(),
        "every candidate that failed its held-out tests must be recorded as blocked",
    );
    for blocked in &run.blocked {
        assert!(!blocked.reason.is_empty(), "a blocked class needs a named gap");
        assert!(record.contains(&blocked.reason));
    }

    let ledger = google_trends_adoption_ledger();
    assert_eq!(
        ledger.adopted().len() + ledger.unadopted().len(),
        ledger.pairs.len(),
        "every recorded pair is either adopted or kept as unadopted",
    );
}

#[test]
fn every_idle_dreaming_run_leaves_a_proposal_only_learning_cycle_record() {
    // Issue #701 §5: the loop must run *periodically* in proposal mode. The
    // dreaming runtime is the in-process half of that schedule (the scheduled
    // CI half lives in `.github/workflows/learning-cycle.yml`). A run must
    // leave a reviewable record and must not adopt anything on its own.
    let memory_path = std::env::temp_dir().join(format!(
        "formal-ai-issue-701-learning-{}-memory.lino",
        std::process::id()
    ));
    formal_ai::MemoryStore::from_events(Vec::new())
        .save_to_file(&memory_path)
        .expect("write fixture");

    formal_ai::run_core_dreaming_once(&memory_path).expect("one idle dreaming run");

    let record_path = formal_ai::dreaming_runtime::learning_cycle_record_path(&memory_path);
    let record = std::fs::read_to_string(&record_path).expect("the idle run writes the record");
    parse_indented(&record).expect("the learning-cycle record parses as Links Notation");
    assert!(record.contains("mode \"proposal_only\""), "{record}");
    assert!(record.contains("human_gated \"true\""), "{record}");
    // A cycle that proposes nothing is a broken loop, not a quiet success.
    assert!(record.contains("proposals \"") && !record.contains("proposals \"0\""));
    // The record is the same artifact the CLI dry run produces, so a human
    // reviews exactly what the scheduled loop proposed.
    assert_eq!(
        record,
        format!("{}\n", google_trends_learning_cycle().links_notation()),
    );

    let _ = std::fs::remove_file(&memory_path);
    let _ = std::fs::remove_file(&record_path);
}
