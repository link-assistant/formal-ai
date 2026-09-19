//! Agentic-route precedence loaded from `data/seed/planner-precedence.lino`
//! (plan 10 leaf 18, issue #1138).
//!
//! The agentic planner's cascade is behaviour — the order in which
//! `plan_chat_step_routes` and `plan_settled_routes` try their route arms,
//! first match wins — and behaviour belongs in seed data ("Data Is The
//! Interface"), not only in a code reading. This loader reads the ordered
//! arm rows — each row is a bare arm name, so the precedence stays invisible
//! to the seed's meaning-closure audit (only *value* tokens are grounded,
//! never the head slug a row names).
//! [`crate::agentic_coding::planner`] joins that order with the coded arms
//! (which must stay code) and asserts the two are an exact permutation of
//! each other, so a seed edit can never silently drop, duplicate or reorder
//! a route — the same join `super::handler_precedence` has with
//! `solver_dispatch` (issue #663).

use super::parser::parse_lino;

/// Ordered planner route-arm names, in cascade precedence order (first wins),
/// as declared by the shipped `data/seed/planner-precedence.lino`.
/// Built once from the seed links network, like [`super::handler_precedence`].
#[must_use]
pub fn planner_precedence() -> &'static [String] {
    static CELL: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CELL.get_or_init(load_planner_precedence)
}

fn load_planner_precedence() -> Vec<String> {
    let network = crate::seed_links::network();
    let Some(root) = network
        .top_level(PLANNER_PRECEDENCE_PATH)
        .into_iter()
        .next()
    else {
        return Vec::new();
    };
    network
        .nodes_under(&root.index)
        .into_iter()
        .map(|row| row.to.clone())
        .collect()
}

/// Repository path of the precedence seed, its name in the seed links network.
pub const PLANNER_PRECEDENCE_PATH: &str = "data/seed/planner-precedence.lino";

/// Parse an arbitrary planner-precedence document into its ordered arm names.
///
/// Exposed so tests can reorder rows in a fixture and observe the permutation
/// assertion fire.
#[must_use]
pub fn planner_precedence_from(seed: &str) -> Vec<String> {
    let tree = parse_lino(seed);
    let Some(root) = tree.children.first() else {
        return Vec::new();
    };
    // Each row under the root is a bare arm name (comments already stripped
    // by the parser), in cascade precedence order.
    root.children
        .iter()
        .map(|child| child.name.clone())
        .collect()
}
