//! Show that `meta_language::LinkNetwork::parse` is quadratic in input size.
//!
//! `convert_node` resolves every node's byte offsets to `(row, column)` through
//! `point_at_byte`, which rescans the source from byte 0. That is `O(bytes)` per
//! node, so a whole parse is `O(nodes x bytes)`.
//!
//! Run with `cargo run --release` and watch the last column: a linear parser
//! holds nanoseconds-per-byte flat, a quadratic one doubles it every time the
//! input doubles.

use std::time::Instant;

use meta_language::{LinkNetwork, ParseConfiguration};

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
    println!("{:>6}  {:>9}  {:>10}  {:>14}", "units", "bytes", "parse ms", "ns per byte");
    for units in [64_usize, 128, 256, 512, 1024] {
        let source: String = (0..units).map(unit).collect();
        let started = Instant::now();
        let network = LinkNetwork::parse(&source, "rust", ParseConfiguration::default());
        let elapsed = started.elapsed();
        assert_eq!(network.reconstruct_text(), source, "the round-trip must stay faithful");
        let bytes = source.len();
        let per_byte = elapsed.as_nanos() as f64 / bytes as f64;
        println!("{units:>6}  {bytes:>9}  {:>10}  {per_byte:>14.0}", elapsed.as_millis());
    }
}
