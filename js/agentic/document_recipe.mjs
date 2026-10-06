// The shared generate -> verify -> final recipe, and the self-inspection
// recipes expressed through it (rust/src/agentic_coding/document_recipe.rs).
//
// Every self-referential recipe writes a generated document, reads it back to
// verify it, then answers with what was generated; they differ only in the
// document, so they share `planDocumentRecipe` and one recipe shape
// (`{path, document, verify_command, final_answer}`).
//
// The per-recipe planners are async only because they await the recipe
// modules' renderers (some of which consult the host solver); the plan they
// return is the Rust plan.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import * as changeRequest from './change_request.mjs';
import * as diagram from './diagram.mjs';
import * as dreamingAudit from './dreaming_audit.mjs';
import * as explain from './explain.mjs';
import * as googleTrendsCatalog from './google_trends_catalog.mjs';
import * as googleTrendsLearning from './google_trends_learning.mjs';
import * as ledger from './ledger.mjs';
import * as meaningDetail from './meaning_detail.mjs';
import { fetchArguments, finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { Progress } from './progress.mjs';
import * as questionCatalog from './question_catalog.mjs';
import * as rebuildPlan from './rebuild_plan.mjs';
import * as repairStrategy from './repair_strategy.mjs';
import * as selfAst from './self_ast.mjs';
import * as selfHeal from './self_heal.mjs';
import * as sourceLinks from './source_links.mjs';

/**
 * Mirrors `fn plan_document_recipe` in rust/src/agentic_coding/document_recipe.rs:
 * write -> verify -> final, skipping steps whose tool is not advertised.
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {{path: string, document: string, verify_command: string, final_answer: string}} recipe
 */
export function planDocumentRecipe(messages, toolNames, recipe) {
  const progress = Progress.scan(messages);
  const writeTool = toolFor(toolNames, Capability.Write);
  if (writeTool !== null && !progress.done(Capability.Write)) {
    return planOne(writeTool, writeArguments(recipe.path, recipe.document));
  }
  const runTool = toolFor(toolNames, Capability.Run);
  if (runTool !== null && !progress.done(Capability.Run)) {
    return planOne(runTool, jsonText({ command: recipe.verify_command }));
  }
  return finalAnswer(recipe.final_answer);
}

/** A recipe over `module.renderDocument` / `module.finalAnswer` at `path`, verified with `cat`. */
async function catRecipe(module, path) {
  const document = await module.renderDocument();
  return { path, verify_command: `cat ${path}`, final_answer: await module.finalAnswer(document), document };
}

/**
 * Mirrors `fn plan_meaning_detail_step` in rust/src/agentic_coding/document_recipe.rs:
 * search -> fetch (Wikidata lexemes) -> write the enriched block -> verify -> final.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planMeaningDetailStep(task, messages, toolNames) {
  const concept = meaningDetail.conceptForTask(task) ?? meaningDetail.TOMATO;
  const searchTool = toolFor(toolNames, Capability.Search);
  const fetchTool = toolFor(toolNames, Capability.Fetch);
  const writeTool = toolFor(toolNames, Capability.Write);
  const runTool = toolFor(toolNames, Capability.Run);
  const progress = Progress.scan(messages);
  if (searchTool !== null && !progress.done(Capability.Search)) {
    return planOne(searchTool, jsonText({ query: meaningDetail.searchQuery(concept) }));
  }
  if (fetchTool !== null && !progress.done(Capability.Fetch)) {
    return planOne(fetchTool, fetchArguments(concept.source_url));
  }
  const block = meaningDetail.enrichBlock(concept, progress.fetched_text);
  if (writeTool !== null && !progress.done(Capability.Write)) {
    return planOne(writeTool, writeArguments(concept.kb_path, block));
  }
  if (runTool !== null && !progress.done(Capability.Run)) {
    return planOne(runTool, jsonText({ command: `cat ${concept.kb_path}` }));
  }
  return finalAnswer(meaningDetail.finalAnswerFor(concept, block));
}

/** Mirrors `fn plan_diagram_step`. */
export async function planDiagramStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(diagram, diagram.DIAGRAM_PATH));
}

/** Mirrors `fn plan_self_ast_step`. */
export async function planSelfAstStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(selfAst, selfAst.AST_PATH));
}

/** Mirrors `fn plan_self_heal_step`. */
export async function planSelfHealStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(selfHeal, selfHeal.SELF_HEAL_PATH));
}

/** Mirrors `fn plan_source_links_step`. */
export async function planSourceLinksStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(sourceLinks, sourceLinks.SOURCE_LINKS_PATH));
}

/** Mirrors `fn plan_ledger_step`. */
export async function planLedgerStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(ledger, ledger.LEDGER_PATH));
}

/** Mirrors `fn plan_explain_step`. */
export async function planExplainStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(explain, explain.EXPLAIN_PATH));
}

/** Mirrors `fn plan_change_request_step`. */
export async function planChangeRequestStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(changeRequest, changeRequest.CHANGE_PATH));
}

/** Mirrors `fn plan_repair_strategy_step`. */
export async function planRepairStrategyStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(repairStrategy, repairStrategy.REPAIR_STRATEGY_PATH));
}

/** Mirrors `fn plan_rebuild_step`. */
export async function planRebuildStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(rebuildPlan, rebuildPlan.REBUILD_PATH));
}

/** Mirrors `fn plan_question_catalog_step`. */
export async function planQuestionCatalogStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(questionCatalog, questionCatalog.QUESTION_CATALOG_PATH));
}

/** Mirrors `fn plan_dreaming_audit_step`. */
export async function planDreamingAuditStep(messages, toolNames) {
  return planDocumentRecipe(messages, toolNames, await catRecipe(dreamingAudit, dreamingAudit.DREAMING_AUDIT_PATH));
}

/** Mirrors `fn plan_google_trends_learning_step`. */
export async function planGoogleTrendsLearningStep(messages, toolNames) {
  const document = await googleTrendsLearning.renderDocument();
  return planDocumentRecipe(messages, toolNames, {
    path: googleTrendsLearning.GOOGLE_TRENDS_LEARNING_PATH,
    verify_command: await googleTrendsLearning.verificationCommand(),
    final_answer: await googleTrendsLearning.finalAnswer(document),
    document,
  });
}

/** Mirrors `fn plan_google_trends_catalog_step`. */
export async function planGoogleTrendsCatalogStep(messages, toolNames) {
  const document = await googleTrendsCatalog.renderDocument();
  return planDocumentRecipe(messages, toolNames, {
    path: googleTrendsCatalog.GOOGLE_TRENDS_CATALOG_PATH,
    verify_command: await googleTrendsCatalog.verificationCommand(),
    final_answer: await googleTrendsCatalog.finalAnswer(document),
    document,
  });
}
