//! Issue #1138, plan 02 L4 and L16–L18 — the seed becomes a deletable bootstrap.
//!
//! Two `include_str!` calls make the idiom seed a compile-time dependency today,
//! so "delete the seed and see what the system can still do" cannot even be
//! expressed. After plan 02 the catalog reads the seed at runtime, an absent
//! seed yields an empty catalog rather than a panic, a forgotten fragment is
//! rediscovered to the same content id, and no rediscovery query names a
//! benchmark identifier.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::fragment_catalog::{Fragment, FragmentCatalog, FragmentLedger, FragmentOrigin};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

/// The two seed files the bootstrap catalog reads.
const BOOTSTRAP_SEED: [&str; 2] = [
    "data/seed/meanings-coding-structure.lino",
    "data/seed/coding-discovery-runtime.lino",
];

fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-1138-fragments-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temporary directory");
    path
}

#[test]
fn bootstrap_catalog_is_absent_without_the_seed_and_never_panics() {
    // A seed directory that exists but holds none of the bootstrap files is
    // exactly the state the deletability job runs the suite in.
    let empty_seed = temp_dir("no-seed");
    let catalog = FragmentCatalog::bootstrap_from(&empty_seed);

    assert!(
        catalog.fragments().is_empty(),
        "a missing seed yields an empty catalog, not a panic and not a guess"
    );
    assert_eq!(
        catalog.get("extend_run"),
        None,
        "a missing fragment id is an absence, never a panic"
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in BOOTSTRAP_SEED {
        assert!(
            root.join(relative).is_file(),
            "{relative} is the bootstrap the catalog reads at runtime"
        );
    }
    assert!(
        !FragmentCatalog::bootstrap().fragments().is_empty(),
        "with the seed present the same code path yields the bootstrap fragments"
    );
}

#[test]
fn forgotten_fragments_are_rediscovered_to_the_same_content_id() {
    let before = FragmentCatalog::bootstrap();
    let expected = before.content_id();
    assert!(
        !expected.is_empty(),
        "the catalog is content-addressed over its fragments"
    );

    // Forget: rebuild with the seed moved aside. Rediscover: replay every
    // fragment through the ledger the offline rediscovery writes.
    let ledger_dir = temp_dir("rediscover");
    let ledger = FragmentLedger::new(&ledger_dir);
    for fragment in before.fragments() {
        let rediscovered = Fragment {
            origin: FragmentOrigin::Rediscovered,
            ..fragment.clone()
        };
        assert!(
            !rediscovered.rediscovery_query.is_empty(),
            "{} has no query that could rediscover it",
            rediscovered.id
        );
        ledger.remember(&rediscovered).expect("remember a fragment");
    }

    let after = FragmentCatalog::bootstrap_from(temp_dir("forgotten")).with_rediscovered(&ledger);
    assert_eq!(
        after.content_id(),
        expected,
        "forget → rediscover → the same content id, or the seed was the capability"
    );
    assert!(
        after
            .fragments()
            .iter()
            .all(|fragment| fragment.origin == FragmentOrigin::Rediscovered),
        "every fragment in the rebuilt catalog came back from a source"
    );
}

#[test]
fn rediscovery_queries_name_no_benchmark_identifier() {
    for fragment in FragmentCatalog::bootstrap().fragments() {
        let query = &fragment.rediscovery_query;
        assert!(
            !query.contains("http"),
            "{} rediscovers itself by a phrase, not a URL: {query}",
            fragment.id
        );
        assert!(
            query.split_whitespace().count() >= 2,
            "{} rediscovers itself by a natural-language phrase: {query}",
            fragment.id
        );
        for forbidden in ["humaneval", "mbpp", "run_length", "find_position", "edit_steps"] {
            assert!(
                !query.to_lowercase().contains(forbidden),
                "{} names a benchmark identifier in its query: {query}",
                fragment.id
            );
        }
        assert!(
            !fragment.grounding.is_empty() && !fragment.license.is_empty(),
            "{} must carry its grounding and license",
            fragment.id
        );
    }
}
