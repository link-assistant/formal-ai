//! Sense → concept / predicate / entity links with provenance (issue #1138,
//! plan 04 L6).
//!
//! A gloss is split into genus and differentiae using the cues declared in
//! `data/seed/formalization-relations.lino`; the relation seed declares how a
//! gloss is read, never what a word means. No substring of a grounded gloss may
//! reach generated source — that is the gloss-not-code gate plan 01 and this
//! plan share.

use crate::concept_lookup::ConceptSense;
use crate::formalization::graph::ConceptGraph;
use crate::formalization::segment::Segment;

/// A concept the formalizer grounded, and the exact bytes that ground it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedConcept {
    /// `"concept:<slug of lemma>"`.
    pub id: String,
    pub label: String,
    pub language: String,
    pub gloss: String,
    /// The "is a" term the gloss names, when it names one.
    pub genus: Option<String>,
    pub differentiae: Vec<String>,
    /// Seeded structural meaning ids the gloss evidences.
    pub structures: Vec<String>,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub license_name: String,
    pub depth: usize,
}

/// A relation evidenced between two grounded terms in one clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedRelation {
    pub id: String,
    /// One of the eight seeded relation kinds.
    pub kind: String,
    pub subject: String,
    pub object: String,
    /// The seeded cue that evidenced the relation.
    pub cue: String,
    /// `"<doc_id>@<start>:<end>"`, the clause span the cue was read from.
    pub source_span: String,
    pub language: String,
}

/// A named individual the formalizer recognised in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedEntity {
    pub id: String,
    pub label: String,
    pub language: String,
    /// `"<doc_id>@<start>:<end>"`, the exact span of the mention.
    pub source_span: String,
}

/// Ground one retrieved sense as a concept: split the gloss into genus and
/// differentiae using the seeded relation cues, then re-run the differentiae
/// through the structural lexicon so the concept reaches seeded idioms.
#[must_use]
pub fn concept_from_sense(_sense: &ConceptSense) -> Option<ExtractedConcept> {
    todo!("plan 04 leaf L6")
}

/// Relations evidenced between two grounded terms in one clause, from the cues
/// declared in `data/seed/formalization-relations.lino`. Never invents a
/// relation the cues do not evidence.
#[must_use]
pub fn relations_in(_clause: &Segment, _graph: &ConceptGraph) -> Vec<ExtractedRelation> {
    todo!("plan 04 leaf L6")
}
