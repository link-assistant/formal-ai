//! Total reference-closure and multi-source infrastructure CI gates (issue #398, PR #399).
//!
//! PR #399 review (comment 4668929105) requires two things the narrower
//! `reference_closure.rs` backbone gate does not cover, and asks that CI fail
//! immediately if either is missing or not working:
//!
//!   1. **Total closure** — *every* non-keyword, non-quoted value token anywhere
//!      in `data/seed/**.lino` must resolve to a defined meaning, a grounded
//!      source id with a cache record, or an override. Not just the structured
//!      `defined-by`/facet/role backbone.
//!   2. **The multi-source `view`** — `WordNet` cached and used, `data/view/`
//!      present with deterministic `M-…` ids, per-field provenance, a working
//!      merge, and a `sources-registry.lino` listing every ingested source with
//!      an API endpoint and a permissive license.
//!
//! The resolver and merge logic live in one place — the Python migrations
//! (`scripts/audit-total-closure.py`, `scripts/build-views.py`). These gates
//! shell out to their machine-readable / `--check` modes so the logic is never
//! duplicated, exactly as the audit module documents.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Run a repo Python script from the repo root, returning (stdout, stderr, success).
fn run_python(args: &[&str]) -> (String, String, bool) {
    let output = Command::new("python3")
        .args(args)
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|err| panic!("failed to run `python3 {}`: {err}", args.join(" ")));
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

/// Total closure: the seed must have zero unresolved value tokens. On failure,
/// name the offending tokens so the gap is actionable — never a bare count.
#[test]
fn seed_has_total_reference_closure() {
    let (stdout, stderr, _) = run_python(&["scripts/audit-total-closure.py", "--json", "."]);
    let report: Value = serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!("audit did not emit JSON ({err}); stderr: {stderr}\nstdout: {stdout}")
    });

    let distinct = report["unresolved_distinct"].as_u64().unwrap_or(u64::MAX);
    if distinct != 0 {
        let unresolved = report["unresolved"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        let sample: Vec<String> = unresolved.keys().take(40).cloned().collect();
        panic!(
            "total reference-closure is not at zero: {distinct} distinct tokens \
             ({} occurrences) resolve to no defined meaning, grounded source, or \
             override. Define or ground them (e.g. `python3 scripts/close-total.py`, \
             `scripts/ground-wordnet.py`, `scripts/ground-wiktionary.py`). \
             First offenders: {sample:?}",
            report["unresolved_occurrences"].as_u64().unwrap_or(0)
        );
    }
}

/// The total-closure backbone counts must not silently collapse: a healthy seed
/// has hundreds of defined meanings and a populated set of grounded sources.
#[test]
fn closure_resolver_sees_a_populated_seed() {
    let (stdout, stderr, _) = run_python(&["scripts/audit-total-closure.py", "--json", "."]);
    let report: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|err| panic!("audit did not emit JSON ({err}); stderr: {stderr}"));
    assert!(
        report["defined"].as_u64().unwrap_or(0) >= 400,
        "expected the seed to define hundreds of meanings, got {}",
        report["defined"]
    );
    assert!(
        report["distinct_value_tokens"].as_u64().unwrap_or(0) >= 1000,
        "expected the audit to inspect the full token surface, got {}",
        report["distinct_value_tokens"]
    );
}

/// `WordNet` must be present and used: the OEWN 2024 per-lemma cache is the
/// keystone source for English content words.
#[test]
fn wordnet_cache_is_present_and_used() {
    let dir = repo_root().join("data/cache/wordnet/en");
    assert!(
        dir.is_dir(),
        "data/cache/wordnet/en is missing; run scripts/ground-wordnet.py"
    );
    let entries = std::fs::read_dir(&dir)
        .expect("wordnet cache dir should be readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("lino"))
        .count();
    assert!(
        entries >= 200,
        "WordNet cache breadth regressed: {entries} .lino entries, floor is 200. \
         Ground more via scripts/ground-wordnet.py rather than removing entries."
    );
}

/// `sources-registry.lino` must enumerate every ingested source with an API
/// endpoint and a permissive license, and must list every source that has a
/// populated cache directory (no silently-unregistered source).
#[test]
fn sources_registry_lists_every_ingested_source() {
    let path = repo_root().join("data/seed/sources-registry.lino");
    let registry = std::fs::read_to_string(&path)
        .expect("data/seed/sources-registry.lino must exist and be UTF-8");
    for source in ["wikidata", "wiktionary", "wordnet"] {
        assert!(
            registry.contains(&format!("source {source}")),
            "sources-registry.lino does not list the `{source}` source"
        );
    }
    for field in ["api ", "license_name ", "license_url ", "cache_path "] {
        assert!(
            registry.contains(field),
            "sources-registry.lino is missing `{field}` entries"
        );
    }
    // Every cache directory that actually holds records must be registered.
    let cache_root = repo_root().join("data/cache");
    if let Ok(entries) = std::fs::read_dir(&cache_root) {
        for entry in entries.filter_map(Result::ok) {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let populated = std::fs::read_dir(entry.path()).is_ok_and(|mut it| it.any(|_| true));
            if populated {
                assert!(
                    registry.contains(&format!("source {name}")),
                    "cache dir data/cache/{name} holds records but is not listed in \
                     sources-registry.lino"
                );
            }
        }
    }
}

/// Issue #444: the external trusted services the procedural how-to handler may
/// consult (wikiHow, Stack Exchange, the MediaWiki family, GitHub) must be
/// enumerated in the registry as an `external_trusted` group, each carrying the
/// settings key the UI toggles bind to and a `default_enabled` flag. This keeps
/// the "available services" list data-driven and the settings opt-in/opt-out
/// section in sync with a single source of truth.
#[test]
fn external_trusted_services_are_registered_with_settings_toggles() {
    let path = repo_root().join("data/seed/sources-registry.lino");
    let registry = std::fs::read_to_string(&path)
        .expect("data/seed/sources-registry.lino must exist and be UTF-8");

    // (source id, settings key the settings UI toggle binds to)
    let services = [
        ("wikihow", "externalServiceWikihow"),
        ("stackexchange", "externalServiceStackExchange"),
        ("wikibooks", "externalServiceMediawikiFamily"),
        ("wikiversity", "externalServiceMediawikiFamily"),
        ("wikivoyage", "externalServiceMediawikiFamily"),
        ("github", "externalServiceGithub"),
    ];

    for (source, settings_key) in services {
        assert!(
            registry.contains(&format!("source {source}")),
            "sources-registry.lino does not list the external trusted source `{source}`"
        );
        assert!(
            registry.contains(&format!("settings_key {settings_key}")),
            "external trusted source `{source}` must declare settings_key `{settings_key}`"
        );
    }

    for field in ["service_group external_trusted", "default_enabled "] {
        assert!(
            registry.contains(field),
            "sources-registry.lino is missing `{field}` on the external trusted services"
        );
    }
}

/// The `data/view/` merge layer must exist, be deterministic, carry per-field
/// provenance, and match its builder (no drift). `--check` reruns the build in
/// memory, reconfirms `M-…` id determinism, and runs the merge-threshold
/// self-tests; it exits non-zero on any failure.
#[test]
fn multi_source_view_is_present_and_consistent() {
    let view_dir = repo_root().join("data/view/en");
    assert!(
        view_dir.is_dir(),
        "data/view/en is missing; run scripts/build-views.py"
    );
    let entities = std::fs::read_dir(&view_dir)
        .expect("view dir should be readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("lino"))
        .count();
    assert!(
        entities >= 100,
        "expected a populated view layer, got {entities}"
    );

    let (stdout, stderr, ok) = run_python(&["scripts/build-views.py", "--check"]);
    assert!(
        ok,
        "scripts/build-views.py --check failed (view drift, non-deterministic id, \
         missing provenance, or merge self-test failure).\nstdout: {stdout}\nstderr: {stderr}"
    );
}

/// At least one view entity must genuinely merge two sources — proof the merge
/// path is exercised on real data, not just unit-tested in isolation.
#[test]
fn view_layer_has_real_multi_source_entities() {
    let view_dir = repo_root().join("data/view/en");
    let mut multi = 0_usize;
    for entry in std::fs::read_dir(&view_dir)
        .expect("view dir should be readable")
        .filter_map(Result::ok)
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("wordnet") && content.contains("wiktionary") {
            multi += 1;
        }
    }
    assert!(
        multi >= 1,
        "no view entity references more than one source; the merge layer is not \
         actually merging anything"
    );
}
