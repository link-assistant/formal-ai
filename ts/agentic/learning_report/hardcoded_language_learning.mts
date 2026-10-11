// The identity of the auto-learning report for R379 (issue #659)
// (rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from '../host.mjs';
import { finalAnswer as reportFinalAnswer, matches, renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from '../learning_report.mjs';
import { agenticMessage } from '../messages.mjs';

/** Mirrors `HARDCODED_LANGUAGE_LEARNING_PATH` in rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs. */
export const HARDCODED_LANGUAGE_LEARNING_PATH = 'hardcoded-language-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs. */
export const REPORT = Object.freeze({
  head: 'hardcoded_language_learning_report',
  issue: '659',
  promotion_gate: 'hardcoded_language_fixture_context_gate_and_agent_cli_e2e_pass',
  path: HARDCODED_LANGUAGE_LEARNING_PATH,
  task: HARDCODED_LANGUAGE_LEARNING_PATH,
  get memory() {
    return readText('data/meta/issue-659-hardcoded-language-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_hardcoded_language_learning');
  },
});

/** Mirrors `HARDCODED_LANGUAGE_LEARNING_TASK` in rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs (the artifact path itself). */
export const HARDCODED_LANGUAGE_LEARNING_TASK = HARDCODED_LANGUAGE_LEARNING_PATH;

/** Mirrors `fn is_hardcoded_language_learning_task` in rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs. @param {string} prompt */
export function isHardcodedLanguageLearningTask(prompt) {
  return matches(REPORT, prompt);
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs. @param {string} document */
export function finalAnswer(document) {
  return reportFinalAnswer(REPORT, document);
}
