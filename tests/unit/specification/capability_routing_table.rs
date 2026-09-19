//! Issue #1138 B10, plan 10 leaf 8: the decision table is total over the
//! declared axes.
//!
//! Capability is a function of `(object, act, locus)`. A triple with no row
//! must be an *explicit* `ask`, not a fall-through — that is what makes the
//! silent UNKNOWN unreachable by construction rather than aspirational.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::capability_routing::{
    Act, Locus, ObjectType, RoutingOutcome, route, route_with, routing_table_from,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn table_text() -> String {
    let path = repo_root().join("data/seed/capability-routing.lino");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("capability-routing.lino readable: {error}"))
}

const OBJECTS: [ObjectType; 14] = [
    ObjectType::Url,
    ObjectType::Path,
    ObjectType::Pattern,
    ObjectType::PathSet,
    ObjectType::PathScope,
    ObjectType::QuotedContent,
    ObjectType::TimeExpression,
    ObjectType::LanguageName,
    ObjectType::QuantityQuestion,
    ObjectType::TaskList,
    ObjectType::Delegation,
    ObjectType::BareTerm,
    ObjectType::SelfSurface,
    ObjectType::None,
];

const ACTS: [Act; 9] = [
    Act::Retrieve,
    Act::Enumerate,
    Act::Transform,
    Act::Compose,
    Act::Schedule,
    Act::Explain,
    Act::Demonstrate,
    Act::Record,
    Act::Unresolved,
];

const LOCI: [Locus; 5] = [
    Locus::Workspace,
    Locus::Web,
    Locus::Dialogue,
    Locus::SelfSurface,
    Locus::Unresolved,
];

const ADVERTISED: &[&str] = &[
    "web_fetch",
    "web_search",
    "read_file",
    "write_file",
    "list_dir",
    "grep",
    "shell",
    "calendar_create_event",
    "response_language_demonstration",
    "concept_measurement_lookup",
    "report_issue",
    "ask_user",
    "glob",
    "read_many",
    "multi_edit",
    "todo",
    "subagent",
];

#[test]
fn table_is_total_over_the_declared_axes() {
    let table = routing_table_from(&table_text()).expect("the decision table parses");
    let mut uncovered: Vec<String> = Vec::new();
    for object in OBJECTS {
        for act in ACTS {
            for locus in LOCI {
                let outcome = route_with(&table, object, act, locus, ADVERTISED);
                let total = matches!(
                    outcome,
                    RoutingOutcome::Routed { .. }
                        | RoutingOutcome::Lowered { .. }
                        | RoutingOutcome::HonestGap { .. }
                        | RoutingOutcome::Ask { .. }
                );
                if !total {
                    uncovered.push(format!("{object:?} x {act:?} x {locus:?}"));
                }
            }
        }
    }
    assert!(
        uncovered.is_empty(),
        "{} triples fall through the table; every one must resolve to a row or to the \
         explicit `default ask`: {:?}",
        uncovered.len(),
        uncovered.iter().take(12).collect::<Vec<_>>()
    );
}

#[test]
fn the_table_declares_an_explicit_default_rather_than_falling_through() {
    let text = table_text();
    assert!(
        text.contains("\n  default ask\n"),
        "data/seed/capability-routing.lino must end in `default ask`: a triple with no \
         row asks rather than guesses"
    );
}

#[test]
fn no_two_rows_claim_the_same_triple() {
    let table = routing_table_from(&table_text()).expect("the decision table parses");
    let mut seen: BTreeSet<(ObjectType, Act, Locus)> = BTreeSet::new();
    for row in &table {
        assert!(
            seen.insert((row.object, row.act, row.locus)),
            "two rows claim ({:?}, {:?}, {:?}); a tie must be resolved in the data, not \
             by declaration order",
            row.object,
            row.act,
            row.locus
        );
    }
    assert!(
        table.len() >= 9,
        "the shipped table declares at least the nine rows plan 10 names, got {}",
        table.len()
    );
}

#[test]
fn a_fallback_is_named_in_data_not_in_a_rust_cascade() {
    // #758's specialized-first, bash-as-universal-fallback policy is a `fallback`
    // field, not the cascade at `capability_router.rs:278-281`.
    let table = routing_table_from(&table_text()).expect("the decision table parses");
    let with_fallback: Vec<&str> = table
        .iter()
        .filter(|row| row.fallback.is_some())
        .map(|row| row.capability.as_str())
        .collect();
    assert!(
        with_fallback.contains(&"grep"),
        "the `(bare_term, retrieve, workspace)` row must name its fallback, so code \
         navigation lowers to the shell instead of reaching web_search: {with_fallback:?}"
    );

    // With the preferred capability withheld the outcome is a *declared*
    // lowering that logs both names.
    let outcome = route_with(
        &table,
        ObjectType::BareTerm,
        Act::Retrieve,
        Locus::Workspace,
        &["shell", "web_search"],
    );
    match outcome {
        RoutingOutcome::Lowered {
            preferred,
            capability,
        } => {
            assert_eq!(preferred, "grep");
            assert_eq!(capability, "shell");
        }
        other => panic!("an unadvertised preferred capability must lower, got {other:?}"),
    }
}

/// Issue #1138 B10, benchmark `en_news_07` (and its zh variation): "Summarise
/// the last few hours for me, with links." evidences a relative period (a
/// `calendar_hour` seed meaning) and no narrower act, so its triple is
/// (time_expression, retrieve, dialogue). That row must route to the fresh-web
/// digest capability -- in every language the corpus names -- instead of
/// letting the request fall through to the unknown opener.
#[test]
fn a_recent_period_summary_routes_to_the_fresh_web_digest() {
    for prompt in [
        "Summarise the last few hours for me, with links.",
        "把最近几个小时的情况总结一下，附上链接。",
    ] {
        let outcome = route(prompt, &["web_search"]);
        assert_eq!(
            outcome,
            RoutingOutcome::Routed {
                capability: String::from("web_search"),
            },
            "a recent-period digest request must route to web_search, got {prompt:?} -> {outcome:?}"
        );
    }
}
