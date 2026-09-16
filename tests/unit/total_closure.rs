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

/// The reviewed closure ceiling, read from `data/meta/closure-audit.lino`.
///
/// Issue #1138 B9, plan 09 leaves 6-7. The number lives in one ledger so a
/// grounding commit touches one file and cannot leave a second copy stale.
fn reviewed_closure_ceiling() -> u64 {
    let path = repo_root().join("data/meta/closure-audit.lino");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} readable: {err}", path.display()));
    let mut measure: Option<&str> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("measure ") {
            measure = Some(rest.trim().trim_matches('"'));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("value ") {
            if measure.take() == Some("unresolved_distinct_honest") {
                return rest
                    .trim()
                    .trim_matches('"')
                    .parse()
                    .unwrap_or_else(|err| panic!("closure-audit.lino value: {err}"));
            }
        }
    }
    panic!("data/meta/closure-audit.lino names no `unresolved_distinct_honest` ceiling");
}

/// Total closure, measured honestly and ratcheted strictly in **both**
/// directions (issue #1138 B9, plan 09 leaves 6-8; renamed from
/// `seed_has_total_reference_closure`).
///
/// The old assertion was self-satisfying: `scripts/close-total.py` wrote
/// `data/seed/closure-generated-*.lino`, and `scripts/audit-total-closure.py`
/// then counted those generated glosses as definitions, so the gap it reported
/// was zero by construction while 3,555 English-only glosses no runtime loads
/// stood in for grounding. Leaf 6 excludes the generated prefix from the audit's
/// definition set and emits `unresolved_distinct_honest`; leaf 7 records the
/// measured value in `data/meta/closure-audit.lino` and this test adopts the
/// strict two-sided rule of `scripts/check-minimal-core-boundary.rs:286-290`:
/// above the ceiling fails ("grew"), below it also fails ("improved; lower the
/// reviewed ceiling").
#[test]
fn seed_closure_gap_only_shrinks() {
    let (stdout, stderr, _) = run_python(&["scripts/audit-total-closure.py", "--json", "."]);
    let report: Value = serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!("audit did not emit JSON ({err}); stderr: {stderr}\nstdout: {stdout}")
    });

    let honest = report["unresolved_distinct_honest"].as_u64().unwrap_or_else(|| {
        panic!(
            "scripts/audit-total-closure.py does not report `unresolved_distinct_honest`. \
             Plan 09 leaf 6 excludes the `closure-generated-` prefix from the audit's \
             definition set and emits the honest number beside the old one, so the gate \
             measures grounding rather than the generator's own output. Report keys: {:?}",
            report.as_object().map(|map| map.keys().cloned().collect::<Vec<_>>())
        )
    });
    let ceiling = reviewed_closure_ceiling();

    let unresolved = report["unresolved"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    let sample: Vec<String> = unresolved.keys().take(40).cloned().collect();

    assert!(
        honest <= ceiling,
        "the honest closure gap grew from {ceiling} to {honest} distinct tokens \
         ({} occurrences) that resolve to no defined meaning, grounded source, or \
         override. Ground them in an authored meanings file the runtime loads; the \
         generator may propose work, never satisfy the gate. First offenders: {sample:?}",
        report["unresolved_occurrences"].as_u64().unwrap_or(0)
    );
    assert!(
        honest >= ceiling,
        "the honest closure gap improved from {ceiling} to {honest}; lower the reviewed \
         ceiling in data/meta/closure-audit.lino in this commit, so the ratchet records \
         the improvement instead of quietly allowing a later regression back to {ceiling}"
    );
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
/// consult (wikiHow, Stack Exchange, the `MediaWiki` family, GitHub) must be
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
        ("wikifunctions", "externalServiceMediawikiFamily"),
        ("rosetta_code", "externalServiceMediawikiFamily"),
        ("wikibooks", "externalServiceMediawikiFamily"),
        ("wikiversity", "externalServiceMediawikiFamily"),
        ("wikivoyage", "externalServiceMediawikiFamily"),
        ("github", "externalServiceGithub"),
        // Issue #535: the original-journalism source the document-verification
        // handler weighs statements against, opt-out-able via the same toggle.
        ("wikinews", "externalServiceMediawikiFamily"),
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

/// Generated closure shards must be **content-addressed** (PR #965 review).
///
/// `data/seed` conflicted in nearly every pull request because `close-total.py`
/// filled `closure-generated-NN.lino` sequentially up to a line cap: each shard's
/// contents then depended on the total size of every block sorted before it, so
/// one new token rewrote its own shard and shifted a block into each following
/// one. Measured on the pre-fix tree, a single new token dirtied 11 of 11 shards
/// when it sorted early and 9 of 11 mid-alphabet.
///
/// The fix places each meaning in the shard chosen by a digest of its own slug,
/// making the file a function of the block alone. This gate pins that invariant
/// so a future refactor cannot quietly restore size-dependent packing — the
/// symptom (merge conflicts) shows up in other people's branches, not in this
/// one, so it would otherwise go unnoticed for a long time.
#[test]
fn generated_closure_shards_are_content_addressed() {
    use sha2::{Digest, Sha256};

    /// Mirrors `SHARD_COUNT` in `scripts/close-total.py`.
    const SHARD_COUNT: u64 = 16;

    fn expected_shard(slug: &str) -> u64 {
        let digest = Sha256::digest(slug.as_bytes());
        let head = u64::from_be_bytes(digest[..8].try_into().expect("8 bytes"));
        head % SHARD_COUNT + 1
    }

    let seed_dir = repo_root().join("data/seed");
    let mut checked = 0usize;
    let mut misplaced: Vec<String> = Vec::new();
    let mut shards_seen = 0u64;

    for index in 1..=SHARD_COUNT {
        let path = seed_dir.join(format!("closure-generated-{index:02}.lino"));
        let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!(
                "missing generated shard {}: {err} — run `python3 scripts/close-total.py`",
                path.display()
            )
        });
        shards_seen += 1;

        for line in text.lines() {
            // Meaning slugs are the two-space-indented heads under `meanings`.
            let Some(rest) = line.strip_prefix("  ") else {
                continue;
            };
            if rest.starts_with(' ') || rest.is_empty() {
                continue;
            }
            let slug = rest.split_whitespace().next().unwrap_or_default();
            checked += 1;
            let want = expected_shard(slug);
            if want != index {
                misplaced.push(format!(
                    "{slug} is in shard {index:02} but hashes to {want:02}"
                ));
            }
        }
    }

    assert_eq!(
        shards_seen, SHARD_COUNT,
        "expected exactly {SHARD_COUNT} generated shards; the file set must stay \
         stable even when a shard empties out"
    );
    assert!(
        checked > 500,
        "only {checked} generated meanings found — the shards look truncated, so \
         this gate would pass vacuously"
    );
    assert!(
        misplaced.is_empty(),
        "{} of {checked} generated meanings are not in the shard their digest \
         selects. Sharding must stay content-addressed (sha256(slug) % {SHARD_COUNT}) \
         so one new token dirties one file; re-run `python3 scripts/close-total.py`. \
         First offenders: {:?}",
        misplaced.len(),
        misplaced.iter().take(10).collect::<Vec<_>>()
    );
}
