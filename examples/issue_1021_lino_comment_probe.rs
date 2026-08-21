//! Issue #1021: probe how canonical Links Notation treats `#`-prefixed lines.
//!
//! `data/seed/*.lino` files open with `#`-prefixed prose that reads like a
//! comment, but `links-notation` 0.13.0 has no comment syntax: those lines parse
//! as ordinary links whose first reference happens to be `#`. That works until
//! the prose contains a character the notation reserves. A bare `:` is the one
//! that bit issue #1021: `parser.rs` accepts a colon only directly after a
//! line's leading reference, where it separates a link id from its members, so
//! `a: b` parses but `# issue #1021 pins: an action …` fails with a `code: Eof`
//! syntax error. Inside backticks or quotes the same colon is accepted, which is
//! why one seed comment carrying `renderer: {subject}` never tripped this.
//!
//! Run: `cargo run --example issue_1021_lino_comment_probe`, or pass documents
//! of your own as arguments to check them.

use links_notation::parse_lino;

/// Documents probed when no argument is given: the cases that map the boundary.
const DEFAULT_CASES: &[&str] = &[
    "# a plain prose line",
    "# a colon: breaks it",
    "# `a colon: inside backticks` is fine",
    "# \"a colon: inside quotes\" is fine",
    "# see https://example.com/x — a URL carries a colon too",
    "# an em dash — and a double hyphen -- are fine",
    "a: b",
    "a:",
];

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let cases: Vec<String> = if arguments.is_empty() {
        DEFAULT_CASES
            .iter()
            .map(|case| (*case).to_string())
            .collect()
    } else {
        arguments
    };

    for case in &cases {
        match parse_lino(case.trim()) {
            Ok(_) => println!("parses  {case:?}"),
            Err(error) => println!("rejects {case:?} -> {error:?}"),
        }
    }
}
