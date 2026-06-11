use super::{
    cache_capacity, within_cache_capacity, CodingOracle, KnowledgeSource, KNOWLEDGE_CACHE_FLOOR,
};

#[test]
fn cache_capacity_floors_small_sources_at_512() {
    // 1% of a small source is below the floor, so the floor wins — but never
    // more than the source actually holds.
    assert_eq!(cache_capacity(10), 10);
    assert_eq!(cache_capacity(600), KNOWLEDGE_CACHE_FLOOR);
    assert_eq!(cache_capacity(KNOWLEDGE_CACHE_FLOOR), KNOWLEDGE_CACHE_FLOOR);
}

#[test]
fn cache_capacity_caps_large_sources_at_one_percent() {
    // 1% of 100_000 is 1_000, comfortably above the 512 floor.
    assert_eq!(cache_capacity(100_000), 1_000);
    // 1% rounds up so we never cap below a single item for a non-empty source.
    assert_eq!(cache_capacity(51_201), 513);
    assert_eq!(cache_capacity(0), 0);
}

#[test]
fn within_cache_capacity_matches_the_cap() {
    assert!(within_cache_capacity(1_000, 100_000));
    assert!(!within_cache_capacity(1_001, 100_000));
}

#[test]
fn committed_snapshots_stay_within_the_cache_cap() {
    // Ratchet: the committed popular-case cache must never grow past the policy
    // for any source, so it can never silently become a mirror.
    for source in [
        KnowledgeSource::RosettaCode,
        KnowledgeSource::Wikifunctions,
        KnowledgeSource::HelloWorldCollection,
        KnowledgeSource::StackOverflow,
    ] {
        let cached = CodingOracle::cached_count(source);
        assert!(
            within_cache_capacity(cached, source.approximate_catalog_size()),
            "{} caches {cached} items, over the cap",
            source.display_name()
        );
    }
}

#[test]
fn oracle_resolves_hello_world_beyond_the_builtin_catalogue() {
    let kotlin = CodingOracle::lookup("hello_world", "Kotlin").expect("kotlin hello world");
    assert_eq!(kotlin.language_slug, "kotlin");
    assert_eq!(kotlin.expected_output, "Hello, World!");
    assert!(kotlin.code.contains("println"));
    assert_eq!(kotlin.source, KnowledgeSource::HelloWorldCollection);

    // Case-insensitive label and bare slug both resolve.
    assert!(CodingOracle::lookup("hello_world", "swift").is_some());
    assert!(CodingOracle::lookup("hello_world", "BASH").is_some());
}

#[test]
fn oracle_covers_a_non_trivial_rosetta_task() {
    let factorial = CodingOracle::lookup("factorial", "kotlin").expect("kotlin factorial");
    assert_eq!(factorial.source, KnowledgeSource::RosettaCode);
    assert_eq!(factorial.expected_output, "120");
}

#[test]
fn oracle_declines_unknown_pairs() {
    // Languages already in the static catalogue are not the oracle's job, and a
    // genuinely unknown language returns None so the caller stays on its path.
    assert!(CodingOracle::lookup("hello_world", "klingon").is_none());
    assert!(CodingOracle::lookup("unsupported_task", "kotlin").is_none());
}

#[test]
fn oracle_knows_language_and_lists_them() {
    assert!(CodingOracle::knows_language("Kotlin"));
    assert!(!CodingOracle::knows_language("rust"));
    let labels = CodingOracle::languages();
    assert!(labels.contains(&"Kotlin"));
    assert!(labels.contains(&"Haskell"));
}

#[test]
fn knowledge_source_slugs_are_filesystem_safe_and_distinct() {
    let slugs: Vec<&str> = [
        KnowledgeSource::RosettaCode,
        KnowledgeSource::Wikifunctions,
        KnowledgeSource::HelloWorldCollection,
        KnowledgeSource::StackOverflow,
    ]
    .iter()
    .map(|source| source.slug())
    .collect();
    for slug in &slugs {
        assert!(slug.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
    }
    let mut unique = slugs.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), slugs.len());
}
