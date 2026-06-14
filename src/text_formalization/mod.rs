//! Deterministic text-to-knowledge formalization (issue #468).
//!
//! This module implements Igor Martynov's *Formal protocol for translating texts
//! into a knowledge base* — the nine primitives (Concept, Entity, Predicate,
//! Assertion, Procedure, Context, Temporal, Modal, Annotation) — "as is", and
//! demonstrates that every one of them reduces to plain links/doublets, honoring
//! the project's standing position that *everything is a link*.
//!
//! ## What is deterministic here
//!
//! - The nine [primitive types](primitives) and their canonical JSON wire format
//!   ([`ProtocolDocument`]), which round-trips the article's own example exactly.
//! - A native, readable Links-Notation serialization ([`KnowledgeBase::to_lino`])
//!   and the fully reduced doublet stream ([`KnowledgeBase::to_links`]).
//! - A declarative conjunctive [`Query`] language over assertions (article §9).
//! - A curated knowledge base for «Сказка о рыбаке и рыбке» ([`tale`]) that
//!   exercises all nine primitives.
//! - A constrained, closed-class [`Extractor`] for the article's worked-example
//!   sentence template.
//!
//! ## What is intentionally out of scope
//!
//! General open-domain natural-language extraction (POS tagging, dependency
//! parsing, semantic role labeling, NER, coreference) is a learned-model problem.
//! formal-ai performs no neural-network inference, so that capability is scoped
//! as future work; see `docs/case-studies/issue-468/README.md`.

pub mod extract;
pub mod knowledge_base;
pub mod links;
pub mod lino;
pub mod primitives;
pub mod query;
pub mod tale;

pub use extract::{Extraction, Extractor, Lexicon, EXTRACTOR_ID};
pub use knowledge_base::{Directory, KnowledgeBase, PrimitiveCoverage, ProtocolDocument};
pub use links::Link;
pub use primitives::{
    Annotation, Assertion, AssertionTag, Concept, Context, Entity, Modal, Predicate, PredicateRef,
    Procedure, Provenance, Temporal, Term,
};
pub use query::{Query, QueryError};
pub use tale::{tale_knowledge_base, TALE_DOC_ID, TALE_GLOSS};

use std::fmt;
use std::str::FromStr;

/// Output serialization for a [`KnowledgeBase`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KbFormat {
    /// Structured, readable Links Notation (one record per primitive).
    #[default]
    Lino,
    /// The protocol's canonical pretty-printed JSON wire format.
    Json,
    /// The fully reduced doublet (link) stream.
    Links,
}

impl KbFormat {
    /// Render a knowledge base in this format.
    #[must_use]
    pub fn render(self, kb: &KnowledgeBase) -> String {
        match self {
            Self::Lino => kb.to_lino(),
            Self::Json => kb.to_json_pretty(),
            Self::Links => kb.to_links_lino(),
        }
    }

    /// The canonical lowercase name of the format.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lino => "lino",
            Self::Json => "json",
            Self::Links => "links",
        }
    }
}

impl fmt::Display for KbFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for KbFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            "lino" | "links-notation" => Ok(Self::Lino),
            "json" => Ok(Self::Json),
            "links" | "doublets" => Ok(Self::Links),
            other => Err(format!("unknown knowledge-base format: {other}")),
        }
    }
}

/// Render the curated tale knowledge base in the requested format.
#[must_use]
pub fn formalize_tale(format: KbFormat) -> String {
    format.render(&tale_knowledge_base())
}

/// Run the constrained extractor over a sentence and render the result.
///
/// Returns [`None`] when the sentence falls outside the extractor's template or
/// lexicon (the extractor never guesses).
#[must_use]
pub fn formalize_sentence(doc_id: &str, sentence: &str, format: KbFormat) -> Option<String> {
    Extractor::new()
        .extract_kb(doc_id, sentence)
        .map(|kb| format.render(&kb))
}
