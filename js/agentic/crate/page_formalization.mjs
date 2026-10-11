// Page formalization (R1188-U18): a whole page of text, every sentence of it,
// becomes formal statements, and the statements are deformalized back to
// text to see which facts survive. The Rust twin is
// rust/src/formalization/page.rs.
//
// Nothing is dropped silently: every sentence of the page is reported with
// its statements and the words no meaning was found for. Three measures:
//
// * sentence coverage: a sentence is covered when at least one of its
//   statements has two or more terms and no unknown term;
// * fact survival: a statement with a known term survives when its
//   deformalization, read again in the same language, names the same known
//   terms;
// * unknown terms: the terms that stayed unknown, against all terms.
//
// Representation: a page report is `{sentences, covered, statements,
// survived, terms, unknown}` (counts) with `sentences` a list of `{text,
// statements, covered, unknown}`.
//
// `pageAnswer` renders a report as the chat answer to "formalize <url>": the
// page's counts, then each statement with its formal notation.

import {
  Polarity, clauseStatement, deformalizeStatement, formalizeSentences, hasNoUnknown, knownIds, statementTerms,
} from './text_formalization.mjs';

/** Mirrors `ANSWER_STATEMENT_LIMIT` in rust/src/formalization/page.rs: the most statements a page answer lists. */
export const ANSWER_STATEMENT_LIMIT = 40;

/** Mirrors `fn fact_survives` in rust/src/formalization/page.rs: the known terms of a statement come back from its deformalization. */
export function factSurvives(statement) {
  const known = knownIds(statement);
  if (known.length === 0) return false;
  const reread = clauseStatement(deformalizeStatement(statement, statement.language), statement.language, null, false);
  const back = knownIds(reread);
  return back.length === known.length && back.every((id, index) => id === known[index]);
}

/**
 * Mirrors `fn formalize_page` in rust/src/formalization/page.rs: every sentence of `text` with its statements,
 * and the page's counts.
 * @param {string} text
 * @param {string} language
 */
export function formalizePage(text, language) {
  const report = { sentences: [], covered: 0, statements: 0, factual: 0, survived: 0, terms: 0, unknown: 0 };
  for (const sentence of formalizeSentences(text, language)) {
    const covered = sentence.statements.some(hasNoUnknown);
    const unknown = sentence.statements.flatMap((statement) => statement.unknown);
    report.sentences.push({ text: sentence.text, statements: sentence.statements, covered, unknown });
    if (covered) report.covered += 1;
    for (const statement of sentence.statements) {
      report.statements += 1;
      report.terms += statementTerms(statement).length;
      report.unknown += statement.unknown.length;
      if (knownIds(statement).length === 0) continue;
      report.factual += 1;
      if (factSurvives(statement)) report.survived += 1;
    }
  }
  return report;
}

/**
 * Mirrors `fn statement_notation` in rust/src/formalization/page.rs: a statement in formal notation, its term ids
 * (the subject first) in parentheses, after `¬` when the statement is denied.
 */
export function statementNotation(statement) {
  const identifiers = statementTerms(statement).map((term) => term.id);
  const negation = statement.polarity === Polarity.Denied ? '¬ ' : '';
  return `${negation}(${identifiers.join(' ')})`;
}

/**
 * Mirrors `fn page_answer` in rust/src/formalization/page.rs: the answer to a request to formalize the page at
 * `url`. `render(intent, values)` is the seeded response of `intent` in the answer's language with `values`
 * filled in (the Rust twin calls `crate::seed::render_response`).
 * @param {object} report a `formalizePage` report
 * @param {string} url
 * @param {(intent: string, values: object) => string} render
 */
export function pageAnswer(report, url, render) {
  if (report.statements === 0) return render('page_formalization_empty', { url });
  const lines = [render('page_formalization_summary', {
    url,
    sentences: report.sentences.length,
    statements: report.statements,
    covered: report.covered,
    terms: report.terms,
    unknown: report.unknown,
    factual: report.factual,
    survived: report.survived,
  })];
  lines.push('');
  const statements = report.sentences.flatMap((sentence) => sentence.statements);
  for (const statement of statements.slice(0, ANSWER_STATEMENT_LIMIT)) {
    lines.push(`- ${statement.text} → ${statementNotation(statement)}`);
  }
  if (report.statements > ANSWER_STATEMENT_LIMIT) {
    lines.push(render('page_formalization_more', { count: report.statements - ANSWER_STATEMENT_LIMIT }));
  }
  return lines.join('\n');
}
