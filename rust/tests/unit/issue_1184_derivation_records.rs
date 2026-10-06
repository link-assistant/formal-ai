//! Issue #1184 (E148): white-box derivation for every answer.
//!
//! The derivation record is a projection of the same append-only
//! [`EventLog`] the `--thinking` trace narrates, so the two surfaces cannot
//! disagree (R6). These tests pin:
//! - the projection recovers search queries, fetched URLs, and SHA-256
//!   hashes from both `source:http` payload spellings the tree emits,
//! - the answer id is content-addressed and stable (R1),
//! - a stage a route never populated reports "not recorded", never a
//!   fabricated entry (R5),
//! - the durable store round-trips a record by answer id (R3) and refuses
//!   path-traversing ids,
//! - the `explain` surface and the in-chat self-explanation recipe stay on
//!   disjoint triggers (R7).
//!
//! The live variant — running an online fetch scenario and explaining the
//! resulting answer — lands with the E128 (#1163) stage events that fill
//! `formalized_fragments`; it is recorded as that issue's follow-up, not
//! fabricated here.

use formal_ai::agentic_coding::explain::{EXPLAIN_TASK, is_explain_task};
use formal_ai::derivation::{
    Derivation, FetchRecord, VerificationRecord, answer_derivation_id, explain_answer, store_path,
};
use formal_ai::event_log::EventLog;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn isolated_directory(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-derivation-{}-{test_name}-{nonce}",
        std::process::id()
    ))
}

/// The log of an online answer: two issued queries and two fetches, one in
/// each `source:http` spelling the tree emits.
fn online_answer_log() -> EventLog {
    let mut log = EventLog::new();
    log.append("web_search:request", "how to compile a Kotlin program");
    log.append(
        "source:http",
        "https://kotlinlang.org/docs/command-line.html fetched_at=2026-09-29T00:00:00Z \
         sha256=9f86d081884c7d65a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08 cached=false",
    );
    log.append(
        "source:http",
        "url=https://example.com/kotlin;fetched_at=2026-09-29T00:01:00Z;\
         sha256=2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae;\
         catalog_match=none",
    );
    log.append("web_search:request", "kotlinc command line options");
    log
}

#[test]
fn derivation_carries_search_queries_fetches_and_hashes_for_an_online_answer() {
    let log = online_answer_log();
    let derivation = Derivation::record_for(&log, "answer_0123456789abcdef");

    assert_eq!(
        derivation.search_queries,
        vec![
            String::from("how to compile a Kotlin program"),
            String::from("kotlinc command line options"),
        ],
        "queries are recovered in append order"
    );
    assert_eq!(derivation.fetches.len(), 2, "both payload spellings parse");
    assert_eq!(
        derivation.fetches[0],
        FetchRecord {
            url: String::from("https://kotlinlang.org/docs/command-line.html"),
            sha256: String::from("9f86d081884c7d65a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"),
            fetched_at: String::from("2026-09-29T00:00:00Z"),
        },
        "the SourceCapture trace_payload form is read"
    );
    assert_eq!(
        derivation.fetches[1].url, "https://example.com/kotlin",
        "the semicolon-separated synthesis-runtime form is read"
    );
    assert_eq!(derivation.fetches[1].sha256.len(), 64);
}

#[test]
fn every_symbolic_answer_carries_a_stable_derivation_id() {
    let once = answer_derivation_id("Kotlin programs are compiled with kotlinc.");
    let twice = answer_derivation_id("Kotlin programs are compiled with kotlinc.");
    assert_eq!(once, twice, "the same answer text yields the same id");
    assert_ne!(
        once,
        answer_derivation_id("Scala programs are compiled with scalac."),
        "a different answer text yields a different id"
    );
    let hex = once.strip_prefix("answer_").expect("answer_ prefix");
    assert_eq!(hex.len(), 16, "sixteen hex digits, like every stable_id");
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "the id body is hex"
    );
}

#[test]
fn explain_reports_not_recorded_for_stages_a_route_did_not_populate() {
    // A pure-arithmetic answer: no search, no fetch, no decomposition.
    let mut log = EventLog::new();
    log.append("calculation", "2+2=4");
    let derivation = Derivation::record_for(&log, "answer_fedcba9876543210");

    assert!(derivation.fetches.is_empty(), "nothing is fabricated");
    assert!(derivation.search_queries.is_empty());
    let explanation = derivation.explain_text();
    for stage in [
        "stage search_queries",
        "stage fetches",
        "stage formalized_fragments",
        "stage decomposed_parts",
        "stage verification",
    ] {
        let block = explanation
            .split(stage)
            .nth(1)
            .unwrap_or_else(|| panic!("explanation names `{stage}`"));
        assert!(
            block.starts_with("\n    not recorded"),
            "`{stage}` reports not recorded, got:{block}"
        );
    }
    assert!(explanation.contains("stage recomposition: not recorded"));
    assert!(explanation.contains("stage rendering: not recorded"));
}

#[test]
fn explain_and_thinking_trace_agree_on_the_same_event_log() {
    let log = online_answer_log();

    // Both projections read this one log; neither replays the request.
    let steps = log.thinking_steps_for_answer("Use kotlinc from the command line.");
    let derivation = Derivation::record_for(&log, "answer_0123456789abcdef");

    let narrated_queries: Vec<&str> = steps
        .iter()
        .filter(|step| step.step == "http_chat")
        .map(|step| step.detail.as_str())
        .collect();
    for query in &derivation.search_queries {
        assert!(
            narrated_queries.contains(&query.as_str()),
            "the thinking trace narrates the same query the derivation records: {query}"
        );
    }
}

#[test]
fn durable_record_round_trips_by_answer_id() {
    let root = isolated_directory("round-trip");
    let log = online_answer_log();
    let mut derivation = Derivation::record_for(&log, "answer_0123456789abcdef");
    derivation.recomposition = Some(String::from("bound literal=Hello, Formal AI!"));
    derivation.rendering = Some(String::from("kotlin"));
    derivation.verification.push(VerificationRecord {
        evidence_id: String::from("evidence_0000000000000000"),
        command: String::from("python3 solution.py"),
        exit_code: Some(0),
    });

    let path = derivation
        .persist(&root)
        .expect("the record persists under data/cache/derivations");
    assert!(
        path.starts_with(root.join("data/cache/derivations")),
        "the store follows the sources-registry cache_path convention"
    );

    let loaded = Derivation::load(&root, "answer_0123456789abcdef")
        .expect("the record loads back without replaying the request");
    assert_eq!(loaded, derivation, "the lino round-trip is lossless");

    let explained =
        explain_answer(&root, "answer_0123456789abcdef").expect("explain reads the durable record");
    assert!(explained.contains("sha256 9f86d0"));
    assert!(explained.contains("python3 solution.py exit=0"));
    assert!(
        Derivation::load(&root, "answer_ffffffffffffffff").is_none(),
        "an unknown id is an honest miss"
    );
}

#[test]
fn verification_payload_round_trips_through_its_event_spelling() {
    let record = VerificationRecord {
        evidence_id: String::from("evidence_1234567890abcdef"),
        command: String::from("python3 solution.py"),
        exit_code: Some(0),
    };
    let parsed = VerificationRecord::parse_payload(&record.payload())
        .expect("the payload spelling parses back");
    assert_eq!(parsed, record);

    let mut log = EventLog::new();
    log.append("verify:evidence", record.payload());
    let derivation = Derivation::record_for(&log, "answer_0123456789abcdef");
    assert_eq!(derivation.verification, vec![record]);
}

#[test]
fn explain_command_does_not_collide_with_the_self_explanation_recipe() {
    // The in-chat recipe keeps its keyword triggers untouched…
    assert!(is_explain_task(EXPLAIN_TASK));
    assert!(is_explain_task("explain how formal ai works"));
    // …and a derivation answer id is not one of them, so `formal-ai explain
    // <answer-id>` and the self-explanation recipe route disjointly (R7).
    assert!(!is_explain_task("answer_0123456789abcdef"));

    // The durable store refuses ids that could traverse the tree.
    assert!(store_path(Path::new("/repo"), "answer_ok").is_some());
    assert!(store_path(Path::new("/repo"), "../secrets").is_none());
    assert!(store_path(Path::new("/repo"), "").is_none());
    assert!(store_path(Path::new("/repo"), "a/b").is_none());
}

#[test]
fn lino_values_survive_quoting_and_line_breaks() {
    let mut derivation = Derivation::record_for(&EventLog::new(), "answer_0123456789abcdef");
    derivation
        .search_queries
        .push(String::from("what is \"1 + 1\"?\nsecond line"));
    let text = derivation.to_lino();
    let parsed = Derivation::from_lino(&text).expect("the record parses back");
    assert_eq!(parsed, derivation, "quoted and escaped values round-trip");
}

#[test]
fn solver_early_returns_expose_serialized_id_and_live_record() {
    for prompt in ["hello", "", "1 + 1", "unrecognized qwerty xyz"] {
        let answer = formal_ai::solve(prompt);
        let id = answer.derivation_id();
        let wire = serde_json::to_value(&answer).expect("serialize answer");
        assert_eq!(wire["derivation_id"].as_str(), Some(id.as_str()));
        assert!(answer.evidence_links.contains(&format!("derivation:{id}")));
        let root = std::env::current_dir().expect("working directory");
        let record = Derivation::load(&root, &id).expect("solver persisted derivation");
        assert!(record.rendering.is_some());
        assert!(answer.links_notation.contains(&record.to_lino()));
    }
}

#[test]
fn verification_payload_preserves_shell_separators_and_legacy_spelling() {
    let record = VerificationRecord {
        evidence_id: "evidence_123".into(),
        command: "printf first; printf second exit=inside".into(),
        exit_code: Some(0),
    };
    assert_eq!(
        VerificationRecord::parse_payload(&record.payload()),
        Some(record)
    );
    let legacy = VerificationRecord::parse_payload(
        "evidence_id=evidence_old command=python3 solution.py exit=0",
    )
    .expect("historical payload remains readable");
    assert_eq!(legacy.command, "python3 solution.py");
    assert_eq!(legacy.exit_code, Some(0));
}
