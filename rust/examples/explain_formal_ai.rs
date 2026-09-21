//! Explain how Formal AI itself works, grounded in its own source, data, and tests
//! (issue #558, `R558-08`).
//!
//! Prints the canonical grounded explanation — an ordered set of topics, each citing
//! the *real* artifacts it rests on (source files content-addressed through the owned
//! manifest, the generated data artifacts they emit, and the tests that lock the
//! behaviour). Every source citation is verified against the owned manifest at
//! construction, so the explanation cannot cite anything the repository does not ship.
//!
//! Like the whole-repository source-links projection, the document depends on the
//! whole source tree (source citation content ids and the manifest id change with
//! every edit), so it is a *workspace-only* artifact — asserted live in the tests,
//! never committed byte-for-byte. Usage: `cargo run --example explain_formal_ai`. The
//! one-line summary prints to stderr; the full grounded explanation (Links Notation)
//! prints to stdout.

use formal_ai::canonical_explanation;

fn main() {
    let explanation = canonical_explanation();

    eprintln!("{}", explanation.summary());

    println!("{}", explanation.links_notation());
}
