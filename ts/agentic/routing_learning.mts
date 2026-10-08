// The identity of the auto-learning report for the issue-712 tool-routing failures
// (rust/src/agentic_coding/routing_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from './host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from './learning_report.mjs';
import { agenticMessage } from './messages.mjs';

/** Mirrors `ROUTING_LEARNING_PATH` in rust/src/agentic_coding/routing_learning.rs. */
export const ROUTING_LEARNING_PATH = 'tool-routing-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/routing_learning.rs. */
export const REPORT = Object.freeze({
  head: 'tool_routing_learning_report',
  issue: '712',
  promotion_gate: 'reported_matrix_and_unseen_paraphrases_pass',
  path: ROUTING_LEARNING_PATH,
  get task() {
    return agenticMessage('learning_report_task_tool_routing_learning');
  },
  get memory() {
    return readText('data/meta/issue-712-routing-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_tool_routing_learning');
  },
});

/** Mirrors `ROUTING_LEARNING_TASK` in rust/src/agentic_coding/routing_learning.rs. */
export function task() {
  return REPORT.task;
}

/** Mirrors `fn is_routing_learning_task` in rust/src/agentic_coding/routing_learning.rs. @param {string} prompt */
export function isRoutingLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/routing_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/routing_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/routing_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
