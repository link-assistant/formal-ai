// The identity of the auto-learning report for the issue-716 client-execution failures
// (rust/src/agentic_coding/execution_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from './host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from './learning_report.mjs';
import { agenticMessage } from './messages.mjs';

/** Mirrors `EXECUTION_LEARNING_PATH` in rust/src/agentic_coding/execution_learning.rs. */
export const EXECUTION_LEARNING_PATH = 'client-execution-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/execution_learning.rs. */
export const REPORT = Object.freeze({
  head: 'client_execution_learning_report',
  issue: '716',
  promotion_gate: 'protocol_matrix_presentation_variations_and_agent_cli_e2e_pass',
  path: EXECUTION_LEARNING_PATH,
  get task() {
    return agenticMessage('learning_report_task_client_execution_learning');
  },
  get memory() {
    return readText('data/meta/issue-716-execution-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_client_execution_learning');
  },
});

/** Mirrors `EXECUTION_LEARNING_TASK` in rust/src/agentic_coding/execution_learning.rs. */
export function task() {
  return REPORT.task;
}

/** Mirrors `fn is_execution_learning_task` in rust/src/agentic_coding/execution_learning.rs. @param {string} prompt */
export function isExecutionLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/execution_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/execution_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/execution_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
