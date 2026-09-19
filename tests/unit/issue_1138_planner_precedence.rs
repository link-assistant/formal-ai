//! Plan 10 leaf 18 (issue #1138): the planner's route precedence is data.
//!
//! `data/seed/planner-precedence.lino` names every route arm of
//! `plan_chat_step_routes` and `plan_settled_routes` in cascade order, and the
//! planner joins that order to its coded arms exactly the way
//! `solver_dispatch::specialized_handlers` joins `handler-precedence.lino`
//! (issue #663): every arm present once, in the position the cascade runs it,
//! panicking otherwise. These tests pin the join and prove it is
//! order-sensitive, so a seed edit can never silently drop, duplicate, or
//! reorder a route.

use formal_ai::agentic_coding::planner::checked_route_precedence;
use formal_ai::seed::{
    PLANNER_PRECEDENCE_LINO, PLANNER_PRECEDENCE_PATH, planner_precedence, planner_precedence_from,
};
use formal_ai::seed_links::{SeedLinkNetwork, network};

#[test]
fn planner_precedence_read_through_link_queries_equals_the_document() {
    let from_links = planner_precedence();
    assert_eq!(
        from_links,
        planner_precedence_from(PLANNER_PRECEDENCE_LINO).as_slice(),
        "the link-query reading and the direct parse must agree"
    );
    assert!(
        from_links.len() > 50,
        "the cascade has dozens of arms; the seed must name them all, got {}",
        from_links.len()
    );
    // The same rows through the generic pattern `(root $child)`.
    let loaded = network();
    let root = loaded
        .top_level(PLANNER_PRECEDENCE_PATH)
        .into_iter()
        .next()
        .expect("the precedence document has a root");
    let matched = loaded.query(&SeedLinkNetwork::children_pattern(&root.index));
    let names: Vec<String> = matched
        .iter()
        .filter(|link| !link.index.ends_with('='))
        .map(|link| link.to.clone())
        .collect();
    assert_eq!(names, from_links);
}

#[test]
fn the_seed_names_exactly_the_arms_the_cascade_runs_in_order() {
    let coded = checked_route_precedence();
    assert_eq!(
        planner_precedence(),
        coded,
        "the seed's arm order must be exactly the cascade's run order"
    );
    // Arms are unique: no two rows may name the same route.
    let mut unique = coded.to_vec();
    unique.sort_unstable();
    let count = unique.len();
    unique.dedup();
    assert_eq!(
        unique.len(),
        count,
        "every arm name in planner-precedence.lino must be distinct"
    );
}

/// The precedence document as its parseable lines (root plus indented rows),
/// with comment-only and blank lines dropped — the shape
/// `planner_precedence_from` parses. Trailing row comments stay; the parser
/// strips them, exactly as it does for the shipped document.
fn bare_rows() -> Vec<String> {
    PLANNER_PRECEDENCE_LINO
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn swapping_two_rows_is_not_a_permutation_the_join_accepts() {
    let mut rows = bare_rows();
    assert!(rows.len() > 3, "the fixture needs the root plus arms");
    rows.swap(1, 2);
    let fixture = rows.join("\n");
    let reordered = planner_precedence_from(&fixture);
    assert_ne!(
        reordered,
        planner_precedence(),
        "precedence is order-sensitive: a swapped seed must read differently"
    );
    assert_ne!(
        reordered.as_slice(),
        checked_route_precedence(),
        "the coded join must reject a reordered seed"
    );
}

#[test]
fn dropping_a_row_is_not_a_permutation_the_join_accepts() {
    let mut rows = bare_rows();
    let dropped = rows.remove(1);
    let fixture = rows.join("\n");
    let shorter = planner_precedence_from(&fixture);
    assert!(
        !shorter.contains(&dropped),
        "the fixture must no longer name {dropped}"
    );
    assert_ne!(
        shorter.as_slice(),
        checked_route_precedence(),
        "the coded join must reject a seed that silently drops an arm"
    );
}
