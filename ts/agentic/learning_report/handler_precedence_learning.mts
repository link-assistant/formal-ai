// The identity of the auto-learning report for specialized-handler precedence (issue #663)
// (rust/src/agentic_coding/learning_report/handler_precedence_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from '../learning_report_api.mjs';


/** Mirrors `HANDLER_PRECEDENCE_LEARNING_PATH` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
export const HANDLER_PRECEDENCE_LEARNING_PATH = 'handler-precedence-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'handler_precedence_learning_report',
  issue: '663',
  promotion_gate: 'routing_precedence_from_seed_and_parity_fixture_pass',
  path: HANDLER_PRECEDENCE_LEARNING_PATH,
  taskMessage: 'learning_report_task_handler_precedence_learning',
  memoryPath: 'data/meta/issue-663-handler-precedence-learning.lino',
  subjectMessage: 'learning_report_subject_handler_precedence_learning',
}, 'isHandlerPrecedenceLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `HANDLER_PRECEDENCE_LEARNING_TASK` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
export const task = reportApi.task;

/** Mirrors `fn is_handler_precedence_learning_task` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. @param {string} prompt */
export const isHandlerPrecedenceLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/handler_precedence_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
