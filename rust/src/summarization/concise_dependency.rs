//! R1188-U21: verified shorter rendering of the unchanged dependency core.
//! Equivalence is bounded to the maintained formal model; unmodeled semantics are unknown.
use super::dependency::{CoreSummary, summarize_by_dependency};
use crate::formalization::statement_rendering::deformalize_statement;
use crate::formalization::text_statements::{
    Polarity, Statement, Word, WordKind, formalize_text, read_words,
};
use crate::normal_markov::{quote_fault, quoted_segment_spans};

/// An ordered typed fact with its exact source surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactTerm {
    pub kind: WordKind,
    pub id: String,
    pub surface: String,
}
/// The complete modeled statement contract used by the renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestatementFacts {
    pub language: String,
    pub polarity: Polarity,
    pub subject: Option<FactTerm>,
    pub terms: Vec<FactTerm>,
    pub unknown: Vec<String>,
}
/// Why original wording was retained, or a shorter rendering was admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestatementReason {
    QuotedOrMalformedSource,
    MultipleNegationScopes,
    NotShorter,
    FormalFactsNotPreserved,
    VerifiedShorterFormalRestatement,
}
/// A source-preserving proposal, not a claim of unmodeled semantic equivalence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Restatement {
    pub text: String,
    pub original_text: String,
    pub original_facts: RestatementFacts,
    pub rewritten: bool,
    pub reason: RestatementReason,
}
/// One retained graph node and its rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreRestatementItem {
    pub index: usize,
    pub depends_on: Vec<usize>,
    pub dependents: Vec<usize>,
    pub restatement: Restatement,
    pub rendered: String,
}
/// Additive rendering; the complete original summary remains unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreRestatement {
    pub source_summary: CoreSummary,
    pub restatements: Vec<CoreRestatementItem>,
    pub restated_text: String,
    pub core_restatement_verified: bool,
}
fn fact_term(word: &Word) -> FactTerm {
    FactTerm {
        kind: word.kind,
        id: word.id.clone(),
        surface: word.surface.clone(),
    }
}
#[must_use]
pub fn restatement_facts(statement: &Statement) -> RestatementFacts {
    RestatementFacts {
        language: statement.language.clone(),
        polarity: statement.polarity,
        subject: statement.subject.as_ref().map(fact_term),
        terms: statement.terms.iter().map(fact_term).collect(),
        unknown: statement.unknown.clone(),
    }
}
#[must_use]
pub fn verified_restatement(statement: &Statement) -> Restatement {
    let original_facts = restatement_facts(statement);
    let retained = |reason| Restatement {
        text: statement.text.clone(),
        original_text: statement.text.clone(),
        original_facts: original_facts.clone(),
        rewritten: false,
        reason,
    };
    if quote_fault(&statement.text).is_some() || !quoted_segment_spans(&statement.text).is_empty() {
        return retained(RestatementReason::QuotedOrMalformedSource);
    }
    if read_words(&statement.text, &statement.language, true)
        .iter()
        .filter(|word| word.kind == WordKind::Negation)
        .count()
        > 1
    {
        return retained(RestatementReason::MultipleNegationScopes);
    }
    let candidate = deformalize_statement(statement, &statement.language);
    if candidate.chars().count() >= statement.text.chars().count() {
        return retained(RestatementReason::NotShorter);
    }
    let recovered = formalize_text(&candidate, &statement.language);
    if recovered.len() != 1 || restatement_facts(&recovered[0]) != original_facts {
        return retained(RestatementReason::FormalFactsNotPreserved);
    }
    Restatement {
        text: candidate,
        original_text: statement.text.clone(),
        original_facts,
        rewritten: true,
        reason: RestatementReason::VerifiedShorterFormalRestatement,
    }
}
#[must_use]
pub fn summarize_with_restatement(text: &str, language: &str) -> CoreRestatement {
    let summary = summarize_by_dependency(text, language);
    let restatements = summary
        .kept
        .iter()
        .map(|&index| {
            let node = &summary.statements[index];
            let restatement = verified_restatement(&node.entry.statement);
            let closing = &node.entry.closing;
            let rendered = if !closing.is_empty() && !restatement.text.ends_with(closing) {
                format!("{}{closing}", restatement.text)
            } else {
                restatement.text.clone()
            };
            CoreRestatementItem {
                index,
                depends_on: node.depends_on.clone(),
                dependents: node.dependents.clone(),
                restatement,
                rendered,
            }
        })
        .collect::<Vec<_>>();
    let generated = restatements
        .iter()
        .map(|item| item.rendered.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let recovered = formalize_text(&generated, language)
        .iter()
        .map(restatement_facts)
        .collect::<Vec<_>>();
    let expected = summary
        .kept
        .iter()
        .map(|&index| restatement_facts(&summary.statements[index].entry.statement))
        .collect::<Vec<_>>();
    let core_restatement_verified = recovered == expected;
    let restated_text = if core_restatement_verified {
        generated
    } else {
        summary.text.clone()
    };
    CoreRestatement {
        source_summary: summary,
        restatements,
        restated_text,
        core_restatement_verified,
    }
}
