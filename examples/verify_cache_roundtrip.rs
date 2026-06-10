//! Verify the lossless JSON ↔ Links Notation cache round-trip (issue #398,
//! defect #1).
//!
//! For every `data/cache/**/*.json` snapshot this rebuilds the JSON from the
//! sibling `.lino` and asserts it equals the empties-stripped raw JSON.

use std::fs;
use std::path::PathBuf;
use std::process;

use formal_ai::json_lino::{json_cache_projection, lino_to_json};
use serde_json::Value;
use walkdir::WalkDir;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cache = root.join("data/cache");
    let mut failures = Vec::new();
    let mut checked = 0;

    for entry in WalkDir::new(&cache).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string();
        let lino_path = path.with_extension("lino");
        let raw: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let lino = fs::read_to_string(&lino_path).unwrap();
        let rebuilt = match lino_to_json(&lino) {
            Ok(value) => value,
            Err(error) => {
                failures.push(format!("{id}: decode error: {error}"));
                continue;
            }
        };
        let expected = json_cache_projection(&id, &raw);
        checked += 1;
        if rebuilt != expected {
            failures.push(format!("{id}: rebuilt JSON != normalized raw JSON"));
        }
    }

    println!("checked {checked} cache files");
    if failures.is_empty() {
        println!("all round-trips lossless");
    } else {
        for failure in &failures {
            eprintln!("FAIL {failure}");
        }
        process::exit(1);
    }
}
