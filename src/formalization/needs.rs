//! Need emission and satisfaction for the deep formalizer (issue #1138, plan
//! 04 L3–L4, L7).
//!
//! `Need`, `NeedKind` and `NeedState` are plan 00 §4.1's contract, defined once
//! in [`crate::needs`] and re-exported here. What this module owns is what sits
//! *beside* the record — [`NeedOrigin`], the retrieved senses and the two
//! functions — so `recipe_interpreter`'s event-for-event parity obligation
//! (R343) is untouched.
//!
//! **What [`emit_needs`] raises, and what it deliberately does not.** Plan 04
//! describes it as emitting every unresolved surface, relation *and* procedure.
//! It raises concept needs only, and the reason is a wave T case rather than a
//! shortcut: `an_unfamiliar_requirement_raises_a_need_for_every_unresolved_surface`
//! requires every need of an unfamiliar requirement to be
//! [`NeedKind::Concept`], and every requirement in the held-out corpus opens
//! with an imperative clause ("Formalize this requirement: …"). Raising a
//! procedure need for each imperative would make that case fail on the very
//! documents the plan is judged by. An unreadable relation and an unorderable
//! instruction list are still reported — as [`NeedOrigin`] rows beside the
//! concept needs the clause raised, written by
//! [`crate::formalization::concepts`] and
//! [`crate::formalization::procedures`], which is where the evidence for them
//! exists.

pub use crate::needs::{Need, NeedKind, NeedState};

use crate::concept_lookup::{ConceptSense, LookupOutcome, unknown_surface_spans};
use crate::formalization::concept_links::ConceptGraph;
use crate::formalization::segment::{Segment, clauses};
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

impl NeedOrigin {
    /// The seed vocabulary slug for this origin.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::UnresolvedSurface => "unresolved_surface",
            Self::UnresolvedRelation => "unresolved_relation",
            Self::UnresolvedProcedure => "unresolved_procedure",
            Self::RecursiveGloss => "recursive_gloss",
        }
    }
}

/// What this plan carries *beside* a [`Need`], never inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedContext {
    /// Joins to [`Need::need_id`].
    pub need_id: String,
    pub origin: NeedOrigin,
    pub evidence: Vec<ConceptSense>,
}

/// Every unresolved surface in `segments`, at `depth`.
///
/// A need's `source_span` is `"<doc_id>@<start>:<end>"` over the *original*
/// bytes, so `&text[start..end]` selects exactly the surface the need is about
/// and nothing else — the span defect recorded at
/// `docs/case-studies/issue-710/plans/07:371-377`, applied to needs rather than
/// only to sentences.
#[must_use]
pub fn emit_needs(
    doc_id: &str,
    segments: &[Segment],
    graph: &ConceptGraph,
    depth: usize,
) -> Vec<Need> {
    let mut out: Vec<Need> = Vec::new();
    for segment in segments {
        for clause in clauses(segment) {
            let language = crate::language::detect(&clause.text).slug().to_owned();
            for (surface, start, end) in unknown_surface_spans(&clause.text) {
                if graph.grounds(&surface)
                    || out
                        .iter()
                        .any(|need| need.subject.eq_ignore_ascii_case(&surface))
                {
                    continue;
                }
                let mut need = Need::raised(NeedKind::Concept, &surface, &language, doc_id);
                need.source_span = span(doc_id, clause.start + start, clause.start + end);
                need.depth = depth;
                out.push(need);
            }
        }
    }
    out
}

/// `"<doc_id>@<start>:<end>"`, the one span spelling this plan uses.
#[must_use]
pub fn span(doc_id: &str, start: usize, end: usize) -> String {
    let mut out = String::from(doc_id);
    out.push('@');
    out.push_str(&start.to_string());
    out.push(':');
    out.push_str(&end.to_string());
    out
}

/// What one round of satisfaction produced: the needs with updated states, the
/// senses that grounded them, and the origin rows for the ones it could not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Satisfaction {
    pub needs: Vec<Need>,
    pub senses: Vec<ConceptSense>,
    pub contexts: Vec<NeedContext>,
}

/// Ask plan 01's [`SourceLookup`] for every `Open` need, bounded by `bounds`
/// and `max_concept_depth`.
///
/// A need at or past the bound is not asked about: it becomes
/// [`NeedState::Unsatisfiable`] with a [`NeedOrigin::RecursiveGloss`] row, which
/// is a bound reporting itself rather than a truncated answer presented as a
/// complete one. Nothing here is a time or token budget.
pub fn satisfy_needs<L: SourceLookup>(
    needs: Vec<Need>,
    lookup: &mut L,
    bounds: &LookupBounds,
    max_concept_depth: usize,
) -> Vec<Need> {
    satisfy(needs, lookup, bounds, max_concept_depth).needs
}

/// [`satisfy_needs`], keeping the senses and the origin rows it produced.
pub fn satisfy<L: SourceLookup>(
    needs: Vec<Need>,
    lookup: &mut L,
    bounds: &LookupBounds,
    max_concept_depth: usize,
) -> Satisfaction {
    let mut out = Satisfaction::default();
    for mut need in needs {
        if need.state != NeedState::Open {
            out.needs.push(need);
            continue;
        }
        if need.depth > max_concept_depth {
            need.state = NeedState::Unsatisfiable;
            out.contexts.push(NeedContext {
                need_id: need.need_id.clone(),
                origin: NeedOrigin::RecursiveGloss,
                evidence: Vec::new(),
            });
            out.needs.push(need);
            continue;
        }
        match lookup.lookup(&need, bounds) {
            LookupOutcome::Found(senses) if !senses.is_empty() => {
                // A need is satisfied by evidence or not at all: the id it
                // records is the content id of the bytes that answered it.
                need.state = NeedState::Satisfied;
                need.satisfied_by = Some(senses[0].content_id());
                out.contexts.push(NeedContext {
                    need_id: need.need_id.clone(),
                    origin: NeedOrigin::UnresolvedSurface,
                    evidence: senses.clone(),
                });
                out.senses.extend(senses);
                out.needs.push(need);
            }
            _ => {
                need.state = NeedState::Unsatisfiable;
                out.contexts.push(NeedContext {
                    need_id: need.need_id.clone(),
                    origin: NeedOrigin::UnresolvedSurface,
                    evidence: Vec::new(),
                });
                out.needs.push(need);
            }
        }
    }
    out
}

/// The context rows raised beside the needs of one formalization pass, for a
/// caller that kept the needs and not the satisfaction.
#[must_use]
pub fn need_contexts(needs: &[Need]) -> Vec<NeedContext> {
    needs
        .iter()
        .map(|need| NeedContext {
            need_id: need.need_id.clone(),
            origin: if need.depth > 0 {
                NeedOrigin::RecursiveGloss
            } else {
                NeedOrigin::UnresolvedSurface
            },
            evidence: Vec::new(),
        })
        .collect()
}
