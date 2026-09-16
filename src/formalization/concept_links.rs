//! Everything one formalization grounded, plus everything it could not
//! (issue #1138, plan 04 L8).
//!
//! [`ConceptGraph::identity`] is language-independent: it is taken over grounded
//! concept ids, relation kinds and structural meaning ids, never over surface
//! text and never over the source language, so one requirement written in five
//! languages produces one identity.

use std::collections::BTreeSet;

use crate::formalization::concepts::{
    ExtractedConcept, ExtractedEntity, ExtractedRelation, concept_from_sense, relations_in,
};
use crate::formalization::needs::{emit_needs, satisfy};
use crate::formalization::procedures::ExtractedProcedure;
use crate::formalization::segment::{Segment, clauses, sentences};
use crate::needs::{Need, NeedState};
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
    /// structural meaning ids and relation kinds.
    ///
    /// Retrieved headwords deliberately stay out of this identity. They remain
    /// fully represented in `concepts`, but a headword is a property of the
    /// language and source that answered a need rather than of the requirement
    /// itself. This is the same boundary used by `ConceptMap::identity`: source
    /// evidence proves the reduction without making its content id depend on
    /// whether the source called the concept "isogram", "isograma", or another
    /// equivalent surface.
    #[must_use]
    pub fn identity(&self) -> String {
        // A partial lookup is evidence about individual headwords, but it is
        // not yet a language-independent reduction of the whole document.
        // Including structures seen on whichever source happened to answer
        // would make the same requirement change identity with connectivity.
        // Keep those structures available through `structure_ids`; admit them
        // to the semantic identity only once every raised need is grounded.
        let structures = if self.unresolved().is_empty() {
            self.structure_ids()
        } else {
            Vec::new()
        };
        let mut relations: Vec<String> = self
            .relations
            .iter()
            .map(|relation| relation.kind.clone())
            .collect();
        relations.sort();
        relations.dedup();
        let mut procedure_shapes: Vec<String> = self
            .procedures
            .iter()
            .map(|procedure| procedure.steps.len().to_string())
            .collect();
        procedure_shapes.sort();
        crate::engine::stable_id(
            "concept_graph",
            &format!(
                "structures={};relations={};procedure_shapes={}",
                structures.join(","),
                relations.join(","),
                procedure_shapes.join(",")
            ),
        )
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        crate::links_format::push_lino_node(&mut out, 0, "concept_graph", Some(&self.identity()));
        crate::links_format::push_lino_node(&mut out, 2, "document", Some(&self.doc_id));

        let mut concepts = self.concepts.iter().collect::<Vec<_>>();
        concepts.sort_by(|left, right| left.id.cmp(&right.id));
        for concept in concepts {
            crate::links_format::push_lino_node(&mut out, 2, "concept", Some(&concept.id));
            crate::links_format::push_lino_node(&mut out, 4, "label", Some(&concept.label));
            crate::links_format::push_lino_node(&mut out, 4, "language", Some(&concept.language));
            crate::links_format::push_lino_node(&mut out, 4, "gloss", Some(&concept.gloss));
            if let Some(genus) = &concept.genus {
                crate::links_format::push_lino_node(&mut out, 4, "genus", Some(genus));
            }
            for differentia in &concept.differentiae {
                crate::links_format::push_lino_node(&mut out, 4, "differentia", Some(differentia));
            }
            for structure in &concept.structures {
                crate::links_format::push_lino_node(&mut out, 4, "structure", Some(structure));
            }
            crate::links_format::push_lino_node(&mut out, 4, "source", Some(&concept.source_id));
            crate::links_format::push_lino_node(
                &mut out,
                4,
                "source_url",
                Some(&concept.source_url),
            );
            crate::links_format::push_lino_node(&mut out, 4, "sha256", Some(&concept.sha256));
            crate::links_format::push_lino_node(
                &mut out,
                4,
                "license",
                Some(&concept.license_name),
            );
            crate::links_format::push_lino_node(
                &mut out,
                4,
                "depth",
                Some(&concept.depth.to_string()),
            );
        }

        let mut relations = self.relations.iter().collect::<Vec<_>>();
        relations.sort_by(|left, right| left.id.cmp(&right.id));
        for relation in relations {
            crate::links_format::push_lino_node(&mut out, 2, "relation", Some(&relation.id));
            crate::links_format::push_lino_node(&mut out, 4, "kind", Some(&relation.kind));
            crate::links_format::push_lino_node(&mut out, 4, "subject", Some(&relation.subject));
            crate::links_format::push_lino_node(&mut out, 4, "object", Some(&relation.object));
            crate::links_format::push_lino_node(&mut out, 4, "cue", Some(&relation.cue));
            crate::links_format::push_lino_node(
                &mut out,
                4,
                "source_span",
                Some(&relation.source_span),
            );
        }

        let mut needs = self.needs.iter().collect::<Vec<_>>();
        needs.sort_by(|left, right| left.need_id.cmp(&right.need_id));
        for need in needs {
            out.push_str(&need.to_links_notation());
        }

        out.trim_end().to_owned()
    }

    /// The projection the composer consumes.
    #[must_use]
    pub fn structure_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .concepts
            .iter()
            .flat_map(|concept| concept.structures.iter().cloned())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Whether `surface` is already accounted for by something this graph
    /// grounded, so a second pass does not ask about it again.
    #[must_use]
    pub fn grounds(&self, surface: &str) -> bool {
        self.concepts
            .iter()
            .any(|concept| concept.label.eq_ignore_ascii_case(surface))
            || self
                .entities
                .iter()
                .any(|entity| entity.label.eq_ignore_ascii_case(surface))
    }

    /// Every need no source grounded, in the order they were raised.
    #[must_use]
    pub fn unresolved(&self) -> Vec<&Need> {
        self.needs
            .iter()
            .filter(|need| need.state != NeedState::Satisfied)
            .collect()
    }

    /// `(grounded, total needs)` — a document with an unresolved need can never
    /// be reported as covered.
    #[must_use]
    pub fn grounded_ratio(&self) -> (usize, usize) {
        let grounded = self
            .needs
            .iter()
            .filter(|need| need.state == NeedState::Satisfied)
            .count();
        (grounded, self.needs.len())
    }

    /// The deepest recursion this formalization actually reached.
    #[must_use]
    pub fn max_depth_reached(&self) -> usize {
        self.needs.iter().map(|need| need.depth).max().unwrap_or(0)
    }
}

/// The entry point. Deterministic for a given text, lookup and bounds. An
/// offline caller passes an offline lookup and every unmet need becomes
/// `NeedState::Unsatisfiable` with its consulted-source rows intact.
pub fn formalize_deeply<L: SourceLookup>(
    text: &str,
    doc_id: &str,
    lookup: &mut L,
    bounds: &LookupBounds,
    max_concept_depth: usize,
) -> ConceptGraph {
    let segments = sentences(text);
    let mut graph = ConceptGraph {
        doc_id: doc_id.to_owned(),
        segments,
        ..ConceptGraph::default()
    };
    let mut pending = emit_needs(doc_id, &graph.segments, &graph, 0);
    let mut seen: BTreeSet<String> = pending.iter().map(|need| need.need_id.clone()).collect();

    while !pending.is_empty() {
        let satisfaction = satisfy(pending, lookup, bounds, max_concept_depth);
        let answered = satisfaction.needs.clone();

        for need in answered {
            if let Some(existing) = graph
                .needs
                .iter_mut()
                .find(|existing| existing.need_id == need.need_id)
            {
                *existing = need;
            } else {
                graph.needs.push(need);
            }
        }

        let mut next = Vec::new();
        for sense in satisfaction.senses {
            let parent = graph.needs.iter().find(|need| {
                need.state == NeedState::Satisfied
                    && need.subject.eq_ignore_ascii_case(&sense.surface)
            });
            let Some(parent) = parent.cloned() else {
                continue;
            };

            let mut grounded_sense = sense.clone();
            grounded_sense.depth = parent.depth;
            if let Some(concept) = concept_from_sense(&grounded_sense)
                && !graph
                    .concepts
                    .iter()
                    .any(|existing| existing.id == concept.id)
            {
                graph.concepts.push(concept);
            }

            if parent.depth >= max_concept_depth {
                continue;
            }
            let sense_doc = sense.content_id();
            let gloss_segments = sentences(&sense.gloss);
            for mut need in emit_needs(&sense_doc, &gloss_segments, &graph, parent.depth + 1) {
                need.raised_by = parent.need_id.clone();
                if seen.insert(need.need_id.clone()) {
                    next.push(need);
                }
            }
        }
        pending = next;
    }

    graph.concepts.sort_by(|left, right| left.id.cmp(&right.id));
    graph.needs.sort_by(|left, right| {
        left.depth
            .cmp(&right.depth)
            .then_with(|| left.source_span.cmp(&right.source_span))
            .then_with(|| left.need_id.cmp(&right.need_id))
    });
    let clauses = graph.segments.iter().flat_map(clauses).collect::<Vec<_>>();
    graph.relations = clauses
        .iter()
        .flat_map(|clause| relations_in(clause, &graph))
        .collect();
    graph
        .relations
        .sort_by(|left, right| left.id.cmp(&right.id));
    graph.relations.dedup_by(|left, right| left.id == right.id);
    graph
}
