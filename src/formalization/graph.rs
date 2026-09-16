//! Everything one formalization grounded, plus everything it could not
//! (issue #1138, plan 04 L8).
//!
//! [`ConceptGraph::identity`] is language-independent: it is taken over grounded
//! concept ids, relation kinds and structural meaning ids, never over surface
//! text and never over the source language, so one requirement written in five
//! languages produces one identity.

use crate::formalization::concepts::{ExtractedConcept, ExtractedEntity, ExtractedRelation};
use crate::formalization::procedures::ExtractedProcedure;
use crate::formalization::segment::Segment;
use crate::needs::Need;
use crate::source_walk::{LookupBounds, SourceLookup};

/// Everything one formalization grounded, plus everything it could not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConceptGraph {
    pub doc_id: String,
    pub concepts: Vec<ExtractedConcept>,
    pub relations: Vec<ExtractedRelation>,
    pub procedures: Vec<ExtractedProcedure>,
    pub entities: Vec<ExtractedEntity>,
    pub needs: Vec<Need>,
    pub segments: Vec<Segment>,
}

impl ConceptGraph {
    /// The five-language invariant: language-independent identity over grounded
    /// concept ids, relation kinds and structural meaning ids.
    #[must_use]
    pub fn identity(&self) -> String {
        todo!("plan 04 leaf L8")
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 04 leaf L8")
    }

    /// The projection the composer consumes.
    #[must_use]
    pub fn structure_ids(&self) -> Vec<String> {
        todo!("plan 04 leaf L8")
    }

    #[must_use]
    pub fn unresolved(&self) -> Vec<&Need> {
        todo!("plan 04 leaf L8")
    }

    /// `(grounded, total needs)` — a document with an unresolved need can never
    /// be reported as covered.
    #[must_use]
    pub fn grounded_ratio(&self) -> (usize, usize) {
        todo!("plan 04 leaf L8")
    }
}

/// The entry point. Deterministic for a given text, lookup and bounds. An
/// offline caller passes an offline lookup and every unmet need becomes
/// `NeedState::Unsatisfiable` with its consulted-source rows intact.
pub fn formalize_deeply<L: SourceLookup>(
    _text: &str,
    _doc_id: &str,
    _lookup: &mut L,
    _bounds: &LookupBounds,
    _max_concept_depth: usize,
) -> ConceptGraph {
    todo!("plan 04 leaf L8")
}
