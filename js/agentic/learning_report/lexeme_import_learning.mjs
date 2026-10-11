// The identity of the auto-learning report for bulk lexeme import (issue #660)
// (rust/src/agentic_coding/learning_report/lexeme_import_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from '../host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from '../learning_report.mjs';
import { agenticMessage } from '../messages.mjs';

/** Mirrors `LEXEME_IMPORT_LEARNING_PATH` in rust/src/agentic_coding/learning_report/lexeme_import_learning.rs. */
export const LEXEME_IMPORT_LEARNING_PATH = 'lexeme-import-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/lexeme_import_learning.rs. */
export const REPORT = Object.freeze({
  head: 'lexeme_import_learning_report',
  issue: '660',
  promotion_gate: 'bulk_lexeme_import_integrity_and_dual_agent_cli_e2e_pass',
  path: LEXEME_IMPORT_LEARNING_PATH,
  task: LEXEME_IMPORT_LEARNING_PATH,
  get memory() {
    return readText('data/meta/issue-660-lexeme-import-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_lexeme_import_learning');
  },
});

/** Mirrors `LEXEME_IMPORT_LEARNING_TASK` in rust/src/agentic_coding/learning_report/lexeme_import_learning.rs (the artifact path itself). */
export const LEXEME_IMPORT_LEARNING_TASK = LEXEME_IMPORT_LEARNING_PATH;

/** Mirrors `fn is_lexeme_import_learning_task` in rust/src/agentic_coding/learning_report/lexeme_import_learning.rs. @param {string} prompt */
export function isLexemeImportLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/lexeme_import_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/lexeme_import_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/lexeme_import_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
