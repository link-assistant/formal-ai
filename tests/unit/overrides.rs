//! Discipline checks for the `data/overrides/` grounding override layer
//! (issue #398, PR #399 review defect 3).
//!
//! Overrides decorate cached external-source records: resolution is
//! `(cache or live API) then overrides`. This suite walks the *entire*
//! `data/overrides` tree (no hard-coded filename) and enforces that every
//! override:
//!
//! 1. maps to an id that has a checked-in cache record at the mirrored path,
//! 2. carries a non-empty `reason`, and
//! 3. is non-redundant — it never repeats a value the cache already holds, so
//!    the layer self-prunes the moment a cache refresh catches up to upstream.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::seed::{cache_contains, override_facts, override_reason, parse_record, resolve};
use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn override_lino_files(overrides_dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(overrides_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("lino"))
        .collect()
}

#[test]
fn overrides_layer_mirrors_the_cache_directory_structure() {
    let root = repo_root();
    let overrides_dir = root.join("data/overrides");
    assert!(
        overrides_dir.is_dir(),
        "data/overrides must exist beside data/cache as the override layer"
    );
    let readme = overrides_dir.join("README.md");
    assert!(
        readme.is_file(),
        "data/overrides/README.md must document the (cache|API) then overrides resolution order"
    );
    let readme_text = fs::read_to_string(&readme).expect("overrides README should be readable");
    assert!(
        readme_text.contains("then") && readme_text.contains("overrides"),
        "overrides README must spell out the cache-then-overrides resolution order"
    );

    // The override tree must mirror the cache's per-source layout so a cached
    // id and its override live at the same relative path.
    for source in ["wikidata/entity", "wikidata/property", "wikidata/lexeme"] {
        assert!(
            overrides_dir.join(source).is_dir(),
            "data/overrides/{source} must mirror data/cache/{source}"
        );
    }
}

#[test]
fn resolve_decorates_the_cache_with_override_facts() {
    // `(cache) then overrides`: the override supplies a key the cache lacks and
    // wins on conflict, while untouched cache values survive.
    let cache = parse_record("Q1\n  labels\n    en First\n    ru Первый\n  type item\n")
        .expect("cache record");
    let over = parse_record(
        "Q1\n  reason \"add hi label; correct ru label\"\n  labels\n    hi पहला\n    ru Перв\n",
    )
    .expect("override record");

    let merged = resolve(&cache, &over);
    let labels = merged
        .children
        .iter()
        .find(|child| child.name == "labels")
        .expect("merged record keeps a labels section");

    let value = |key: &str| {
        labels
            .children
            .iter()
            .find(|entry| entry.name == key)
            .map(|entry| entry.id.as_str())
    };
    assert_eq!(value("en"), Some("First"), "untouched cache value survives");
    assert_eq!(value("hi"), Some("पहला"), "override adds the missing key");
    assert_eq!(value("ru"), Some("Перв"), "override wins on conflict");
    assert!(
        merged.children.iter().any(|child| child.name == "type"),
        "sections the override never mentions are preserved"
    );
}

#[test]
fn overrides_are_disciplined_and_non_redundant() {
    let root = repo_root();
    let overrides_dir = root.join("data/overrides");
    let cache_dir = root.join("data/cache");

    let mut problems = Vec::new();
    let mut checked = 0usize;

    for path in override_lino_files(&overrides_dir) {
        checked += 1;
        let relative = path
            .strip_prefix(&overrides_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        let text = fs::read_to_string(&path).expect("override file should be readable");
        let Some(record) = parse_record(&text) else {
            problems.push(format!(
                "{relative}: override file has no top-level id node"
            ));
            continue;
        };

        // (1) The override id must equal the file stem and have a checked-in
        // cache record at the mirrored relative path.
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();
        if record.name != stem {
            problems.push(format!(
                "{relative}: top-level id `{}` must match the file name `{stem}`",
                record.name
            ));
        }
        let cache_path = cache_dir.join(
            path.strip_prefix(&overrides_dir)
                .expect("override path is under overrides dir"),
        );
        if !cache_path.is_file() {
            problems.push(format!(
                "{relative}: overrides decorate cached records, but no cache file exists at {}",
                cache_path
                    .strip_prefix(&root)
                    .unwrap_or(&cache_path)
                    .display()
            ));
            continue;
        }

        // (2) Every override must record why it exists.
        if override_reason(&record).is_none() {
            problems.push(format!(
                "{relative}: override must carry a non-empty `reason \"...\"` explaining why upstream is insufficient"
            ));
        }

        // (3) An override that repeats a value the cache already holds is
        // redundant and must be removed.
        let cache_text = fs::read_to_string(&cache_path).expect("cache file should be readable");
        let cache_record =
            parse_record(&cache_text).expect("cache file should have a top-level id node");
        let facts = override_facts(&record);
        if facts.is_empty() {
            problems.push(format!(
                "{relative}: override carries no facts; an override must supply at least one corrected value"
            ));
        }
        for fact in facts {
            if cache_contains(&cache_record, &fact) {
                problems.push(format!(
                    "{relative}: redundant override `{} {} {}` already present in the cache record — remove it",
                    fact.section, fact.key, fact.value
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "data/overrides discipline violations:\n{}",
        problems.join("\n")
    );
    assert!(
        checked > 0,
        "the override layer should carry at least one example override so the discipline checks have coverage"
    );
}
