//! Bounded typed enumeration over the available fragments (issue #1138, plan
//! 02 L7).
//!
//! The search replaces the forty hand-authored composition blocks: an unseen
//! combination of seeded meanings composes because the enumeration is over
//! fragment *type signatures*, not over authored shapes. Order is deterministic
//! — by [`crate::coding::program_ir::ProgramIr::action_cost`], then content id.

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::program_ir::ProgramIr;
use crate::coding::task_spec::CodingTaskSpec;

/// Declared enumeration bounds: a width and a depth, never a time budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBounds {
    pub max_candidates: usize,
    pub max_depth: usize,
}

impl Default for SearchBounds {
    fn default() -> Self {
        Self {
            max_candidates: 64,
            max_depth: 4,
        }
    }
}

/// Enumerate typed candidate programs for `spec` from `catalog`, cheapest
/// first and deterministic across runs.
#[must_use]
pub fn search(
    _spec: &CodingTaskSpec,
    _catalog: &FragmentCatalog,
    _bounds: SearchBounds,
) -> Vec<ProgramIr> {
    todo!("plan 02 leaf L7")
}
