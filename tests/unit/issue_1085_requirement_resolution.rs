//! Issue #1085 (D2.2, D2.3, D4): a requirement that names behaviour resolves to
//! its file through the self-AST census, every ladder leaf is a committed
//! link-edit rule, and the ladder compiles, tests and merges what it verifies.

use std::fs;
use std::path::Path;

use formal_ai::agentic_coding::{
    LinkEditRule, apply_link_edit, parse_rule_document, resolve_requirement_target,
};

const LADDER: &str = "experiments/issue_1028_agent_cli_ladder";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn leaves() -> Vec<Vec<String>> {
    fs::read_to_string(root().join(LADDER).join("leaves.tsv"))
        .expect("the ladder leaf table is committed")
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect()
}

#[test]
fn a_requirement_without_a_file_name_resolves_through_the_census() {
    let target = resolve_requirement_target("Add wikiquote as a web search provider.")
        .expect("the census declares one web search provider list");
    assert_eq!(target.module_path, "src/web_search_core.rs");
    assert_eq!(target.symbol, "WEB_SEARCH_PROVIDERS");
    assert_eq!(target.kind, "const");

    let named = resolve_requirement_target("Rename the constant ERROR_JOIN to ERROR_LIST_JOIN.")
        .expect("an identifier named verbatim resolves to its declaring module");
    assert_eq!(named.module_path, "src/cli_context.rs");
    assert_eq!(named.symbol, "ERROR_JOIN");

    assert_eq!(
        resolve_requirement_target("Improve the code."),
        None,
        "a requirement that names nothing resolvable is not guessed"
    );
}

#[test]
fn every_ladder_leaf_requirement_resolves_to_its_leaf_file() {
    let rows = leaves();
    assert_eq!(rows.len(), 32);
    for row in &rows {
        let (leaf, path, requirement) = (&row[0], &row[2], &row[5]);
        let target = resolve_requirement_target(requirement)
            .unwrap_or_else(|| panic!("{leaf}: {requirement:?} must resolve"));
        assert_eq!(&target.module_path, path, "{leaf}: {requirement:?}");
    }
}

#[cfg(feature = "meta-language")]
#[test]
fn each_committed_leaf_rule_applies_to_the_current_source() {
    let rows = leaves();
    for row in &rows {
        let leaf = &row[0];
        let text = fs::read_to_string(root().join(LADDER).join(format!("rules/{leaf}.lino")))
            .unwrap_or_else(|error| panic!("{leaf}: rule file must exist: {error}"));
        let document = parse_rule_document(&text).unwrap_or_else(|error| panic!("{leaf}: {error}"));
        assert_eq!(&document.leaf, leaf);
        assert_eq!(
            &document.path, &row[2],
            "{leaf}: the rule targets the leaf file"
        );
        let source = fs::read_to_string(root().join(&document.path))
            .unwrap_or_else(|error| panic!("{leaf}: {}: {error}", document.path));
        let (edited, report) = apply_link_edit(&source, &document.language, &document.rule)
            .unwrap_or_else(|error| panic!("{leaf}: {error}"));
        assert_ne!(edited, source, "{leaf}: the rule must change the source");
        assert!(report.clean_after, "{leaf}: the edited network must verify");
        for expected in &document.expect {
            assert!(
                edited.contains(expected.as_str()),
                "{leaf}: edited source must contain {expected:?}"
            );
        }
        match (&document.rule, leaf.trim_start_matches('L').parse::<u32>()) {
            (LinkEditRule::InsertMember { .. }, Ok(n)) => assert!(n <= 11, "{leaf}"),
            (LinkEditRule::ReplaceLiteral { .. }, Ok(n)) => {
                assert!((12..=22).contains(&n), "{leaf}");
            }
            (LinkEditRule::RenameIdentifier { .. }, Ok(n)) => assert!(n >= 23, "{leaf}"),
            (_, Err(error)) => panic!("{leaf}: {error}"),
        }
    }
}

#[test]
fn the_ladder_compiles_tests_merges_and_verifies_requirement_levels() {
    let verifier =
        fs::read_to_string(root().join(LADDER).join("verify-node.sh")).expect("verifier");
    for needle in [
        "cargo check --lib",
        "cargo test --test unit",
        "unmergeable_child_diffs",
        "uncompilable_composite_change",
        "node_kind=requirement",
        "missing_requirement_change",
    ] {
        assert!(
            verifier.contains(needle),
            "verify-node.sh must contain {needle:?}"
        );
    }
    let runner = fs::read_to_string(root().join(LADDER).join("run.sh")).expect("runner");
    for needle in [
        "leaves.tsv",
        "ladder-result.lino",
        "deepest_passing_level",
        "leaf_nodes_passing",
        "change.diff",
    ] {
        assert!(runner.contains(needle), "run.sh must contain {needle:?}");
    }
    let workflow = fs::read_to_string(root().join(".github/workflows/issue-1028-agent-ladder.yml"))
        .expect("workflow");
    for needle in [
        "pull_request:",
        "schedule:",
        "data/meta/ladder-ratchet.lino",
    ] {
        assert!(
            workflow.contains(needle),
            "the ladder workflow must contain {needle:?}"
        );
    }
    let ratchet =
        fs::read_to_string(root().join("data/meta/ladder-ratchet.lino")).expect("ratchet");
    // What is ratcheted is how many of the 32 leaves Formal AI actually
    // changed. The first run under the compile-and-test criteria passed 15;
    // the record may only rise (issue #1085 D4).
    assert!(ratchet.contains("  leaf_nodes_selected 32\n"));
    let passing = ratchet
        .lines()
        .find_map(|line| line.trim().strip_prefix("leaf_nodes_passing "))
        .and_then(|value| value.parse::<u32>().ok())
        .expect("the ratchet records how many leaves pass");
    assert!(
        (15..=32).contains(&passing),
        "leaf_nodes_passing is {passing}, outside the 15 the first measured run passed and the 32 leaves there are"
    );
}
