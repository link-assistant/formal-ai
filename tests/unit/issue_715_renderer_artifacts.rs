//! Every published record survives *both* readers of the same document.
//!
//! Issue #715 is about code reaching a Links Notation artifact. Code carries
//! quotes, and a quote is the one character the notation and the hand-rolled
//! C-style escapers disagreed about: the notation *doubles* a delimiter and has
//! no backslash escape at all, so `println!(\"hi\")` written the backslash way
//! is not Links Notation — the grammar rejects it.
//!
//! A renderer is only correct if the document it writes satisfies every reader
//! that will read it:
//!
//! - `links_notation::parse_lino` — the real grammar, the oracle here.
//! - `formal_ai::seed::parser` — the repository's own reader, reached through
//!   the public `from_links_notation` round trips below.
//!
//! Pinning one reader is what let the two drift apart in the first place, so
//! each case checks both.

use formal_ai::associative_package::AssociativePackage;
use formal_ai::associative_package::PackageHandler;
use formal_ai::substitution::SubstitutionRuleSet;
use formal_ai::translation::formalization::formalize_prompt;
use links_notation::parse_lino as parse_canonical_lino;

/// Values that reach a record through user text or source code. Each is a
/// plausible payload for issue #715's "change this code fragment" request.
const HOSTILE: [(&str, &str); 6] = [
    ("plain", "hello world"),
    ("apostrophe", "it doesn't matter"),
    ("double quote", "say \"hi\""),
    ("code", "println!(\"hi\");"),
    ("both quotes", "it's a \"test\""),
    ("link-cli query", "((\"Hello\")) ((terminal: \"Goodbye\"))"),
];

#[track_caller]
fn assert_canonical(label: &str, artifact: &str) {
    assert!(
        parse_canonical_lino(artifact).is_ok(),
        "the {label} artifact is not Links Notation; \
         the grammar rejected it:\n{artifact}"
    );
}

#[test]
fn substitution_rule_documents_carrying_quotes_satisfy_both_readers() {
    for (label, value) in HOSTILE {
        // A rule set is the sharpest case: `text_manipulation` writes this
        // document and *immediately* reads it back, so a renderer that only
        // pleased one reader broke the handler outright.
        let source = format!(
            "substitution_rules\n  id {id}\n  rule {rule}\n    order \"10\"\n    \
             event \"manual\"\n    replace {replace}\n      with {with}\n",
            id = quoted(value),
            rule = quoted(&format!("rule for {value}")),
            replace = quoted(&format!("stage:0 -> text:{value}")),
            with = quoted(&format!("stage:1 -> text:{value}")),
        );

        let set = SubstitutionRuleSet::from_links_notation(&source)
            .unwrap_or_else(|error| panic!("{label}: reading a quoted value failed: {error:?}"));

        let artifact = set.links_notation();
        assert_canonical(&format!("substitution rule set ({label})"), &artifact);

        let reparsed = SubstitutionRuleSet::from_links_notation(&artifact)
            .unwrap_or_else(|error| panic!("{label}: re-reading our own output failed: {error:?}"));
        assert_eq!(
            set, reparsed,
            "{label}: a rule set changed meaning when written and read back"
        );
    }
}

#[test]
fn associative_package_documents_carrying_quotes_satisfy_both_readers() {
    for (label, value) in HOSTILE {
        let mut package = AssociativePackage::new("pkg_hostile", value, "1.0.0");
        package
            .handlers
            .push(PackageHandler::new("handler_hostile", "answer", "respond").with_response(value));

        let artifact = package.links_notation();
        assert_canonical(&format!("associative package ({label})"), &artifact);

        let reparsed = AssociativePackage::from_links_notation(&artifact).unwrap_or_else(|error| {
            panic!("{label}: re-reading our own package failed: {error:?}")
        });
        assert_eq!(
            reparsed.name, value,
            "{label}: a package name changed when written and read back"
        );
        assert_eq!(
            reparsed.handlers[0].response, value,
            "{label}: a handler response changed when written and read back"
        );
    }
}

#[test]
fn formalization_candidate_artifacts_are_canonical_links_notation() {
    for (label, value) in HOSTILE {
        let candidate = formalize_prompt(value, "en");
        assert_canonical(
            &format!("formalization candidate ({label})"),
            &candidate.to_links_notation(),
        );
    }
}

/// The notation quotes a value by doubling the delimiter inside it, so the
/// value's own quote is preserved. This is the shape the corpus already
/// carries — `data/cache/wikidata/property/P138.lino` writes `the subject''s
/// name` — which the reader silently failed to decode until it learned the
/// rule, handing callers back the raw line with the quotes still in it.
#[test]
fn the_reader_collapses_a_doubled_delimiter() {
    let source = "substitution_rules\n  id 'the subject''s name'\n";
    let set = SubstitutionRuleSet::from_links_notation(source)
        .expect("a doubled delimiter is how the notation escapes a quote");
    assert_eq!(
        set.id, "the subject's name",
        "a doubled delimiter must collapse to one quote, not survive as two"
    );
}

fn quoted(value: &str) -> String {
    // Quote the way the notation does, so the fixture itself is valid input:
    // pick a delimiter the value does not carry, else double the delimiter.
    if value.contains('"') && !value.contains('\'') {
        format!("'{value}'")
    } else if value.contains('"') {
        format!("'{}'", value.replace('\'', "''"))
    } else {
        format!("\"{value}\"")
    }
}
