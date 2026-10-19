// The identity of the auto-learning report for issue #686's associative auto-learning loop
// (rust/src/agentic_coding/associative_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from './learning_report_api.mjs';


/** Mirrors `ASSOCIATIVE_LEARNING_PATH` in rust/src/agentic_coding/associative_learning.rs. */
export const ASSOCIATIVE_LEARNING_PATH = 'associative-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/associative_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'associative_learning_report',
  issue: '686',
  promotion_gate: null,
  path: ASSOCIATIVE_LEARNING_PATH,
  taskMessage: 'learning_report_task_associative_learning',
  memoryPath: 'data/meta/associative-learning-case.lino',
  subjectMessage: 'learning_report_subject_associative_learning',
}, 'isAssociativeLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `ASSOCIATIVE_LEARNING_TASK` in rust/src/agentic_coding/associative_learning.rs. */
export const task = reportApi.task;

/** Mirrors `fn is_associative_learning_task` in rust/src/agentic_coding/associative_learning.rs. @param {string} prompt */
export const isAssociativeLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/associative_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/associative_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/associative_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
