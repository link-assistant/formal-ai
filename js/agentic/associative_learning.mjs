// The identity of the auto-learning report for issue #686's associative auto-learning loop
// (rust/src/agentic_coding/associative_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from './host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from './learning_report.mjs';
import { agenticMessage } from './messages.mjs';

/** Mirrors `ASSOCIATIVE_LEARNING_PATH` in rust/src/agentic_coding/associative_learning.rs. */
export const ASSOCIATIVE_LEARNING_PATH = 'associative-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/associative_learning.rs. */
export const REPORT = Object.freeze({
  head: 'associative_learning_report',
  issue: '686',
  promotion_gate: null,
  path: ASSOCIATIVE_LEARNING_PATH,
  get task() {
    return agenticMessage('learning_report_task_associative_learning');
  },
  get memory() {
    return readText('data/meta/associative-learning-case.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_associative_learning');
  },
});

/** Mirrors `ASSOCIATIVE_LEARNING_TASK` in rust/src/agentic_coding/associative_learning.rs. */
export function task() {
  return REPORT.task;
}

/** Mirrors `fn is_associative_learning_task` in rust/src/agentic_coding/associative_learning.rs. @param {string} prompt */
export function isAssociativeLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/associative_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/associative_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/associative_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
