//! Per-statement verification planning for the document-verification class.
//!
//! Issue #535 asks us to *"use our web search to check for each statement in the
//! text"* and to weigh those statements with
//! [`relative_meta_logic`](crate::relative_meta_logic): assume a statement true,
//! raise its probability with trusted original-first evidence, lower it with
//! contradicting evidence, and ignore reposts.
//!
//! This module turns a raw text sample into a deterministic, inspectable plan:
//! it splits the sample into statements across scripts, builds a grounding
//! web-search query for each, and produces an assumed-true
//! [`StatementAssessment`](crate::relative_meta_logic::StatementAssessment) plus
//! the trusted-source tier policy that governs how live evidence would move each
//! statement. The solver runs offline and deterministically, so no network call
//! is made here; instead the plan records exactly what would be checked and how
//! the resulting evidence would be weighed, which the handler replays into the
//! append-only event log.

use crate::relative_meta_logic::{
    RelativeEvidence, SourceTier, Stance, StatementAssessment, TruthValue, ASSUMED_TRUE_PRIOR,
};

/// Sentence terminators across the scripts the solver recognises: ASCII stops,
/// CJK full stop / exclamation / question, the Devanagari danda and double
/// danda, and the Arabic question mark.
const SENTENCE_TERMINATORS: &[char] = &['.', '!', '?', '。', '！', '？', '।', '॥', '؟', '।', '\n'];

/// Minimum number of words a fragment must contain to count as a checkable
/// statement. Below this it is treated as a heading or fragment and skipped.
const MIN_STATEMENT_WORDS: usize = 3;

/// Minimum number of non-whitespace characters an otherwise word-sparse
/// fragment must contain to count as a statement. This is the fallback gate for
/// scripts that do not separate words with spaces (Chinese, Japanese), where a
/// whole sentence is a single whitespace token.
const MIN_STATEMENT_CHARS: usize = 6;

/// The trusted-source tiers, in descending trust order, that govern how live
/// evidence for a statement would be weighed. Original first-party and original
/// journalism sources are trusted most; unoriginal reposts are ignored.
pub const TRUSTED_SOURCE_POLICY: &[SourceTier] = &[
    SourceTier::OriginalFirstParty,
    SourceTier::OriginalJournalism,
    SourceTier::IndependentCorroboration,
    SourceTier::Unoriginal,
];

/// A single checkable statement with its grounding query and assumed-true
/// assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementPlan {
    /// The statement text as extracted from the sample.
    pub statement: String,
    /// The web-search query that would ground this statement.
    pub query: String,
    /// The relative-meta-logic assessment given the evidence weighed so far.
    pub assessment: StatementAssessment,
}

impl StatementPlan {
    /// Build a plan for `statement`, weighing any already-collected `evidence`
    /// (empty in the deterministic offline path, non-empty when a caller has
    /// gathered grounding results).
    #[must_use]
    pub fn new(statement: impl Into<String>, evidence: &[RelativeEvidence]) -> Self {
        let statement = statement.into();
        let query = grounding_query(&statement);
        let assessment = StatementAssessment::assess(
            statement.clone(),
            TruthValue::new(ASSUMED_TRUE_PRIOR),
            evidence,
        );
        Self {
            statement,
            query,
            assessment,
        }
    }
}

/// A verification plan over every statement extracted from a text sample.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementVerificationPlan {
    /// One plan per extracted statement, in source order.
    pub statements: Vec<StatementPlan>,
}

impl StatementVerificationPlan {
    /// Extract statements from `sample` and plan grounding for each, with no
    /// evidence collected yet (the deterministic offline path).
    #[must_use]
    pub fn from_sample(sample: &str) -> Self {
        let statements = extract_statements(sample)
            .into_iter()
            .map(|statement| StatementPlan::new(statement, &[]))
            .collect();
        Self { statements }
    }

    /// Whether any statement was extracted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// The number of statements planned.
    #[must_use]
    pub fn len(&self) -> usize {
        self.statements.len()
    }
}

/// Split `sample` into checkable statements across scripts, trimming
/// whitespace and dropping fragments shorter than [`MIN_STATEMENT_WORDS`].
#[must_use]
pub fn extract_statements(sample: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    for character in sample.chars() {
        if SENTENCE_TERMINATORS.contains(&character) {
            push_statement(&mut statements, &current);
            current.clear();
        } else {
            current.push(character);
        }
    }
    push_statement(&mut statements, &current);
    statements
}

fn push_statement(statements: &mut Vec<String>, candidate: &str) {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return;
    }
    let word_count = trimmed.split_whitespace().count();
    let char_count = trimmed
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if word_count < MIN_STATEMENT_WORDS && char_count < MIN_STATEMENT_CHARS {
        return;
    }
    statements.push(trimmed.to_owned());
}

/// Build the web-search query that grounds `statement`: the quoted statement
/// paired with fact-check intent terms so the fusion layer surfaces original
/// first sources for or against it.
#[must_use]
pub fn grounding_query(statement: &str) -> String {
    let condensed = statement.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("\"{condensed}\" fact check source")
}

/// Whether an evidence stance would raise (`Supports`) or lower (`Contradicts`)
/// a statement's probability, exposed for callers that translate grounding
/// results into [`RelativeEvidence`].
#[must_use]
pub fn stance_for_agreement(agrees: bool) -> Stance {
    if agrees {
        Stance::Supports
    } else {
        Stance::Contradicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_multiscript_statements() {
        let sample = "The company launched in 2020. Компания запустилась в 2020 году. \
                      कंपनी 2020 में शुरू हुई। 公司在2020年成立。";
        let statements = extract_statements(sample);
        assert_eq!(statements.len(), 4);
        assert!(statements[0].starts_with("The company"));
        assert!(statements[3].contains("公司"));
    }

    #[test]
    fn drops_short_fragments() {
        let sample = "Yes. This is a real statement worth checking. OK.";
        let statements = extract_statements(sample);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].starts_with("This is a real"));
    }

    #[test]
    fn grounding_query_quotes_and_adds_intent() {
        let query = grounding_query("The tower is 300 metres tall");
        assert_eq!(query, "\"The tower is 300 metres tall\" fact check source");
    }

    #[test]
    fn grounding_query_condenses_whitespace() {
        let query = grounding_query("spaced   out\n\tclaim");
        assert_eq!(query, "\"spaced out claim\" fact check source");
    }

    #[test]
    fn plan_assumes_statements_true_before_evidence() {
        let plan = StatementVerificationPlan::from_sample(
            "The bridge opened in 1937. It spans the strait.",
        );
        assert_eq!(plan.len(), 2);
        for statement in &plan.statements {
            assert_eq!(
                statement.assessment.posterior,
                TruthValue::new(ASSUMED_TRUE_PRIOR),
            );
            assert!(statement.assessment.is_probable());
        }
    }

    #[test]
    fn plan_weighs_supplied_evidence() {
        let evidence = [RelativeEvidence::new(
            "gov.example",
            SourceTier::OriginalFirstParty,
            Stance::Contradicts,
            0.9,
        )];
        let plan = StatementPlan::new("A contested claim about policy", &evidence);
        assert!(plan.assessment.posterior.get() < ASSUMED_TRUE_PRIOR);
    }

    #[test]
    fn empty_sample_yields_no_statements() {
        assert!(StatementVerificationPlan::from_sample("   \n  ").is_empty());
    }

    #[test]
    fn trusted_source_policy_orders_original_first() {
        assert_eq!(TRUSTED_SOURCE_POLICY[0], SourceTier::OriginalFirstParty);
        assert_eq!(
            TRUSTED_SOURCE_POLICY.last().copied(),
            Some(SourceTier::Unoriginal),
        );
    }

    #[test]
    fn stance_for_agreement_maps_both_directions() {
        assert_eq!(stance_for_agreement(true), Stance::Supports);
        assert_eq!(stance_for_agreement(false), Stance::Contradicts);
    }
}
