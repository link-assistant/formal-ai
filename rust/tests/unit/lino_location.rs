//! Locating a Links Notation parse failure.
//!
//! Issue #1076, D18. `data/meta/ci-gates/check-job-headroom.lino` was added
//! with a `#` prose paragraph reading `a commit *can* break: two of the tests
//! parse the ...`. Links Notation had no comment syntax then, so that one bare
//! colon made the whole file unparseable, and the error named no line: it
//! printed the entire unconsumed tail.
//!
//! links-notation 0.23 retired both halves. A `#` that opens a line or follows
//! whitespace is a comment (`links_notation::comments`), and a syntax error is
//! a `SyntaxError` carrying its line, column, expectation and the offending
//! line. The line-by-line locator this file used to export is gone with them;
//! these tests pin the upstream behaviour it was standing in for.

use links_notation::{ParseError, parse_lino as parse_canonical_lino};

/// The gate file as it was written, colon and all.
const PROSE_GATE: &str = "\
# One CI gate, one file. Issue #991 moved the gate list out of
# `.github/workflows/release.yml`, the repository's third most conflicted path,
# because its step list was append-only.
#
# What this gate holds is the part a commit *can* break: two of the tests parse
# the repository's real `.github/workflows/**`.
ci_gate check_job_headroom
  stage rust
  description \"Every declared job cap can still be read.\"
  run \"rust-script --test scripts/check-job-headroom.rs\"
";

#[test]
fn a_bare_colon_in_a_comment_line_is_no_longer_a_trap() {
    parse_canonical_lino(PROSE_GATE.trim())
        .expect("a `#` line is a comment, so the colon inside it is prose");
}

#[test]
fn the_failure_is_reported_with_the_line_and_column_that_caused_it() {
    let error = parse_canonical_lino("ci_gate x\nstage: rust: nextest\n")
        .expect_err("a second colon on one line is a syntax error");
    let ParseError::SyntaxError(error) = error else {
        panic!("expected a located syntax error, got {error:?}");
    };
    assert_eq!((error.line, error.column), (2, 12));
    assert_eq!(error.found, Some(':'));
    assert_eq!(error.line_text, "stage: rust: nextest");
}

#[test]
fn a_hash_inside_a_word_or_a_quoted_reference_is_content() {
    // `issue#1047` is one reference, and a `#` inside a delimited reference is
    // text, so neither starts a comment.
    let quoted = "ci_gate x\n  stage rust\n  note issue#1047\n  run \"echo # kept\"\n";
    let parsed = parse_canonical_lino(quoted).expect("both hashes are content");
    let rendered = format!("{parsed}");
    assert!(rendered.contains("issue#1047"), "{rendered}");
    assert!(rendered.contains("echo # kept"), "{rendered}");
}

#[test]
fn every_checked_in_gate_file_parses() {
    // The registry is the concentration of hand-written prose in `data/`, and
    // `run-ci-gates.rs` reads gates with its own line parser, so a gate can
    // break canonical parsing without the gate runner noticing.
    let registry = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/meta/ci-gates");
    let mut checked = 0_usize;
    for entry in std::fs::read_dir(&registry).expect("the gate registry should exist") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("lino") {
            continue;
        }
        checked += 1;
        let content = std::fs::read_to_string(&path).expect("gate files are UTF-8");
        if let Err(error) = parse_canonical_lino(content.trim()) {
            panic!(
                "{} is not canonical Links Notation: {error}",
                path.display()
            );
        }
    }
    assert!(checked >= 10, "expected the gate registry to be populated");
}
