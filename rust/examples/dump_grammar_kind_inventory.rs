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
//! dump is how that checklist is measured rather than remembered.

use std::path::Path;

use formal_ai::grammar_kinds::{corpus_inventory, corpus_sources};

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
