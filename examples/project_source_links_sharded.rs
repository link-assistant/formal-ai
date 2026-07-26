//! Write the exhaustive whole-repository source ↔ links projection as Links
//! Notation, sharded so each file stays under the 1500-line `.lino` ceiling
//! (`scripts/check-file-size.rs`) — the same way `scripts/close-total.py` shards
//! its generated closure into `closure-generated-NN.lino` "capped well under the
//! 1500-line data-file limit".
//!
//! Each shard is an independently-valid Links Notation tree carrying a disjoint,
//! path-sorted slice of the owned modules; concatenated they are the full
//! `SourceLinks::owned()` projection — issue #558's *"translate the entire source
//! code of our system to links / meta language and back"* at full fidelity, every
//! module proven to round-trip byte-for-byte. Like [`project_source_links`], the
//! parse is deliberately expensive, so this is a tool, not a hot path.
//!
//! Usage: `cargo run --example project_source_links_sharded -- <out-dir>`
//! (default out-dir: `docs/case-studies/issue-819/self-hosting-evidence`). The
//! per-shard summary prints to stderr; the shard files are written in place.

use std::path::PathBuf;

use formal_ai::{owned_source_files, SourceLinks};

/// Modules per shard. At roughly seven Links Notation lines per module plus a small
/// per-shard header, this keeps every shard comfortably under the 1500-line `.lino`
/// file-size limit; the loop asserts it rather than trusting the estimate.
const MODULES_PER_SHARD: usize = 150;

/// The hard `.lino` line ceiling `scripts/check-file-size.rs` enforces.
const LINO_LINE_LIMIT: usize = 1_500;

fn main() {
    let out_dir = std::env::args().nth(1).map_or_else(
        || PathBuf::from("docs/case-studies/issue-819/self-hosting-evidence"),
        PathBuf::from,
    );

    let files = owned_source_files();
    let shard_count = files.len().div_ceil(MODULES_PER_SHARD);
    let mut total_lines = 0usize;

    for (index, chunk) in files.chunks(MODULES_PER_SHARD).enumerate() {
        let graph = SourceLinks::compile(chunk);
        assert!(
            graph.is_fully_faithful(),
            "shard {index} has modules that did not round-trip: {:?}",
            graph
                .unfaithful_modules()
                .iter()
                .map(|module| &module.path)
                .collect::<Vec<_>>()
        );
        let notation = format!("{}\n", graph.links_notation().trim_end());
        let lines = notation.lines().count();
        assert!(
            lines < LINO_LINE_LIMIT,
            "shard {index} is {lines} lines, over the {LINO_LINE_LIMIT}-line .lino limit; \
             lower MODULES_PER_SHARD"
        );
        total_lines += lines;

        let path = out_dir.join(format!("whole-repository-projection-{:02}.lino", index + 1));
        std::fs::write(&path, &notation)
            .unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
        eprintln!(
            "shard {}/{}: {} modules, {} lines -> {}",
            index + 1,
            shard_count,
            graph.module_count(),
            lines,
            path.display()
        );
    }

    eprintln!(
        "wrote {shard_count} shards, {total_lines} total projection lines, {} owned modules",
        files.len()
    );
}
