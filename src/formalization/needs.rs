//! Need emission and satisfaction for the deep formalizer (issue #1138, plan
//! 04 L3–L4, L7).
//!
//! `Need`, `NeedKind` and `NeedState` are plan 00 §4.1's contract, defined once
//! in [`crate::needs`] and re-exported here. What this module owns is what sits
//! *beside* the record — [`NeedOrigin`], the retrieved senses and the two
//! functions — so `recipe_interpreter`'s event-for-event parity obligation
//! (R343) is untouched.

pub use crate::needs::{Need, NeedKind, NeedState};

use crate::concept_lookup::ConceptSense;
use crate::formalization::concept_links::ConceptGraph;
use crate::formalization::segment::Segment;
use crate::source_walk::{LookupBounds, SourceLookup};

/// A finer reason *inside* a `NeedKind::Concept` or `NeedKind::Procedure` need,
/// so a reader can tell an unknown word from an unknown relation from an
/// exhausted recursion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedOrigin {
    /// A surface no seeded meaning, memory link or retrieved sense accounts for.
    UnresolvedSurface,
    /// A clause whose relation between two grounded terms is not recognised.
    UnresolvedRelation,
    /// An imperative clause with no ordered procedure behind it.
    UnresolvedProcedure,
    /// A retrieved gloss whose own surfaces are unresolved at depth+1.
    RecursiveGloss,
}

/// What this plan carries *beside* a [`Need`], never inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedContext {
    /// Joins to [`Need::need_id`].
    pub need_id: String,
    pub origin: NeedOrigin,
    pub evidence: Vec<ConceptSense>,
}

/// Every unresolved surface, relation and procedure in `segments`, at `depth`.
#[must_use]
pub fn emit_needs(
    _doc_id: &str,
    _segments: &[Segment],
    _graph: &ConceptGraph,
    _depth: usize,
) -> Vec<Need> {
    todo!("plan 04 leaf L4")
}

/// Ask plan 01's [`SourceLookup`] for every `Open` need, bounded by `bounds`
/// and `max_concept_depth`; each satisfied need's gloss is itself segmented and
/// may emit needs at `depth + 1`.
pub fn satisfy_needs<L: SourceLookup>(
    _needs: Vec<Need>,
    _lookup: &mut L,
    _bounds: &LookupBounds,
    _max_concept_depth: usize,
) -> Vec<Need> {
    todo!("plan 04 leaf L7")
}

/// The context rows raised beside the needs of one formalization pass.
#[must_use]
pub fn need_contexts(_needs: &[Need]) -> Vec<NeedContext> {
    todo!("plan 04 leaf L4")
}
