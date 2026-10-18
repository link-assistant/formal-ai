// The identity of the auto-learning report for the issue-716 client-execution failures
// (rust/src/agentic_coding/execution_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from './learning_report_api.mjs';


/** Mirrors `EXECUTION_LEARNING_PATH` in rust/src/agentic_coding/execution_learning.rs. */
export const EXECUTION_LEARNING_PATH = 'client-execution-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/execution_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'client_execution_learning_report',
  issue: '716',
  promotion_gate: 'protocol_matrix_presentation_variations_and_agent_cli_e2e_pass',
  path: EXECUTION_LEARNING_PATH,
  taskMessage: 'learning_report_task_client_execution_learning',
  memoryPath: 'data/meta/issue-716-execution-learning.lino',
  subjectMessage: 'learning_report_subject_client_execution_learning',
}, 'isExecutionLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `EXECUTION_LEARNING_TASK` in rust/src/agentic_coding/execution_learning.rs. */
export const task = reportApi.task;

/** Mirrors `fn is_execution_learning_task` in rust/src/agentic_coding/execution_learning.rs. @param {string} prompt */
export const isExecutionLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/execution_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/execution_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/execution_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
