//! Refresh the seeded raw-API-response cache from the live Wikipedia /
//! Wikidata / Wiktionary APIs.
//!
//! Run with:
//!
//! ```bash
//! FORMAL_AI_LIVE_API=1 cargo run --example refresh_translation_cache
//! ```
//!
//! The example drives the full
//! `source → formalize → meaning → deformalize → target` pipeline against
//! a curated set of source surfaces, populates the gitignored on-disk
//! accelerator under `data/wikidata-cache/` and `data/wiktionary-cache/`
//! with the verbatim API bodies, then bundles those bodies into the legacy
//! `.lino` replay files at `data/seed/api-cache/*.lino`.
//!
//! Issue #398 removed those replay files from the committed seed data in favor
//! of explicit source snapshots under `data/cache/wikidata/`. This example is
//! still useful for local cache refresh experiments or older checkouts, but its
//! output is not the current reviewed seed source.
//!
//! Caps (issue #221):
//! - At most [`MAX_SEED_RECORDS_PER_BUCKET`] records per bucket.
//! - Each `.lino` file stays under [`MAX_SEED_LINES_PER_FILE`] lines;
//!   the bundler splits into `<bucket>-partN.lino` files when needed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::translation::cache::{
    cache_location, escape_lino_string, split_body_into_chunks, CacheLocation,
    MAX_SEED_LINES_PER_FILE, MAX_SEED_RECORDS_PER_BUCKET, SEED_BODY_CHUNK_CHARS, SEED_CACHE_DIR,
};
use formal_ai::translation::{CachedHttpClient, CurlClient, TranslationPipeline};

fn main() {
    let pairs = curated_pairs();
    let cache_dir = std::env::var("FORMAL_AI_TRANSLATION_CACHE_DIR")
        .unwrap_or_else(|_| formal_ai::translation::cache::DEFAULT_CACHE_DIR.to_owned());
    println!("cache_dir = {cache_dir}");
    println!(
        "live = {} (set FORMAL_AI_LIVE_API=1 to hit the network)",
        std::env::var("FORMAL_AI_LIVE_API").unwrap_or_default(),
    );

    let http = CachedHttpClient::new(&cache_dir, CurlClient::default());
    let pipeline = TranslationPipeline::new(&http);

    let total = pairs.len();
    let mut gaps: Vec<String> = Vec::new();
    for (surface, source, target) in &pairs {
        match pipeline.translate(surface, source, target) {
            Ok(translation) => {
                let candidate = translation
                    .primary_surface()
                    .unwrap_or("<empty>")
                    .to_owned();
                println!(
                    "{source}→{target} \"{surface}\" -> \"{candidate}\" \
                     (meaning={}, provenance={:?})",
                    translation.meaning, translation.provenance,
                );
                if translation.candidates.is_empty() {
                    gaps.push(format!("{source}→{target} \"{surface}\""));
                }
            }
            Err(error) => {
                eprintln!("{source}→{target} \"{surface}\" -> ERROR: {error}");
                gaps.push(format!("{source}→{target} \"{surface}\""));
            }
        }
    }

    println!("\nBundling on-disk accelerator into seed bundle at {SEED_CACHE_DIR}...");
    match bundle_disk_cache_into_seed(&cache_dir) {
        Ok(report) => {
            for line in &report {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("seed-bundle failure: {error}");
            std::process::exit(2);
        }
    }

    if !gaps.is_empty() {
        eprintln!("\n{} translation gap(s):", gaps.len());
        for gap in &gaps {
            eprintln!("  - {gap}");
        }
        std::process::exit(1);
    }
    println!("\nAll {total} pairs cached successfully.");
}

/// Curated translation pairs. Sized to fit comfortably under
/// [`MAX_SEED_RECORDS_PER_BUCKET`] for every bucket so the resulting
/// `.lino` files stay reviewer-friendly.
///
/// The list is *test-driven*: every surface here is referenced by a
/// spec test (`tests/unit/specification/translation_via_links.rs`).
/// New tests should append the surfaces they exercise.
fn curated_pairs() -> Vec<(String, &'static str, &'static str)> {
    let mut pairs: Vec<(String, &str, &str)> = Vec::new();
    // Russian → English regressions (issues #210, #216, #217, #221).
    for surface in [
        "как у тебя дела",
        "как дела",
        "доброе яблоко",
        "спасибо",
        "привет",
        "да",
        "нет",
        "яблоко",
        "помидор",
        "огурец",
        "картофель",
        "морковь",
        "хлеб",
        "вода",
    ] {
        pairs.push((surface.to_owned(), "ru", "en"));
    }
    // English → Russian (issue #216 + #221 common-noun coverage).
    for surface in [
        "hello",
        "thank you",
        "apple",
        "tomato",
        "cucumber",
        "potato",
        "carrot",
        "bread",
        "water",
    ] {
        pairs.push((surface.to_owned(), "en", "ru"));
    }
    // English → Hindi / Chinese fan-out for the apple + hello cases.
    for (surface, target) in [
        ("hello", "hi"),
        ("hello", "zh"),
        ("apple", "hi"),
        ("apple", "zh"),
    ] {
        pairs.push((surface.to_owned(), "en", target));
    }
    pairs
}

/// Read every `.body` / `.url` pair under `cache_dir` and rebuild the
/// committed seed bundle from them.
///
/// Bucketing follows [`cache_location`]: Wikidata Q-ids and Lexeme L-ids
/// go to `wikidata-entities`, P-ids to `wikidata-properties`, search
/// queries to `wikidata-search`, SPARQL queries to `wikidata-sparql`,
/// and Wiktionary pages to `wiktionary-pages`. Records inside each
/// bucket are sorted by URL so the bundle is deterministic.
fn bundle_disk_cache_into_seed(cache_dir: &str) -> Result<Vec<String>, String> {
    let root = Path::new(cache_dir);
    if !root.is_dir() {
        return Err(format!(
            "cache dir {} does not exist; run the pipeline first",
            root.display()
        ));
    }

    let mut buckets: BTreeMap<&'static str, Vec<(String, String)>> = BTreeMap::new();
    for bucket in BUCKETS {
        buckets.insert(bucket.0, Vec::new());
    }

    for (body_path, url_path) in walk_cache_entries(root) {
        let url = fs::read_to_string(&url_path)
            .map_err(|e| format!("read url marker {}: {e}", url_path.display()))?;
        let body = fs::read_to_string(&body_path)
            .map_err(|e| format!("read body {}: {e}", body_path.display()))?;
        let location = cache_location(url.trim());
        let bucket_name = bucket_for_location(&location, url.trim());
        if let Some(entries) = buckets.get_mut(bucket_name) {
            entries.push((url.trim().to_owned(), body));
        }
    }

    let seed_root = Path::new(SEED_CACHE_DIR);
    fs::create_dir_all(seed_root).map_err(|e| format!("create {}: {e}", seed_root.display()))?;

    let mut report: Vec<String> = Vec::new();
    for (bucket_name, header_block) in BUCKETS {
        let entries_owned = buckets.remove(bucket_name).unwrap_or_default();
        let mut entries = entries_owned;
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries.dedup_by(|a, b| a.0 == b.0);
        let capped = entries.len().min(MAX_SEED_RECORDS_PER_BUCKET);
        let dropped = entries.len().saturating_sub(capped);
        entries.truncate(capped);

        let parts = write_bucket_parts(bucket_name, header_block, &entries, seed_root)?;
        report.push(format!(
            "  {bucket_name}: {} records, {parts} part(s){}",
            entries.len(),
            if dropped > 0 {
                format!(", {dropped} dropped (cap={MAX_SEED_RECORDS_PER_BUCKET})")
            } else {
                String::new()
            },
        ));
    }
    Ok(report)
}

/// Walk the cache root and yield `(body, url)` file pairs.
fn walk_cache_entries(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut out: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_owned()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip the seed directory itself — we are *writing* to it.
                if path == Path::new(SEED_CACHE_DIR) || path.ends_with("seed") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("body") {
                let url_path = path.with_extension("url");
                if url_path.exists() {
                    out.push((path, url_path));
                }
            }
        }
    }
    out
}

/// Map a cache location to a seed bucket name.
fn bucket_for_location(location: &CacheLocation, url: &str) -> &'static str {
    let directory = location.directory.to_string_lossy();
    if directory.contains("wiktionary-cache") {
        return "wiktionary-pages";
    }
    if directory.contains("wikidata-cache/sparql") {
        return "wikidata-sparql";
    }
    if directory.contains("wikidata-cache/search") {
        return "wikidata-search";
    }
    if directory.contains("wikidata-cache") {
        // Distinguish properties (P-ids) from entities (Q-ids, L-ids).
        if url_mentions_property(url) {
            return "wikidata-properties";
        }
        return "wikidata-entities";
    }
    // Unknown / misc — fold into entities so we don't lose the bytes.
    "wikidata-entities"
}

/// Heuristic: a URL refers to a Wikidata property when its `ids=` or
/// `titles=` query parameter starts with `P` followed by digits.
fn url_mentions_property(url: &str) -> bool {
    let Some(query_start) = url.find('?') else {
        return false;
    };
    for pair in url[query_start + 1..].split('&') {
        for prefix in ["ids=", "titles=", "search="] {
            if let Some(value) = pair.strip_prefix(prefix) {
                if looks_like_property_id(value) {
                    return true;
                }
            }
        }
    }
    false
}

fn looks_like_property_id(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('P'))
        && chars.clone().all(|c| c.is_ascii_digit())
        && chars.next().is_some()
}

/// Write the bucket as one or more `.lino` parts, each strictly under
/// [`MAX_SEED_LINES_PER_FILE`] lines.
///
/// If a single record's body would not fit in one part, its `body`
/// chunks are spread across multiple records that all share the same
/// URL. `cache::seed_index` concatenates same-URL chunks, so the split
/// is transparent to consumers.
fn write_bucket_parts(
    bucket_name: &str,
    header_block: &str,
    entries: &[(String, String)],
    seed_root: &Path,
) -> Result<usize, String> {
    // Drop any stale `<bucket>-partN.lino` files from earlier runs before
    // writing fresh parts. Keeps the seed directory canonical even when a
    // later refresh shrinks the bucket.
    remove_existing_parts(seed_root, bucket_name)?;

    let header_lines = header_block.lines().count();
    let mut parts_written = 0usize;
    let mut part_index = 0usize;
    let mut buffer = String::new();
    buffer.push_str(header_block);
    let mut lines_in_buffer = header_lines;
    let mut wrote_records_in_part = false;

    for (idx, (url, body)) in entries.iter().enumerate() {
        let chunks = split_body_into_chunks(body, SEED_BODY_CHUNK_CHARS);
        let mut remaining: &[String] = &chunks;
        let mut split_index = 0usize;
        loop {
            if remaining.is_empty() {
                break;
            }
            let separator = usize::from(wrote_records_in_part);
            let record_header = 2; // `response_<id>` + `  url "..."`
            let used = lines_in_buffer + separator + record_header;
            let body_budget = MAX_SEED_LINES_PER_FILE
                .saturating_sub(used)
                .saturating_sub(1); // keep one slack line, never hit the cap
            if body_budget == 0 {
                // Not enough room for even a single body line — flush and retry.
                flush_part(
                    seed_root,
                    bucket_name,
                    &mut part_index,
                    &mut buffer,
                    &mut parts_written,
                )?;
                buffer.push_str(header_block);
                lines_in_buffer = header_lines;
                wrote_records_in_part = false;
                continue;
            }

            if wrote_records_in_part {
                buffer.push('\n');
                lines_in_buffer += 1;
            }
            let short_id = if split_index == 0 {
                short_record_id(idx, url)
            } else {
                format!("{}_p{}", short_record_id(idx, url), split_index + 1)
            };
            buffer.push_str("response_");
            buffer.push_str(&short_id);
            buffer.push('\n');
            buffer.push_str("  url \"");
            buffer.push_str(&escape_lino_string(url));
            buffer.push_str("\"\n");
            lines_in_buffer += 2;

            let take = body_budget.min(remaining.len());
            for chunk in &remaining[..take] {
                buffer.push_str("  body \"");
                buffer.push_str(&escape_lino_string(chunk));
                buffer.push_str("\"\n");
                lines_in_buffer += 1;
            }
            remaining = &remaining[take..];
            split_index += 1;
            wrote_records_in_part = true;

            if !remaining.is_empty() {
                flush_part(
                    seed_root,
                    bucket_name,
                    &mut part_index,
                    &mut buffer,
                    &mut parts_written,
                )?;
                buffer.push_str(header_block);
                lines_in_buffer = header_lines;
                wrote_records_in_part = false;
            }
        }
    }

    // Flush the trailing part (or the empty header-only file for an
    // empty bucket).
    flush_part(
        seed_root,
        bucket_name,
        &mut part_index,
        &mut buffer,
        &mut parts_written,
    )?;
    Ok(parts_written)
}

fn flush_part(
    seed_root: &Path,
    bucket_name: &str,
    part_index: &mut usize,
    buffer: &mut String,
    parts_written: &mut usize,
) -> Result<(), String> {
    let path = part_path(seed_root, bucket_name, *part_index);
    fs::write(&path, buffer.as_str()).map_err(|e| format!("write {}: {e}", path.display()))?;
    *parts_written += 1;
    *part_index += 1;
    buffer.clear();
    Ok(())
}

/// Remove `<bucket>-partN.lino` files (any N ≥ 1) from a previous run.
/// The main `<bucket>.lino` file is overwritten in place by the next
/// flush, so it is left alone here.
fn remove_existing_parts(seed_root: &Path, bucket_name: &str) -> Result<(), String> {
    let Ok(entries) = fs::read_dir(seed_root) else {
        return Ok(());
    };
    let prefix = format!("{bucket_name}-part");
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let has_lino_extension = std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lino"));
        if name.starts_with(&prefix) && has_lino_extension {
            fs::remove_file(&path).map_err(|e| format!("remove stale {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn part_path(seed_root: &Path, bucket_name: &str, part_index: usize) -> PathBuf {
    if part_index == 0 {
        seed_root.join(format!("{bucket_name}.lino"))
    } else {
        seed_root.join(format!("{bucket_name}-part{part_index}.lino"))
    }
}

/// Build a short, filesystem-friendly identifier for the record. Uses
/// the entry index plus a slug of the URL so reviewers can grep records.
fn short_record_id(idx: usize, url: &str) -> String {
    let mut slug = String::with_capacity(24);
    let tail = url.rsplit_once('/').map_or(url, |(_, after)| after);
    for ch in tail.chars().take(24) {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' {
            slug.push(ch);
        } else {
            slug.push('_');
        }
    }
    if slug.is_empty() {
        slug.push_str("record");
    }
    format!("{idx:04}_{slug}")
}

/// Buckets in declaration order. The header block is the same valid
/// Links Notation `seed_metadata` record shipped in the placeholders so
/// the file remains parseable even when empty.
const BUCKETS: &[(&str, &str)] = &[
    (
        "wikidata-entities",
        "seed_metadata\n  bucket \"wikidata-entities\"\n  format \"Links Notation; one `response_<short_id>` record per fetched URL\"\n  populated_by \"examples/refresh_translation_cache.rs\"\n  max_records \"128\"\n  max_lines \"1500\"\n  split_marker \"wikidata-entities-partN.lino\"\n  body_format \"raw response text; concatenate `body` chunks; values escape one literal quote as two consecutive quotes per Links Notation\"\n",
    ),
    (
        "wikidata-properties",
        "seed_metadata\n  bucket \"wikidata-properties\"\n  format \"Links Notation; one `response_<short_id>` record per fetched URL\"\n  populated_by \"examples/refresh_translation_cache.rs\"\n  max_records \"128\"\n  max_lines \"1500\"\n  split_marker \"wikidata-properties-partN.lino\"\n  body_format \"raw response text; concatenate `body` chunks; values escape one literal quote as two consecutive quotes per Links Notation\"\n",
    ),
    (
        "wikidata-search",
        "seed_metadata\n  bucket \"wikidata-search\"\n  format \"Links Notation; one `response_<short_id>` record per fetched URL\"\n  populated_by \"examples/refresh_translation_cache.rs\"\n  max_records \"128\"\n  max_lines \"1500\"\n  split_marker \"wikidata-search-partN.lino\"\n  body_format \"raw response text; concatenate `body` chunks; values escape one literal quote as two consecutive quotes per Links Notation\"\n",
    ),
    (
        "wikidata-sparql",
        "seed_metadata\n  bucket \"wikidata-sparql\"\n  format \"Links Notation; one `response_<short_id>` record per fetched URL\"\n  populated_by \"examples/refresh_translation_cache.rs\"\n  max_records \"128\"\n  max_lines \"1500\"\n  split_marker \"wikidata-sparql-partN.lino\"\n  body_format \"raw response text; concatenate `body` chunks; values escape one literal quote as two consecutive quotes per Links Notation\"\n",
    ),
    (
        "wiktionary-pages",
        "seed_metadata\n  bucket \"wiktionary-pages\"\n  format \"Links Notation; one `response_<short_id>` record per fetched URL\"\n  populated_by \"examples/refresh_translation_cache.rs\"\n  max_records \"256\"\n  max_lines \"1500\"\n  split_marker \"wiktionary-pages-partN.lino\"\n  body_format \"raw response text; concatenate `body` chunks; values escape one literal quote as two consecutive quotes per Links Notation\"\n",
    ),
];
