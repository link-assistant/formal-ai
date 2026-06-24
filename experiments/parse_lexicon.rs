//! Reproduction: which constructs does `links_notation::parse_lino` (the
//! canonical Links Notation parser used by the `data_files` guard test) accept?
//!
//! Issue #468: the committed `data/agentic-coding/fisherman-lexicon.lino` failed
//! `tests/unit/data_files.rs::lino_data_files_are_parseable_human_readable_and_bounded`
//! with `code: Eof`. This bisect pins the root cause: the canonical parser
//! rejects **blank lines between top-level entries** ("two entries blank-line"
//! FAILs, "two entries no-blank" passes). No file under `data/` may contain a
//! blank line. Our own `Lexicon::load` already skips blank lines, so dropping
//! them is parse-equivalent for us while satisfying the canonical guard.
//!
//! Run it (the crate dependency `links-notation` is on the lib target, so copy
//! this into `examples/` to build it against the workspace):
//!
//! ```sh
//! cp experiments/parse_lexicon.rs examples/_tmp_parse_lexicon.rs
//! cargo run --example _tmp_parse_lexicon
//! rm examples/_tmp_parse_lexicon.rs
//! ```

use links_notation::parse_lino;

fn try_parse(label: &str, src: &str) {
    match parse_lino(src) {
        Ok(_) => println!("OK    {label}"),
        Err(e) => println!("FAIL  {label}: {e}"),
    }
}

fn main() {
    try_parse("bare doublet", "lexeme old\n  work tale\n  kind entity");
    try_parse(
        "quoted value",
        "lexeme \"старик\"\n  work \"tale:fisherman-and-fish\"\n  kind \"entity\"",
    );
    try_parse("quoted w/ colon only", "work \"tale:fisherman-and-fish\"");
    try_parse(
        "two entries blank-line",
        "lexeme \"a\"\n  kind \"entity\"\n\nlexeme \"b\"\n  kind \"entity\"",
    );
    try_parse(
        "two entries no-blank",
        "lexeme \"a\"\n  kind \"entity\"\nlexeme \"b\"\n  kind \"entity\"",
    );
    try_parse(
        "colon inside quote child",
        "lexeme \"a\"\n  modal \"commitment:0.95\"",
    );
    try_parse(
        "space inside quote",
        "context \"ctx:final\"\n  label \"У синего моря\"",
    );
    let full = std::fs::read_to_string("data/agentic-coding/fisherman-lexicon.lino").unwrap();
    try_parse("FULL FILE", full.trim());
}
