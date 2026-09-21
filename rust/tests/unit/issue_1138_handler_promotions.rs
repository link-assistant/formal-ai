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
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
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
        let document = format!("handler_promotions\n{}", row.to_links_notation());
        promotions_from(&document).unwrap_or_else(|error| {
            panic!(
                "promotion `{}` must round-trip through the shared condition parser: {error}",
                row.handler
            )
        });
    }

    let malformed = "handler_promotions\n  promotion bad\n    rank 1\n    when\n      invented_predicate value\n    because invalid\n";
    assert!(
        promotions_from(malformed).is_err(),
        "an unknown condition must be rejected by the handler-rules parser"
    );
}

#[test]
fn a_seeded_promotion_reads_a_phrasal_verb_around_its_object() {
    let promoted = promoted_relevants(
        &promotions(),
        "Break the customer import rewrite into sub-tasks.",
    );
    assert!(
        promoted.contains(&"handler:task_decomposition".to_owned()),
        "the data-owned promotion must preserve the lexicon's discontinuous `break into` \
         meaning: {promoted:?}"
    );
}

/// A prefix surface ("prove …") is a phrase, so its lead half must be present as
/// complete words. The Spanish word for "providers" embeds the English "prove",
/// and a raw substring read of the prefix promoted `proof_request` for the
/// held-out Spanish repository prompt of plan 03, preempting the capability
/// table's typed workspace handoff (issue #1138, plan 03 wave F).
#[test]
fn a_prefix_surface_does_not_match_inside_an_embedding_word() {
    let spanish = "la lista de proveedores de búsqueda de confianza";
    let promoted = promoted_relevants(&promotions(), spanish);
    assert!(
        !promoted.contains(&"handler:proof_request".to_owned()),
        "{spanish:?} embeds `prove` inside `proveedores`; that is not a proof request, \
         so the promotion must not fire: {promoted:?}"
    );

    // The genuine English prefix readings keep promoting.
    for prompt in [
        "prove that 2 + 2 = 4",
        "show that every even sum has two primes",
    ] {
        let promoted = promoted_relevants(&promotions(), prompt);
        assert!(
            promoted.contains(&"handler:proof_request".to_owned()),
            "{prompt:?} leads with a seeded proof surface and must still promote: {promoted:?}"
        );
    }
}

/// A semantic form with an open slot is still a seeded role surface. The
/// promotion evaluator must interpret that slot instead of looking for a
/// literal ellipsis, or generic source retrieval can steal a locally
/// derivable, checkable task before the verifiable-task interpreter runs.
#[test]
fn slot_backed_expectations_promote_the_verifiable_interpreter() {
    let corpus =
        fs::read_to_string(repo_root().join("data/benchmarks/verifiable-task-paraphrases.lino"))
            .expect("the held-out verifiable-task corpus should be readable");
    let mut prompts = Vec::new();
    for line in corpus.lines() {
        let trimmed = line.trim();
        if let Some(raw) = trimmed.strip_prefix("prompt ") {
            let prompt = raw
                .trim()
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(raw)
                .replace("\"\"", "\"");
            prompts.push(prompt);
        }
    }
    assert_eq!(
        prompts.len(),
        30,
        "the six-by-five corpus must stay complete"
    );

    let rows = promotions();
    let mut missed = Vec::new();
    for prompt in prompts {
        if !promoted_relevants(&rows, &prompt).contains(&"handler:verifiable_task".to_owned()) {
            missed.push(prompt);
        }
    }
    assert!(
        missed.is_empty(),
        "every seeded checkable expectation must promote verifiable_task: {missed:?}"
    );
}

#[test]
fn no_handler_name_appears_in_prompt_relevants() {
    // Plan 09 leaf 11 deletes the nineteen-entry array and the
    // `contains("в ")` / `contains(':')` glue; after it, the file evaluates the
    // seed and keeps no handler names at all.
    let source =
        fs::read_to_string(repo_root().join("src/intent_formalization/prompt_relevants.rs"))
            .expect("prompt_relevants.rs readable");
    let literals = source.matches("\"handler:").count();
    assert_eq!(
        literals, 0,
        "src/intent_formalization/prompt_relevants.rs still names {literals} handlers as \
         literals; plan 09 leaf 11 moves every one into data/seed/handler-promotions.lino"
    );
}
