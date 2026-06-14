//! The nine knowledge primitives of the text-formalization protocol.
//!
//! These types are a faithful, `serde`-serializable encoding of the protocol
//! described in issue #468 (Igor Martynov, *Formal protocol for translating
//! texts into a knowledge base*). The authoritative wire format is the compact
//! JSON shown in the article; every type here round-trips that JSON. See
//! `docs/case-studies/issue-468/README.md` for the full mapping and
//! `docs/case-studies/issue-468/formal-protocol-mapping.md` for how each
//! primitive reduces to plain links/doublets.
//!
//! The nine primitives are: [`Concept`], [`Entity`], [`Predicate`],
//! [`Assertion`], [`Procedure`], [`Context`], [`Temporal`], [`Modal`] and
//! [`Annotation`]. An [`Assertion`] is the atomic block of knowledge; every
//! other primitive supports it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A `Concept`: an abstract unit of meaning that need not denote a concrete
/// object (e.g. *greed*, *wish*, *ransom*).
///
/// Fields follow the protocol: `id`, `label`, `type` and `attributes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Concept {
    /// Stable identifier, conventionally namespaced (e.g. `concept:wish`).
    pub id: String,
    /// Human-readable label in the source language.
    pub label: String,
    /// The kind of concept (the protocol's `type` field).
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub concept_type: String,
    /// Free-form attributes as ordered key/value pairs.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl Concept {
    /// Build a concept from an id, a label and a concept type.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        concept_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            concept_type: concept_type.into(),
            attributes: BTreeMap::new(),
        }
    }

    /// Attach an attribute, returning `self` for chaining.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// An `Entity`: a concrete object or referent (e.g. *the old man*, *the sea*).
///
/// Fields follow the protocol: `id`, `label`, `canonical_forms` and
/// `attributes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// Stable identifier, conventionally namespaced (e.g. `ent:old_man`).
    pub id: String,
    /// Primary human-readable label in the source language.
    pub label: String,
    /// Alternative surface forms that refer to the same entity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_forms: Vec<String>,
    /// Free-form attributes as ordered key/value pairs.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl Entity {
    /// Build an entity from an id and a label.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            canonical_forms: Vec::new(),
            attributes: BTreeMap::new(),
        }
    }

    /// Record an alternative surface form, returning `self` for chaining.
    #[must_use]
    pub fn with_form(mut self, form: impl Into<String>) -> Self {
        self.canonical_forms.push(form.into());
        self
    }

    /// Attach an attribute, returning `self` for chaining.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// A `Predicate` (relation): an operation or relation between terms.
///
/// Fields follow the protocol: `id`, `name`, `arity` and `semantics`. This is
/// the reference declaration of a relation; an [`Assertion`] refers to it by id
/// through a [`PredicateRef`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Predicate {
    /// Stable identifier, conventionally namespaced (e.g. `pred:catch`).
    pub id: String,
    /// Surface name in the source language (e.g. `поймал`).
    pub name: String,
    /// Number of arguments the relation takes.
    #[serde(default = "default_arity")]
    pub arity: u8,
    /// Optional formal semantics (a formula or type description).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub semantics: String,
}

const fn default_arity() -> u8 {
    2
}

impl Predicate {
    /// Build a binary predicate from an id and a name.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arity: 2,
            semantics: String::new(),
        }
    }

    /// Set the arity, returning `self` for chaining.
    #[must_use]
    pub const fn with_arity(mut self, arity: u8) -> Self {
        self.arity = arity;
        self
    }

    /// Set the formal semantics, returning `self` for chaining.
    #[must_use]
    pub fn with_semantics(mut self, semantics: impl Into<String>) -> Self {
        self.semantics = semantics.into();
        self
    }
}

/// A `Procedure`: a transformation, template or inference rule expressed as a
/// function.
///
/// Fields follow the protocol: `id`, `signature`, `body` and `triggers`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Procedure {
    /// Stable identifier, conventionally namespaced (e.g. `proc:escalate`).
    pub id: String,
    /// Call signature of the procedure.
    pub signature: String,
    /// Human-readable description of the procedure body.
    pub body: String,
    /// Identifiers of predicates or events that trigger the procedure.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
}

impl Procedure {
    /// Build a procedure from an id, a signature and a body description.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        signature: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            signature: signature.into(),
            body: body.into(),
            triggers: Vec::new(),
        }
    }

    /// Register a trigger, returning `self` for chaining.
    #[must_use]
    pub fn with_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.triggers.push(trigger.into());
        self
    }
}

/// A `Context`: the situation or bounds of validity of an assertion (e.g.
/// *within RF law*, *the interval 1990-2000*, *a location*).
///
/// The same type serves both as a reference catalogue entry (with `label` and
/// `description`) and as the per-assertion binding shown in the article
/// (`{"id": "ctx:loc", "properties": {"location": "Москва"}}`); unused fields
/// are omitted from the JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Context {
    /// Stable identifier, conventionally namespaced (e.g. `ctx:loc`).
    pub id: String,
    /// Optional human-readable label.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Optional human-readable description of the validity bounds.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Ordered key/value properties (e.g. `location`, `jurisdiction`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
}

impl Context {
    /// Build a context binding from an id.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            description: String::new(),
            properties: BTreeMap::new(),
        }
    }

    /// Set the human-readable label, returning `self` for chaining.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the description, returning `self` for chaining.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a property, returning `self` for chaining.
    #[must_use]
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Look up a property value by key.
    #[must_use]
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }

    /// Convenience accessor for the conventional `location` property.
    #[must_use]
    pub fn location(&self) -> Option<&str> {
        self.property("location")
    }
}

/// A `Temporal` value: unified time as an instant, an interval or a relative
/// expression.
///
/// Serialized with a `type` tag, matching the article's
/// `{"type": "Instant", "value": "2019-00-00", "granularity": "year"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Temporal {
    /// A single point in time.
    Instant {
        /// The instant value, conventionally an ISO-like string.
        value: String,
        /// Optional granularity (e.g. `year`, `day`).
        #[serde(default, skip_serializing_if = "String::is_empty")]
        granularity: String,
    },
    /// A closed interval between two points.
    Interval {
        /// Interval start.
        start: String,
        /// Interval end.
        end: String,
    },
    /// A relative expression (e.g. `after the first wish`).
    Relative {
        /// The relative description.
        value: String,
    },
}

impl Temporal {
    /// Build a year-granularity instant from a calendar year.
    #[must_use]
    pub fn year(year: i32) -> Self {
        Self::Instant {
            value: format!("{year:04}-00-00"),
            granularity: String::from("year"),
        }
    }

    /// Build a relative temporal expression.
    #[must_use]
    pub fn relative(value: impl Into<String>) -> Self {
        Self::Relative {
            value: value.into(),
        }
    }

    /// Extract a calendar year, if one is determinable from the value.
    ///
    /// Parses the leading run of ASCII digits of an [`Temporal::Instant`] or the
    /// start of an [`Temporal::Interval`]. Returns `None` for relative times or
    /// values without a leading year.
    #[must_use]
    pub fn calendar_year(&self) -> Option<i32> {
        let raw = match self {
            Self::Instant { value, .. } | Self::Interval { start: value, .. } => value,
            Self::Relative { .. } => return None,
        };
        let digits: String = raw.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    }
}

/// A `Modal` value: the modality of an assertion plus its confidence.
///
/// Matches the article's `{"type": "assertion", "confidence": 0.95}`. The
/// protocol's `confidence` field rides inside the modal record; the
/// [`Assertion::confidence`] accessor exposes it as the BNF's `CONF` slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Modal {
    /// Modality kind: `assertion`, `belief`, `obligation`, `possibility`, ...
    #[serde(rename = "type")]
    pub kind: String,
    /// Confidence in `[0, 1]`.
    pub confidence: f64,
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            kind: String::from("assertion"),
            confidence: 1.0,
        }
    }
}

impl Modal {
    /// Build a plain assertion modality with the given confidence.
    #[must_use]
    pub fn assertion(confidence: f64) -> Self {
        Self {
            kind: String::from("assertion"),
            confidence,
        }
    }

    /// Build a modality of an arbitrary kind with the given confidence.
    #[must_use]
    pub fn new(kind: impl Into<String>, confidence: f64) -> Self {
        Self {
            kind: kind.into(),
            confidence,
        }
    }
}

/// Provenance: where an assertion came from in the source material.
///
/// Matches the article's
/// `{"source_doc": "doc-0001", "offsets": [0, 37], "extractor": "nlp_v1"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// Identifier of the source document.
    pub source_doc: String,
    /// Optional `[start, end]` character offsets into the source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offsets: Option<[usize; 2]>,
    /// Identifier of the extractor that produced the assertion.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub extractor: String,
    /// Optional reference to an [`Annotation`] capturing the exact span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotation: Option<String>,
}

impl Provenance {
    /// Build provenance that only names the source document.
    #[must_use]
    pub fn source(source_doc: impl Into<String>) -> Self {
        Self {
            source_doc: source_doc.into(),
            offsets: None,
            extractor: String::new(),
            annotation: None,
        }
    }

    /// Set the character offsets, returning `self` for chaining.
    #[must_use]
    pub const fn with_offsets(mut self, start: usize, end: usize) -> Self {
        self.offsets = Some([start, end]);
        self
    }

    /// Set the extractor identifier, returning `self` for chaining.
    #[must_use]
    pub fn with_extractor(mut self, extractor: impl Into<String>) -> Self {
        self.extractor = extractor.into();
        self
    }
}

/// An `Annotation`: a link to a span of source text with offsets, language and
/// tokenization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    /// Stable identifier, conventionally namespaced (e.g. `ann:s1`).
    pub id: String,
    /// Identifier of the source document.
    pub source_doc: String,
    /// `[start, end]` character offsets into the source.
    pub offsets: [usize; 2],
    /// BCP-47-like language tag of the span.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub language: String,
    /// Tokenization of the span.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokenization: Vec<String>,
}

impl Annotation {
    /// Build an annotation for a source span.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        source_doc: impl Into<String>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            id: id.into(),
            source_doc: source_doc.into(),
            offsets: [start, end],
            language: String::new(),
            tokenization: Vec::new(),
        }
    }

    /// Set the language tag, returning `self` for chaining.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set the tokenization, returning `self` for chaining.
    #[must_use]
    pub fn with_tokens<I, S>(mut self, tokens: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tokenization = tokens.into_iter().map(Into::into).collect();
        self
    }
}

/// A `Term`: an argument position in an assertion.
///
/// A term is an [`Entity`] reference, a [`Concept`] reference, a typed literal,
/// or a reference to another [`Assertion`] (the protocol's nested,
/// higher-order assertions). The assertion-reference variant is what makes
/// assertions first-class: an assertion may be the subject or object of another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Term {
    /// Reference to a concrete entity.
    Entity {
        /// Entity identifier.
        id: String,
        /// Optional inline label for readability.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        label: String,
    },
    /// Reference to an abstract concept.
    Concept {
        /// Concept identifier.
        id: String,
        /// Optional inline label for readability.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        label: String,
    },
    /// A typed literal value.
    Literal {
        /// The literal's datatype (e.g. `xsd:string`, `number`).
        datatype: String,
        /// The literal's value rendered as text.
        value: String,
    },
    /// Reference to another assertion (a nested, higher-order assertion).
    #[serde(rename = "AssertionRef")]
    AssertionRef {
        /// Identifier of the referenced assertion.
        id: String,
    },
}

impl Term {
    /// Build an entity term.
    #[must_use]
    pub fn entity(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Entity {
            id: id.into(),
            label: label.into(),
        }
    }

    /// Build a concept term.
    #[must_use]
    pub fn concept(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Concept {
            id: id.into(),
            label: label.into(),
        }
    }

    /// Build a typed literal term.
    #[must_use]
    pub fn literal(datatype: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Literal {
            datatype: datatype.into(),
            value: value.into(),
        }
    }

    /// Build a reference to another assertion.
    #[must_use]
    pub fn assertion_ref(id: impl Into<String>) -> Self {
        Self::AssertionRef { id: id.into() }
    }

    /// The referenced identifier, if the term is a reference (not a literal).
    #[must_use]
    pub fn reference_id(&self) -> Option<&str> {
        match self {
            Self::Entity { id, .. } | Self::Concept { id, .. } | Self::AssertionRef { id } => {
                Some(id.as_str())
            }
            Self::Literal { .. } => None,
        }
    }

    /// A stable node identifier for the term in the links reduction.
    ///
    /// References use their id; literals use a `lit:<datatype>:<value>` node so
    /// equal literals collapse to the same node.
    #[must_use]
    pub fn node_id(&self) -> String {
        match self {
            Self::Entity { id, .. } | Self::Concept { id, .. } | Self::AssertionRef { id } => {
                id.clone()
            }
            Self::Literal { datatype, value } => format!("lit:{datatype}:{value}"),
        }
    }

    /// The short type tag used in the wire format and links typing.
    #[must_use]
    pub const fn type_tag(&self) -> &'static str {
        match self {
            Self::Entity { .. } => "Entity",
            Self::Concept { .. } => "Concept",
            Self::Literal { .. } => "Literal",
            Self::AssertionRef { .. } => "AssertionRef",
        }
    }
}

/// A `PredicateRef`: an in-assertion reference to a [`Predicate`] by id and
/// name.
///
/// Serialized as a single-variant tagged enum so the wire form is exactly the
/// article's `{"type": "Predicate", "id": "pred:open", "name": "открыл"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PredicateRef {
    /// The predicate reference.
    Predicate {
        /// Predicate identifier.
        id: String,
        /// Optional inline name for readability.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
    },
}

impl PredicateRef {
    /// Build a predicate reference from an id and a name.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Predicate {
            id: id.into(),
            name: name.into(),
        }
    }

    /// The referenced predicate identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        let Self::Predicate { id, .. } = self;
        id.as_str()
    }

    /// The inline predicate name.
    #[must_use]
    pub fn name(&self) -> &str {
        let Self::Predicate { name, .. } = self;
        name.as_str()
    }
}

/// The `type` tag carried by every [`Assertion`] in the wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AssertionTag {
    /// The only value; serializes to the string `"Assertion"`.
    #[default]
    Assertion,
}

/// An `Assertion`: the atomic block of knowledge — a statement that a subject
/// stands in a predicate relation to one or more objects, with optional time,
/// context, modality, confidence and provenance.
///
/// This is the unit over which the protocol performs search, inference and
/// aggregation. It mirrors the article's JSON exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    /// Stable identifier (e.g. `a1`).
    pub id: String,
    /// The constant `type` tag (`"Assertion"`).
    #[serde(rename = "type", default)]
    pub kind: AssertionTag,
    /// The subject term.
    pub subject: Term,
    /// The predicate relation.
    pub predicate: PredicateRef,
    /// The object terms (one or more).
    #[serde(default)]
    pub object: Vec<Term>,
    /// Optional temporal qualifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<Temporal>,
    /// Optional context binding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Modality and confidence (always present).
    #[serde(default)]
    pub modal: Modal,
    /// Optional provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Assertion {
    /// Build a minimal assertion (subject, predicate, single object).
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        subject: Term,
        predicate: PredicateRef,
        object: Term,
    ) -> Self {
        Self {
            id: id.into(),
            kind: AssertionTag::Assertion,
            subject,
            predicate,
            object: vec![object],
            time: None,
            context: None,
            modal: Modal::default(),
            provenance: None,
        }
    }

    /// Replace the object list, returning `self` for chaining.
    #[must_use]
    pub fn with_objects(mut self, objects: Vec<Term>) -> Self {
        self.object = objects;
        self
    }

    /// Set the temporal qualifier, returning `self` for chaining.
    #[must_use]
    pub fn with_time(mut self, time: Temporal) -> Self {
        self.time = Some(time);
        self
    }

    /// Set the context binding, returning `self` for chaining.
    #[must_use]
    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    /// Set the modality/confidence, returning `self` for chaining.
    #[must_use]
    pub fn with_modal(mut self, modal: Modal) -> Self {
        self.modal = modal;
        self
    }

    /// Set the confidence (keeping the current modality kind), returning `self`.
    #[must_use]
    pub const fn with_confidence(mut self, confidence: f64) -> Self {
        self.modal.confidence = confidence;
        self
    }

    /// Set the provenance, returning `self` for chaining.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// The confidence carried by the modality (the BNF's `CONF` slot).
    #[must_use]
    pub const fn confidence(&self) -> f64 {
        self.modal.confidence
    }

    /// The subject's referenced identifier, if it is a reference.
    #[must_use]
    pub fn subject_id(&self) -> Option<&str> {
        self.subject.reference_id()
    }

    /// The predicate identifier.
    #[must_use]
    pub fn predicate_id(&self) -> &str {
        self.predicate.id()
    }
}
