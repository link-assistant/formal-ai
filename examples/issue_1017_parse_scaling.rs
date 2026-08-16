//! Issue #1017: show that a whole-module CST/AST parse grows quadratically.
//!
//! `meta_language::LinkNetwork::parse` converts every tree-sitter node into a
//! link, and each conversion resolves its byte offsets to `(row, column)` with
//! `point_at_byte`, which rescans the source from byte 0. That is `O(bytes)` per
//! node, so a parse is `O(nodes x bytes)` — quadratic in file size.
//!
//! This binary parses the same synthetic Rust module at growing sizes and prints
//! the wall time plus the time-per-byte ratio. A linear parser holds the ratio
//! flat; a quadratic one doubles it every time the input doubles.
//!
//! Run with: `cargo run --example issue_1017_parse_scaling`
//! (a `dev` build on purpose — that is what `cargo nextest` runs in CI).

use std::time::Instant;

use formal_ai::agentic_coding::self_ast::ast_census;

/// One self-contained function, repeated to grow the module.
fn unit(index: usize) -> String {
    format!(
        "/// Doc comment for item {index}.\n\
         pub fn item_{index}(input: &str) -> usize {{\n\
         \x20   let trimmed = input.trim();\n\
         \x20   if trimmed.is_empty() {{\n\
         \x20       return {index};\n\
         \x20   }}\n\
         \x20   trimmed.len() + {index}\n\
         }}\n\n"
    )
}

fn main() {
    println!(
        "{:>6}  {:>9}  {:>10}  {:>14}",
        "units", "bytes", "parse ms", "ns per byte"
    );
    for units in [64_usize, 128, 256, 512, 1024] {
        let source: String = (0..units).map(unit).collect();
        let started = Instant::now();
        let census = ast_census(&source);
        let elapsed = started.elapsed();
        assert!(census.text_preserved, "the round-trip must stay faithful");
        let bytes = source.len();
        #[allow(clippy::cast_precision_loss)]
        let per_byte = elapsed.as_nanos() as f64 / bytes as f64;
        println!(
            "{units:>6}  {bytes:>9}  {:>10}  {per_byte:>14.0}",
            elapsed.as_millis()
        );
    }
}
