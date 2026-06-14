//! A constrained, fully deterministic extractor.
//!
//! formal-ai performs **no neural-network inference**. General open-domain
//! text-to-knowledge extraction (the article's §7 pipeline: POS tagging,
//! dependency parsing, semantic role labeling, named-entity recognition and
//! coreference resolution) is a learned-model problem and is therefore out of
//! scope; the case study scopes it as future work and surveys the prior art in
//! `docs/case-studies/issue-468/raw-data/online-research.md`.
//!
//! What *is* deterministic — and what this module implements — is a closed-class
//! rule extractor for one explicit sentence template, the article's worked
//! example «Пётр открыл магазин в Москве в 2019 году.» and sentences of exactly
//! that shape over a fixed [`Lexicon`]. Any sentence outside the template or the
//! lexicon yields [`None`]: the extractor never guesses.
//!
//! Template:
//!
//! ```text
//! <Subject> <Predicate> <Object> в <Location> в <Year> [году]
//! ```

use std::collections::BTreeMap;

use super::knowledge_base::KnowledgeBase;
use super::primitives::{
    Annotation, Assertion, Context, Entity, Predicate, PredicateRef, Provenance, Temporal, Term,
};

/// Identifier this extractor records in provenance.
pub const EXTRACTOR_ID: &str = "deterministic_v1";

/// A closed vocabulary mapping surface forms to primitive identifiers.
///
/// The extractor only recognizes words present here; this is what keeps it
/// deterministic and honest about its (deliberately narrow) coverage.
#[derive(Debug, Clone)]
pub struct Lexicon {
    /// Subject surface form -> (entity id, canonical label).
    subjects: BTreeMap<String, (String, String)>,
    /// Predicate surface form -> (predicate id, name).
    predicates: BTreeMap<String, (String, String)>,
    /// Object surface form -> (entity id, canonical label).
    objects: BTreeMap<String, (String, String)>,
    /// Location surface form (often locative case) -> canonical place name.
    locations: BTreeMap<String, String>,
}

impl Default for Lexicon {
    fn default() -> Self {
        let mut subjects = BTreeMap::new();
        subjects.insert(
            String::from("Пётр"),
            (String::from("ent:petrov_petr"), String::from("Пётр")),
        );
        subjects.insert(
            String::from("Петр"),
            (String::from("ent:petrov_petr"), String::from("Пётр")),
        );

        let mut predicates = BTreeMap::new();
        predicates.insert(
            String::from("открыл"),
            (String::from("pred:open"), String::from("открыл")),
        );

        let mut objects = BTreeMap::new();
        objects.insert(
            String::from("магазин"),
            (String::from("ent:shop_001"), String::from("магазин")),
        );

        let mut locations = BTreeMap::new();
        locations.insert(String::from("Москве"), String::from("Москва"));
        locations.insert(String::from("Москва"), String::from("Москва"));

        Self {
            subjects,
            predicates,
            objects,
            locations,
        }
    }
}

impl Lexicon {
    /// Build an empty lexicon.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            subjects: BTreeMap::new(),
            predicates: BTreeMap::new(),
            objects: BTreeMap::new(),
            locations: BTreeMap::new(),
        }
    }

    /// Register a subject surface form.
    pub fn add_subject(
        &mut self,
        surface: impl Into<String>,
        id: impl Into<String>,
        label: impl Into<String>,
    ) -> &mut Self {
        self.subjects
            .insert(surface.into(), (id.into(), label.into()));
        self
    }

    /// Register a predicate surface form.
    pub fn add_predicate(
        &mut self,
        surface: impl Into<String>,
        id: impl Into<String>,
        name: impl Into<String>,
    ) -> &mut Self {
        self.predicates
            .insert(surface.into(), (id.into(), name.into()));
        self
    }

    /// Register an object surface form.
    pub fn add_object(
        &mut self,
        surface: impl Into<String>,
        id: impl Into<String>,
        label: impl Into<String>,
    ) -> &mut Self {
        self.objects
            .insert(surface.into(), (id.into(), label.into()));
        self
    }

    /// Register a location surface form and its canonical place name.
    pub fn add_location(
        &mut self,
        surface: impl Into<String>,
        canonical: impl Into<String>,
    ) -> &mut Self {
        self.locations.insert(surface.into(), canonical.into());
        self
    }
}

/// The result of a successful extraction: an assertion, the source annotation it
/// is grounded in, and the supporting declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct Extraction {
    /// The extracted assertion.
    pub assertion: Assertion,
    /// The source-text annotation grounding the assertion.
    pub annotation: Annotation,
    /// The supporting entity declarations (subject and object).
    pub entities: Vec<Entity>,
    /// The supporting predicate declaration.
    pub predicate: Predicate,
}

impl Extraction {
    /// Assemble a self-contained knowledge base from this extraction.
    #[must_use]
    pub fn into_knowledge_base(self, doc_id: impl Into<String>) -> KnowledgeBase {
        let mut kb = KnowledgeBase::new(doc_id);
        for entity in self.entities {
            kb.push_entity(entity);
        }
        kb.push_predicate(self.predicate);
        kb.push_annotation(self.annotation);
        kb.push_assertion(self.assertion);
        kb
    }
}

/// The deterministic, closed-class extractor.
#[derive(Debug, Clone, Default)]
pub struct Extractor {
    lexicon: Lexicon,
}

impl Extractor {
    /// Build an extractor with the default worked-example lexicon.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an extractor over a custom lexicon.
    #[must_use]
    pub const fn with_lexicon(lexicon: Lexicon) -> Self {
        Self { lexicon }
    }

    /// Attempt to extract an assertion from a single sentence.
    ///
    /// Returns [`None`] when the sentence does not match the template or uses a
    /// word outside the lexicon — the extractor never guesses.
    #[must_use]
    pub fn extract(&self, doc_id: &str, sentence: &str) -> Option<Extraction> {
        let tokens: Vec<&str> = sentence
            .split_whitespace()
            .map(|token| token.trim_matches(|character: char| !character.is_alphanumeric()))
            .filter(|token| !token.is_empty())
            .collect();

        // Template: SUBJ PRED OBJ "в" LOC "в" YEAR [году]
        if tokens.len() < 7 {
            return None;
        }
        if tokens[3] != "в" || tokens[5] != "в" {
            return None;
        }

        let (subject_id, subject_label) = self.lexicon.subjects.get(tokens[0])?.clone();
        let (predicate_id, predicate_name) = self.lexicon.predicates.get(tokens[1])?.clone();
        let (object_id, object_label) = self.lexicon.objects.get(tokens[2])?.clone();
        let location = self.lexicon.locations.get(tokens[4])?.clone();
        let year: i32 = tokens[6].parse().ok()?;

        let span_len = sentence.chars().count();

        let assertion = Assertion::new(
            "a1",
            Term::entity(subject_id.clone(), subject_label.clone()),
            PredicateRef::new(predicate_id.clone(), predicate_name.clone()),
            Term::entity(object_id.clone(), object_label.clone()),
        )
        .with_time(Temporal::year(year))
        .with_context(Context::new("ctx:loc").with_property("location", location))
        .with_confidence(1.0)
        .with_provenance(
            Provenance::source(doc_id)
                .with_offsets(0, span_len)
                .with_extractor(EXTRACTOR_ID),
        );

        let annotation = Annotation::new("ann:s1", doc_id, 0, span_len)
            .with_language("ru")
            .with_tokens(tokens.iter().copied());

        let entities = vec![
            Entity::new(subject_id, subject_label),
            Entity::new(object_id, object_label),
        ];
        let predicate = Predicate::new(predicate_id, predicate_name);

        Some(Extraction {
            assertion,
            annotation,
            entities,
            predicate,
        })
    }

    /// Extract a sentence directly into a self-contained knowledge base.
    #[must_use]
    pub fn extract_kb(&self, doc_id: &str, sentence: &str) -> Option<KnowledgeBase> {
        self.extract(doc_id, sentence)
            .map(|extraction| extraction.into_knowledge_base(doc_id))
    }
}
