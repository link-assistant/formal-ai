//! Dump the tree-sitter kind inventory of the committed corpora (issue
//! #1138, plan 16 L8), with occurrence counts.
//!
//! ```bash
//! cargo run --example dump_grammar_kind_inventory
//! ```
//!
//! The inventory is the rule checklist the grammar-projection seed must
//! cover — every kind a committed module parses into is either ruled or
//! refused-by-name in `data/seed/grammar-projection-rules.lino`, and this
//! dump is how that checklist is measured rather than remembered. The
//! owned rust corpus is the census set (the same derived set the round-trip
//! proof walks); the ES corpora are the committed `js/` and `ts/` trees.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::grammar_kinds::corpus_inventory;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    for corpus in formal_ai::grammar_kinds::CORPORA {
        let sources = corpus_sources(root, corpus);
        let inventory = corpus_inventory(corpus.label, &sources);
        println!(
            "== {label}: {files} files, {kinds} syntax kinds",
            label = corpus.label,
            files = sources.len(),
            kinds = inventory.len(),
        );
        for (kind, count) in &inventory {
            println!("{kind}\t{count}");
        }
    }
}

fn corpus_sources(root: &Path, corpus: formal_ai::grammar_kinds::Corpus) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    if corpus.census_derived {
        collect(
            &root.join("data").join("meta").join("self-ast"),
            "lino",
            &mut sources,
        );
        sources = sources
            .into_iter()
            .map(|census_path| {
                let relative = census_path
                    .strip_prefix(root.join("data").join("meta").join("self-ast"))
                    .expect("collected under the census tree")
                    .with_extension("rs");
                root.join("rust").join(relative)
            })
            .collect();
    } else {
        collect(&root.join(corpus.directory), corpus.extension, &mut sources);
    }
    sources
}

fn collect(directory: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, extension, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    out.sort();
}
