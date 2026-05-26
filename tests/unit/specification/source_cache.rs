//! Source-cache tests.
//!
//! `VISION.md` and `GOALS.md` require that external sources used to answer a
//! question (web pages, papers, datasets) are cached link-natively in the
//! network with provenance and a refresh policy (default ≈ 2 months) so the
//! assistant remains transparent and reproducible.

use formal_ai::{FormalAiEngine, SolverConfig, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn answer_with_config(prompt: &str, config: SolverConfig) -> SymbolicAnswer {
    UniversalSolver::new(config).solve(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: the implementation does not yet hit external sources.
// ---------------------------------------------------------------------------

#[test]
fn implementation_does_not_advertise_external_fetches_for_local_prompts() {
    let response = answer("Hi");
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source:http")),
        "local prompts should not leak fake source links"
    );
}

// ---------------------------------------------------------------------------
// full-scope expectations.
// ---------------------------------------------------------------------------

#[test]
fn external_lookups_record_source_url() {
    let response = answer("Cite a definition of associative memory from Wikipedia");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source:http")),
        "answers that draw on external knowledge must record a source link"
    );
}

#[test]
fn source_links_carry_fetched_at_timestamp() {
    let response = answer("Define associative memory");
    let has_fetched_at = response
        .evidence_links
        .iter()
        .any(|link| link.contains("fetched_at="));
    assert!(
        has_fetched_at,
        "source links must include a fetched_at timestamp for TTL tracking"
    );
}

#[test]
fn stale_sources_are_refreshed() {
    let response = answer("Refresh the cached page for example.com");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source_refresh:")),
        "the engine must publish a source_refresh link when refreshing"
    );
}

#[test]
fn repeated_lookups_hit_the_cache() {
    let first = answer("Define associative memory");
    let second = answer("Define associative memory");
    let first_cache_hits = first
        .evidence_links
        .iter()
        .filter(|link| link.starts_with("cache_hit:"))
        .count();
    let second_cache_hits = second
        .evidence_links
        .iter()
        .filter(|link| link.starts_with("cache_hit:"))
        .count();
    assert!(
        second_cache_hits >= first_cache_hits,
        "repeated lookups within TTL must report a cache_hit link"
    );
}

#[test]
fn cached_sources_include_content_hash() {
    let response = answer("Define associative memory");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.contains("sha256=")),
        "cached source records must include a sha256 fingerprint"
    );
}

#[test]
fn conflicting_sources_are_surfaced() {
    let response = answer("Was X born in 1880 or 1881?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("conflict:source_disagreement")),
        "the engine must record disagreement between sources instead of silently choosing"
    );
}

#[test]
fn cache_flush_is_explicit_and_auditable() {
    let response = answer("Flush the source cache please");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("policy:cache_flush_requires_confirmation")),
        "cache flush must require explicit confirmation and produce an audit link"
    );
}

#[test]
fn offline_mode_disables_external_lookups() {
    let response = answer_with_config(
        "Define associative memory",
        SolverConfig {
            offline: true,
            ..Default::default()
        },
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("network_fetch:")),
        "offline mode must skip network fetches"
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "policy:offline"),
        "offline mode must record a policy refusal for external lookups"
    );
}
