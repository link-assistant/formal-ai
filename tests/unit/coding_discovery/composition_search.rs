//! Issue #1138, plan 02 L7 — bounded typed enumeration replaces the forty
//! authored composition blocks.
//!
//! The point of the search is that a combination nobody wrote down composes,
//! because the enumeration is over fragment *type signatures* rather than over
//! authored shapes. Determinism is part of the contract: the same prompt and the
//! same catalog must yield the same candidates in the same order.

use formal_ai::coding_task_spec::recognise;
use formal_ai::composition_search::{SearchBounds, search};
use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::program_ir::ProgramIr;

/// A held-out request whose shape is in none of the deleted authored blocks.
const UNSEEN_COMBINATION: &str = concat!(
    "Write a Python function run_length(text) that returns a list of ",
    "(character, count) pairs for each run of equal characters in text."
);

fn candidates() -> Vec<ProgramIr> {
    let spec = recognise(UNSEEN_COMBINATION).expect("the request is a coding task");
    search(&spec, &FragmentCatalog::bootstrap(), SearchBounds::default())
}

#[test]
fn an_unseen_combination_of_seeded_meanings_composes() {
    let found = candidates();
    assert!(
        !found.is_empty(),
        "a combination absent from the authored blocks must still compose"
    );

    let first = &found[0];
    assert!(
        first.type_check(&FragmentCatalog::bootstrap()).is_ok(),
        "every enumerated candidate is well-typed"
    );
    assert!(
        found
            .windows(2)
            .all(|pair| pair[0].action_cost() <= pair[1].action_cost()),
        "candidates are enumerated cheapest first"
    );
    assert!(
        !first.fragments.is_empty(),
        "a composed program names the fragments it was built from"
    );
}

#[test]
fn search_is_deterministic_across_runs() {
    let first: Vec<String> = candidates().iter().map(ProgramIr::content_id).collect();
    let second: Vec<String> = candidates().iter().map(ProgramIr::content_id).collect();

    assert_eq!(
        first, second,
        "same prompt, same catalog, same order — enumeration is by cost then id"
    );
    let mut sorted = first.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        first.len(),
        "the enumeration never yields the same candidate twice"
    );
}
