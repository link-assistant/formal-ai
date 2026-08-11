#!/usr/bin/env rust-script
//! Enforce the cached-record budget for `data/cache/**`.
//!
//! Issue #960 (R222-1, <https://github.com/link-assistant/formal-ai/pull/222#issuecomment-4513844358>):
//! "we should cache not more than 128 the most frequently used words ... our
//! system should be lightweight by default". That number lives in code as
//! [`MAX_SEED_RECORDS_PER_BUCKET`] (`src/translation/cache.rs`), but nothing
//! failed when a bucket grew past it: `data/cache/wikidata/entity` holds 406
//! records, over 3x the cap, with CI green throughout.
//!
//! ## What a bucket is
//!
//! A bucket is a leaf directory under `data/cache/` (for example
//! `data/cache/wikidata/entity`, `data/cache/wordnet/en`). A *record* is a
//! distinct file stem inside it: `Q1860.json` + `Q1860.lino` are one record,
//! not two.
//!
//! ## Why two rules instead of one
//!
//! A flat 128-record cap collides head-on with a later, equally binding
//! requirement: the total reference-closure gate
//! (`scripts/audit-total-closure.py`) demands that *every* bare token in
//! `data/seed/*.lino` resolve to a cached record. Three buckets are already
//! above 128 purely because the seed references that many ids and lemmas —
//! deleting records to satisfy the cap would fail closure, and closure is what
//! keeps natural language out of the engine. Sharding the directory would keep
//! both numbers happy while changing nothing about the repository's weight,
//! which is the thing the cap actually protects.
//!
//! So the budget is enforced as the requirement's *intent*:
//!
//!   * **Capped buckets** (everything not listed in [`CLOSURE_DRIVEN_BUCKETS`])
//!     fail the build above [`MAX_RECORDS_PER_BUCKET`]. Speculative offline
//!     bulk-caching is blocked outright.
//!   * **Closure-driven buckets** are exempt from the count, and pay for the
//!     exemption with a stricter rule: every record must be *referenced* from
//!     `data/seed/**`, `src/**`, or another cache record (the recursive
//!     grounding closure `tests/unit/semantic_grounding.rs` walks). An
//!     unreferenced record is speculative caching by another name and fails.
//!     Their overflow above the cap is reported as a warning on every run, so
//!     the debt stays visible and attributable.
//!
//! Usage:
//!   rust-script scripts/check-cache-budget.rs          # check (CI/local)
//!   rust-script --test scripts/check-cache-budget.rs   # inline unit tests
//!
//! ```cargo
//! [dependencies]
//! walkdir = "2"
//! ```

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::Path;
#[cfg(not(test))]
use std::process::exit;
use walkdir::WalkDir;

/// Mirror of `formal_ai::translation::cache::MAX_SEED_RECORDS_PER_BUCKET`.
/// Kept in sync by `records_per_bucket_cap_matches_the_library_constant`.
const MAX_RECORDS_PER_BUCKET: usize = 128;

/// Path of the library constant this script enforces.
const CACHE_CONSTANT_PATH: &str = "src/translation/cache.rs";

/// Buckets whose size is dictated by the total reference-closure gate rather
/// than by a caching policy decision. Each entry states why it cannot simply be
/// trimmed to the cap; adding one is a deliberate, reviewable act.
const CLOSURE_DRIVEN_BUCKETS: &[ExemptBucket] = &[
    ExemptBucket {
        path: "data/cache/wikidata/entity",
        reason: "every Q-id referenced by data/seed/**, by src/**, or by another cached record must have a checked-in record (tests/unit/semantic_grounding.rs); trimming to 128 fails the closure gate",
    },
    ExemptBucket {
        path: "data/cache/wordnet/en",
        reason: "one record per seed lemma; scripts/audit-total-closure.py resolves bare seed tokens through this bucket",
    },
    ExemptBucket {
        path: "data/cache/wiktionary/en",
        reason: "one record per seed lemma; scripts/audit-total-closure.py resolves bare seed tokens through this bucket",
    },
];

/// Directories scanned for references to cached records.
const REFERENCE_ROOTS: &[&str] = &["data/seed", "data/cache", "data/overrides", "src"];

/// File extensions scanned for references.
const REFERENCE_EXTENSIONS: &[&str] = &["lino", "rs", "json", "rq"];

/// Root of the cache tree.
const CACHE_ROOT: &str = "data/cache";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExemptBucket {
    path: &'static str,
    reason: &'static str,
}

/// One bucket's measured contents.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Bucket {
    /// Repository-relative path, `/`-separated.
    path: String,
    /// Distinct record stems, sorted.
    records: Vec<String>,
}

impl Bucket {
    fn exemption(&self) -> Option<&'static ExemptBucket> {
        CLOSURE_DRIVEN_BUCKETS
            .iter()
            .find(|exempt| exempt.path == self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OverflowFinding {
    bucket: String,
    records: usize,
    cap: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OrphanFinding {
    bucket: String,
    record: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct CheckResult {
    /// Capped buckets above the cap: hard failures.
    overflow_violations: Vec<OverflowFinding>,
    /// Closure-driven buckets above the cap: reported, never blocking.
    overflow_warnings: Vec<OverflowFinding>,
    /// Records nothing refers to: hard failures.
    orphan_violations: Vec<OrphanFinding>,
}

impl CheckResult {
    const fn is_clean(&self) -> bool {
        self.overflow_violations.is_empty() && self.orphan_violations.is_empty()
    }
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn relative_path(path: &Path, root: &Path) -> String {
    normalized_path(path.strip_prefix(root).unwrap_or(path))
}

/// Collect every bucket under `data/cache`, keyed by its relative path.
///
/// Only directories that directly contain files are buckets; intermediate
/// directories (`data/cache/wikidata`) are not counted, so a sharded layout
/// would be measured shard by shard.
fn collect_buckets(root: &Path) -> Vec<Bucket> {
    let cache_root = root.join(CACHE_ROOT);
    let mut buckets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for entry in WalkDir::new(&cache_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if stem.starts_with('.') || path.file_name().is_some_and(|name| name == "README.md") {
            continue;
        }
        let Some(parent) = path.parent() else {
            continue;
        };
        buckets
            .entry(relative_path(parent, root))
            .or_default()
            .insert(stem.to_string());
    }

    buckets
        .into_iter()
        .map(|(path, records)| Bucket {
            path,
            records: records.into_iter().collect(),
        })
        .collect()
}

/// Split a line into identifier-ish tokens: Wikidata ids and lemmas survive,
/// punctuation and Links Notation structure do not.
fn tokenize(line: &str) -> impl Iterator<Item = &str> {
    line.split(|character: char| !(character.is_ascii_alphanumeric() || character == '-'))
        .filter(|token| !token.is_empty())
}

/// Record stems referenced from somewhere other than the record's own files.
///
/// A cached record legitimately names itself (`Q1860.lino` starts with
/// `Q1860`), so a file is only evidence of a reference when its own stem
/// differs from the token.
fn referenced_records(root: &Path, wanted: &HashSet<String>) -> HashSet<String> {
    let mut referenced = HashSet::new();

    for reference_root in REFERENCE_ROOTS {
        for entry in WalkDir::new(root.join(reference_root))
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let has_scanned_extension = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| REFERENCE_EXTENSIONS.contains(&extension));
            if !has_scanned_extension {
                continue;
            }
            let own_stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_string();

            let Ok(content) = fs::read_to_string(path) else {
                continue;
            };
            for line in content.lines() {
                for token in tokenize(line) {
                    if token == own_stem {
                        continue;
                    }
                    if wanted.contains(token) {
                        referenced.insert(token.to_string());
                    }
                }
            }
        }
    }

    referenced
}

fn check_repository(root: &Path) -> CheckResult {
    let buckets = collect_buckets(root);
    check_buckets(root, &buckets)
}

fn check_buckets(root: &Path, buckets: &[Bucket]) -> CheckResult {
    let mut result = CheckResult::default();

    let exempt_records: HashSet<String> = buckets
        .iter()
        .filter(|bucket| bucket.exemption().is_some())
        .flat_map(|bucket| bucket.records.iter().cloned())
        .collect();
    let referenced = if exempt_records.is_empty() {
        HashSet::new()
    } else {
        referenced_records(root, &exempt_records)
    };

    for bucket in buckets {
        let finding = OverflowFinding {
            bucket: bucket.path.clone(),
            records: bucket.records.len(),
            cap: MAX_RECORDS_PER_BUCKET,
        };

        match bucket.exemption() {
            None => {
                if bucket.records.len() > MAX_RECORDS_PER_BUCKET {
                    result.overflow_violations.push(finding);
                }
            }
            Some(_) => {
                if bucket.records.len() > MAX_RECORDS_PER_BUCKET {
                    result.overflow_warnings.push(finding);
                }
                for record in &bucket.records {
                    if !referenced.contains(record) {
                        result.orphan_violations.push(OrphanFinding {
                            bucket: bucket.path.clone(),
                            record: record.clone(),
                        });
                    }
                }
            }
        }
    }

    result
}

/// The cap this script enforces must be the constant the library documents;
/// two numbers that can drift are the failure mode issue #960 is about.
fn library_cap(source: &str) -> Option<usize> {
    source
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pub const MAX_SEED_RECORDS_PER_BUCKET: usize = ")
        })
        .and_then(|value| value.trim_end_matches(';').trim().parse().ok())
}

#[cfg(not(test))]
fn report(result: &CheckResult, cap_matches: bool) {
    for warning in &result.overflow_warnings {
        println!(
            "::notice file={}::Closure-driven cache bucket holds {} records (cap {}). Exempt from the hard cap because the total-closure gate requires one record per referenced id; every record must stay referenced.",
            warning.bucket, warning.records, warning.cap
        );
        println!(
            "NOTICE: {} holds {} records, above the {}-record cap (closure-driven exemption)",
            warning.bucket, warning.records, warning.cap
        );
    }

    if !result.overflow_violations.is_empty() {
        println!("\nCache buckets over the {MAX_RECORDS_PER_BUCKET}-record cap:\n");
        for violation in &result.overflow_violations {
            println!(
                "  {}: {} records (exceeds cap of {})",
                violation.bucket, violation.records, violation.cap
            );
        }
        println!(
            "\nCache by actual need, not in bulk: drop records the seed does not reference, or\n\
             split the bucket. See src/translation/cache.rs::MAX_SEED_RECORDS_PER_BUCKET.\n"
        );
    }

    if !result.orphan_violations.is_empty() {
        println!("\nCached records nothing references:\n");
        for violation in &result.orphan_violations {
            println!("  {}/{}", violation.bucket, violation.record);
        }
        println!(
            "\nA closure-driven bucket is exempt from the record cap only while every record is\n\
             required. Delete unreferenced records or reference them from data/seed.\n"
        );
    }

    if !cap_matches {
        println!(
            "\n  {CACHE_CONSTANT_PATH}: MAX_SEED_RECORDS_PER_BUCKET no longer equals {MAX_RECORDS_PER_BUCKET}; update this script so the gate keeps enforcing the documented constant.\n"
        );
    }
}

#[cfg(not(test))]
fn main() {
    println!("\nChecking cached-record budget for data/cache/**...\n");

    let root = std::env::current_dir().expect("Failed to get current directory");
    let result = check_repository(&root);
    let cap_matches = fs::read_to_string(root.join(CACHE_CONSTANT_PATH))
        .ok()
        .and_then(|source| library_cap(&source))
        .is_some_and(|cap| cap == MAX_RECORDS_PER_BUCKET);

    report(&result, cap_matches);

    if result.is_clean() && cap_matches {
        println!("All cache buckets are within their record budget\n");
        exit(0);
    }
    exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("check-cache-budget-{name}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_records(dir: &Path, prefix: &str, count: usize) {
        fs::create_dir_all(dir).unwrap();
        for index in 0..count {
            fs::write(dir.join(format!("{prefix}{index}.json")), "{}\n").unwrap();
            fs::write(dir.join(format!("{prefix}{index}.lino")), "record\n").unwrap();
        }
    }

    /// `Q1860.json` and `Q1860.lino` are one cached record, not two: the cap
    /// counts what was fetched, not how many encodings we keep of it.
    #[test]
    fn json_and_lino_siblings_count_as_one_record() {
        let repo = temp_dir("record-identity");
        write_records(&repo.join("data/cache/wikidata/property"), "P", 3);

        let buckets = collect_buckets(&repo);

        assert_eq!(
            buckets,
            vec![Bucket {
                path: "data/cache/wikidata/property".to_string(),
                records: vec!["P0".to_string(), "P1".to_string(), "P2".to_string()],
            }]
        );
    }

    /// Issue #960: this is the failure the repository shipped for months — a
    /// bucket 3x over the documented cap with nothing red.
    #[test]
    fn capped_bucket_over_the_cap_fails() {
        let repo = temp_dir("over-cap");
        write_records(
            &repo.join("data/cache/wikidata/property"),
            "P",
            MAX_RECORDS_PER_BUCKET + 1,
        );

        let result = check_repository(&repo);

        assert_eq!(
            result.overflow_violations,
            vec![OverflowFinding {
                bucket: "data/cache/wikidata/property".to_string(),
                records: MAX_RECORDS_PER_BUCKET + 1,
                cap: MAX_RECORDS_PER_BUCKET,
            }]
        );
        assert!(!result.is_clean());
    }

    #[test]
    fn capped_bucket_at_the_cap_passes() {
        let repo = temp_dir("at-cap");
        write_records(
            &repo.join("data/cache/wikidata/property"),
            "P",
            MAX_RECORDS_PER_BUCKET,
        );

        let result = check_repository(&repo);

        assert_eq!(result, CheckResult::default());
        assert!(result.is_clean());
    }

    /// The exemption buys count, not licence: an exempt bucket still fails when
    /// it caches something nothing asked for.
    #[test]
    fn closure_driven_bucket_warns_on_overflow_but_fails_on_orphans() {
        let repo = temp_dir("closure-driven");
        let entity_dir = repo.join("data/cache/wikidata/entity");
        write_records(&entity_dir, "Q", MAX_RECORDS_PER_BUCKET + 1);
        let seed_dir = repo.join("data/seed");
        fs::create_dir_all(&seed_dir).unwrap();
        let mut seed = String::from("meanings\n");
        for index in 0..MAX_RECORDS_PER_BUCKET {
            seed.push_str(&format!("  thing-{index}\n    wikidata Q{index}\n"));
        }
        fs::write(seed_dir.join("meanings.lino"), seed).unwrap();

        let result = check_repository(&repo);

        assert_eq!(
            result.overflow_warnings,
            vec![OverflowFinding {
                bucket: "data/cache/wikidata/entity".to_string(),
                records: MAX_RECORDS_PER_BUCKET + 1,
                cap: MAX_RECORDS_PER_BUCKET,
            }]
        );
        assert_eq!(result.overflow_violations, Vec::new());
        assert_eq!(
            result.orphan_violations,
            vec![OrphanFinding {
                bucket: "data/cache/wikidata/entity".to_string(),
                record: format!("Q{MAX_RECORDS_PER_BUCKET}"),
            }]
        );
        assert!(!result.is_clean());
    }

    /// Records reference each other: `L3412.lino` naming `Q4833830` is why that
    /// entity is checked in, and the recursive grounding closure in
    /// `tests/unit/semantic_grounding.rs` walks exactly those edges.
    #[test]
    fn a_record_referenced_only_by_another_record_is_not_an_orphan() {
        let repo = temp_dir("record-to-record");
        let entity_dir = repo.join("data/cache/wikidata/entity");
        write_records(&entity_dir, "Q", 1);
        let lexeme_dir = repo.join("data/cache/wikidata/lexeme");
        fs::create_dir_all(&lexeme_dir).unwrap();
        fs::write(lexeme_dir.join("L1.lino"), "L1\n  sense\n    item Q0\n").unwrap();

        let result = check_repository(&repo);

        assert_eq!(result.orphan_violations, Vec::new());
        assert!(result.is_clean());
    }

    /// A record naming itself is not a reference to itself.
    #[test]
    fn self_naming_record_is_still_an_orphan() {
        let repo = temp_dir("self-naming");
        let entity_dir = repo.join("data/cache/wikidata/entity");
        fs::create_dir_all(&entity_dir).unwrap();
        fs::write(entity_dir.join("Q0.lino"), "Q0\n  labels\n    en zero\n").unwrap();

        let result = check_repository(&repo);

        assert_eq!(
            result.orphan_violations,
            vec![OrphanFinding {
                bucket: "data/cache/wikidata/entity".to_string(),
                record: "Q0".to_string(),
            }]
        );
    }

    #[test]
    fn records_per_bucket_cap_matches_the_library_constant() {
        // `rust-script --test` runs from its own build directory, so locate the
        // repository through this file's path rather than the process cwd.
        let repository_root = Path::new(file!())
            .parent()
            .and_then(Path::parent)
            .expect("script lives in <repo>/scripts");
        let source = fs::read_to_string(repository_root.join(CACHE_CONSTANT_PATH))
            .expect("src/translation/cache.rs should be readable");

        assert_eq!(library_cap(&source), Some(MAX_RECORDS_PER_BUCKET));
    }

    #[test]
    fn library_cap_reads_the_documented_constant() {
        let source = "/// docs\npub const MAX_SEED_RECORDS_PER_BUCKET: usize = 128;\n";

        assert_eq!(library_cap(source), Some(128));
    }

    #[test]
    fn every_exempt_bucket_states_a_reason() {
        for bucket in CLOSURE_DRIVEN_BUCKETS {
            assert!(
                bucket.reason.len() > 40,
                "{} must document why it cannot be trimmed to the cap",
                bucket.path
            );
        }
    }
}
