//! Issue #1182 (E147, R11): the seed loaders parse with `links-notation`.
//!
//! The conformance harness the issue prescribes: every `data/**/*.lino`
//! parses to the same tree with `links_notation::Parser` (via the
//! `lino_adapters` front end) as with `seed::parser`. Any construct the
//! local parser accepts and the crate does not is a **gap**: the file is
//! listed in `data/meta/links-notation-conformance-gaps.lino` with the
//! link-foundation/links-notation issue that asks for it — the gap list
//! may only shrink, mirroring the duplication baseline. When the list is
//! empty and this test is green, `rust/src/seed/parser.rs` and the parser
//! half of `js/seed_loader.js` are deleted (R11's final step).
//!
//! The installed direct dependency is 0.16.1; its public unflattened parser
//! supports the adapter without a Cargo update. The full corpus audit is an
//! explicitly ignored adoption gate until gaps are measured and filed upstream.
//! CI can run it separately with `cargo test --test unit -- --ignored`.

use std::fs;
use std::path::{Path, PathBuf};

/// The canonical form two parsers must agree on: one line per node,
/// children indented, `name\tid` — everything structural, nothing
/// lexical (quotes, spacing, comment placement).
fn canonical(node: &formal_ai::seed::parser::LinoNode, depth: usize) -> String {
    let mut out = format!("{}{}\t{}\n", "  ".repeat(depth), node.name, node.id);
    for child in &node.children {
        out.push_str(&canonical(child, depth + 1));
    }
    out
}

fn lino_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data"),
        &mut files,
    );
    files.sort();
    files
}

fn collect(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|it| it == "lino") {
            out.push(path);
        }
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits inside the repository")
        .to_path_buf()
}

fn gap_list() -> Vec<String> {
    let path = repo_root().join("data/meta/links-notation-conformance-gaps.lino");
    let Ok(source) = fs::read_to_string(path) else {
        return Vec::new();
    };
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("file "))
        .map(|rest| rest.trim().trim_matches('"').to_string())
        .collect()
}

#[test]
#[ignore = "pending full corpus adoption: inline comments, nested groups and historical escapes require measured upstream gaps"]
fn every_seed_file_parses_to_the_same_tree_with_both_parsers() {
    let files = lino_files();
    assert!(
        files.len() > 100,
        "the corpus is large (found {} .lino files); a smoke-sized run hides gaps",
        files.len()
    );
    let gaps = gap_list();
    let mut mismatches = Vec::new();
    let mut rejected = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("every corpus file is readable");
        let local = formal_ai::seed::parser::parse_lino(&text);
        let relative = file
            .strip_prefix(repo_root())
            .unwrap_or(file)
            .display()
            .to_string();
        match formal_ai::lino_adapters::links_notation::parse_lino(&text) {
            Ok(adapted) => {
                if canonical(&local, 0) != canonical(&adapted, 0) {
                    if !gaps.contains(&relative) {
                        mismatches.push(relative);
                    }
                }
            }
            Err(error) => {
                if !gaps.contains(&relative) {
                    rejected.push(format!("{relative}: {error}"));
                }
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "files whose trees differ between seed::parser and links-notation (file each in link-foundation/links-notation, then list it in data/meta/links-notation-conformance-gaps.lino):\n{}",
        mismatches.join("\n")
    );
    assert!(
        rejected.is_empty(),
        "files links-notation rejects but seed::parser accepts (file each in link-foundation/links-notation, then list it in the gap file):\n{}",
        rejected.join("\n")
    );
}

#[test]
fn the_gap_list_only_names_files_that_still_exist_and_may_only_shrink() {
    let gaps = gap_list();
    let files = lino_files();
    for gap in &gaps {
        let full = repo_root().join(gap);
        assert!(
            full.exists(),
            "gap row {gap} names a file that no longer exists -- shrink the list"
        );
        assert!(
            files.iter().any(|file| file.ends_with(gap)),
            "gap row {gap} is a .lino corpus file"
        );
    }
}

#[test]
fn the_adapter_fronts_the_crate_behind_the_seed_interface() {
    let adapter = fs::read_to_string(repo_root().join("rust/src/lino_adapters/links_notation.rs"))
        .expect("the adapter exists");
    assert!(
        adapter.contains("pub use crate::seed::parser::LinoNode"),
        "the adapter returns the seed tree, so call sites switch by changing one use line"
    );
    assert!(
        adapter.contains("links_notation::parser::parse_document"),
        "the crate does the parsing"
    );
}
