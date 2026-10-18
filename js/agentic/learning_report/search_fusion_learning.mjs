// The identity of the auto-learning report for search fusion (issue #709)
// (rust/src/agentic_coding/learning_report/search_fusion_learning.rs). The derivation is shared:
// js/agentic/learning_report.mjs renders, routes and plans it.


import {createLearningReportApi} from '../learning_report_api.mjs';


/** Mirrors `SEARCH_FUSION_LEARNING_PATH` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. */
export const SEARCH_FUSION_LEARNING_PATH = 'search-fusion-learning-report.lino';

/** Mirrors `static REPORT` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. */
const reportApi = createLearningReportApi({
  head: 'search_fusion_learning_report',
  issue: '709',
  promotion_gate: 'issue_709_held_out_zero_failures_and_named_review',
  path: SEARCH_FUSION_LEARNING_PATH,
  taskValue: SEARCH_FUSION_LEARNING_PATH,
  memoryPath: 'data/meta/issue-709-search-fusion-learning.lino',
  subjectMessage: 'learning_report_subject_search_fusion_learning',
}, 'isSearchFusionLearningTask');
export const REPORT = reportApi.REPORT;

/** Mirrors `SEARCH_FUSION_LEARNING_TASK` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs (the artifact path itself). */
export const SEARCH_FUSION_LEARNING_TASK = SEARCH_FUSION_LEARNING_PATH;

/** Mirrors `fn is_search_fusion_learning_task` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. @param {string} prompt */
export const isSearchFusionLearningTask = reportApi.isTask;

/** Mirrors `fn render_document` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. */
export const renderDocument = reportApi.renderDocument;

/** Mirrors `fn render_document_from` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. @param {string} memoryDocument */
export const renderDocumentFrom = reportApi.renderDocumentFrom;

/** Mirrors `fn final_answer` in rust/src/agentic_coding/learning_report/search_fusion_learning.rs. @param {string} document */
export const finalAnswer = reportApi.finalAnswer;
