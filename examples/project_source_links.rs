//! Project the *entire* owned source tree to the links / meta language and back,
//! verifying the byte-for-byte round-trip for every file (issue #558).
//!
//! This is the exhaustive whole-repository projection — the "recompile itself"
//! round-trip proven over all owned modules, not just the representative slice the
//! agentic recipe verifies inline. Parsing every file through the CST/AST engine is
//! deliberately expensive (seconds per file in debug), so it lives here and in an
//! ignored-by-default test rather than on the hot path.
//!
//! Usage: `cargo run --example project_source_links`. The one-line summary and any
//! unfaithful modules print to stderr; the full projection (Links Notation) prints
//! to stdout, so it can be redirected to a file for review.

use formal_ai::SourceLinks;

fn main() {
    let graph = SourceLinks::owned();

    eprintln!("{}", graph.summary());
    if graph.is_fully_faithful() {
        eprintln!(
            "All {} owned source modules round-trip losslessly (source -> links -> source).",
            graph.module_count()
        );
    } else {
        eprintln!("Modules that did NOT round-trip:");
        for module in graph.unfaithful_modules() {
            eprintln!("  {}", module.path);
        }
    }

    println!("{}", graph.links_notation());
}
