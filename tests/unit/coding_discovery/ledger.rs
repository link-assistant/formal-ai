use std::fs;
use std::path::PathBuf;

use formal_ai::coding_task_spec::recognise;
use formal_ai::composition::VerifiedDraft;
use formal_ai::concept_discovery::{CandidatePart, ConceptMap, ConceptNeed};
use formal_ai::discovered_procedures::DiscoveredProcedureLedger;

fn temporary_directory(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-coding-ledger-{name}-{}-{nonce}",
        std::process::id()
    ))
}

fn solution() -> (
    formal_ai::coding_task_spec::CodingTaskSpec,
    ConceptMap,
    VerifiedDraft,
) {
    let spec = recognise(
        "Implement Python function greatest_divisor(a: int, b: int) -> int. Return the greatest common divisor.",
    )
    .expect("coding task spec");
    let candidate = CandidatePart {
        id: "math.gcd".to_owned(),
        kind: "stdlib".to_owned(),
        label: "greatest common divisor".to_owned(),
        language: Some("python".to_owned()),
        code: None,
        callable_name: None,
        source_tests: Vec::new(),
        license: "PSF-2.0".to_owned(),
        source_url: "https://docs.python.org/3.12/library/math.html#math.gcd".to_owned(),
        sha256: "a".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
        score: 9.0,
    };
    let concepts = ConceptMap {
        needs: vec![ConceptNeed {
            phrase: "greatest common divisor".to_owned(),
            structures: Vec::new(),
            candidates: vec![candidate],
            status: "satisfied".to_owned(),
        }],
        evidence: Vec::new(),
    };
    let draft = VerifiedDraft {
        id: "stdlib:math.gcd".to_owned(),
        source: "import math\n\ndef greatest_divisor(a: int, b: int):\n    return math.gcd(a, b)\n"
            .to_owned(),
        assertion_count: 2,
        source_urls: vec!["https://docs.python.org/3.12/library/math.html#math.gcd".to_owned()],
        source_licenses: vec!["PSF-2.0".to_owned()],
        composition: "direct_stdlib math.gcd".to_owned(),
    };
    (spec, concepts, draft)
}

#[test]
fn verified_procedure_round_trips_with_provenance_and_becomes_a_cache_hit() {
    let directory = temporary_directory("round-trip");
    let ledger = DiscoveredProcedureLedger::new(&directory);
    let (spec, concepts, draft) = solution();

    assert!(ledger.recall(&spec).expect("empty ledger").is_none());
    let recorded = ledger
        .remember(&spec, &concepts, &draft)
        .expect("record verified procedure");
    let recalled = ledger
        .recall(&spec)
        .expect("read ledger")
        .expect("cache hit");

    assert_eq!(recalled.id, recorded.id);
    assert_eq!(recalled.composition, draft.composition);
    assert_eq!(recalled.tests_passed, 2);
    assert_eq!(recalled.parts.len(), 1);
    assert_eq!(recalled.parts[0].license, "PSF-2.0");
    assert_eq!(recalled.parts[0].sha256, "a".repeat(64));
    assert!(ledger.path().ends_with("discovered-procedures.lino"));

    fs::remove_dir_all(directory).ok();
}

#[test]
fn forgotten_procedures_are_rediscovered_from_the_same_sources() {
    let directory = temporary_directory("forget");
    let ledger = DiscoveredProcedureLedger::new(&directory);
    let (spec, concepts, draft) = solution();
    let first = ledger.remember(&spec, &concepts, &draft).expect("first");

    fs::remove_file(ledger.path()).expect("forget the procedure ledger");
    assert!(ledger.recall(&spec).expect("forgotten ledger").is_none());
    let rediscovered = ledger
        .remember(&spec, &concepts, &draft)
        .expect("rediscovered");

    assert_eq!(rediscovered.id, first.id);
    fs::remove_dir_all(directory).ok();
}

#[test]
fn a_tampered_entry_is_ignored_and_rederived() {
    let directory = temporary_directory("tamper");
    let ledger = DiscoveredProcedureLedger::new(&directory);
    let (spec, concepts, draft) = solution();
    let original = ledger.remember(&spec, &concepts, &draft).expect("original");
    let text = fs::read_to_string(ledger.path()).expect("ledger text");
    fs::write(
        ledger.path(),
        text.replace(&"a".repeat(64), &"b".repeat(64)),
    )
    .expect("tamper with source digest");

    assert!(ledger.recall(&spec).expect("tampered ledger").is_none());
    let rederived = ledger
        .remember(&spec, &concepts, &draft)
        .expect("rederived");
    assert_eq!(rederived.id, original.id);

    fs::remove_dir_all(directory).ok();
}
