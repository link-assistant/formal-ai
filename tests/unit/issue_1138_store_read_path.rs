//! Issue #1138 B9, plan 09 leaf 39: the doublets store is written but never
//! read. Stage 2 extracts condition evaluation behind a `ConditionSource` with
//! two backends — today's `SeedTables` and the `LinkStore` — and a parity
//! fixture replays every rule and every promotion through both.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::event_log::EventLog;
use formal_ai::handler_promotion::{promotion_condition_verdicts, promotions};
use formal_ai::rule_interpreter::{HandlerRules, LinkStoreSource, SeedTables};
use formal_ai::seed::parse_lexicon_text;
use formal_ai::seed_links::{SeedLinkNetwork, network};

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

fn fixture_prompts(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("prompt "))
        .map(|value| value.trim_matches('"').replace("\"\"", "\""))
        .collect()
}

#[test]
fn condition_sources_agree_on_every_rule_and_promotion() {
    let fixture = parity_fixture();
    let prompts = fixture_prompts(&fixture);
    assert!(
        prompts.len() >= 20,
        "the parity fixture must exercise varied shapes"
    );

    let rules = formal_ai::rule_interpreter::rules();
    let promotion_rows = promotions();
    let seed_tables = SeedTables::new(formal_ai::seed::lexicon());
    let link_store = LinkStoreSource::from_store(network());
    let log = EventLog::default();
    let mut matched = 0usize;
    let mut rejected = 0usize;

    for prompt in &prompts {
        let normalized = formal_ai::web_engine_core::normalize_prompt(prompt);
        let normalized = formal_ai::seed::operation_vocabulary().canonicalized_prompt(&normalized);
        let from_seed = rules.condition_verdicts(&seed_tables, prompt, &normalized, &log);
        let from_store = rules.condition_verdicts(&link_store, prompt, &normalized, &log);
        assert_eq!(
            from_seed, from_store,
            "{prompt:?}: the seed-table backend and the link-store backend disagree; \
             the store may not become the default until every rule and promotion agrees"
        );
        assert_eq!(from_seed.len(), rules.rule_count());
        matched += from_seed.iter().filter(|verdict| verdict.matched).count();
        rejected += from_seed.iter().filter(|verdict| !verdict.matched).count();

        let seed_promotions = promotion_condition_verdicts(&promotion_rows, prompt, &seed_tables);
        let store_promotions = promotion_condition_verdicts(&promotion_rows, prompt, &link_store);
        assert_eq!(
            seed_promotions, store_promotions,
            "promotion parity for {prompt:?}"
        );
        assert_eq!(seed_promotions.len(), promotion_rows.len());
        matched += seed_promotions
            .iter()
            .filter(|(_, verdict)| *verdict)
            .count();
        rejected += seed_promotions
            .iter()
            .filter(|(_, verdict)| !*verdict)
            .count();
    }

    assert!(
        matched > 0,
        "the fixture must contain positive condition evidence"
    );
    assert!(rejected > 0, "the fixture must retain negative controls");
}

#[test]
fn the_link_backend_reads_an_injected_store_instead_of_seed_tables() {
    let store = SeedLinkNetwork::from_documents(&[
        (
            "meanings-fixture.lino",
            "meanings\n  store_probe\n    role store_probe\n    lexeme en\n      surface\n        text storeword\n",
        ),
        (
            "data/seed/intent-routing.lino",
            "intent_routing\n  intent fixture\n    slug store_route\n    phrase \"storeword storeroute\"\n",
        ),
    ]);
    let source = LinkStoreSource::from_store(&store);
    let empty = parse_lexicon_text("meanings\n");
    let seed_tables = SeedTables::new(&empty);
    let rules = HandlerRules::parse(
        "handler_rules\n  handler probe\n    rule probe\n      when\n        role store_probe raw\n        route_exact store_route\n      respond_unknown\n",
    )
    .expect("fixture rule parses");
    let handler = rules.handler("probe").expect("fixture handler");
    let prompt = "storeword storeroute";

    assert!(handler.matches_with_source(&source, prompt, prompt));
    assert!(!handler.matches_with_source(&seed_tables, prompt, prompt));
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
