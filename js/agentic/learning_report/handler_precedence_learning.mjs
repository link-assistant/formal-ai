// The identity of the auto-learning report for specialized-handler precedence (issue #663)
// (rust/src/agentic_coding/learning_report/handler_precedence_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from '../host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from '../learning_report.mjs';
import { agenticMessage } from '../messages.mjs';

/** Mirrors `HANDLER_PRECEDENCE_LEARNING_PATH` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
export const HANDLER_PRECEDENCE_LEARNING_PATH = 'handler-precedence-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
export const REPORT = Object.freeze({
  head: 'handler_precedence_learning_report',
  issue: '663',
  promotion_gate: 'routing_precedence_from_seed_and_parity_fixture_pass',
  path: HANDLER_PRECEDENCE_LEARNING_PATH,
  get task() {
    return agenticMessage('learning_report_task_handler_precedence_learning');
  },
  get memory() {
    return readText('data/meta/issue-663-handler-precedence-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_handler_precedence_learning');
  },
});

/** Mirrors `HANDLER_PRECEDENCE_LEARNING_TASK` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
export function task() {
  return REPORT.task;
}

/** Mirrors `fn is_handler_precedence_learning_task` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. @param {string} prompt */
export function isHandlerPrecedenceLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
