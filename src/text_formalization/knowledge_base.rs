//! The [`KnowledgeBase`] aggregate and its JSON wire document.
//!
//! A [`KnowledgeBase`] is the in-memory model that holds all nine primitives.
//! [`ProtocolDocument`] is its canonical JSON serialization — exactly the
//! compact format from the protocol article: a `doc_id`, an optional reference
//! `directory`, and an `annotations` array of [`Assertion`] records.

use serde::{Deserialize, Serialize};

use super::primitives::{Annotation, Assertion, Concept, Context, Entity, Predicate, Procedure};

/// The reference catalogue (the "ontology" part of the protocol): declarations
/// that carry no facts of their own.
///
/// Empty by default and omitted entirely from the JSON when empty, so the
/// article's `{"doc_id": ..., "annotations": [...]}` round-trips byte-for-byte
/// in structure.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Directory {
    /// Abstract concepts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concepts: Vec<Concept>,
    /// Concrete entities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<Entity>,
    /// Relation declarations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<Predicate>,
    /// Procedures / inference rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedures: Vec<Procedure>,
    /// Context declarations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<Context>,
    /// Source-text annotations (the ninth primitive).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_annotations: Vec<Annotation>,
}

impl Directory {
    /// Whether the catalogue holds no declarations at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
            && self.entities.is_empty()
            && self.predicates.is_empty()
            && self.procedures.is_empty()
            && self.contexts.is_empty()
            && self.source_annotations.is_empty()
    }
}

/// The canonical JSON document: `{doc_id, directory?, annotations}`.
///
/// This is the protocol's wire format. [`ProtocolDocument::from_json`] and
/// [`ProtocolDocument::to_json_pretty`] parse and emit it; the round-trip is
/// exercised against the article's own example in the unit tests.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProtocolDocument {
    /// Identifier of the document the assertions were extracted from.
    pub doc_id: String,
    /// Reference catalogue; omitted from the JSON when empty.
    #[serde(default, skip_serializing_if = "Directory::is_empty")]
    pub directory: Directory,
    /// The assertions extracted from the document.
    #[serde(default)]
    pub annotations: Vec<Assertion>,
}

impl ProtocolDocument {
    /// Parse a document from the protocol's JSON wire format.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Emit the document as compact JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ProtocolDocument should always serialize")
    }

    /// Emit the document as pretty-printed JSON.
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("ProtocolDocument should always serialize")
    }
}

/// Counts of how many of each primitive a knowledge base realizes.
///
/// Used to confirm a knowledge base genuinely exercises all nine primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveCoverage {
    /// Number of concepts.
    pub concepts: usize,
    /// Number of entities.
    pub entities: usize,
    /// Number of predicate declarations.
    pub predicates: usize,
    /// Number of assertions.
    pub assertions: usize,
    /// Number of procedures.
    pub procedures: usize,
    /// Number of distinct contexts (catalogue plus per-assertion bindings).
    pub contexts: usize,
    /// Number of temporal qualifiers carried by assertions.
    pub temporals: usize,
    /// Number of modality records (one per assertion).
    pub modals: usize,
    /// Number of source-text annotations.
    pub annotations: usize,
}

impl PrimitiveCoverage {
    /// Whether at least one instance of every one of the nine primitives is
    /// present.
    #[must_use]
    pub const fn covers_all_nine(&self) -> bool {
        self.concepts > 0
            && self.entities > 0
            && self.predicates > 0
            && self.assertions > 0
            && self.procedures > 0
            && self.contexts > 0
            && self.temporals > 0
            && self.modals > 0
            && self.annotations > 0
    }
}

/// The in-memory aggregate of all nine primitives for one document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KnowledgeBase {
    /// Identifier of the source document.
    pub doc_id: String,
    /// Abstract concepts.
    pub concepts: Vec<Concept>,
    /// Concrete entities.
    pub entities: Vec<Entity>,
    /// Relation declarations.
    pub predicates: Vec<Predicate>,
    /// Procedures / inference rules.
    pub procedures: Vec<Procedure>,
    /// Context declarations.
    pub contexts: Vec<Context>,
    /// Source-text annotations.
    pub source_annotations: Vec<Annotation>,
    /// The assertions — the atomic blocks of knowledge.
    pub assertions: Vec<Assertion>,
}

impl KnowledgeBase {
    /// Build an empty knowledge base for the given document id.
    #[must_use]
    pub fn new(doc_id: impl Into<String>) -> Self {
        Self {
            doc_id: doc_id.into(),
            ..Self::default()
        }
    }

    /// Add a concept.
    pub fn push_concept(&mut self, concept: Concept) -> &mut Self {
        self.concepts.push(concept);
        self
    }

    /// Add an entity.
    pub fn push_entity(&mut self, entity: Entity) -> &mut Self {
        self.entities.push(entity);
        self
    }

    /// Add a predicate declaration.
    pub fn push_predicate(&mut self, predicate: Predicate) -> &mut Self {
        self.predicates.push(predicate);
        self
    }

    /// Add a procedure.
    pub fn push_procedure(&mut self, procedure: Procedure) -> &mut Self {
        self.procedures.push(procedure);
        self
    }

    /// Add a context declaration.
    pub fn push_context(&mut self, context: Context) -> &mut Self {
        self.contexts.push(context);
        self
    }

    /// Add a source-text annotation.
    pub fn push_annotation(&mut self, annotation: Annotation) -> &mut Self {
        self.source_annotations.push(annotation);
        self
    }

    /// Add an assertion.
    pub fn push_assertion(&mut self, assertion: Assertion) -> &mut Self {
        self.assertions.push(assertion);
        self
    }

    /// Convert into the canonical JSON document.
    #[must_use]
    pub fn to_document(&self) -> ProtocolDocument {
        ProtocolDocument {
            doc_id: self.doc_id.clone(),
            directory: Directory {
                concepts: self.concepts.clone(),
                entities: self.entities.clone(),
                predicates: self.predicates.clone(),
                procedures: self.procedures.clone(),
                contexts: self.contexts.clone(),
                source_annotations: self.source_annotations.clone(),
            },
            annotations: self.assertions.clone(),
        }
    }

    /// Build a knowledge base from a parsed JSON document.
    #[must_use]
    pub fn from_document(document: ProtocolDocument) -> Self {
        Self {
            doc_id: document.doc_id,
            concepts: document.directory.concepts,
            entities: document.directory.entities,
            predicates: document.directory.predicates,
            procedures: document.directory.procedures,
            contexts: document.directory.contexts,
            source_annotations: document.directory.source_annotations,
            assertions: document.annotations,
        }
    }

    /// Parse a knowledge base from the protocol's JSON wire format.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        ProtocolDocument::from_json(json).map(Self::from_document)
    }

    /// Emit the knowledge base as compact protocol JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        self.to_document().to_json()
    }

    /// Emit the knowledge base as pretty-printed protocol JSON.
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        self.to_document().to_json_pretty()
    }

    /// Look up an entity declaration by id.
    #[must_use]
    pub fn entity(&self, id: &str) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }

    /// Look up a predicate declaration by id.
    #[must_use]
    pub fn predicate(&self, id: &str) -> Option<&Predicate> {
        self.predicates.iter().find(|predicate| predicate.id == id)
    }

    /// Look up an assertion by id.
    #[must_use]
    pub fn assertion(&self, id: &str) -> Option<&Assertion> {
        self.assertions.iter().find(|assertion| assertion.id == id)
    }

    /// Count how many of each primitive this knowledge base realizes.
    #[must_use]
    pub fn coverage(&self) -> PrimitiveCoverage {
        let temporals = self
            .assertions
            .iter()
            .filter(|assertion| assertion.time.is_some())
            .count();
        let binding_contexts = self
            .assertions
            .iter()
            .filter(|assertion| assertion.context.is_some())
            .count();
        PrimitiveCoverage {
            concepts: self.concepts.len(),
            entities: self.entities.len(),
            predicates: self.predicates.len(),
            assertions: self.assertions.len(),
            procedures: self.procedures.len(),
            contexts: self.contexts.len() + binding_contexts,
            temporals,
            modals: self.assertions.len(),
            annotations: self.source_annotations.len(),
        }
    }
}
