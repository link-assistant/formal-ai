// The identity of the auto-learning report for nested context resolution (issue #702)
// (rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from '../learning_report_api.mjs';


/** Mirrors `CONTEXT_HIERARCHY_LEARNING_PATH` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. */
export const CONTEXT_HIERARCHY_LEARNING_PATH = 'context-hierarchy-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'context_hierarchy_learning_report',
  issue: '702',
  promotion_gate: 'nested_context_runtime_and_parity_fixtures_pass',
  path: CONTEXT_HIERARCHY_LEARNING_PATH,
  taskValue: CONTEXT_HIERARCHY_LEARNING_PATH,
  memoryPath: 'data/meta/issue-702-context-hierarchy-learning.lino',
  subjectMessage: 'learning_report_subject_context_hierarchy_learning',
}, 'isContextHierarchyLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `CONTEXT_HIERARCHY_LEARNING_TASK` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs (the artifact path itself). */
export const CONTEXT_HIERARCHY_LEARNING_TASK = CONTEXT_HIERARCHY_LEARNING_PATH;

/** Mirrors `fn is_context_hierarchy_learning_task` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. @param {string} prompt */
export const isContextHierarchyLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
