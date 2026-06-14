//! Deterministic text → Links Notation knowledge base.
//!
//! The maintainer's framing for issue #468 is explicit: *"for us everything is a
//! link"*, and the formalization must use *"meta-language, that is already in our
//! code base"* — Links Notation — rather than the typed-struct
//! entities/ontologies a previous draft hand-coded. This module is that
//! formalizer. It takes a source document (the text an agentic step fetched) and
//! emits a knowledge base in which **all nine protocol primitives are links**:
//! concept, entity, predicate, assertion, procedure, context, temporal, modal,
//! annotation. No record is a bespoke Rust struct in the output — every record is
//! `id\n  key "value"` Links Notation produced by [`format_lino_record`].
//!
//! Extraction is deliberately shallow and honest (open-domain information
//! extraction needs neural inference, which is a documented NON-GOAL):
//!
//! * **Annotations** are produced for *every* sentence of *any* input, with real
//!   character offsets — fully general, never guessed.
//! * **Assertions** use a closed-class lexicon stored as data (see
//!   [`super::lexicon`]) — recognised subject/predicate/object triples become
//!   structured assertion links; unrecognised sentences become natural-language
//!   assertion links that still carry the raw span. The recogniser does not
//!   hallucinate relations it cannot ground.
//! * Concept / procedure / context catalogue records for a *recognised work* are
//!   declared from the lexicon and marked `source "lexicon:<work>"`, kept
//!   distinct from the text-derived assertions.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::lexicon::{Lexicon, PredicateUse, Procedure, Term, TermKind, Work, WorkContext};
use crate::links_format::format_lino_record;

/// A public-domain plot synopsis of «Сказка о рыбаке и рыбке».
///
/// These are plain facts in our own wording, **not** Pushkin's verse. This is the
/// canonical source text a fetch step returns, and the deterministic fallback the
/// planner formalizes when a live `web_fetch` fails. Keeping fetched-text ==
/// fallback-text makes the agentic loop produce a stable knowledge base either
/// way.
pub const CANONICAL_FISHERMAN_SYNOPSIS: &str =
    include_str!("../../data/agentic-coding/fisherman-synopsis.txt");

/// The default document id used when a caller does not supply one and the text is
/// recognised as the canonical tale.
pub const FISHERMAN_DOC_ID: &str = "tale:fisherman-and-fish";

/// The nine protocol primitives, as the record kinds the formalizer emits.
pub const PRIMITIVE_KINDS: [&str; 9] = [
    "concept",
    "entity",
    "predicate",
    "assertion",
    "procedure",
    "context",
    "temporal",
    "modal",
    "annotation",
];

/// Summary of a formalization, returned alongside the Links Notation document so
/// callers (the planner's final answer, tests) can report counts and coverage
/// without re-parsing the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalizationSummary {
    pub doc_id: String,
    pub concepts: usize,
    pub entities: usize,
    pub predicates: usize,
    pub assertions: usize,
    pub procedures: usize,
    pub contexts: usize,
    pub temporals: usize,
    pub modals: usize,
    pub annotations: usize,
    /// The distinct primitive kinds present, in [`PRIMITIVE_KINDS`] order.
    pub covered: Vec<String>,
}

impl FormalizationSummary {
    #[must_use]
    pub fn covers_all_nine(&self) -> bool {
        self.covered.len() == PRIMITIVE_KINDS.len()
    }

    #[must_use]
    pub const fn total_records(&self) -> usize {
        // +1 for the knowledge_base header record.
        1 + self.concepts
            + self.entities
            + self.predicates
            + self.assertions
            + self.procedures
            + self.contexts
            + self.temporals
            + self.modals
            + self.annotations
    }
}

/// A formalized knowledge base: the Links Notation document plus its summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalizedKnowledgeBase {
    pub links_notation: String,
    pub summary: FormalizationSummary,
}

/// Formalize `text` into a Links Notation knowledge base.
///
/// When `doc_id` is empty, a stable id is chosen: [`FISHERMAN_DOC_ID`] if the
/// text is recognised as the canonical tale, otherwise `doc:input`.
#[must_use]
pub fn formalize_text_to_links(text: &str, doc_id: &str) -> FormalizedKnowledgeBase {
    let lexicon = Lexicon::standard();
    let work = lexicon.best_work_for(text);
    let resolved_doc_id = resolve_doc_id(doc_id, work);

    let sentences = segment_sentences(text);
    let language = if has_cyrillic(text) { "ru" } else { "en" };

    let mut annotations: Vec<Annotation> = Vec::new();
    let mut assertions: Vec<Assertion> = Vec::new();
    let mut used_entities: BTreeMap<String, String> = BTreeMap::new();
    let mut used_predicates: BTreeMap<String, String> = BTreeMap::new();
    let mut used_concepts: BTreeMap<String, String> = BTreeMap::new();
    let mut temporals: BTreeMap<String, Temporal> = BTreeMap::new();
    let mut modals: BTreeMap<String, Modal> = BTreeMap::new();

    for (index, sentence) in sentences.iter().enumerate() {
        let annotation_id = format!("ann:{index}");
        annotations.push(Annotation {
            id: annotation_id.clone(),
            doc: resolved_doc_id.clone(),
            start: sentence.start,
            end: sentence.end,
            text: sentence.text.clone(),
            language: language.to_owned(),
        });

        let provenance = format!("{}@{}:{}", resolved_doc_id, sentence.start, sentence.end);
        let extracted = work.and_then(|work| work.extract(&sentence.text));
        match extracted {
            Some(triple) => {
                if matches!(triple.subject.kind, TermKind::Entity) {
                    used_entities.insert(triple.subject.id.clone(), triple.subject.label.clone());
                } else if matches!(triple.subject.kind, TermKind::Concept) {
                    used_concepts.insert(triple.subject.id.clone(), triple.subject.label.clone());
                }
                used_predicates.insert(triple.predicate.id.clone(), triple.predicate.label.clone());
                match triple.object.kind {
                    TermKind::Entity => {
                        used_entities.insert(triple.object.id.clone(), triple.object.label.clone());
                    }
                    TermKind::Concept => {
                        used_concepts.insert(triple.object.id.clone(), triple.object.label.clone());
                    }
                    TermKind::Literal => {}
                }

                let time_ref = triple.predicate.time.as_ref().map(|expression| {
                    let temporal = Temporal::from_expression(expression);
                    let id = temporal.id.clone();
                    temporals.insert(id.clone(), temporal);
                    id
                });
                let modal_ref = triple.predicate.modal.as_ref().map(|raw| {
                    let modal = Modal::from_raw(raw);
                    let id = modal.id.clone();
                    modals.insert(id.clone(), modal);
                    id
                });
                let context = if triple.predicate.id == "pred:remain" {
                    work.and_then(Work::final_context)
                } else {
                    work.and_then(Work::primary_context)
                };

                assertions.push(Assertion {
                    id: format!("a:{index}"),
                    subject: triple.subject,
                    predicate: triple.predicate.as_ref(),
                    object: triple.object,
                    time: time_ref,
                    modal: modal_ref,
                    context,
                    annotation: annotation_id,
                    provenance,
                    natural_language: None,
                });
            }
            None => assertions.push(Assertion {
                id: format!("a:{index}"),
                subject: Term::literal("—"),
                predicate: PredicateUse {
                    id: "pred:states".to_owned(),
                    label: "states".to_owned(),
                },
                object: Term::literal(&sentence.text),
                time: None,
                modal: None,
                context: None,
                annotation: annotation_id,
                provenance,
                natural_language: Some(sentence.text.clone()),
            }),
        }
    }

    // Thematic catalogue from the recognised work (lexicon-sourced declarations,
    // kept distinct from text-derived assertions via `source`).
    let mut concept_records: Vec<ConceptDecl> = Vec::new();
    let mut procedure_records: Vec<&Procedure> = Vec::new();
    let mut context_records: Vec<&WorkContext> = Vec::new();
    if let Some(work) = work {
        for concept in &work.concepts {
            concept_records.push(ConceptDecl {
                id: concept.id.clone(),
                label: concept.label.clone(),
                kind: concept.kind.clone(),
                source: format!("lexicon:{}", work.id),
            });
        }
        procedure_records.extend(work.procedures.iter());
        context_records.extend(work.contexts.iter());
    }
    // Concepts referenced by extracted assertions (text-derived).
    for (id, label) in &used_concepts {
        if concept_records.iter().any(|concept| &concept.id == id) {
            continue;
        }
        concept_records.push(ConceptDecl {
            id: id.clone(),
            label: label.clone(),
            kind: "extracted".to_owned(),
            source: resolved_doc_id.clone(),
        });
    }
    concept_records.sort_by(|left, right| left.id.cmp(&right.id));

    let summary = FormalizationSummary {
        doc_id: resolved_doc_id.clone(),
        concepts: concept_records.len(),
        entities: used_entities.len(),
        predicates: used_predicates.len(),
        assertions: assertions.len(),
        procedures: procedure_records.len(),
        contexts: context_records.len(),
        temporals: temporals.len(),
        modals: modals.len(),
        annotations: annotations.len(),
        covered: Vec::new(),
    };

    let mut document = String::new();
    let header = format_lino_record(
        "knowledge_base",
        &[
            ("id", resolved_doc_id.clone()),
            (
                "source",
                work.map_or_else(|| resolved_doc_id.clone(), |work| work.title.clone()),
            ),
            ("primitive_scheme", PRIMITIVE_KINDS.join(" ")),
            (
                "generator",
                "formal-ai/agentic-coding/formalize@links-v1".to_owned(),
            ),
            ("concepts", summary.concepts.to_string()),
            ("entities", summary.entities.to_string()),
            ("predicates", summary.predicates.to_string()),
            ("assertions", summary.assertions.to_string()),
            ("procedures", summary.procedures.to_string()),
            ("contexts", summary.contexts.to_string()),
            ("temporals", summary.temporals.to_string()),
            ("modals", summary.modals.to_string()),
            ("annotations", summary.annotations.to_string()),
        ],
    );
    push_record(&mut document, &header);

    for concept in &concept_records {
        push_record(
            &mut document,
            &format_lino_record(
                "concept",
                &[
                    ("id", concept.id.clone()),
                    ("label", concept.label.clone()),
                    ("type", concept.kind.clone()),
                    ("source", concept.source.clone()),
                ],
            ),
        );
    }
    for (id, label) in &used_entities {
        push_record(
            &mut document,
            &format_lino_record(
                "entity",
                &[
                    ("id", id.clone()),
                    ("label", label.clone()),
                    ("source", resolved_doc_id.clone()),
                ],
            ),
        );
    }
    for (id, label) in &used_predicates {
        push_record(
            &mut document,
            &format_lino_record(
                "predicate",
                &[
                    ("id", id.clone()),
                    ("label", label.clone()),
                    ("source", resolved_doc_id.clone()),
                ],
            ),
        );
    }
    for procedure in &procedure_records {
        push_record(
            &mut document,
            &format_lino_record(
                "procedure",
                &[
                    ("id", procedure.id.clone()),
                    ("signature", procedure.signature.clone()),
                    ("description", procedure.description.clone()),
                    ("trigger", procedure.trigger.clone()),
                    ("source", format!("lexicon:{resolved_doc_id}")),
                ],
            ),
        );
    }
    for context in &context_records {
        push_record(
            &mut document,
            &format_lino_record(
                "context",
                &[
                    ("id", context.id.clone()),
                    ("label", context.label.clone()),
                    ("description", context.description.clone()),
                ],
            ),
        );
    }
    for temporal in temporals.values() {
        push_record(
            &mut document,
            &format_lino_record(
                "temporal",
                &[
                    ("id", temporal.id.clone()),
                    ("expression", temporal.expression.clone()),
                    ("kind", temporal.kind.clone()),
                ],
            ),
        );
    }
    for modal in modals.values() {
        push_record(
            &mut document,
            &format_lino_record(
                "modal",
                &[
                    ("id", modal.id.clone()),
                    ("kind", modal.kind.clone()),
                    ("degree", modal.degree.clone()),
                ],
            ),
        );
    }
    for annotation in &annotations {
        push_record(
            &mut document,
            &format_lino_record(
                "annotation",
                &[
                    ("id", annotation.id.clone()),
                    ("doc", annotation.doc.clone()),
                    ("span", format!("{}:{}", annotation.start, annotation.end)),
                    ("text", annotation.text.clone()),
                    ("language", annotation.language.clone()),
                ],
            ),
        );
    }
    for assertion in &assertions {
        let mut pairs: Vec<(&str, String)> = vec![
            ("id", assertion.id.clone()),
            ("subject", assertion.subject.id.clone()),
            ("subject_kind", assertion.subject.kind.slug().to_owned()),
            ("predicate", assertion.predicate.id.clone()),
            ("object", assertion.object.id.clone()),
            ("object_kind", assertion.object.kind.slug().to_owned()),
        ];
        if let Some(time) = &assertion.time {
            pairs.push(("time", time.clone()));
        }
        if let Some(modal) = &assertion.modal {
            pairs.push(("modal", modal.clone()));
        }
        if let Some(context) = &assertion.context {
            pairs.push(("context", context.clone()));
        }
        if let Some(text) = &assertion.natural_language {
            pairs.push(("natural_language", text.clone()));
        }
        pairs.push(("annotation", assertion.annotation.clone()));
        pairs.push(("provenance", assertion.provenance.clone()));
        push_record(&mut document, &format_lino_record("assertion", &pairs));
    }

    let covered = PRIMITIVE_KINDS
        .iter()
        .filter(|kind| record_count(&summary, kind) > 0)
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let summary = FormalizationSummary { covered, ..summary };

    FormalizedKnowledgeBase {
        links_notation: document,
        summary,
    }
}

fn record_count(summary: &FormalizationSummary, kind: &str) -> usize {
    match kind {
        "concept" => summary.concepts,
        "entity" => summary.entities,
        "predicate" => summary.predicates,
        "assertion" => summary.assertions,
        "procedure" => summary.procedures,
        "context" => summary.contexts,
        "temporal" => summary.temporals,
        "modal" => summary.modals,
        "annotation" => summary.annotations,
        _ => 0,
    }
}

fn resolve_doc_id(requested: &str, work: Option<&Work>) -> String {
    let trimmed = requested.trim();
    if !trimmed.is_empty() {
        return trimmed.to_owned();
    }
    work.map_or_else(|| "doc:input".to_owned(), |work| work.doc_id.clone())
}

fn push_record(document: &mut String, record: &str) {
    if !document.is_empty() {
        document.push('\n');
    }
    document.push_str(record.trim_end());
    document.push('\n');
}

// ---------------------------------------------------------------------------
// Sentence segmentation (general, with character offsets).
// ---------------------------------------------------------------------------

struct Sentence {
    text: String,
    start: usize,
    end: usize,
}

fn segment_sentences(text: &str) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut buffer = String::new();
    for (char_index, character) in text.chars().enumerate() {
        buffer.push(character);
        if matches!(character, '.' | '!' | '?') {
            let trimmed = buffer.trim().to_owned();
            if !trimmed.is_empty() {
                sentences.push(Sentence {
                    text: trimmed,
                    start,
                    end: char_index + 1,
                });
            }
            start = char_index + 1;
            buffer.clear();
        }
    }
    let trimmed = buffer.trim().to_owned();
    if !trimmed.is_empty() {
        let end = text.chars().count();
        sentences.push(Sentence {
            text: trimmed,
            start,
            end,
        });
    }
    sentences
}

fn has_cyrillic(text: &str) -> bool {
    text.chars()
        .any(|character| ('\u{0400}'..='\u{04FF}').contains(&character))
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for character in text.trim().chars() {
        if character.is_alphanumeric() {
            for lowered in character.to_lowercase() {
                slug.push(lowered);
            }
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_owned()
}

// ---------------------------------------------------------------------------
// Output term/record models.
// ---------------------------------------------------------------------------

struct Assertion {
    id: String,
    subject: Term,
    predicate: PredicateUse,
    object: Term,
    time: Option<String>,
    modal: Option<String>,
    context: Option<String>,
    annotation: String,
    provenance: String,
    natural_language: Option<String>,
}

struct Annotation {
    id: String,
    doc: String,
    start: usize,
    end: usize,
    text: String,
    language: String,
}

struct ConceptDecl {
    id: String,
    label: String,
    kind: String,
    source: String,
}

struct Temporal {
    id: String,
    expression: String,
    kind: String,
}

impl Temporal {
    fn from_expression(expression: &str) -> Self {
        Self {
            id: format!("temporal:{}", slugify(expression)),
            expression: expression.to_owned(),
            kind: "relative".to_owned(),
        }
    }
}

struct Modal {
    id: String,
    kind: String,
    degree: String,
}

impl Modal {
    fn from_raw(raw: &str) -> Self {
        let (kind, degree) = raw.split_once(':').unwrap_or((raw, ""));
        Self {
            id: format!("modal:{}", slugify(kind)),
            kind: kind.to_owned(),
            degree: degree.to_owned(),
        }
    }
}

/// Render a one-line, human-readable trace of which primitives were covered.
#[must_use]
pub fn coverage_line(summary: &FormalizationSummary) -> String {
    let mut line = String::new();
    for kind in &summary.covered {
        if !line.is_empty() {
            let _ = write!(line, ", ");
        }
        let _ = write!(line, "{kind}");
    }
    line
}
