//! Issue #1165 (E130): discovery on the production path.
//!
//! Covers the rediscoverable procedure cache: the policy seed governs what a
//! row must carry, `content_id` is recomputed so a stored address can never
//! drift, deleting the cache is total and the miss path re-runs
//! `research_coding_skill_gap`, and the embedded `ORACLE_SNAPSHOTS` bootstrap
//! answers only while the policy seed says it is active — the deletable-cache
//! contract applied to the compiled-in tier. The production wiring points the
//! issue names (the `WriteProgram` branch of the solver and
//! `plan_work_item_execution`) call `cached_or_research`; the six-language
//! online rediscovery run of R1165-7 lands with that wiring and is gated by
//! `FORMAL_AI_LIVE_FETCH` there.

use std::path::PathBuf;

use formal_ai::coding_research_learning::{
    CodingResearchApproval, CodingResearchGap, ResearchedCodingProcedureLedger,
};
use formal_ai::discovery_production::{
    CachedOrDiscovered, ProcedureCache, RediscoverableRecipe, bootstrap_cache_active, fnv1a64,
    grammar_exists, knows_language,
};

/// An isolated cache file per test, so a failing test never destroys the
/// committed cache and tests never see each other's rows.
fn isolated_cache(name: &str) -> PathBuf {
    let directory = std::env::temp_dir()
        .join("formal-ai-issue-1165")
        .join(name.to_string().replace("::", "-"));
    let _unused = std::fs::remove_dir_all(&directory);
    directory
}

fn kotlin_hello_recipe() -> RediscoverableRecipe {
    RediscoverableRecipe {
        language: String::from("kotlin"),
        task: String::from("hello_world"),
        rediscovery_query: String::from(
            "kotlin hello_world verified coding procedure SPDX license",
        ),
        rediscovery_source: String::from("http://helloworldcollection.de/#Kotlin"),
        entry: String::from("fun main() {\n    println(\"Hello, World!\")\n}"),
        verified_output: String::from("Hello, World!"),
        content_id: 0,
    }
}

/// A row without a rediscovery query and source is memorization, not a cache
/// entry: `store` refuses it (R1165-3).
#[test]
fn store_rejects_procedure_without_rediscovery_fields() {
    let mut cache = ProcedureCache::load_at(&isolated_cache("reject"));
    let mut recipe = kotlin_hello_recipe();
    recipe.rediscovery_query.clear();
    recipe.rediscovery_source.clear();
    let error = cache.store(recipe).expect_err("policy must refuse the row");
    assert!(
        error.contains("rediscovery"),
        "the refusal names the missing policy fields: {error}"
    );
    assert!(cache.recipes().is_empty());
}

/// The happy path: store, reload from disk, look up case-insensitively.
#[test]
fn store_lookup_reload_roundtrip() {
    let path = isolated_cache("roundtrip");
    let mut cache = ProcedureCache::load_at(&path);
    cache
        .store(kotlin_hello_recipe())
        .expect("policy-complete row stores");
    let reloaded = ProcedureCache::load_at(&path);
    let recipe = reloaded
        .lookup("Kotlin", "hello_world")
        .expect("lookup is case-insensitive on the language");
    assert_eq!(recipe.entry, kotlin_hello_recipe().entry);
    assert_eq!(recipe.verified_output, "Hello, World!");
    assert_eq!(
        recipe.content_id,
        RediscoverableRecipe::content_address(&recipe.entry),
        "the stored address matches the program"
    );
}

/// `content_id` is the FNV-1a of the program bytes and is recomputed on
/// store, so a caller cannot drift a stored address from its program.
#[test]
fn content_id_is_fnv1a_of_entry_and_recomputed_on_store() {
    let recipe = kotlin_hello_recipe();
    assert_eq!(
        recipe.content_address(&recipe.entry),
        fnv1a64(recipe.entry.as_bytes())
    );
    let mut cache = ProcedureCache::load_at(&isolated_cache("content-id"));
    let mut poisoned = kotlin_hello_recipe();
    poisoned.content_id = u64::MAX;
    cache.store(poisoned).expect("stores");
    assert_eq!(
        cache
            .lookup("kotlin", "hello_world")
            .expect("row")
            .content_id,
        fnv1a64(recipe.entry.as_bytes()),
        "store recomputes the address from the entry"
    );

    // A file row whose stored address disagrees with its program is dropped
    // on load: it is corruption, not a cache entry, and the miss path
    // rediscovers.
    let path = isolated_cache("content-id-drift");
    let mut drifted = ProcedureCache::load_at(&path);
    drifted.store(kotlin_hello_recipe()).expect("stores");
    let text = std::fs::read_to_string(&path).expect("cache file");
    std::fs::write(&path, text.replace("content_id \"0x", "content_id \"0xf")).expect("drift");
    assert!(
        ProcedureCache::load_at(&path)
            .lookup("kotlin", "hello_world")
            .is_none(),
        "a drifted address must not answer"
    );
}

/// R1165-7's first half, offline: `delete_all` empties the cache file
/// completely — the whole cache is one deletable artifact.
#[test]
fn delete_all_empties_the_cache_file() {
    let path = isolated_cache("delete-all");
    let mut cache = ProcedureCache::load_at(&path);
    cache.store(kotlin_hello_recipe()).expect("stores");
    cache.delete_all();
    assert!(cache.recipes().is_empty());
    let reloaded = ProcedureCache::load_at(&path);
    assert!(
        reloaded.lookup("kotlin", "hello_world").is_none(),
        "no row survives deletion"
    );
    assert!(
        std::fs::read_to_string(&path)
            .expect("the file survives as an empty cache")
            .contains("coding_procedure_cache"),
        "the empty cache keeps its document head"
    );
}

/// The miss path routes through `research_coding_skill_gap` and stores the
/// verified procedure with its rediscovery query (R1165-1), on the same
/// offline capture-transport idiom issue #919 uses: the search endpoint
/// answers with one documented page, research verifies the procedure against
/// it, and no network is touched.
#[test]
fn miss_path_runs_research_and_stores_the_verified_procedure() {
    use formal_ai::{CachedSourceClient, FetchError, SourceTransport};

    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let requests = Arc::new(AtomicUsize::new(0));
    struct ServingCount {
        requests: Arc<AtomicUsize>,
    }
    impl SourceTransport for ServingCount {
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

    let path = isolated_cache("miss-path");
    let mut cache = ProcedureCache::load_at(&path);
    let transport = ServingCount {
        requests: Arc::clone(&requests),
    };
    let client = CachedSourceClient::new(path.join("source-cache"), transport);
    let candidate = "def main\n  __COUNT_TO_THREE__\nend\n";
    let verified = "def main\n  1.upto(3) { |number| puts number }\nend\n";
    let mut gap = CodingResearchGap::for_program_task("count_to_three", "ruby");
    let mut ledger = ResearchedCodingProcedureLedger::new();
    let answer = formal_ai::discovery_production::cached_or_research(
        &mut cache,
        &mut gap,
        &mut ledger,
        &client,
        candidate,
        verified,
        &CodingResearchApproval::granted("pull_request_review"),
        "https://research.invalid/ruby-count",
    )
    .expect("research verifies the candidate");
    assert!(
        !answer.was_cached(),
        "an empty cache must route to research"
    );
    let recipe = answer.recipe();
    assert_eq!(
        recipe.entry, verified,
        "the row reuses the verified produced program"
    );
    assert_eq!(recipe.verified_output, verified);
    assert_eq!(
        recipe.rediscovery_source,
        "https://research.invalid/ruby-count"
    );
    assert!(
        !recipe.rediscovery_query.is_empty(),
        "the row names where a rediscovery starts"
    );
    assert!(
        cache.lookup("ruby", "count_to_three").is_some(),
        "the verified procedure is stored"
    );

    // The second ask is a cache hit: no research round, the row answers.
    let mut gap = CodingResearchGap::for_program_task("count_to_three", "ruby");
    let mut ledger = ResearchedCodingProcedureLedger::new();
    let answer = formal_ai::discovery_production::cached_or_research(
        &mut cache,
        &mut gap,
        &mut ledger,
        &client,
        candidate,
        verified,
        &CodingResearchApproval::granted("pull_request_review"),
        "https://research.invalid/ruby-count",
    )
    .expect("lookup answers");
    assert!(matches!(answer, CachedOrDiscovered::Cached(_)));
    assert!(
        requests.load(Ordering::SeqCst) < 6,
        "a cache hit must not re-run the research round"
    );
}

/// A grammar exists for the languages the CST seed carries and not for ones
/// it does not — the first half of the R1165-4 `knows_language` reading.
#[test]
fn grammar_exists_reads_the_cst_seed() {
    for language in ["kotlin", "php", "swift", "scala", "rust", "r"] {
        assert!(grammar_exists(language), "{language} carries a CST grammar");
    }
    assert!(!grammar_exists("latin-vulgate"));
}

/// `knows_language` is "a grammar exists and discovery found a procedure":
/// kotlin has both a grammar and a bootstrap-recorded discovery, rust has a
/// grammar but no recorded procedure yet (its row appears with the
/// rediscovery run of R1165-7), and a grammarless language never answers.
#[test]
fn knows_language_requires_grammar_and_procedure() {
    assert!(
        knows_language("kotlin"),
        "grammar plus a bootstrap-recorded discovery answers"
    );
    assert!(
        !knows_language("rust"),
        "a grammar with no recorded procedure is a rediscovery away, not knowledge"
    );
    assert!(
        !knows_language("latin-vulgate"),
        "no grammar means no amount of caching answers"
    );
}

/// The bootstrap tier obeys the policy seed: while `bootstrap` carries
/// `active "true"` the embedded snapshots are the record of past discovery.
/// Flipping the seed is the deletion path the issue's wiring completes.
#[test]
fn bootstrap_tier_is_governed_by_the_policy_seed() {
    assert!(
        bootstrap_cache_active(),
        "the committed policy keeps the bootstrap active until the wiring retires it"
    );
    assert!(formal_ai::knowledge::CodingOracle::knows_language("kotlin"));
    assert!(formal_ai::knowledge::CodingOracle::lookup("hello_world", "kotlin").is_some());
}
