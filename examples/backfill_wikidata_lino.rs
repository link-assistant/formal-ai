//! Backfill canonical `.lino` siblings for Wikidata `.json` cache records.
//!
//! The bulk lexeme importer (issue #660) and the curation helper
//! `experiments/gather_common_nouns.py` write trimmed `data/cache/wikidata/…`
//! `.json` snapshots. Every cache record must also carry its canonical `.lino`
//! sibling (enforced by `tests/unit/semantic_grounding.rs`). This tool walks a
//! cache directory and writes the missing `.lino` files with
//! [`json_cache_file`], so a freshly gathered batch becomes closure-complete.
//!
//! Usage: `cargo run --example backfill_wikidata_lino -- <cache_dir> [--force]`

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use formal_ai::json_lino::json_cache_file;
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    let force = args.iter().any(|arg| arg == "--force");
    let dir = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| String::from("data/cache/wikidata/entity"));

    let mut written = 0usize;
    let mut skipped = 0usize;
    walk(Path::new(&dir), force, &mut written, &mut skipped);
    eprintln!("backfilled {written} .lino sibling(s); {skipped} already present");
}

fn walk(dir: &Path, force: bool, written: &mut usize, skipped: &mut usize) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("cannot read {}: {error}", dir.display());
            process::exit(1);
        }
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, force, written, skipped);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let lino_path = path.with_extension("lino");
        if !force && lino_path.exists() {
            *skipped += 1;
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("cannot read {}: {error}", path.display());
                continue;
            }
        };
        let value: Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("invalid json {}: {error}", path.display());
                continue;
            }
        };
        if let Err(error) = fs::write(&lino_path, json_cache_file(stem, &value)) {
            eprintln!("cannot write {}: {error}", lino_path.display());
            continue;
        }
        *written += 1;
    }
}
