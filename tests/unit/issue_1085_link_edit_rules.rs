//! Issue #1085 (D2.1): a source edit is a substitution over the file's links
//! network, not a byte offset.
//!
//! The three shapes below are the three the Agent CLI ladder's 32 leaves use.
//! Each test parses Rust into the meta-language links network, applies one
//! rule, and asserts the rendered source, that the rewritten file still
//! round-trips through the network, and that the network verified clean.

use formal_ai::agentic_coding::self_ast::round_trips;
use formal_ai::agentic_coding::{LinkEditError, LinkEditRule, apply_link_edit, rule_shapes};

const SOURCE: &str = "pub const WEB_SEARCH_PROVIDERS: &[&str] = &[\n    \"wikipedia\",\n    \"wikidata\",\n];\n\nconst QUERY_PLACEHOLDER: &str = \"{query}\";\nconst QUERY_PLACEHOLDER_2: &str = \"second {query}\";\n\nfn describe() -> String {\n    format!(\"search {}\", QUERY_PLACEHOLDER)\n}\n";

#[test]
fn the_ladder_leaf_shapes_are_declared_as_data() {
    let names = rule_shapes()
        .into_iter()
        .map(|shape| shape.rule)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        ["insert_member", "replace_literal", "rename_identifier"]
    );
}

#[test]
fn a_member_is_inserted_in_the_lists_own_style_through_the_links_network() {
    let rule = LinkEditRule::InsertMember {
        list: "WEB_SEARCH_PROVIDERS".to_owned(),
        member: "wikiquote".to_owned(),
    };
    let (text, report) = apply_link_edit(SOURCE, "rust", &rule).expect("the list is reachable");
    assert!(
        text.contains("    \"wikidata\",\n    \"wikiquote\",\n];"),
        "the new member repeats the list's separator: {text}"
    );
    assert_eq!(report.edits, 1);
    assert!(report.round_trip_before);
    assert!(report.clean_after, "{report:?}");
    assert!(round_trips(&text));

    let again =
        apply_link_edit(&text, "rust", &rule).expect_err("a present member is not inserted twice");
    assert!(
        matches!(again, LinkEditError::MemberPresent { .. }),
        "{again}"
    );
}

#[test]
fn a_literal_is_replaced_inside_string_literals_only() {
    let rule = LinkEditRule::ReplaceLiteral {
        old: "{query}".to_owned(),
        new: "{search_query}".to_owned(),
    };
    let (text, report) = apply_link_edit(SOURCE, "rust", &rule).expect("the literal occurs");
    assert!(
        text.contains("const QUERY_PLACEHOLDER: &str = \"{search_query}\";"),
        "{text}"
    );
    assert!(text.contains("\"second {search_query}\""), "{text}");
    assert!(
        text.contains("QUERY_PLACEHOLDER)"),
        "identifiers are not literals: {text}"
    );
    assert_eq!(report.edits, 2);
    assert!(report.clean_after, "{report:?}");
    assert!(round_trips(&text));
}

#[test]
fn an_identifier_is_renamed_as_a_whole_token() {
    let rule = LinkEditRule::RenameIdentifier {
        old: "QUERY_PLACEHOLDER".to_owned(),
        new: "TRENDS_QUERY_PLACEHOLDER".to_owned(),
    };
    let (text, report) = apply_link_edit(SOURCE, "rust", &rule).expect("the identifier occurs");
    assert!(
        text.contains("const TRENDS_QUERY_PLACEHOLDER: &str = \"{query}\";"),
        "{text}"
    );
    assert!(
        text.contains("format!(\"search {}\", TRENDS_QUERY_PLACEHOLDER)"),
        "{text}"
    );
    assert!(
        text.contains("const QUERY_PLACEHOLDER_2:"),
        "a longer identifier that contains the old name is a different token: {text}"
    );
    assert_eq!(report.edits, 2, "{report:?}");
    assert!(report.clean_after, "{report:?}");
    assert!(round_trips(&text));
}

#[test]
fn an_absent_needle_is_a_named_error_not_a_silent_no_op() {
    let rule = LinkEditRule::RenameIdentifier {
        old: "NOT_IN_THE_FILE".to_owned(),
        new: "STILL_NOT".to_owned(),
    };
    let error = apply_link_edit(SOURCE, "rust", &rule).expect_err("nothing to rename");
    assert!(
        matches!(error, LinkEditError::NoMatchingLink { .. }),
        "{error}"
    );
    assert_eq!(
        error.to_string(),
        "link_edit:no_matching_link:rename_identifier:NOT_IN_THE_FILE"
    );
}
