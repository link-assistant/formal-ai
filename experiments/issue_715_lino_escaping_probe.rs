//! Why the published mutation trace was not Links Notation (#715).
//!
//! Standalone probe kept for re-use: it compares the escaping the codebase used
//! to hand-roll against the codec's own escaper, over the values a code change
//! actually carries. Run it with:
//!
//! ```sh
//! rust-script experiments/issue_715_lino_escaping_probe.rs
//! ```
//!
//! ```cargo
//! [dependencies]
//! lino-objects-codec = "0.2.1"
//! ```
//!
//! Findings, reproduced by this probe:
//!
//! * Links Notation escapes a quote by *doubling* it and picks a delimiter the
//!   value does not contain. A backslash escape leaves the quote visible to the
//!   reader, so `"println!(\"hi\");"` ends the string at `(\"`, and the now-bare
//!   `(` opens a group that never closes — a hard `Eof` parse error.
//! * `Ok` from `parse_indented` is not proof of a round trip: a mis-escaped
//!   field can parse cleanly and be silently dropped. Always compare the
//!   recovered value, never just the status.
//! * The codec's `escape_reference` round-trips every value tried here,
//!   including multi-line code, backslashes and both quote kinds — which is why
//!   `code_artifact.rs` delegates to it instead of hand-rolling a variant.

use lino_objects_codec::format::{escape_reference, parse_indented};

/// The escaping `code_artifact.rs` used before the fix.
fn hand_rolled_backslash(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn round_trip(value: &str, escaped: &str) -> &'static str {
    let doc = format!("normal_markov_program\n  pattern {escaped}\n");
    match parse_indented(&doc) {
        Ok((_root, fields)) => match fields.get("pattern") {
            Some(got) if got == value => "survives",
            Some(_) => "MANGLED",
            None => "DROPPED",
        },
        Err(_) => "PARSE ERROR",
    }
}

fn compare(label: &str, value: &str) {
    let hand = round_trip(value, &hand_rolled_backslash(value));
    let codec = round_trip(value, &escape_reference(value));
    println!("{label:<24} hand-rolled: {hand:<12} codec: {codec}");
}

fn main() {
    for (label, value) in [
        ("plain word", "main.rs"),
        ("spaces", "Hello, world!"),
        ("apostrophe", "it doesn't matter"),
        ("double quote", "say \"hi\""),
        ("code w/ parens+quotes", "println!(\"Hello, world!\");"),
        ("both quote kinds", "it's a \"test\""),
        ("newline", "line1\nline2"),
        ("multiline code", "fn main() {\n    println!(\"hi\");\n}"),
        ("backslash", "C:\\path"),
        ("colon", "stage:0"),
        ("a substitution query", "((\"Hello\")) ((terminal: \"Goodbye\"))"),
    ] {
        compare(label, value);
    }
}
