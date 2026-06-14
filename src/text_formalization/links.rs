//! Reduction of the nine primitives to plain links (doublets).
//!
//! The maintainer's standing position on issue #468 is that *everything is a
//! link*. This module makes that literal: every primitive — including the
//! [`Assertion`], which the protocol treats as irreducible — is decomposed into
//! a flat set of [`Link`] doublets, each a named directed edge `source ->
//! target`. The role of each edge (subject, predicate, object, time, …) lives in
//! the link's identifier, which is the legible inline form of a fully reduced,
//! untyped links store.
//!
//! The reduction is deterministic: identical knowledge bases produce identical
//! link sets in identical order, so the link count and membership can be pinned
//! by tests and golden files.

use serde::{Deserialize, Serialize};

use super::knowledge_base::KnowledgeBase;
use super::primitives::{
    Annotation, Assertion, Concept, Context, Entity, Predicate, Procedure, Provenance, Temporal,
};
use crate::links_format::format_lino_record;

/// A single link (doublet): a named directed edge from `source` to `target`.
///
/// In a fully reduced, untyped links store a link is just an ordered pair; the
/// `id` here names the pair so the reduction stays human-legible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// Stable, content-derived identifier (e.g. `lnk:a1:subject`).
    pub id: String,
    /// The source node identifier.
    pub source: String,
    /// The target node identifier.
    pub target: String,
}

impl Link {
    /// Build a link from an id, a source node and a target node.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
        }
    }

    /// Render the link as a Links-Notation record.
    #[must_use]
    pub fn to_lino(&self) -> String {
        format_lino_record(
            &self.id,
            &[
                ("source", self.source.clone()),
                ("target", self.target.clone()),
            ],
        )
    }
}

/// The literal node identifier for a typed literal value.
fn literal_node(datatype: &str, value: &str) -> String {
    format!("lit:{datatype}:{value}")
}

/// A node identifier for a temporal value.
fn temporal_node(temporal: &Temporal) -> String {
    match temporal {
        Temporal::Instant { value, .. } => format!("time:instant:{value}"),
        Temporal::Interval { start, end } => format!("time:interval:{start}..{end}"),
        Temporal::Relative { value } => format!("time:relative:{value}"),
    }
}

/// A node identifier for a provenance record.
fn provenance_node(provenance: &Provenance) -> String {
    provenance.offsets.map_or_else(
        || format!("prov:{}", provenance.source_doc),
        |[start, end]| format!("prov:{}:{start}-{end}", provenance.source_doc),
    )
}

/// Reduce a concept declaration to links.
fn concept_links(concept: &Concept) -> Vec<Link> {
    let base = &concept.id;
    let mut links = vec![
        Link::new(format!("lnk:{base}:type"), base.clone(), "Concept"),
        Link::new(
            format!("lnk:{base}:label"),
            base.clone(),
            literal_node("string", &concept.label),
        ),
    ];
    if !concept.concept_type.is_empty() {
        links.push(Link::new(
            format!("lnk:{base}:kind"),
            base.clone(),
            literal_node("string", &concept.concept_type),
        ));
    }
    for (key, value) in &concept.attributes {
        links.push(Link::new(
            format!("lnk:{base}:attr:{key}"),
            base.clone(),
            literal_node("string", value),
        ));
    }
    links
}

/// Reduce an entity declaration to links.
fn entity_links(entity: &Entity) -> Vec<Link> {
    let base = &entity.id;
    let mut links = vec![
        Link::new(format!("lnk:{base}:type"), base.clone(), "Entity"),
        Link::new(
            format!("lnk:{base}:label"),
            base.clone(),
            literal_node("string", &entity.label),
        ),
    ];
    for (index, form) in entity.canonical_forms.iter().enumerate() {
        links.push(Link::new(
            format!("lnk:{base}:form:{index}"),
            base.clone(),
            literal_node("string", form),
        ));
    }
    for (key, value) in &entity.attributes {
        links.push(Link::new(
            format!("lnk:{base}:attr:{key}"),
            base.clone(),
            literal_node("string", value),
        ));
    }
    links
}

/// Reduce a predicate declaration to links.
fn predicate_links(predicate: &Predicate) -> Vec<Link> {
    let base = &predicate.id;
    let mut links = vec![
        Link::new(format!("lnk:{base}:type"), base.clone(), "Predicate"),
        Link::new(
            format!("lnk:{base}:name"),
            base.clone(),
            literal_node("string", &predicate.name),
        ),
        Link::new(
            format!("lnk:{base}:arity"),
            base.clone(),
            literal_node("number", &predicate.arity.to_string()),
        ),
    ];
    if !predicate.semantics.is_empty() {
        links.push(Link::new(
            format!("lnk:{base}:semantics"),
            base.clone(),
            literal_node("string", &predicate.semantics),
        ));
    }
    links
}

/// Reduce a procedure declaration to links.
fn procedure_links(procedure: &Procedure) -> Vec<Link> {
    let base = &procedure.id;
    let mut links = vec![
        Link::new(format!("lnk:{base}:type"), base.clone(), "Procedure"),
        Link::new(
            format!("lnk:{base}:signature"),
            base.clone(),
            literal_node("string", &procedure.signature),
        ),
        Link::new(
            format!("lnk:{base}:body"),
            base.clone(),
            literal_node("string", &procedure.body),
        ),
    ];
    for (index, trigger) in procedure.triggers.iter().enumerate() {
        links.push(Link::new(
            format!("lnk:{base}:trigger:{index}"),
            base.clone(),
            trigger.clone(),
        ));
    }
    links
}

/// Reduce a context declaration to links.
fn context_links(context: &Context) -> Vec<Link> {
    let base = &context.id;
    let mut links = vec![Link::new(
        format!("lnk:{base}:type"),
        base.clone(),
        "Context",
    )];
    if !context.label.is_empty() {
        links.push(Link::new(
            format!("lnk:{base}:label"),
            base.clone(),
            literal_node("string", &context.label),
        ));
    }
    if !context.description.is_empty() {
        links.push(Link::new(
            format!("lnk:{base}:description"),
            base.clone(),
            literal_node("string", &context.description),
        ));
    }
    for (key, value) in &context.properties {
        links.push(Link::new(
            format!("lnk:{base}:prop:{key}"),
            base.clone(),
            literal_node("string", value),
        ));
    }
    links
}

/// Reduce a source annotation to links.
fn annotation_links(annotation: &Annotation) -> Vec<Link> {
    let base = &annotation.id;
    let [start, end] = annotation.offsets;
    let mut links = vec![
        Link::new(format!("lnk:{base}:type"), base.clone(), "Annotation"),
        Link::new(
            format!("lnk:{base}:source"),
            base.clone(),
            annotation.source_doc.clone(),
        ),
        Link::new(
            format!("lnk:{base}:span"),
            base.clone(),
            format!("span:{start}-{end}"),
        ),
    ];
    if !annotation.language.is_empty() {
        links.push(Link::new(
            format!("lnk:{base}:language"),
            base.clone(),
            literal_node("string", &annotation.language),
        ));
    }
    for (index, token) in annotation.tokenization.iter().enumerate() {
        links.push(Link::new(
            format!("lnk:{base}:token:{index}"),
            base.clone(),
            literal_node("string", token),
        ));
    }
    links
}

impl Assertion {
    /// Reduce this assertion to its constituent links (doublets).
    ///
    /// Each structural slot of the assertion becomes one link whose role is
    /// encoded in the identifier suffix: `:type`, `:subject`, `:predicate`,
    /// `:object:<i>`, `:time`, `:context`, `:modal`, `:confidence` and
    /// `:provenance`. The modal and confidence links are always present (the
    /// protocol mandates a modality and confidence on every assertion); the
    /// temporal, context and provenance links appear only when set.
    #[must_use]
    pub fn to_links(&self) -> Vec<Link> {
        let base = &self.id;
        let mut links = vec![
            Link::new(format!("lnk:{base}:type"), base.clone(), "Assertion"),
            Link::new(
                format!("lnk:{base}:subject"),
                base.clone(),
                self.subject.node_id(),
            ),
            Link::new(
                format!("lnk:{base}:predicate"),
                base.clone(),
                self.predicate_id().to_string(),
            ),
        ];
        for (index, object) in self.object.iter().enumerate() {
            links.push(Link::new(
                format!("lnk:{base}:object:{index}"),
                base.clone(),
                object.node_id(),
            ));
        }
        if let Some(time) = &self.time {
            links.push(Link::new(
                format!("lnk:{base}:time"),
                base.clone(),
                temporal_node(time),
            ));
        }
        if let Some(context) = &self.context {
            links.push(Link::new(
                format!("lnk:{base}:context"),
                base.clone(),
                context.id.clone(),
            ));
        }
        links.push(Link::new(
            format!("lnk:{base}:modal"),
            base.clone(),
            format!("modal:{}", self.modal.kind),
        ));
        links.push(Link::new(
            format!("lnk:{base}:confidence"),
            base.clone(),
            literal_node("number", &self.modal.confidence.to_string()),
        ));
        if let Some(provenance) = &self.provenance {
            links.push(Link::new(
                format!("lnk:{base}:provenance"),
                base.clone(),
                provenance_node(provenance),
            ));
        }
        links
    }
}

impl KnowledgeBase {
    /// Reduce the entire knowledge base to a flat, ordered set of links.
    ///
    /// Declarations are reduced first (concepts, entities, predicates,
    /// procedures, contexts, annotations), then the assertions, so the output is
    /// a single deterministic doublet stream that fully reconstructs the base.
    #[must_use]
    pub fn to_links(&self) -> Vec<Link> {
        let mut links = Vec::new();
        links.extend(self.concepts.iter().flat_map(concept_links));
        links.extend(self.entities.iter().flat_map(entity_links));
        links.extend(self.predicates.iter().flat_map(predicate_links));
        links.extend(self.procedures.iter().flat_map(procedure_links));
        links.extend(self.contexts.iter().flat_map(context_links));
        links.extend(self.source_annotations.iter().flat_map(annotation_links));
        links.extend(self.assertions.iter().flat_map(Assertion::to_links));
        links
    }

    /// Render the entire knowledge base as a Links-Notation doublet stream.
    #[must_use]
    pub fn to_links_lino(&self) -> String {
        let records: Vec<String> = self.to_links().iter().map(Link::to_lino).collect();
        records.join("\n")
    }

    /// The number of links the knowledge base reduces to.
    #[must_use]
    pub fn link_count(&self) -> usize {
        self.to_links().len()
    }
}
