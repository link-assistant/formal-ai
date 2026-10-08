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

import {
  clauseStatement, deformalizeStatement, formalizeSentences, hasNoUnknown, knownIds, statementTerms,
} from './text_formalization.mjs';

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
