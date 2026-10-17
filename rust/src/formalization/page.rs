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
//!
//! [`page_answer`] renders a report as the chat answer to `formalize <url>`:
//! the page's counts, then each statement with its formal notation.

use super::dialogue::source_term_count;
use super::statement_rendering::deformalize_statement;
use super::text_statements::{
    Polarity, Statement, clause_statement, formalize_sentences, has_no_unknown, known_ids,
    statement_terms,
};

/// The most statements a page answer lists; the rest are counted.
pub const ANSWER_STATEMENT_LIMIT: usize = 40;

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
            report.terms += source_term_count(statement);
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

/// A statement in formal notation: its term ids, the subject first, in
/// parentheses, after `¬` when the statement is denied.
#[must_use]
pub fn statement_notation(statement: &Statement) -> String {
    let ids: Vec<&str> = statement_terms(statement)
        .into_iter()
        .map(|term| term.id.as_str())
        .collect();
    let negation = if statement.polarity == Polarity::Denied {
        "¬ "
    } else {
        ""
    };
    format!("{negation}({})", ids.join(" "))
}

/// The answer to a request to formalize the page at `url`, in `language`:
///
/// The seeded summary of the report's counts, then one line per statement
/// (its text and its notation), up to [`ANSWER_STATEMENT_LIMIT`], then the
/// count of the rest. A page with no statement is answered with the seeded
/// empty-page response.
#[must_use]
pub fn page_answer(report: &PageReport, url: &str, language: &str) -> String {
    let render = |intent: &str, values: &[(&str, &str)]| {
        crate::seed::render_response(intent, language, values).unwrap_or_default()
    };
    if report.statements == 0 {
        return render("page_formalization_empty", &[("url", url)]);
    }
    let counts = [
        report.sentences.len(),
        report.statements,
        report.covered,
        report.terms,
        report.unknown,
        report.factual,
        report.survived,
    ]
    .map(|count| count.to_string());
    let mut lines = vec![render(
        "page_formalization_summary",
        &[
            ("url", url),
            ("sentences", &counts[0]),
            ("statements", &counts[1]),
            ("covered", &counts[2]),
            ("terms", &counts[3]),
            ("unknown", &counts[4]),
            ("factual", &counts[5]),
            ("survived", &counts[6]),
        ],
    )];
    lines.push(String::new());
    let statements = report
        .sentences
        .iter()
        .flat_map(|sentence| sentence.statements.iter());
    for statement in statements.take(ANSWER_STATEMENT_LIMIT) {
        lines.push(format!(
            "- {} → {}",
            statement.text,
            statement_notation(statement)
        ));
    }
    if report.statements > ANSWER_STATEMENT_LIMIT {
        let rest = (report.statements - ANSWER_STATEMENT_LIMIT).to_string();
        lines.push(render("page_formalization_more", &[("count", &rest)]));
    }
    lines.join("\n")
}

#[cfg(test)]
mod numeric_source_evidence_tests {
    use super::*;
    use crate::formalization::text_statements::clause_statement;
    #[test]
    fn canonical_numbers_and_independent_boundaries_round_trip() {
        for (text, language) in [
            ("695,000千米", "zh"),
            ("3,827⋅1026", "ru"),
            ("225—250", "ru"),
            ("1：397", "zh"),
            ("१४,९६,००,००० किलोमीटर", "hi"),
        ] {
            let statement = clause_statement(text, language, None, false);
            assert!(fact_survives(&statement), "{text}");
        }
    }
    #[test]
    fn logical_grouping_retains_source_word_counts_unknowns_and_byte_spans() {
        for (text, value, components) in [
            ("987 654 blarg", "987654", 2),
            ("987654 blarg", "987654", 1),
            ("9,007,199,254,740,993 blarg", "9007199254740993", 1),
        ] {
            let statement = clause_statement(text, "en", None, false);
            assert_eq!(statement_terms(&statement).len(), 2);
            assert_eq!(source_term_count(&statement), components + 1);
            assert_eq!(statement.unknown, ["blarg"]);
            let quantity = &statement.dialogue.quantities[0];
            assert_eq!(quantity.value, value);
            assert_eq!(quantity.source_components.len(), components);
            for component in &quantity.source_components {
                assert_eq!(&text[component.start..component.end], component.surface);
            }
        }
    }
}

#[cfg(test)]
mod locale_numeric_meaning_tests {
    use crate::formalization::dialogue::dialogue_quantities;
    #[test]
    fn locale_decimal_grouping_and_unsupported_values_are_not_invented() {
        for (surface, language, expected) in [
            ("3,827", "ru", "3.827"),
            ("450 000", "ru", "450000"),
            ("9,007,199,254,740,993", "en", "9007199254740993"),
            ("12,34,567.89", "hi", "1234567.89"),
            ("12.345,67", "es", "12345.67"),
            ("12,345.67", "zh", "12345.67"),
        ] {
            let values = dialogue_quantities(surface, language);
            assert_eq!(values.len(), 1, "{surface}");
            assert_eq!(values[0].value, expected, "{surface}");
            assert!(values[0].uncertainty.is_none());
        }
        for (surface, language) in [("1,23", "en"), ("1,234,567", "hi"), ("3.827", "ru")] {
            let values = dialogue_quantities(surface, language);
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].value, surface);
            assert_eq!(
                values[0].uncertainty.as_deref(),
                Some("unsupported-locale-number-format")
            );
        }
    }
}

#[cfg(test)]
mod translated_numeric_meaning_tests {
    use crate::formalization::text_statements::{clause_statement, statement_terms};
    use crate::translation::round_trip::render_term;
    #[test]
    fn target_locale_rendering_preserves_numeric_value() {
        for (surface, source, target, expected) in [
            ("3,827", "ru", "en", "3.827"),
            ("1.25", "en", "ru", "1,25"),
            ("12.345,67", "es", "en", "12345.67"),
            ("9,007,199,254,740,993", "en", "ru", "9007199254740993"),
        ] {
            let statement = clause_statement(surface, source, None, false);
            let terms = statement_terms(&statement);
            assert_eq!(render_term(terms[0], source, target).target, expected);
            let translated = clause_statement(expected, target, None, false);
            assert_eq!(statement_terms(&translated)[0].id, terms[0].id);
        }
    }
}
