//! Structured Links-Notation serialization of a [`KnowledgeBase`].
//!
//! This is the readable, record-per-primitive view (one indented record for each
//! concept, entity, predicate, procedure, context, annotation and assertion),
//! mirroring how the rest of the crate emits Links Notation in
//! [`crate::engine`]. The fully reduced doublet view lives in
//! [`super::links`]; the interoperable wire format is the JSON in
//! [`super::knowledge_base`].

use super::knowledge_base::KnowledgeBase;
use super::primitives::{Assertion, Context, Provenance, Temporal, Term};
use crate::links_format::format_lino_record;

/// Format a record, dropping pairs whose value is empty so optional fields do
/// not render as noisy `key ""` lines.
fn record(id: &str, pairs: &[(&str, String)]) -> String {
    let kept: Vec<(&str, String)> = pairs
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .cloned()
        .collect();
    format_lino_record(id, &kept)
}

/// Human-readable rendering of a temporal value.
fn temporal_display(temporal: &Temporal) -> String {
    match temporal {
        Temporal::Instant { value, granularity } => {
            if granularity.is_empty() {
                value.clone()
            } else {
                format!("{value} ({granularity})")
            }
        }
        Temporal::Interval { start, end } => format!("{start}..{end}"),
        Temporal::Relative { value } => format!("relative: {value}"),
    }
}

/// Human-readable rendering of a context binding.
fn context_display(context: &Context) -> String {
    let mut parts = vec![context.id.clone()];
    for (key, value) in &context.properties {
        parts.push(format!("{key}={value}"));
    }
    parts.join(" ")
}

/// Human-readable rendering of a provenance record.
fn provenance_display(provenance: &Provenance) -> String {
    let mut parts = vec![provenance.source_doc.clone()];
    if let Some([start, end]) = provenance.offsets {
        parts.push(format!("[{start},{end}]"));
    }
    if !provenance.extractor.is_empty() {
        parts.push(provenance.extractor.clone());
    }
    parts.join(" ")
}

/// Comma-joined node identifiers of a term list.
fn term_list(terms: &[Term]) -> String {
    terms
        .iter()
        .map(Term::node_id)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render a single assertion as a Links-Notation record.
fn assertion_record(assertion: &Assertion) -> String {
    let mut pairs: Vec<(&str, String)> = vec![
        ("kind", String::from("Assertion")),
        ("subject", assertion.subject.node_id()),
        ("predicate", assertion.predicate_id().to_string()),
        ("object", term_list(&assertion.object)),
    ];
    if let Some(time) = &assertion.time {
        pairs.push(("time", temporal_display(time)));
    }
    if let Some(context) = &assertion.context {
        pairs.push(("context", context_display(context)));
    }
    pairs.push(("modality", assertion.modal.kind.clone()));
    pairs.push(("confidence", assertion.modal.confidence.to_string()));
    if let Some(provenance) = &assertion.provenance {
        pairs.push(("provenance", provenance_display(provenance)));
    }
    record(&assertion.id, &pairs)
}

impl KnowledgeBase {
    /// Render the knowledge base as a structured Links-Notation document.
    ///
    /// Emits a header record with the document id and primitive counts, followed
    /// by one record per declaration and one per assertion.
    #[must_use]
    pub fn to_lino(&self) -> String {
        let coverage = self.coverage();
        let mut records = vec![record(
            "formal_ai_text_formalization",
            &[
                ("doc_id", self.doc_id.clone()),
                (
                    "policy",
                    String::from("deterministic reduction; no neural network inference"),
                ),
                ("concepts", coverage.concepts.to_string()),
                ("entities", coverage.entities.to_string()),
                ("predicates", coverage.predicates.to_string()),
                ("procedures", coverage.procedures.to_string()),
                ("contexts", coverage.contexts.to_string()),
                ("annotations", coverage.annotations.to_string()),
                ("assertions", coverage.assertions.to_string()),
            ],
        )];

        for concept in &self.concepts {
            records.push(record(
                &concept.id,
                &[
                    ("kind", String::from("Concept")),
                    ("label", concept.label.clone()),
                    ("type", concept.concept_type.clone()),
                ],
            ));
        }
        for entity in &self.entities {
            records.push(record(
                &entity.id,
                &[
                    ("kind", String::from("Entity")),
                    ("label", entity.label.clone()),
                    ("canonical_forms", entity.canonical_forms.join(", ")),
                ],
            ));
        }
        for predicate in &self.predicates {
            records.push(record(
                &predicate.id,
                &[
                    ("kind", String::from("Predicate")),
                    ("name", predicate.name.clone()),
                    ("arity", predicate.arity.to_string()),
                ],
            ));
        }
        for procedure in &self.procedures {
            records.push(record(
                &procedure.id,
                &[
                    ("kind", String::from("Procedure")),
                    ("signature", procedure.signature.clone()),
                    ("body", procedure.body.clone()),
                    ("triggers", procedure.triggers.join(", ")),
                ],
            ));
        }
        for context in &self.contexts {
            records.push(record(
                &context.id,
                &[
                    ("kind", String::from("Context")),
                    ("label", context.label.clone()),
                    ("description", context.description.clone()),
                ],
            ));
        }
        for annotation in &self.source_annotations {
            let [start, end] = annotation.offsets;
            records.push(record(
                &annotation.id,
                &[
                    ("kind", String::from("Annotation")),
                    ("source_doc", annotation.source_doc.clone()),
                    ("offsets", format!("[{start},{end}]")),
                    ("language", annotation.language.clone()),
                ],
            ));
        }
        records.extend(self.assertions.iter().map(assertion_record));

        records.join("\n")
    }
}
