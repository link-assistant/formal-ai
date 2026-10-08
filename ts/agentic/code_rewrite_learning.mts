// The identity of the auto-learning report for the issue-715 workspace-rewrite failures
// (rust/src/agentic_coding/code_rewrite_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from './host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from './learning_report.mjs';
import { agenticMessage } from './messages.mjs';

/** Mirrors `CODE_REWRITE_LEARNING_PATH` in rust/src/agentic_coding/code_rewrite_learning.rs. */
export const CODE_REWRITE_LEARNING_PATH = 'code-rewrite-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/code_rewrite_learning.rs. */
export const REPORT = Object.freeze({
  head: 'code_rewrite_learning_report',
  issue: '715',
  promotion_gate: 'normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass',
  path: CODE_REWRITE_LEARNING_PATH,
  get task() {
    return agenticMessage('learning_report_task_code_rewrite_learning');
  },
  get memory() {
    return readText('data/meta/issue-715-code-rewrite-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_code_rewrite_learning');
  },
});

/** Mirrors `CODE_REWRITE_LEARNING_TASK` in rust/src/agentic_coding/code_rewrite_learning.rs. */
export function task() {
  return REPORT.task;
}

/** Mirrors `fn is_code_rewrite_learning_task` in rust/src/agentic_coding/code_rewrite_learning.rs. @param {string} prompt */
export function isCodeRewriteLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/code_rewrite_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/code_rewrite_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/code_rewrite_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
