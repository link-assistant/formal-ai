//! Formalize the canonical issue-#468 source text into a Links Notation
//! knowledge base and print it, together with the primitive coverage.
//!
//! Run with:
//!
//! ```text
//! cargo run --example issue_468_formalize_text
//! ```
//!
//! This demonstrates the *meta-language* formalization in isolation (no agentic
//! loop): text in, Links Notation out, with all nine protocol primitives realised
//! as links.

use formal_ai::agentic_coding::{
    coverage_line, formalize_text_to_links, CANONICAL_FISHERMAN_SYNOPSIS,
};

fn main() {
    let formalized = formalize_text_to_links(CANONICAL_FISHERMAN_SYNOPSIS, "");

    println!("# Source text\n{CANONICAL_FISHERMAN_SYNOPSIS}\n");
    println!(
        "# Knowledge base (Links Notation)\n{}",
        formalized.links_notation
    );
    println!("# Primitive coverage");
    println!("covered: {}", coverage_line(&formalized.summary));
    println!("covers all nine: {}", formalized.summary.covers_all_nine());
    println!("total records: {}", formalized.summary.total_records());
}
