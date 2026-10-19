// The identity of the auto-learning report for the issue-715 workspace-rewrite failures
// (rust/src/agentic_coding/code_rewrite_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from './learning_report_api.mjs';


/** Mirrors `CODE_REWRITE_LEARNING_PATH` in rust/src/agentic_coding/code_rewrite_learning.rs. */
export const CODE_REWRITE_LEARNING_PATH = 'code-rewrite-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/code_rewrite_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'code_rewrite_learning_report',
  issue: '715',
  promotion_gate: 'normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass',
  path: CODE_REWRITE_LEARNING_PATH,
  taskMessage: 'learning_report_task_code_rewrite_learning',
  memoryPath: 'data/meta/issue-715-code-rewrite-learning.lino',
  subjectMessage: 'learning_report_subject_code_rewrite_learning',
}, 'isCodeRewriteLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `CODE_REWRITE_LEARNING_TASK` in rust/src/agentic_coding/code_rewrite_learning.rs. */
export const task = reportApi.task;

/** Mirrors `fn is_code_rewrite_learning_task` in rust/src/agentic_coding/code_rewrite_learning.rs. @param {string} prompt */
export const isCodeRewriteLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/code_rewrite_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/code_rewrite_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/code_rewrite_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
