//! Which dialect survives *both* readers of the same document?
//!
//! `solver_handlers::text_manipulation` is not a write-only publisher, which is
//! what makes it the sharpest case in issue #715. It writes a rules document and
//! then immediately reads it back:
//!
//! ```text
//! let rules_text = build_rules_links_notation(input, &steps);
//! let rules = SubstitutionRuleSet::from_links_notation(&rules_text).ok()?;
//! ```
//!
//! and the same document is *also* published as `response.links_notation` and
//! validated as Links Notation by `tests/unit/specification/substitution_rules`.
//! So one document has two readers that disagree about escaping:
//!
//! - `src/seed/parser.rs` — the repository's own reader, a C-style backslash
//!   dialect, which is what the hand-rolled escapers were written against.
//! - the real grammar (`links-notation`) — which has no backslash escape at all
//!   and doubles a quote instead.
//!
//! A value only reaches this document through user text, so quotes are ordinary.
//! This prints, per dialect, whether each reader accepts *and* returns the value.
//!
//! Run with:
//!
//! ```sh
//! rust-script experiments/issue_715_round_trip_dialects.rs
//! ```
//!
//! ```cargo
//! [dependencies]
//! lino-objects-codec = "0.2.1"
//! links-notation = "0.13.0"
//! ```

use lino_objects_codec::format::format_indented_ordered;

fn sanitize(value: &str) -> String {
    value
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// The notation's own quoting, borrowed from the codec's public encoder.
fn codec_dialect(value: &str) -> String {
    let sanitized = sanitize(value);
    let record = format_indented_ordered("v", &[("v", sanitized.as_str())], "").unwrap();
    record
        .strip_prefix("v\nv ")
        .expect("codec writes `id\\n{indent}{key} {value}`")
        .to_owned()
}

/// The escape the renderers hand-rolled.
fn backslash_dialect(value: &str) -> String {
    format!(
        "\"{}\"",
        sanitize(value).replace('\\', "\\\\").replace('"', "\\\"")
    )
}

/// Does the real grammar accept this line inside a rules document?
fn grammar_accepts(line: &str) -> bool {
    links_notation::parse_lino(&format!("substitution_rules\n  replace {line}\n")).is_ok()
}

/// `src/seed/parser.rs`'s reader, reproduced: a quoted scalar is decoded only if
/// the closing quote is the last thing on the line, and `\\` / `\"` / `\n` are
/// the escapes. Anything else falls back to the raw text, quotes and all.
fn seed_reads_back(line: &str) -> Option<String> {
    fn find_closing(rest: &str, quote: u8) -> Option<usize> {
        let bytes = rest.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == quote {
                return Some(i);
            }
            i += 1;
        }
        None
    }
    fn unescape_double(raw: &str) -> String {
        let mut out = String::new();
        let mut iter = raw.chars();
        while let Some(c) = iter.next() {
            if c == '\\' {
                match iter.next() {
                    Some('n') => out.push('\n'),
                    Some('"') => out.push('"'),
                    Some('\\') | None => out.push('\\'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    for (open, quote) in [('"', b'"'), ('\'', b'\'')] {
        if let Some(rest) = line.strip_prefix(open) {
            if let Some(close) = find_closing(rest, quote) {
                if rest[close + 1..].trim().is_empty() {
                    // The single-quote branch has no `""`/`''` doubling: it
                    // unescapes backslashes and `\x27` only.
                    return Some(unescape_double(&rest[..close]));
                }
            }
        }
    }
    None
}

fn verdict(line: &str, value: &str) -> String {
    let sanitized = sanitize(value);
    let grammar = if grammar_accepts(line) { "ok" } else { "REJECTS" };
    let seed = match seed_reads_back(line) {
        Some(read) if read == sanitized => "ok",
        Some(_) => "CORRUPTS",
        None => "REJECTS",
    };
    format!("grammar {grammar:<8} seed {seed}")
}

fn main() {
    // Values as they actually reach the document: user text inside a pattern.
    let cases = [
        ("plain", "stage:0 -> text:hello"),
        ("apostrophe", "stage:0 -> text:it doesn't matter"),
        ("double quote", "stage:0 -> text:say \"hi\""),
        ("code", "stage:0 -> text:println!(\"hi\");"),
        ("both quotes", "stage:0 -> text:it's a \"test\""),
        ("backslash", "stage:0 -> text:C:\\tmp"),
    ];

    println!(
        "{:<14} | {:<34} | {:<34}",
        "case", "backslash (what renderers wrote)", "codec (the notation itself)"
    );
    println!("{}", "-".repeat(90));
    for (label, value) in cases {
        println!(
            "{label:<14} | {:<34} | {:<34}",
            verdict(&backslash_dialect(value), value),
            verdict(&codec_dialect(value), value)
        );
    }
}
