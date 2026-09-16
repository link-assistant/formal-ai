//! Issue #1138 B9, plan 09 leaves 9-11: which handlers a prompt hoists ahead of
//! `data/seed/handler-precedence.lino` is *data*, not nineteen hard-coded
//! predicates in `src/intent_formalization/prompt_relevants.rs`.
//!
//! Written before the leaves that make it pass (plan 14 wave T). Each test
//! names the leaf that owes it an implementation.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::handler_promotion::{promoted_relevants, promotions, promotions_from};
use formal_ai::rule_interpreter;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn shipped_promotions_text() -> String {
    let path = repo_root().join("data/seed/handler-promotions.lino");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("handler-promotions.lino readable: {error}"))
}

/// The fixture edit an operator would make: one new promotion row, hoisting a
/// handler that the shipped seed never hoists, on a condition the handler-rules
/// grammar already expresses. #959 "How to test" clause 4.
const FIXTURE_ROW: &str = concat!(
    "  promotion proof_request\n",
    "    rank 10\n",
    "    when\n",
    "      substring qed of lowercase\n",
    "    because \"wave T fixture: a promotion is a seed edit, never a Rust edit\"\n",
);

#[test]
fn a_seeded_promotion_row_changes_routing_with_no_rust_edit() {
    // The prompt carries the fixture's cue and nothing else that promotes.
    let prompt = "show the qed line";

    let shipped = promotions_from(&shipped_promotions_text())
        .expect("the shipped promotions document parses");
    let before = promoted_relevants(&shipped, prompt);
    assert!(
        !before.contains(&"handler:proof_request".to_owned()),
        "the shipped seed must not already hoist proof_request for {prompt:?}: {before:?}"
    );

    let patched_text = format!("{}{FIXTURE_ROW}", shipped_promotions_text());
    let patched = promotions_from(&patched_text).expect("the patched promotions document parses");
    let after = promoted_relevants(&patched, prompt);
    assert!(
        after.contains(&"handler:proof_request".to_owned()),
        "plan 09 leaf 11: adding one seed row must change routing with no Rust edit, got {after:?}"
    );

    // Rank orders the promotions that fired; the fixture's rank 10 wins.
    assert_eq!(
        after.first().map(String::as_str),
        Some("handler:proof_request"),
        "promotions are pushed in rank order, lower rank first"
    );
}

#[test]
fn promotion_conditions_use_the_handler_rules_grammar() {
    // One evaluator, two callers: every `when` block must parse and evaluate
    // through the same interpreter `data/seed/handler-rules.lino` already uses.
    let rows = promotions();
    assert!(
        rows.len() >= 19,
        "plan 09 leaf 10 transcribes all nineteen promotion predicates, got {}",
        rows.len()
    );
    let grammar = rule_interpreter::rules();
    assert!(
        grammar.rule_count() > 0,
        "the handler-rules grammar must be loaded before a promotion can reuse it"
    );
    for row in &rows {
        assert!(
            !row.when.trim().is_empty(),
            "promotion `{}` declares no condition",
            row.handler
        );
        assert!(
            !row.because.trim().is_empty(),
            "promotion `{}` names no reason; an unexplained hoist is an omission",
            row.handler
        );
        for keyword in row.when.split_whitespace() {
            let structural = matches!(
                keyword,
                "all"
                    | "any"
                    | "none"
                    | "role"
                    | "role_prefix"
                    | "role_padded"
                    | "role_lead"
                    | "word"
                    | "substring"
                    | "route_exact"
                    | "history_role"
                    | "shape"
                    | "of"
                    | "cleaned"
                    | "trimmed"
                    | "lowercase"
                    | "padded"
            );
            let operand = keyword
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '_');
            assert!(
                structural || operand,
                "promotion `{}` uses `{keyword}`, which is not in the handler-rules grammar",
                row.handler
            );
        }
    }
}

#[test]
fn no_handler_name_appears_in_prompt_relevants() {
    // Plan 09 leaf 11 deletes the nineteen-entry array and the
    // `contains("в ")` / `contains(':')` glue; after it, the file evaluates the
    // seed and keeps no handler names at all.
    let source = fs::read_to_string(repo_root().join("src/intent_formalization/prompt_relevants.rs"))
        .expect("prompt_relevants.rs readable");
    let literals = source.matches("\"handler:").count();
    assert_eq!(
        literals, 0,
        "src/intent_formalization/prompt_relevants.rs still names {literals} handlers as \
         literals; plan 09 leaf 11 moves every one into data/seed/handler-promotions.lino"
    );
}
