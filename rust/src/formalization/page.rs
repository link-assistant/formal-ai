//! Page formalization (R1188-U18).
//!
//! A whole page of text, every sentence of it, becomes formal statements, and
//! the statements are deformalized back to text to see which facts survive.
//! Nothing is dropped silently: every sentence is reported with its statements
//! and the words no meaning was found for. It mirrors
//! `js/agentic/crate/page_formalization.mjs`, the JavaScript root.
//!
//! A sentence is covered when at least one of its statements has two or more
//! terms and no unknown term. A statement with a known term survives when its
//! deformalization, read again in the same language, names the same known
//! terms.

use super::statement_rendering::deformalize_statement;
use super::text_statements::{
    Statement, clause_statement, formalize_sentences, has_no_unknown, known_ids, statement_terms,
};

/// One sentence of a page, with its statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceReport {
    /// The sentence text.
    pub text: String,
    /// Its statements, in order.
    pub statements: Vec<Statement>,
    /// Whether a statement of two or more terms has no unknown term.
    pub covered: bool,
    /// The words of the sentence no meaning was found for.
    pub unknown: Vec<String>,
}

/// Every sentence of a page and the page's counts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageReport {
    /// Every sentence, in order.
    pub sentences: Vec<SentenceReport>,
    /// Sentences covered.
    pub covered: usize,
    /// Statements read.
    pub statements: usize,
    /// Statements with at least one known term.
    pub factual: usize,
    /// Factual statements whose known terms survive deformalization.
    pub survived: usize,
    /// Terms read.
    pub terms: usize,
    /// Terms that stayed unknown.
    pub unknown: usize,
}

/// Whether the known terms of a statement come back from its deformalization.
#[must_use]
pub fn fact_survives(statement: &Statement) -> bool {
    let known = known_ids(statement);
    if known.is_empty() {
        return false;
    }
    let text = deformalize_statement(statement, &statement.language);
    let reread = clause_statement(&text, &statement.language, None, false);
    known_ids(&reread) == known
}

/// Every sentence of `text` with its statements, and the page's counts.
#[must_use]
pub fn formalize_page(text: &str, language: &str) -> PageReport {
    let mut report = PageReport::default();
    for sentence in formalize_sentences(text, language) {
        let covered = sentence.statements.iter().any(has_no_unknown);
        let unknown = sentence
            .statements
            .iter()
            .flat_map(|statement| statement.unknown.iter().cloned())
            .collect();
        if covered {
            report.covered += 1;
        }
        for statement in &sentence.statements {
            report.statements += 1;
            report.terms += statement_terms(statement).len();
            report.unknown += statement.unknown.len();
            if known_ids(statement).is_empty() {
                continue;
            }
            report.factual += 1;
            if fact_survives(statement) {
                report.survived += 1;
            }
        }
        report.sentences.push(SentenceReport {
            text: sentence.text,
            statements: sentence.statements,
            covered,
            unknown,
        });
    }
    report
}
