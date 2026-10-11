// The identity of the auto-learning report for nested context resolution (issue #702)
// (rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from '../host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from '../learning_report.mjs';
import { agenticMessage } from '../messages.mjs';

/** Mirrors `CONTEXT_HIERARCHY_LEARNING_PATH` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. */
export const CONTEXT_HIERARCHY_LEARNING_PATH = 'context-hierarchy-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. */
export const REPORT = Object.freeze({
  head: 'context_hierarchy_learning_report',
  issue: '702',
  promotion_gate: 'nested_context_runtime_and_parity_fixtures_pass',
  path: CONTEXT_HIERARCHY_LEARNING_PATH,
  task: CONTEXT_HIERARCHY_LEARNING_PATH,
  get memory() {
    return readText('data/meta/issue-702-context-hierarchy-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_context_hierarchy_learning');
  },
});

/** Mirrors `CONTEXT_HIERARCHY_LEARNING_TASK` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs (the artifact path itself). */
export const CONTEXT_HIERARCHY_LEARNING_TASK = CONTEXT_HIERARCHY_LEARNING_PATH;

/** Mirrors `fn is_context_hierarchy_learning_task` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. @param {string} prompt */
export function isContextHierarchyLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
