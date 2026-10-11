// The identity of the auto-learning report for real external benchmark failures (issue #698)
// (rust/src/agentic_coding/external_benchmark_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.

import { readText } from './host.mjs';
import { renderDocument as renderReport, renderDocumentFrom as renderReportFrom } from './learning_report.mjs';
import { agenticMessage } from './messages.mjs';

/** Mirrors `EXTERNAL_BENCHMARK_LEARNING_PATH` in rust/src/agentic_coding/external_benchmark_learning.rs. */
export const EXTERNAL_BENCHMARK_LEARNING_PATH = 'external-benchmark-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/external_benchmark_learning.rs. */
export const REPORT = Object.freeze({
  head: 'external_benchmark_learning_report',
  issue: '698',
  promotion_gate: 'external_benchmark_ratchet_and_agent_cli_e2e_pass',
  path: EXTERNAL_BENCHMARK_LEARNING_PATH,
  task: EXTERNAL_BENCHMARK_LEARNING_PATH,
  get memory() {
    return readText('data/meta/issue-698-external-benchmark-learning.lino');
  },
  get subject() {
    return agenticMessage('learning_report_subject_external_benchmark_learning');
  },
});

/** Mirrors `fn render_document` in rust/src/agentic_coding/external_benchmark_learning.rs. */
export function renderDocument() {
  return renderReport(REPORT);
}

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/external_benchmark_learning.rs. @param {string} memoryDocument */
export function renderDocumentFrom(memoryDocument) {
  return renderReportFrom(REPORT, memoryDocument);
}
