//! Issue #1487: the native proposition structure, checked statement relation,
//! statement-merge signature and unread coding-requirement structure agree
//! with the browser (JavaScript) reader on the shared fixture
//! `rust/tests/fixtures/proposition-structure-parity.json`.
//!
//! The fixture is written from the JavaScript modules by
//! `node experiments/issue-1487/parity-fixture.mjs --write`; the JavaScript
//! tests `rust/tests/web/issue-1487-proposition-identity.test.mjs` and
//! `rust/tests/web/issue-1487-coding-acceptance.test.mjs` check the same file,
//! so a drift on either side fails one of the suites.

use formal_ai::formalization::proposition_structure::structure_key;
use formal_ai::formalization::statement_relation::{certifies_restatement, checked_relation};
use formal_ai::formalization::text_statements::{Statement, formalize_text, statement_structure};
use formal_ai::summarization::StatementSignature;
use formal_ai::unread_structure::unread_structure;
use serde_json::Value;

const FIXTURE: &str = include_str!("../fixtures/proposition-structure-parity.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("parity fixture is JSON")
}

fn field<'a>(entry: &'a Value, name: &str) -> &'a str {
    entry[name]
        .as_str()
        .unwrap_or_else(|| panic!("fixture entry {entry} lacks string field {name:?}"))
}

fn entries<'a>(fixture: &'a Value, name: &str) -> &'a [Value] {
    fixture[name]
        .as_array()
        .unwrap_or_else(|| panic!("fixture lacks array {name:?}"))
}

fn one(text: &str, language: &str) -> Statement {
    let mut statements = formalize_text(text, language);
    assert_eq!(statements.len(), 1, "one statement expected for {text:?}");
    statements.remove(0)
}

#[test]
fn proposition_structure_keys_match_the_javascript_reader() {
    let fixture = fixture();
    let structures = entries(&fixture, "structures");
    assert!(!structures.is_empty());
    for entry in structures {
        let text = field(entry, "text");
        let statement = one(text, field(entry, "language"));
        assert_eq!(
            structure_key(&statement_structure(&statement)),
            field(entry, "key"),
            "structure of {text:?}"
        );
    }
}

#[test]
fn checked_relations_match_the_javascript_reader() {
    let fixture = fixture();
    let relations = entries(&fixture, "relations");
    assert!(!relations.is_empty());
    for entry in relations {
        let later = field(entry, "later");
        let earlier = field(entry, "earlier");
        let language = field(entry, "language");
        let relation = checked_relation(&one(later, language), &one(earlier, language));
        let label = format!("{later:?} against {earlier:?}");
        assert_eq!(relation.verdict.slug(), field(entry, "verdict"), "{label}");
        assert_eq!(relation.reason.slug(), field(entry, "reason"), "{label}");
        assert_eq!(
            certifies_restatement(relation),
            entry["restates"].as_bool().expect("restates is a boolean"),
            "{label}"
        );
    }
}

#[test]
fn statement_merge_signatures_match_the_javascript_reader() {
    let fixture = fixture();
    let signatures = entries(&fixture, "signatures");
    assert!(!signatures.is_empty());
    for entry in signatures {
        let text = field(entry, "text");
        assert_eq!(
            StatementSignature::of(text).key(),
            field(entry, "key"),
            "signature of {text:?}"
        );
    }
}

#[test]
fn ordered_and_connective_pairs_keep_distinct_signatures() {
    for (left, right) in [
        ("1 is less than 2.", "2 is less than 1."),
        ("Compute a - b.", "Compute b - a."),
        ("1 < 2", "1 <= 2"),
        (
            "The program returns true or false.",
            "The program returns true and false.",
        ),
        ("Not all tests pass.", "All tests do not pass."),
        ("Print hello twice.", "Print hello."),
    ] {
        assert_ne!(
            StatementSignature::of(left).key(),
            StatementSignature::of(right).key(),
            "{left:?} and {right:?} must not merge"
        );
    }
    let asserted = StatementSignature::of("The parser is fast.");
    let denied = StatementSignature::of("The parser is not fast.");
    assert!(denied.denies(&asserted), "a plain denial still contradicts");
}

#[test]
fn unread_coding_structure_matches_the_javascript_reader() {
    let fixture = fixture();
    let unread = entries(&fixture, "unread");
    assert!(!unread.is_empty());
    for entry in unread {
        let text = field(entry, "text");
        let ids: Vec<&str> = entry["ids"]
            .as_array()
            .unwrap_or_else(|| panic!("fixture entry {entry} lacks array ids"))
            .iter()
            .map(|id| id.as_str().expect("meaning ids are strings"))
            .collect();
        assert_eq!(
            unread_structure(text, &ids),
            field(entry, "unread"),
            "unread structure of {text:?} under {ids:?}"
        );
    }
}
