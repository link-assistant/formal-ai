// The identity of the auto-learning report for search fusion (issue #709)
// (rust/src/agentic_coding/learning_report/search_fusion_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from '../host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from '../learning_report.mjs';
import { agenticMessage } from '../messages.mjs';

/** Mirrors `SEARCH_FUSION_LEARNING_PATH` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. */
export const SEARCH_FUSION_LEARNING_PATH = 'search-fusion-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. */
export const REPORT = Object.freeze({
  head: 'search_fusion_learning_report',
  issue: '709',
  promotion_gate: 'issue_709_held_out_zero_failures_and_named_review',
  path: SEARCH_FUSION_LEARNING_PATH,
  task: SEARCH_FUSION_LEARNING_PATH,
  get memory() {
    return readText('data/meta/issue-709-search-fusion-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_search_fusion_learning');
  },
});

/** Mirrors `SEARCH_FUSION_LEARNING_TASK` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs (the artifact path itself). */
export const SEARCH_FUSION_LEARNING_TASK = SEARCH_FUSION_LEARNING_PATH;

/** Mirrors `fn is_search_fusion_learning_task` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. @param {string} prompt */
export function isSearchFusionLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
