// One descriptor for every review-gated auto-learning report
// (rust/src/agentic_coding/learning_report.rs).
//
// A report is the same derivation every time - rank a persisted observation
// network by the associative-memory adapter's four retention signals, keep the
// evidence links, and hand a human a proposal - under a different identity.
// A `LearningReport` is a plain object `{head, issue, promotion_gate, path,
// task, memory, subject}` (the last three are getters reading the messages
// and data files); the report modules export one as `REPORT` and this module
// renders, routes and plans it.

import * as associativeLearning from './associative_learning.mjs';
import * as codeRewriteLearning from './code_rewrite_learning.mjs';
import { formatLinoValueVerbatim } from './crate/links_format.mjs';
import { AssociativeMemory, sortedQualifiers } from './crate/associative_persistence.mjs';
import { parseLinksNotation } from './crate/memory.mjs';
import { rustLines } from './content.mjs';
import { planDocumentRecipe } from './document_recipe.mjs';
import * as executionLearning from './execution_learning.mjs';
import * as externalBenchmarkLearning from './external_benchmark_learning.mjs';
import * as contextHierarchyLearning from './learning_report/context_hierarchy_learning.mjs';
import * as handlerPrecedenceLearning from './learning_report/handler_precedence_learning.mjs';
import * as hardcodedLanguageLearning from './learning_report/hardcoded_language_learning.mjs';
import * as lexemeImportLearning from './learning_report/lexeme_import_learning.mjs';
import * as searchFusionLearning from './learning_report/search_fusion_learning.mjs';
import * as selfHostingLearning from './learning_report/self_hosting_learning.mjs';
import { agenticMessage } from './messages.mjs';
import * as routingLearning from './routing_learning.mjs';
import { trimStart } from './crate/rust_str.mjs';

/**
 * Mirrors `static REPORTS` in rust/src/agentic_coding/learning_report.rs:
 * every auto-learning report, in routing order. A function so the report
 * modules (which import this one) can finish evaluating first.
 * @returns {Array<object>}
 */
export function reports() {
  return [
    associativeLearning.REPORT,
    routingLearning.REPORT,
    codeRewriteLearning.REPORT,
    executionLearning.REPORT,
    contextHierarchyLearning.REPORT,
    selfHostingLearning.REPORT,
    searchFusionLearning.REPORT,
    hardcodedLanguageLearning.REPORT,
    lexemeImportLearning.REPORT,
    handlerPrecedenceLearning.REPORT,
    externalBenchmarkLearning.REPORT,
  ];
}

/**
 * Mirrors `fn route` in rust/src/agentic_coding/learning_report.rs: the report
 * a task asks for, or null.
 * @param {string} task
 * @returns {object|null}
 */
export function route(task) {
  return reports().find((report) => matches(report, task)) ?? null;
}

/**
 * Mirrors `LearningReport::matches`: the prompt names the artifact it wants.
 * @param {object} report
 * @param {string} prompt
 */
export function matches(report, prompt) {
  return prompt.toLowerCase().includes(report.path);
}

/** Mirrors `LearningReport::render_document`: render from the report's own memory. */
export function renderDocument(report) {
  return renderDocumentFrom(report, report.memory);
}

/** Mirrors `LearningReport::render_document_from`. */
export function renderDocumentFrom(report, memoryDocument) {
  return render(report, memoryDocument);
}

/**
 * Mirrors `LearningReport::plan_step`: the write-then-verify steps that
 * produce the report (the planner's `report.plan_step(messages, tool_names)`).
 * @param {object} report
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planReportStep(report, messages, toolNames) {
  const document = renderDocument(report);
  return planDocumentRecipe(messages, toolNames, {
    path: report.path,
    verify_command: `cat ${report.path}`,
    final_answer: finalAnswer(report, document),
    document,
  });
}

/** Mirrors `LearningReport::final_answer`: what was ranked, and where the proposal is. */
export function finalAnswer(report, document) {
  const expressions = rustLines(document).filter((line) => trimStart(line).startsWith('learned_expression_')).length;
  const params = { expressions, subject: report.subject, path: report.path };
  return report.promotion_gate !== null
    ? agenticMessage('learning_report_final_gated', params)
    : agenticMessage('learning_report_final_ungated', params);
}

/** Mirrors `fn field` in rust/src/agentic_coding/learning_report.rs. */
function field(indent, name, value) {
  return `${' '.repeat(indent)}${name} ${formatLinoValueVerbatim(value)}\n`;
}

/** Mirrors `fn render` in rust/src/agentic_coding/learning_report.rs. */
function render(report, memoryDocument) {
  const memory = AssociativeMemory.fromMemoryEvents(parseLinksNotation(memoryDocument));
  const seed = memory.retentionRanking()[0] ?? '';
  const recalled = memory.recallRelated(seed, 2);
  const ranking = memory.retentionRanking();
  const warningCount = memory.expressions().reduce((sum, [, expression]) => sum + expression.validation_issues.length, 0);
  let out = `${report.head}\n`;
  out += field(2, 'issue', report.issue);
  if (report.promotion_gate !== null) {
    out += field(2, 'decision', 'awaiting_human_review');
    out += field(2, 'promotion_gate', report.promotion_gate);
  }
  out += '  record_type "agent_cli_auto_learning"\n';
  out += '  substrate "links_network"\n';
  out += `  retention_formula "${agenticMessage('learning_report_retention_formula')}"\n`;
  out += `  expression_count "${memory.len()}"\n`;
  out += `  validation_warning_count "${warningCount}"\n`;
  out += field(2, 'multi_hop_seed', seed);
  out += field(2, 'multi_hop_recall', recalled.join('|'));
  ranking.forEach((id, index) => {
    const expression = memory.get(id);
    if (expression === null) return;
    out += `  learned_expression_${String(index + 1).padStart(2, '0')}\n`;
    out += field(4, 'id', id);
    out += field(4, 'text', expression.text);
    out += field(4, 'reads', String(expression.reads));
    out += field(4, 'writes', String(expression.writes));
    out += field(4, 'incoming_links', String(memory.inDegree(id)));
    out += field(4, 'outgoing_links', String(memory.outDegree(id)));
    out += field(4, 'retention_score', String(memory.retentionScore(id)));
    out += field(4, 'qualifiers', sortedQualifiers(expression).map(([name, value]) => `${name}=${value}`).join('|'));
    out += field(4, 'validation', expression.validation_issues.length === 0 ? 'aligned' : 'retained_with_warning');
  });
  return out;
}
