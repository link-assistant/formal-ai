//! How does Links Notation carry a value containing a newline? (#715)
//!
//! Issue #715 is about code reaching a Links Notation artifact, and the
//! recurring defect is a value that one reader accepts and another does not.
//! Quotes are the known case. This probe asks the same question of *newlines*,
//! which reach a record whenever an observation quotes a multi-line message.
//!
//! Three candidate encodings, and the two readers that must both accept them:
//!
//! * the real grammar, `links_notation::parse_lino`;
//! * the repository's own `src/seed/parser.rs`, which is line-based
//!   (`for line in text.lines()`), and so is modelled here by the one question
//!   that decides it — does the value close on the line it opened on?
//!
//! Run it with:
//!
//! ```sh
//! rust-script experiments/issue_715_multiline_value_probe.rs
//! ```
//!
//! ```cargo
//! [dependencies]
//! links-notation = "0.13.0"
//! lino-objects-codec = "0.2.1"
//! ```

use links_notation::LiNo;

/// The value under test: an observation carrying a newline *and* a quote,
/// which is what a real "change this code" message looks like.
const VALUE: &str = "say \"hi\"\nthen stop";

fn main() {
    println!("value under test: {VALUE:?}\n");

    // 1. What `associative_learning.rs::field` emits: backslash-escape `\` and
    //    `"`, pass the newline through raw.
    let hand_rolled = format!(
        "root\n  text \"{}\"\n",
        VALUE.replace('\\', "\\\\").replace('"', "\\\"")
    );
    probe("hand-rolled field() (backslash quotes, raw newline)", &hand_rolled);

    // 2. What `links_format::format_lino_value` emits: sanitize the newline to
    //    a literal `\` + `n`, then hand to the codec's own escaper.
    let sanitized = VALUE
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t");
    probe("links_format (newline sanitized to an escape)", &render(&sanitized));

    // 3. The codec's escaper with the newline passed through untouched.
    probe("codec escaper (raw newline)", &render(VALUE));
}

/// Render one field the way `links_format::format_lino_value` does.
fn render(value: &str) -> String {
    let record =
        lino_objects_codec::format::format_indented_ordered("root", &[("text", value)], "")
            .expect("a one-field record is formattable");
    format!("{record}\n")
}

fn probe(label: &str, document: &str) {
    println!("=== {label}");
    println!("--- document:   {document:?}");

    // Reader 1: the real grammar. Does it parse, and does the value survive?
    match links_notation::parse_lino(document) {
        Ok(tree) => match first_value(&tree) {
            Some(got) if got == VALUE => println!("--- grammar:    parses; value ROUND-TRIPS"),
            Some(got) => println!("--- grammar:    parses; VALUE CHANGED -> {got:?}"),
            None => println!("--- grammar:    parses; but no `text <value>` field found"),
        },
        Err(error) => println!("--- grammar:    REJECTS -> {error:?}"),
    }

    // Reader 2: the seed parser is line-based. A value is reachable only if it
    // opens and closes on one line.
    let field_line = document.lines().nth(1).unwrap_or_default();
    let quotes = field_line.matches('"').count();
    println!(
        "--- line-based: {} ({quotes} quote(s) on {field_line:?})",
        if quotes >= 2 && quotes % 2 == 0 {
            "value closes on its line"
        } else {
            "value DOES NOT CLOSE on its line"
        }
    );
    println!();
}

/// Pull the first `text <value>` pair out of whatever shape the tree took.
fn first_value(tree: &LiNo<String>) -> Option<String> {
    fn walk(node: &LiNo<String>, out: &mut Option<String>) {
        let LiNo::Link { values, .. } = node else {
            return;
        };
        let refs: Vec<&str> = values
            .iter()
            .filter_map(|v| match v {
                LiNo::Ref(r) => Some(r.as_str()),
                LiNo::Link { .. } => None,
            })
            .collect();
        if out.is_none() && refs.len() == 2 && refs[0] == "text" {
            *out = Some(refs[1].to_string());
        }
        for value in values {
            walk(value, out);
        }
    }
    let mut out = None;
    walk(tree, &mut out);
    out
}
