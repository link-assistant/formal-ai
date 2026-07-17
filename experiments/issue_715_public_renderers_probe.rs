//! Do the other public "Links Notation" renderers actually emit Links Notation?
//!
//! `src/agentic_coding/code_artifact.rs` published a trace that claimed to be
//! Links Notation and was not, because it hand-rolled a C-style backslash escape
//! instead of doubling the quote. Nine more copies of that escape survive
//! elsewhere. This probe asks the same question of the ones whose output is a
//! published artifact, using the codec the notation is defined by.
//!
//! Run it with:
//!
//! ```sh
//! rust-script experiments/issue_715_public_renderers_probe.rs
//! ```
//!
//! A `Value` that carries a quote is not exotic: `IntentFormalization`'s
//! `source_text` is the user's own prompt.
//!
//! ```cargo
//! [dependencies]
//! lino-objects-codec = "0.2.1"
//! ```

use lino_objects_codec::format::{escape_reference, parse_indented};

/// The escape `intent_formalization.rs` and friends hand-roll.
fn hand_rolled(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

/// Render one `name value` line the way the renderers do, then ask the codec to
/// read the document back.
fn verdict(escaped: &str, value: &str) -> String {
    let document = format!("record\n  source_text {escaped}\n");
    match parse_indented(&document) {
        Err(error) => format!("PARSE ERROR ({error:?})"),
        Ok((_, fields)) => match fields.get("source_text") {
            Some(read) if read == value => String::from("survives"),
            Some(read) => format!("CORRUPTED (read back {read:?})"),
            None => String::from("DROPPED (field vanished)"),
        },
    }
}

fn main() {
    let cases = [
        ("plain prompt", "remember the kettle"),
        ("prompt with a quote", "he said \"hi\""),
        ("prompt with an apostrophe", "it doesn't matter"),
        ("prompt quoting code", "run println!(\"hi\");"),
        ("prompt with both quotes", "it's a \"test\""),
        ("prompt with a backslash", "path C:\\tmp"),
    ];

    println!("{:<28} {:<38} {}", "value", "hand-rolled backslash", "codec");
    for (label, value) in cases {
        println!(
            "{label:<28} {:<38} {}",
            verdict(&hand_rolled(value), value),
            verdict(&escape_reference(value), value)
        );
    }
}
