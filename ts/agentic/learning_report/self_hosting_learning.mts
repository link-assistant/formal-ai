// The identity of the auto-learning report for self-hosting attribution (issue #657)
// (rust/src/agentic_coding/learning_report/self_hosting_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from '../host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from '../learning_report.mjs';
import { agenticMessage } from '../messages.mjs';

/** Mirrors `SELF_HOSTING_LEARNING_PATH` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
export const SELF_HOSTING_LEARNING_PATH = 'self-hosting-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
export const REPORT = Object.freeze({
  head: 'self_hosting_learning_report',
  issue: '657',
  promotion_gate: 'metric_fixture_exact_share_and_honest_ledger_ratchet_pass',
  path: SELF_HOSTING_LEARNING_PATH,
  get task() {
    return agenticMessage('learning_report_task_self_hosting_learning');
  },
  get memory() {
    return readText('data/meta/issue-657-self-hosting-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_self_hosting_learning');
  },
});

/** Mirrors `SELF_HOSTING_LEARNING_TASK` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
export function task() {
  return REPORT.task;
}

/** Mirrors `fn is_self_hosting_learning_task` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. @param {string} prompt */
export function isSelfHostingLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
