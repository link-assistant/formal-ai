//! Which prose shapes does canonical Links Notation accept inside a seed value? (#674)
//!
//! Moving the procedure-compiler prose into `data/seed/multilingual-responses.lino`
//! (R379) made `tests/unit/data_files.rs` reject the file: `links_notation::parse_lino`
//! stops at a `:` that follows an escaped quote inside the same quoted value, so
//!
//!     text "I cannot compile the step \"{step}\": {gap}."
//!
//! failed with `Syntax error: … code: Eof` while the same row with an em dash parsed.
//! Run this probe (as a `formal-ai` example target or by pasting into one) to re-check
//! the boundary before wording a new seed row:
//!
//!     cargo run --example <copy-of-this-file>
fn main() {
    let cases = [
        (
            "escaped-quote-then-colon",
            "root\n  response r\n    text \"step \\\"{step}\\\": gap.\"",
        ),
        (
            "escaped-quote-then-em-dash",
            "root\n  response r\n    text \"step \\\"{step}\\\" — gap.\"",
        ),
        (
            "escaped-quote-no-colon",
            "root\n  response r\n    text \"say \\\"hello\\\" now.\"",
        ),
        (
            "colon-without-escaped-quote",
            "root\n  response r\n    text \"step {step}: gap.\"",
        ),
        (
            "guillemets-and-colon",
            "root\n  response r\n    text \"шаг «{step}»: {gap}.\"",
        ),
        (
            "fenced-links-block",
            "root\n  response r\n    text \"a\\n\\n```links\\n{program}```\\n\\nb\"",
        ),
    ];
    for (name, text) in cases {
        match links_notation::parse_lino(text) {
            Ok(_) => println!("{name}: ok"),
            Err(error) => println!("{name}: ERROR {error}"),
        }
    }
}
