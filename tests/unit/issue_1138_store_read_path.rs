//! Issue #1138 B9, plan 09 leaf 39: the doublets store is written but never
//! read. Stage 2 extracts condition evaluation behind a `ConditionSource` with
//! two backends — today's `SeedTables` and the `LinkStore` — and a parity
//! fixture replays every rule and every promotion through both.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::handler_promotion::promotions;
use formal_ai::rule_interpreter;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn parity_fixture() -> String {
    let path = repo_root().join("data/parity/condition-source.lino");
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "plan 09 leaf 39 owes {}: the stage-2 parity fixture that replays every rule and \
             promotion through both condition sources ({error})",
            path.display()
        )
    })
}

#[test]
fn condition_sources_agree_on_every_rule_and_promotion() {
    let fixture = parity_fixture();
    let cases: Vec<&str> = fixture
        .lines()
        .filter_map(|line| line.trim().strip_prefix("case "))
        .collect();

    let rules = rule_interpreter::rules();
    let rule_names: Vec<&str> = rules.handler_names().collect();
    let promotion_rows = promotions();
    assert_eq!(
        cases.len(),
        rule_names.len() + promotion_rows.len(),
        "the parity fixture must replay every rule and every promotion, not a sample"
    );

    for case in &cases {
        let (name, verdicts) = case
            .split_once(' ')
            .unwrap_or_else(|| panic!("parity case `{case}` names no verdict pair"));
        let (seed_tables, link_store) = verdicts
            .trim()
            .split_once(' ')
            .unwrap_or_else(|| panic!("parity case `{name}` needs both backends' verdicts"));
        assert_eq!(
            seed_tables, link_store,
            "`{name}`: the seed-table backend and the link-store backend disagree; \
             the store may not become the default until every rule and promotion agrees"
        );
    }
}

#[test]
fn the_store_read_share_is_a_declared_upward_ratchet() {
    // `store_read_share` is the one measure in plan 09 whose strict direction is
    // upward, and the ledger states that in its own `how` field so the rule is
    // never ambiguous (plan 00 section 6.7, section 9 X6).
    let ledger = fs::read_to_string(repo_root().join("data/meta/debt-ratchet.lino"))
        .expect("debt ratchet readable");
    assert!(
        ledger.contains("measure store_read_share"),
        "plan 09 leaf 4 adds `store_read_share` to data/meta/debt-ratchet.lino"
    );
    let block = ledger
        .split("measure store_read_share")
        .nth(1)
        .unwrap_or_default();
    let head: String = block.lines().take(4).collect::<Vec<_>>().join("\n");
    assert!(
        head.contains("up"),
        "the `store_read_share` block must declare its upward direction in its own \
         fields, since every other measure here ratchets down: {head}"
    );
}
