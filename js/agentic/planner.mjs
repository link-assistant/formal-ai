// Deterministic agentic planner: the next tool call or final answer from a
// conversation and its advertised tools, without hidden neural state
// (rust/src/agentic_coding/planner.rs).
//
// Every route arm is the twin of the Rust arm of the same name, run in the
// order data/seed/planner-precedence.lino declares. Arms that consult the
// host solver are async, so the cascade awaits each arm.

import * as algorithmLearning from './algorithm_learning.mjs';
import { Capability } from './capability.mjs';
import * as capabilityRouter from './capability_router.mjs';
import * as changeRequest from './change_request.mjs';
import * as ciWorkflow from './ci_workflow.mjs';
import * as codeArtifact from './code_artifact.mjs';
import * as codeTask from './code_task.mjs';
import * as commandReroute from './command_reroute.mjs';
import * as comparison from './comparison.mjs';
import { programContractAnswer } from './crate/coding_program_contract.mjs';
import { sourceTreeRequest as metaSourceTreeRequest } from './crate/meta_translate.mjs';
import { handlerMatches } from './crate/rule_interpreter.mjs';
import { plannerPrecedence } from './crate/seed.mjs';
import { looksLikeSkillDescription } from './crate/skill_compiler.mjs';
import { computerUsePlanAgenticStep } from './crate/computer_use_planner.mjs';
import { latestUserRequest } from './content.mjs';
import * as conversationRecall from './conversation_recall.mjs';
import * as diagram from './diagram.mjs';
import * as documentRecipe from './document_recipe.mjs';
import * as dreamingAudit from './dreaming_audit.mjs';
import * as evidenceRecord from './evidence_record.mjs';
import * as fileSummary from './file_summary.mjs';
import * as explain from './explain.mjs';
import { fileReadTaskFor, planFileReadStep } from './file_read.mjs';
import * as formalizationRecipe from './formalization_recipe.mjs';
import * as functionExpectation from './function_expectation.mjs';
import { planGeneralChangeStep } from './general_execution.mjs';
import { composeGeneralChangePlan, hasAuthoritativeLiteralWrite, objectiveText } from './general_planner.mjs';
import * as gitCommit from './git_commit.mjs';
import * as googleTrendsCatalog from './google_trends_catalog.mjs';
import * as googleTrendsLearning from './google_trends_learning.mjs';
import * as harnessEnvelope from './harness_envelope.mjs';
import * as intentRouter from './intent_router.mjs';
import * as learningReport from './learning_report.mjs';
import * as ledger from './ledger.mjs';
import * as localSearch from './local_search.mjs';
import * as meaningDetail from './meaning_detail.mjs';
import * as moduleExports from './module_exports.mjs';
import * as moduleFunction from './module_function.mjs';
import * as mutatingAction from './mutating_action.mjs';
import * as noteComposition from './note_composition.mjs';
import { agenticMessage } from './messages.mjs';
import { finalAnswer, isToolCalls, jsonText, planOne } from './plan.mjs';
import { continuedAgentTask, isContinuationCue, traceRoute } from './planner/continuation.mjs';
import * as positionalEdit from './positional_edit.mjs';
import * as procedure from './procedure.mjs';
import { Progress } from './progress.mjs';
import * as questionCatalog from './question_catalog.mjs';
import * as rebuildPlan from './rebuild_plan.mjs';
import * as repairStrategy from './repair_strategy.mjs';
import * as reportIssue from './report_issue.mjs';
import * as restartFeedback from './restart_feedback.mjs';
import * as selfAst from './self_ast.mjs';
import * as selfHeal from './self_heal.mjs';
import * as shellCommand from './shell_command.mjs';
import * as shellFileFallback from './shell_file_fallback.mjs';
import * as sourceLinks from './source_links.mjs';
import * as statementAudit from './statement_audit.mjs';
import * as structuredDocument from './structured_document.mjs';
import * as structuredEdit from './structured_edit.mjs';
import * as taskObligations from './task_obligations.mjs';
import * as taskStructure from './task_structure.mjs';
import * as toolResult from './tool_result.mjs';
import * as webResearch from './web_research.mjs';
import { fill } from './work_item_steps.mjs';
import * as workspaceChange from './workspace_change.mjs';
import * as workspaceInspection from './workspace_inspection.mjs';
import * as workspaceSearch from './workspace_search.mjs';

const { toolFor } = capabilityRouter;

/**
 * Mirrors `PLANNER_ROUTE_ARMS` in rust/src/agentic_coding/planner/precedence.rs: the
 * route arms of the cascade, named and in run order.
 */
export const PLANNER_ROUTE_ARMS = [
  ['plan_chat_step_routes', 'conversation_control_decline'],
  ['plan_chat_step_routes', 'computer_use'],
  ['plan_chat_step_routes', 'authoritative_literal_write'],
  ['plan_chat_step_routes', 'program_contract'],
  ['plan_chat_step_routes', 'evidence_record'],
  ...['git_commit', 'workspace_change', 'generated_source', 'structured_edit', 'structured_document',
    'statement_audit', 'task_obligations', 'literal_write', 'algorithm_learning', 'procedure',
    'learning_report', 'code_artifact', 'self_heal', 'dreaming_audit', 'self_ast', 'source_links',
    'learning_ledger', 'explain', 'change_request', 'repair_strategy', 'rebuild_plan',
    'google_trends_learning', 'google_trends_catalog', 'question_catalog', 'file_analysis', 'workspace_search',
    'report_flow', 'conversation_recall', 'follow_up_answer', 'contextual_reference_clarification',
    'definition_followup', 'intent_edit', 'typed_file_read', 'local_search', 'comparison',
    'named_capability_table', 'shell_command', 'file_read', 'formalization_recipe', 'meaning_detail',
    'diagram', 'workspace_inspection', 'task_structure', 'capability_table_named_or_local',
    'positional_edit_decline', 'web_research_query', 'intent_web_search', 'code_search_fallback',
    'research_continuation', 'latest_turn_answer', 'note_composition', 'general_change_fallback',
    'web_research_final', 'capability_table_open_web'].map((arm) => ['plan_settled_routes', arm]),
];

/**
 * Mirrors `fn checked_route_precedence` in rust/src/agentic_coding/planner/precedence.rs:
 * throws unless planner-precedence.lino names exactly the coded arms in order.
 */
export function checkedRoutePrecedence() {
  const declared = plannerPrecedence();
  if (declared.length !== PLANNER_ROUTE_ARMS.length) {
    throw new Error(agenticMessage('planner_precedence_count', { declared: declared.length, coded: PLANNER_ROUTE_ARMS.length }));
  }
  PLANNER_ROUTE_ARMS.forEach(([fn, name], index) => {
    if (declared[index] !== name) {
      throw new Error(agenticMessage('planner_precedence_order', { declared: declared[index], function: fn, name }));
    }
  });
  return declared;
}

/**
 * The workspace_change arm of `fn plan_settled_routes`: a learned
 * workspace-change procedure, or the module-function composition (PR #1188 T1)
 * that is one of them.
 */
async function planWorkspaceChangeArm(task, messages, toolNames) {
  return (await workspaceChange.planWorkspaceChangeStep(task, messages, toolNames))
    ?? (await moduleFunction.planModuleFunctionStep(task, messages, toolNames))
    // A bug report with a stated expectation is checked before anything is
    // rewritten (PR #1188 T93).
    ?? functionExpectation.planFunctionExpectationStep(task, messages, toolNames);
}

/**
 * Mirrors `fn tool_capability` in rust/src/agentic_coding/planner.rs.
 * @param {string} name
 * @returns {string|null}
 */
export function toolCapability(name) {
  return capabilityRouter.classifyTool(name);
}

/**
 * Mirrors `fn plan_chat_step` in rust/src/agentic_coding/planner.rs.
 * @param {Array<object>} messages parsed chat messages
 * @param {Array<string>} toolNames the grounded advertised tool names
 * @returns {Promise<object|null>} an `AgenticPlan` or null
 */
export async function planChatStep(messages, toolNames) {
  const received = latestUserRequest(messages);
  if (received === null) return null;
  const summary = harnessEnvelope.summarizeRequest(received);
  if (summary !== null) return finalAnswer(summary);
  const effective = continuedAgentTask(messages, received);
  const restart = await restartFeedback.planRestart(effective ?? received, messages, toolNames);
  const plan = restart ?? await planChatStepRoutes(messages, toolNames, received);
  if (plan === null) return null;
  return stopRepeatedFailure(stopRepeatedCall(plan, messages), messages);
}

/** Mirrors `fn stop_repeated_call` in rust/src/agentic_coding/planner.rs. */
function stopRepeatedCall(plan, messages) {
  if (!isToolCalls(plan)) return plan;
  const progress = Progress.scan(messages);
  for (const call of plan.calls) {
    const attempt = progress.repeatedCall(call);
    if (attempt) {
      return finalAnswer(fill('stuck_step_report', [
        ['{step}', `${call.tool} ${call.arguments}`],
        ['{result}', attempt.detail.trim()],
      ]));
    }
  }
  return plan;
}

/** Mirrors `fn stop_repeated_failure` in rust/src/agentic_coding/planner.rs. */
function stopRepeatedFailure(plan, messages) {
  const REPEATED_FAILURES_THAT_STOP = 2;
  if (!isToolCalls(plan)) return plan;
  const progress = Progress.scan(messages);
  const repeated = plan.calls.find((call) => progress.identicalFailuresOf(call.tool) >= REPEATED_FAILURES_THAT_STOP);
  if (!repeated) return plan;
  const failure = progress.latestFailureOfTool(repeated.tool);
  if (!failure) return plan;
  const prompt = latestUserRequest(messages) ?? '';
  return finalAnswer(toolResult.renderFailure(repeated.tool, failure.detail, prompt));
}

/** Mirrors `fn plan_chat_step_routes` in rust/src/agentic_coding/planner.rs. */
async function planChatStepRoutes(messages, toolNames, received) {
  checkedRoutePrecedence();
  traceRoute('agentic_received', received);
  const effective = continuedAgentTask(messages, received) ?? received;
  const task = objectiveText(effective);
  traceRoute('agentic_task', task);
  if (handlerMatches('conversation_control', task) || isContinuationCue(task)
    // An edit request's block is its payload: a `when … then` inside it is text
    // being written, not a skill being taught (PR #1188 T57).
    || looksLikeSkillDescription(positionalEdit.ownText(task))) {
    return null;
  }
  const computerUse = computerUsePlanAgenticStep(messages, toolNames);
  if (computerUse !== null) return computerUse;
  if (hasAuthoritativeLiteralWrite(task) && capabilityRouter.workspaceCreationTool(toolNames) !== null) {
    const general = composeGeneralChangePlan(task);
    if (general !== null) return await planGeneralChangeStep(messages, toolNames, general);
  }
  const answer = programContractAnswer(task);
  if (answer !== null) {
    if (answer.execution_recipe && ciWorkflow.requestedIn(task)) ciWorkflow.attach(answer.execution_recipe);
    const rerouted = await commandReroute.planSymbolicCommandReroute(messages, toolNames, answer);
    if (rerouted !== null) return rerouted;
  }
  const evidence = await evidenceRecord.planEvidenceRecordStep(task, messages, toolNames);
  if (evidence !== null) return evidence;
  return planSettledRoutes(task, messages, toolNames);
}

/**
 * Mirrors `fn plan_settled_routes` in rust/src/agentic_coding/planner.rs.
 * @returns {Promise<object|null>}
 */
export async function planSettledRoutes(task, messages, toolNames) {
  for (const arm of [gitCommit.planCommitStep, planWorkspaceChangeArm, codeTask.planGeneratedSourceStep, structuredEdit.planStructuredEditStep, structuredDocument.planStep]) {
    const plan = await arm(task, messages, toolNames);
    if (plan !== null) return plan;
  }
  if (statementAudit.isStatementAuditTask(task)) {
    return planShellStep(messages, toolNames, statementAudit.commandFor(task));
  }
  if (metaSourceTreeRequest(task) !== null) {
    const step = await conversationRecall.planSharedSolverStep(messages, toolNames);
    if (step.kind === 'ready') return step.plan;
  }
  const shellOwned = shellCommand.semanticShellCommandForTask(task) !== null && composeGeneralChangePlan(task) === null;
  const obligations = capabilityRouter.workspaceCreationTool(toolNames) !== null && !shellOwned
    ? taskObligations.obligations(task)
    : null;
  if (obligations !== null) {
    const next = taskObligations.nextStep(task, messages);
    if (next && (next.kind === 'observe' || next.kind === 'decompose')) {
      const general = composeGeneralChangePlan(next.node.clause);
      return general === null ? null : await planGeneralChangeStep(messages, toolNames, general);
    }
    if (next && next.kind === 'report_gap') {
      return finalAnswer(taskObligations.gapAnswer(next.node_id, next.clause, next.span, next.reason));
    }
    if (taskObligations.successfullyDischarged(task, messages)) {
      for (const obligation of [...obligations].reverse()) {
        const general = composeGeneralChangePlan(obligation.clause);
        if (general !== null) return await planGeneralChangeStep(messages, toolNames, general);
      }
      return null;
    }
    return null;
  }
  if (capabilityRouter.workspaceCreationTool(toolNames) !== null) {
    const general = composeGeneralChangePlan(task);
    if (general !== null) return await planGeneralChangeStep(messages, toolNames, general);
  }
  const algorithm = algorithmLearning.compileTask(task);
  if (algorithm !== null) return algorithmLearning.planStep(messages, toolNames, algorithm);
  const compiled = procedure.compileTask(task);
  if (compiled !== null) return procedure.planStep(messages, toolNames, compiled);
  const report = learningReport.route(task);
  if (report !== null) return learningReport.planReportStep(report, messages, toolNames);
  const artifact = await codeArtifact.planCodeArtifactStep(task, messages, toolNames);
  if (artifact !== null) return artifact;
  for (const [predicate, step] of [
    [selfHeal.isSelfHealTask, documentRecipe.planSelfHealStep],
    [dreamingAudit.isDreamingAuditTask, documentRecipe.planDreamingAuditStep],
    [selfAst.isSelfAstTask, documentRecipe.planSelfAstStep],
    [sourceLinks.isSourceLinksTask, documentRecipe.planSourceLinksStep],
    [ledger.isLedgerTask, documentRecipe.planLedgerStep],
    [explain.isExplainTask, documentRecipe.planExplainStep],
    [changeRequest.isChangeRequestTask, documentRecipe.planChangeRequestStep],
    [repairStrategy.isRepairStrategyTask, documentRecipe.planRepairStrategyStep],
    [rebuildPlan.isRebuildTask, documentRecipe.planRebuildStep],
    [googleTrendsLearning.isGoogleTrendsLearningTask, documentRecipe.planGoogleTrendsLearningStep],
    [googleTrendsCatalog.isGoogleTrendsCatalogTask, documentRecipe.planGoogleTrendsCatalogStep],
    [questionCatalog.isQuestionCatalogTask, documentRecipe.planQuestionCatalogStep],
  ]) {
    if (predicate(task)) return await step(messages, toolNames);
  }
  const analysis = fileReadTaskFor(task);
  if (analysis !== null && analysis.isAnalysis()) return await planFileReadStep(analysis, messages, toolNames);
  // Where a name or literal is used inside the workspace is a content search
  // (PR #1188 T90): grep it, ahead of the file-name locate arm and web search.
  const search = workspaceSearch.planWorkspaceSearchStep(task, messages, toolNames)
    // What a named module exports is answered from its declarations (T99).
    ?? moduleExports.planModuleExportsStep(task, messages, toolNames)
    // A summary of a named file summarizes the file's text (T100).
    ?? await fileSummary.planFileSummaryStep(task, messages, toolNames);
  if (search !== null) return search;
  const reportFlow = await reportIssue.planReportFlow(messages, toolNames);
  if (reportFlow !== null) return reportFlow;
  const shared = await conversationRecall.planSharedSolverStep(messages, toolNames);
  if (shared.kind === 'ready') return shared.plan;
  if (shared.kind === 'defer') return null;
  return planLaterRoutes(task, messages, toolNames);
}

/** The second half of `plan_settled_routes`, from `follow_up_answer` on. */
async function planLaterRoutes(task, messages, toolNames) {
  const followUp = toolResult.followUpAnswer(messages, task);
  if (followUp !== null) return finalAnswer(followUp);
  const clarification = webResearch.contextualReferenceClarification(task);
  if (clarification !== null) return finalAnswer(clarification);
  if (webResearch.isDefinitionFollowup(task)) {
    const query = await webResearch.definitionFollowupTopic(messages, task);
    if (query === null) return finalAnswer(webResearch.definitionFollowupClarification(task));
    const plan = await webResearch.planWebResearchStep(messages, toolNames, query, true);
    if (plan !== null) return plan;
  }
  const edit = await intentRouter.planEditStep(task, messages, toolNames);
  if (edit !== null) return edit;
  if (toolFor(toolNames, Capability.Read) !== null) {
    const fileTask = fileReadTaskFor(task);
    if (fileTask !== null) return await planFileReadStep(fileTask, messages, toolNames);
  }
  for (const step of [
    () => localSearch.planLocalSearchStep(messages, toolNames),
    () => comparison.planComparisonStep(task, messages, toolNames),
    () => capabilityRouter.planNamedCapabilityStep(task, messages, toolNames),
  ]) {
    const plan = await step();
    if (plan !== null) return plan;
  }
  // A destructive shell intent read against a request about text inside a
  // file is declined honestly, never composed (PR #1188).
  const refused = shellCommand.refusedDestructiveEdit(task);
  if (refused !== null) {
    const decline = codeTask.renderSeededChange('file_text_unit', task, refused, []);
    if (decline !== null) return finalAnswer(decline);
  }
  const command = shellCommand.shellCommandForTask(task);
  if (command !== null) {
    const fallback = await shellFileFallback.planStep(task, messages, toolNames, command);
    if (fallback !== null) return fallback;
    const mutating = await mutatingAction.planStep(command, messages, toolNames, task);
    if (mutating !== null) return mutating;
    return planShellStep(messages, toolNames, command);
  }
  const fileTask = fileReadTaskFor(task);
  if (fileTask !== null) return await planFileReadStep(fileTask, messages, toolNames);
  if (formalizationRecipe.isFormalizationTask(task)) {
    return await formalizationRecipe.planFormalizationStep(task, messages, toolNames);
  }
  if (meaningDetail.isMeaningDetailTask(task)) return await documentRecipe.planMeaningDetailStep(task, messages, toolNames);
  if (diagram.isDiagramTask(task)) return await documentRecipe.planDiagramStep(messages, toolNames);
  return planOpenRoutes(task, messages, toolNames);
}

/** The tail of `plan_settled_routes`, from `workspace_inspection` on. */
async function planOpenRoutes(task, messages, toolNames) {
  if (!toolResult.hasLatestTurnResult(messages)) {
    const search = workspaceInspection.workspaceInspectionSearchForTask(task);
    const tool = search === null ? null : toolFor(toolNames, Capability.Grep);
    if (tool !== null) {
      const args = { query: search.query, pattern: search.pattern };
      if (search.include !== null && search.include !== undefined) args.include = search.include;
      return planOne(tool, jsonText(args));
    }
  }
  const structure = await taskStructure.planTaskStructureStep(messages, task);
  if (structure !== null) return structure;
  if (!webResearch.hasSuccessfulSearchResult(messages)) {
    const routed = await capabilityRouter.planRoutedCapabilityStep(task, messages, toolNames, capabilityRouter.RoutingStage.NamedOrLocal);
    if (routed !== null) return routed;
  }
  if (positionalEdit.namesLocalEdit(task)) return null;
  const researchQuery = await webResearch.webResearchQueryFor(messages);
  if (researchQuery !== null) {
    const plan = await webResearch.planWebResearchStep(messages, toolNames, researchQuery, false);
    if (plan !== null) return plan;
  }
  const webSearch = await intentRouter.planWebSearchStep(task, messages, toolNames);
  if (webSearch !== null) return webSearch;
  if (!toolResult.hasLatestTurnResult(messages)) {
    const query = shellCommand.codeSearchQueryForTask(task);
    const tool = query === null ? null : toolFor(toolNames, Capability.Grep);
    if (tool !== null) return planOne(tool, jsonText({ query, pattern: query }));
  }
  if (webResearch.hasSuccessfulSearchResult(messages)) {
    const query = await webResearch.midResearchWebQueryFor(messages);
    if (query !== null) {
      const plan = await webResearch.planWebResearchStep(messages, toolNames, query, false);
      if (plan !== null) return plan;
    }
  }
  const latest = await toolResult.latestTurnAnswer(messages, toolNames, task);
  if (latest !== null) return finalAnswer(latest);
  const note = await noteComposition.planNoteCompositionStep(task, messages);
  if (note !== null) return note;
  const general = composeGeneralChangePlan(task);
  if (general !== null) return await planGeneralChangeStep(messages, toolNames, general);
  const unresolved = await webResearch.unresolvedWebResearchQueryFor(messages);
  if (unresolved !== null) {
    const plan = await webResearch.planWebResearchStep(messages, toolNames, unresolved, false);
    if (plan !== null) return plan;
  }
  return capabilityRouter.planRoutedCapabilityStep(task, messages, toolNames, capabilityRouter.RoutingStage.OpenWeb);
}

/**
 * Mirrors `fn plan_shell_step` in rust/src/agentic_coding/planner.rs.
 * @returns {object} an `AgenticPlan`
 */
export function planShellStep(messages, toolNames, command) {
  const progress = Progress.scan(messages);
  if (progress.done(Capability.Run)) {
    const outputs = progress.run_outputs;
    return finalAnswer(toolResult.render(command, outputs.length ? outputs[outputs.length - 1] : '',
      latestUserRequest(messages) ?? ''));
  }
  const tool = toolFor(toolNames, Capability.Run);
  if (tool !== null) return planOne(tool, jsonText({ command }));
  return finalAnswer(agenticMessage('shell_tool_missing', { command }));
}
