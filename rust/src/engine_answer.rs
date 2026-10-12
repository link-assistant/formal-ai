//! The answer value the symbolic engine returns, and the execution recipe a
//! code answer hands to the requesting agentic client.
//!
//! Split out of `engine.rs` so the engine module stays under the per-file
//! line ceiling; every item is re-exported from [`crate::engine`], so the
//! public paths are unchanged.

use serde::{Deserialize, Serialize};

use crate::thinking::ThinkingStep;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SymbolicAnswer {
    pub intent: String,
    pub answer: String,
    pub confidence: f32,
    pub evidence_links: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thinking_steps: Vec<ThinkingStep>,
    pub links_notation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_recipe: Option<Box<ExecutionRecipe>>,
}

/// A code artifact whose side effects belong to the requesting agentic client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRecipe {
    pub language: String,
    pub source: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_files: Vec<ExecutionRecipeFile>,
    pub commands: Vec<String>,
}

/// An additional file required by a typed execution recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRecipeFile {
    pub path: String,
    pub source: String,
}

// Compute the identifier when serializing rather than storing a redundant field
// that could become stale when a handler edits the answer text.
impl Serialize for SymbolicAnswer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct WireAnswer<'a> {
            intent: &'a str,
            answer: &'a str,
            confidence: f32,
            evidence_links: &'a [String],
            #[serde(skip_serializing_if = "slice_is_empty")]
            thinking_steps: &'a [ThinkingStep],
            links_notation: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            execution_recipe: Option<&'a ExecutionRecipe>,
            derivation_id: String,
        }
        const fn slice_is_empty<T>(value: &&[T]) -> bool {
            value.is_empty()
        }
        WireAnswer {
            intent: &self.intent,
            answer: &self.answer,
            confidence: self.confidence,
            evidence_links: &self.evidence_links,
            thinking_steps: &self.thinking_steps,
            links_notation: &self.links_notation,
            execution_recipe: self.execution_recipe.as_deref(),
            derivation_id: self.derivation_id(),
        }
        .serialize(serializer)
    }
}

impl SymbolicAnswer {
    /// Content address of the answer currently carried by this value.
    #[must_use]
    pub fn derivation_id(&self) -> String {
        crate::derivation::answer_derivation_id(&self.answer)
    }

    /// Whether the answer never reached a conclusion about the prompt.
    ///
    /// The unknown-prompt fallback, an ill-formed prompt, a punctuation-only
    /// prompt and every clarification request answer something *about* the
    /// prompt rather than the prompt itself. A caller that has to act on the
    /// text -- replaying it in another language
    /// (`solver_handlers::response_language_followup`), recording it as
    /// evidence at a path the caller named
    /// ([`crate::agentic_coding`]) -- must be able to tell the two apart, and
    /// both callers have to agree on where the line is, so the test lives with
    /// the type that carries the intent rather than in either caller.
    #[must_use]
    pub fn is_inconclusive(&self) -> bool {
        matches!(
            self.intent.as_str(),
            "unknown" | "ill_formed" | "punctuation_only_prompt" | "concept_lookup_unresolved"
        ) || self.asks_for_clarification()
    }

    /// Whether the answer is a clarifying question. An ask is not an unknown:
    /// the engine knows exactly what is missing and says so, so a caller
    /// admitting "unresolved" requests must exclude it. [`Self::is_inconclusive`]
    /// includes it and states why; this half is the boundary callers such as
    /// the research continuation gate subtract.
    #[must_use]
    pub fn asks_for_clarification(&self) -> bool {
        self.intent.starts_with("clarify")
    }

    /// Whether the answer points at the open web instead of stating a finding.
    ///
    /// The `web_search` intent renders what the browser demo *would* query and
    /// how it would rank the results. That is a plan for a lookup nobody has
    /// performed yet, so the text is about the search rather than about the
    /// subject: nothing in it is true of `IIR` or of `Sunday` in particular.
    ///
    /// `agentic_coding::web_research` already reads the intent this
    /// way when it decides an open-world question is unresolved. A caller that
    /// has to *deliver* an answer -- writing it to a path the request named --
    /// needs the same reading for the opposite reason: recording a description
    /// of a pending search as the evidence a run produced is the hollow proof
    /// issue #1066 exists to stop.
    #[must_use]
    pub fn defers_to_the_open_web(&self) -> bool {
        self.intent == "web_search"
    }

    /// Whether the answer ends on a promise of a list it never makes.
    ///
    /// A reply that closes with the colon introducing an enumeration and stops
    /// there is a heading with nothing under it. The handler that composes such
    /// a reply is the one that knows *why* it has nothing to enumerate and says
    /// so ([`crate::task_decomposition::Decomposition::unenumerable_reason`]);
    /// this is the backstop underneath it, for the callers that deliver an
    /// answer somewhere a reader will later find it. Delivering a heading with
    /// no list is the hollow evidence issue #1066 exists to stop -- it passes
    /// every mechanical check a harness makes, because a file that says
    /// nothing is still a non-empty file.
    ///
    /// The full-width colon is here because Chinese and Japanese introduce a
    /// list with it, and a guard that only reads ASCII would hold for four of
    /// the supported languages and not the fifth.
    #[must_use]
    pub fn announces_a_list_it_does_not_make(&self) -> bool {
        self.answer.trim_end().ends_with([':', '\u{ff1a}'])
    }
}
