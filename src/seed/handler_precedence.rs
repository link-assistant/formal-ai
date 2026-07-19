//! Specialized-handler precedence loaded from `data/seed/handler-precedence.lino`
//! (issue #663).
//!
//! Handler precedence is behaviour — the order in which the universal solver
//! tries its specialized handlers, first-match-wins — and behaviour belongs in
//! seed data ("Data Is The Interface"), not in a Rust constant. This loader
//! reads the ordered handler rows — each row is a bare handler name, so the
//! precedence stays invisible to the seed's meaning-closure audit (only *value*
//! tokens are grounded, never the head slug a row names).
//! [`super::super::solver_dispatch`] joins that order with the Rust function
//! pointers (which must stay code) and asserts the two are an exact permutation
//! of each other, so a seed edit can never silently drop or duplicate a handler.
//!
//! The JavaScript worker loads the synced deployment copy through
//! `src/web/seed_loader.js`, and a routing-parity fixture pins the shared
//! precedence invariants across the Rust and browser surfaces.

use super::parser::parse_lino;
use super::HANDLER_PRECEDENCE_LINO;

/// Ordered specialized-handler names, in dispatch precedence order (first wins),
/// as declared by the shipped `data/seed/handler-precedence.lino`.
#[must_use]
pub fn handler_precedence() -> Vec<String> {
    handler_precedence_from(HANDLER_PRECEDENCE_LINO)
}

/// Parse an arbitrary handler-precedence document into its ordered handler names.
///
/// Exposed so tests can reorder rows in a fixture and observe the routing change
/// (`routing_precedence_from_seed`).
#[must_use]
pub fn handler_precedence_from(seed: &str) -> Vec<String> {
    let tree = parse_lino(seed);
    let Some(root) = tree.children.first() else {
        return Vec::new();
    };
    // Each row under the root is a bare handler name (comments already stripped
    // by the parser), in dispatch precedence order.
    root.children
        .iter()
        .map(|child| child.name.clone())
        .collect()
}
