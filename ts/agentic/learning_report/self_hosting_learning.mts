// The identity of the auto-learning report for self-hosting attribution (issue #657)
// (rust/src/agentic_coding/learning_report/self_hosting_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from '../learning_report_api.mjs';


/** Mirrors `SELF_HOSTING_LEARNING_PATH` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
export const SELF_HOSTING_LEARNING_PATH = 'self-hosting-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'self_hosting_learning_report',
  issue: '657',
  promotion_gate: 'metric_fixture_exact_share_and_honest_ledger_ratchet_pass',
  path: SELF_HOSTING_LEARNING_PATH,
  taskMessage: 'learning_report_task_self_hosting_learning',
  memoryPath: 'data/meta/issue-657-self-hosting-learning.lino',
  subjectMessage: 'learning_report_subject_self_hosting_learning',
}, 'isSelfHostingLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `SELF_HOSTING_LEARNING_TASK` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
export const task = reportApi.task;

/** Mirrors `fn is_self_hosting_learning_task` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. @param {string} prompt */
export const isSelfHostingLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/self_hosting_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
