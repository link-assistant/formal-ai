//! Candidate-draft strategy order loaded from `data/seed/draft-strategies.lino`
//! (issue #704).
//!
//! Which generators a candidate-solution portfolio may spend its draft slots on
//! — and in which order — is behaviour, and behaviour belongs in seed data
//! ("Data Is The Interface"), not in a Rust `match` on the draft count. This
//! loader reads the ordered strategy rows the same way
//! [`super::handler_precedence`] reads dispatch precedence: each row is a bare
//! strategy name, so the catalog stays invisible to the seed's meaning-closure
//! audit (only *value* tokens are grounded, never a head slug).
//!
//! [`crate::draft_portfolio`] intersects this order with the strategies a given
//! solver leaf can actually run, so adding a new draft generator is a seed edit
//! plus a leaf that answers `supports` — never a new branch in the portfolio
//! engine.

use super::parser::parse_lino;
use super::DRAFT_STRATEGIES_LINO;

/// Ordered candidate-draft strategy names, cheapest and most reusable first, as
/// declared by the shipped `data/seed/draft-strategies.lino`.
#[must_use]
pub fn draft_strategies() -> Vec<String> {
    draft_strategies_from(DRAFT_STRATEGIES_LINO)
}

/// Parse an arbitrary draft-strategy document into its ordered strategy names.
///
/// Exposed so tests can reorder rows in a fixture and observe the portfolio
/// change, proving the order is data rather than code.
#[must_use]
pub fn draft_strategies_from(seed: &str) -> Vec<String> {
    let tree = parse_lino(seed);
    let Some(root) = tree.children.first() else {
        return Vec::new();
    };
    root.children
        .iter()
        .map(|child| child.name.clone())
        .collect()
}
