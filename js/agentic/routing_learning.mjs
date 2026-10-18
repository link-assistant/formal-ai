// The identity of the auto-learning report for the issue-712 tool-routing failures
// (rust/src/agentic_coding/routing_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from './learning_report_api.mjs';


/** Mirrors `ROUTING_LEARNING_PATH` in rust/src/agentic_coding/routing_learning.rs. */
export const ROUTING_LEARNING_PATH = 'tool-routing-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/routing_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'tool_routing_learning_report',
  issue: '712',
  promotion_gate: 'reported_matrix_and_unseen_paraphrases_pass',
  path: ROUTING_LEARNING_PATH,
  taskMessage: 'learning_report_task_tool_routing_learning',
  memoryPath: 'data/meta/issue-712-routing-learning.lino',
  subjectMessage: 'learning_report_subject_tool_routing_learning',
}, 'isRoutingLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `ROUTING_LEARNING_TASK` in rust/src/agentic_coding/routing_learning.rs. */
export const task = reportApi.task;

/** Mirrors `fn is_routing_learning_task` in rust/src/agentic_coding/routing_learning.rs. @param {string} prompt */
export const isRoutingLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/routing_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/routing_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/routing_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
