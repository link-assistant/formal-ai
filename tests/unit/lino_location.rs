//! Locating a Links Notation parse failure.
//!
//! Issue #1076, D18. `data/meta/ci-gates/check-job-headroom.lino` was added
//! with a `#` prose paragraph reading `a commit *can* break: two of the tests
//! parse the ...`. That one bare colon made the whole file unparseable, and
//! `Test (ubuntu-latest / full)` failed with
//!
//! ```text
//! .../check-job-headroom.lino contains invalid canonical Links Notation:
//! Syntax error: Error(Error { input: "# holds is the part a commit *can* break: two of the tests parse the\n# repository's ...
//! ```
//!
//! The message names the file but no line, and prints the entire unconsumed
//! tail, so on a 1500-line data file it is a wall of text. The cause is that
//! Links Notation has no comment syntax at all: a `#` line is an ordinary
//! link, `:` is the notation's own delimiter, and `# a: b` is therefore a
//! parse error while `# a b` and `` # `a:b` `` are not. Every `.lino` file in
//! `data/` carries prose in that shape, so this is a trap the next writer
//! walks into as easily as this pull request did.

use links_notation::parse_lino as parse_canonical_lino;

/// Best-effort location for a Links Notation parse failure: the first line
/// that cannot stand on its own. Returns `None` when every line parses in
/// isolation, which is what a multi-line quoted string looks like -- callers
/// print the underlying error either way, so a miss costs nothing.
pub fn first_unparseable_lino_line(content: &str) -> Option<(usize, String)> {
    content
        .lines()
        .enumerate()
        .find(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && parse_canonical_lino(trimmed).is_err()
        })
        .map(|(index, line)| (index + 1, line.trim().to_string()))
}

/// The gate file as it was written, colon and all. Reproduces the CI failure.
const BROKEN_GATE: &str = "\
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
fn a_bare_colon_in_a_prose_line_makes_the_file_unparseable() {
    // The reproduction, held so the diagnostic below has something to explain.
    assert!(
        parse_canonical_lino(BROKEN_GATE.trim()).is_err(),
        "a `#` line with a bare colon should still be a Links Notation error; \
         if this passes the notation grew comments and D18 can be retired"
    );
}

#[test]
fn the_failure_is_reported_with_the_line_that_caused_it() {
    let (line, text) =
        first_unparseable_lino_line(BROKEN_GATE).expect("the colon line should be named");
    assert_eq!(line, 5, "the colon is on line 5 of the fixture");
    assert!(
        text.starts_with("# What this gate holds"),
        "the reported text should be the offending line, got `{text}`"
    );
}

#[test]
fn a_file_that_parses_names_no_line() {
    let fixed = BROKEN_GATE.replace("*can* break:", "*can* break --");
    parse_canonical_lino(fixed.trim()).expect("replacing the colon should make the file parse");
    assert_eq!(first_unparseable_lino_line(&fixed), None);
}

#[test]
fn a_colon_inside_backticks_is_not_the_trap() {
    // `check-test-partition-balance.lino` carries `slice:N/D` in prose and
    // parses, which is why the trap looks arbitrary until you see the rule:
    // the notation reads a backtick span as one reference, delimiter and all.
    let quoted =
        "# Issue #1047 measured `cargo nextest --partition slice:N/D`.\nci_gate x\n  stage rust\n";
    parse_canonical_lino(quoted.trim()).expect("a colon inside backticks is one reference");
    assert_eq!(first_unparseable_lino_line(quoted), None);
}

#[test]
fn every_checked_in_gate_file_still_parses_line_by_line() {
    // The registry is the concentration of hand-written prose in `data/`, and
    // `run-ci-gates.rs` reads gates with its own line parser, so a gate can
    // break canonical parsing without the gate runner noticing.
    let registry = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/meta/ci-gates");
    let mut checked = 0_usize;
    for entry in std::fs::read_dir(&registry).expect("the gate registry should exist") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("lino") {
            continue;
        }
        checked += 1;
        let content = std::fs::read_to_string(&path).expect("gate files are UTF-8");
        assert_eq!(
            first_unparseable_lino_line(&content),
            None,
            "{} has a line that does not parse on its own",
            path.display()
        );
    }
    assert!(checked >= 10, "expected the gate registry to be populated");
}
