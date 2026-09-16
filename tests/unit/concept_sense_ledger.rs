//! Issue #1138, plan 01 L12 — the sense ledger keeps the recipe, not the answer.
//!
//! A remembered sense may be forgotten and rediscovered from the same committed
//! captures, and the rediscovered record must carry the same content id: that is
//! what makes the ledger a cache rather than a second source of truth. A record
//! whose stored bytes no longer match its digest is rejected and re-derived
//! instead of being trusted.

use std::path::{Path, PathBuf};

use formal_ai::concept_lookup::{ConceptSense, LookupOutcome, RegistrySourceLookup};
use formal_ai::concept_sense_ledger::ConceptSenseLedger;
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::needs::{Need, NeedKind, NeedState};
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::source_walk::{LookupBounds, SourceLookup};

const FIXTURE_DIR: &str = "tests/fixtures/issue-1138-b1";
const HELD_OUT_WORD: &str = "isogram";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR)
}

fn ledger_dir(tag: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("formal-ai-issue-1138-senses-{tag}"));
    let _ = std::fs::remove_dir_all(&path);
    path
}

/// Resolve the held-out word from the committed captures, offline.
fn rediscover() -> Vec<ConceptSense> {
    let client = CachedSourceClient::new(fixture_dir(), CurlSourceTransport).with_online(false);
    let preferences = ServicePreferences::default();
    let mut cache = ServiceAccessibilityCache::new(
        std::env::temp_dir().join("formal-ai-issue-1138-sense-ledger"),
    );
    let mut lookup = RegistrySourceLookup::new(
        &client,
        &preferences,
        &mut cache,
        LookupBounds::default(),
        "en",
        u64::MAX / 2,
    );
    let need = Need {
        need_id: "need:isogram".to_owned(),
        kind: NeedKind::Concept,
        subject: HELD_OUT_WORD.to_owned(),
        language: "en".to_owned(),
        raised_by: "prompt:1".to_owned(),
        source_span: "prompt:1@0:7".to_owned(),
        depth: 0,
        state: NeedState::Open,
        satisfied_by: None,
    };
    match lookup.lookup(&need, &LookupBounds::default()) {
        LookupOutcome::Found(senses) => senses,
        LookupOutcome::NotFound { consulted } => {
            panic!("the committed captures must answer offline; consulted {consulted:?}")
        }
    }
}

#[test]
fn forgotten_senses_are_rediscovered_from_the_same_captures_to_the_same_content_id() {
    let ledger = ConceptSenseLedger::new(ledger_dir("forget"));
    let first = rediscover();
    assert!(!first.is_empty(), "the captures must yield at least one sense");
    for sense in &first {
        ledger.remember(sense).expect("remember a sense");
    }

    let remembered = ledger.content_ids().expect("ledger content ids");
    for id in &remembered {
        ledger.forget(id).expect("forget a sense");
        assert_eq!(
            ledger.recall(id).expect("recall after forget"),
            None,
            "a forgotten sense is really gone"
        );
    }

    let second = rediscover();
    let rediscovered: Vec<String> = second.iter().map(ConceptSense::content_id).collect();
    assert_eq!(
        rediscovered, remembered,
        "rediscovery from the same captures must reproduce the same content ids"
    );
}

#[test]
fn a_tampered_ledger_record_is_rejected_and_re_derived() {
    let directory = ledger_dir("tamper");
    let ledger = ConceptSenseLedger::new(&directory);
    let sense = rediscover().first().cloned().expect("one sense");
    let content_id = sense.content_id();
    ledger.remember(&sense).expect("remember a sense");

    // Tamper with the stored bytes directly: a ledger that trusts its own file
    // would now serve a gloss no source ever published.
    let tampered_gloss = "a meaning nobody published";
    for entry in std::fs::read_dir(&directory).expect("ledger directory").flatten() {
        let path = entry.path();
        if path.is_file() {
            let bytes = std::fs::read_to_string(&path).expect("ledger record");
            std::fs::write(&path, bytes.replace(&sense.gloss, tampered_gloss))
                .expect("tamper with the record");
        }
    }

    assert_eq!(
        ledger.recall(&content_id).expect("recall a tampered record"),
        None,
        "a record whose bytes no longer match its digest is refused, not served"
    );
    let rederived = rediscover().first().cloned().expect("one sense");
    assert_eq!(
        rederived.content_id(),
        content_id,
        "the rejected record is re-derived from the captures, not invented"
    );
    assert_eq!(rederived.gloss, sense.gloss);
}
