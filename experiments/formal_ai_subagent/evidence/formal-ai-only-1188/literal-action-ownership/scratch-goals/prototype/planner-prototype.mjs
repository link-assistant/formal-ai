import {planGoalLedger} from './goal-ledger.mjs';
import {ownedSemanticAuthoringLead} from './operation-owner.mjs';
import {instructionView,literalWriteOwnership} from './write-contract.mjs';
// Deterministic agentic planner: the next tool call or final answer from a
// conversation and its advertised tools, without hidden neural state
// (rust/src/agentic_coding/planner.rs).
//
// Every route arm is the twin of the Rust arm of the same name, run in the
// order data/seed/planner-precedence.lino declares. Arms that consult the
// host solver are async, so the cascade awaits each arm.

import * as algorithmLearning from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/algorithm_learning.mjs';
import { Capability } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/capability.mjs';
import * as capabilityRouter from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/capability_router.mjs';
import * as changeRequest from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/change_request.mjs';
import * as ciWorkflow from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/ci_workflow.mjs';
import * as codeArtifact from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/code_artifact.mjs';
import * as codeTask from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/code_task.mjs';
import * as commandReroute from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/command_reroute.mjs';
import * as comparison from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/comparison.mjs';
import { programContractAnswer } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/coding_program_contract.mjs';
import { sourceTreeRequest as metaSourceTreeRequest } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/meta_translate.mjs';
import { handlerMatches } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/rule_interpreter.mjs';
import { quoteFault } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/normal_markov.mjs';
import * as testAssertion from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/test_assertion.mjs';
import * as replaceList from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/replace_list.mjs';
import * as quoteNesting from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/quote_nesting.mjs';
import * as requestSequence from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/request_sequence.mjs';
import { plannerPrecedence } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/seed.mjs';
import { looksLikeSkillDescription } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/skill_compiler.mjs';
import { computerUsePlanAgenticStep } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/computer_use_planner.mjs';
import { latestUserRequest } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/content.mjs';
import * as conversationRecall from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/conversation_recall.mjs';
import * as diagram from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/diagram.mjs';
import * as documentRecipe from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/document_recipe.mjs';
import * as dreamingAudit from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/dreaming_audit.mjs';
import * as evidenceRecord from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/evidence_record.mjs';
import * as fileSummary from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/file_summary.mjs';
import * as explain from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/explain.mjs';
import { fileReadTaskFor, planFileReadStep } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/file_read.mjs';
import * as formalizationRecipe from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/formalization_recipe.mjs';
import * as functionExpectation from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/function_expectation.mjs';
import { planGeneralChangeStep } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/general_execution.mjs';
import { composeEditRequest, composeGeneralChangePlan, hasAuthoritativeLiteralWrite, objectiveText, semanticAuthoringLead } from './write-contract.mjs';
import * as gitCommit from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/git_commit.mjs';
import * as googleTrendsCatalog from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/google_trends_catalog.mjs';
import * as googleTrendsLearning from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/google_trends_learning.mjs';
import * as harnessEnvelope from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/harness_envelope.mjs';
import * as intentRouter from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/intent_router.mjs';
import * as learningReport from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/learning_report.mjs';
import * as ledger from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/ledger.mjs';
import * as localSearch from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/local_search.mjs';
import * as meaningDetail from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/meaning_detail.mjs';
import * as moduleExports from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/module_exports.mjs';
import * as moduleFunction from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/module_function.mjs';
import * as mutatingAction from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/mutating_action.mjs';
import * as noteComposition from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/note_composition.mjs';
import { agenticMessage } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/messages.mjs';
import { FinalDisposition, finalAnswer, isToolCalls, jsonText, planOne, projectPlan, resolvedFinalAnswer } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/plan.mjs';
import { continuedAgentTask, isContinuationCue, traceRoute } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/planner/continuation.mjs';
import * as positionalEdit from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/positional_edit.mjs';
import * as procedure from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/procedure.mjs';
import { Progress } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/progress.mjs';
import * as questionCatalog from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/question_catalog.mjs';
import * as rebuildPlan from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/rebuild_plan.mjs';
import * as repairStrategy from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/repair_strategy.mjs';
import * as reportIssue from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/report_issue.mjs';
import * as restartFeedback from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/restart_feedback.mjs';
import * as selfAst from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/self_ast.mjs';
import * as selfHeal from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/self_heal.mjs';
import * as shellCommand from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/shell_command.mjs';
import * as shellFileFallback from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/shell_file_fallback.mjs';
import * as sourceLinks from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/source_links.mjs';
import * as statementAudit from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/statement_audit.mjs';
import * as structuredDocument from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/structured_document.mjs';
import * as structuredEdit from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/structured_edit.mjs';
import * as taskObligations from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/task_obligations.mjs';
import { planObligationsStep } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/planner/obligations.mjs';
import * as taskStructure from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/task_structure.mjs';
import * as toolResult from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/tool_result.mjs';
import * as webResearch from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/web_research.mjs';
import { fill } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/work_item_steps.mjs';
import * as workspaceChange from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/workspace_change.mjs';
import * as workspaceInspection from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/workspace_inspection.mjs';
import * as workspaceSearch from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/workspace_search.mjs';

const { toolFor } = capabilityRouter;

/**
 * Mirrors `PLANNER_ROUTE_ARMS` in rust/src/agentic_coding/planner/precedence.rs: the
 * route arms of the cascade, named and in run order.
 */
export const PLANNER_ROUTE_ARMS = [
  ['plan_chat_step_routes', 'conversation-control-decline'],
  ['plan_chat_step_routes', 'computer_use'],
  ['plan_chat_step_routes', 'authoritative-literal-write'],
  ['plan_chat_step_routes', 'program_contract'],
  ['plan_chat_step_routes', 'evidence_record'],
  ...['git_commit', 'workspace_change', 'generated-source', 'structured_edit', 'structured_document',
    'statement_audit', 'task_obligations', 'literal-write', 'algorithm_learning', 'procedure',
    'learning_report', 'code_artifact', 'self_heal', 'dreaming_audit', 'self_ast', 'source_links',
    'learning_ledger', 'explain', 'change_request', 'repair_strategy', 'rebuild_plan',
    'google_trends_learning', 'google_trends_catalog', 'question_catalog', 'file-analysis',
    'report-flow', 'conversation_recall', 'follow_up_answer', 'contextual_reference_clarification',
    'definition-followup', 'intent-edit', 'typed-file-read', 'local_search', 'comparison',
    'named-capability-table', 'shell_command', 'file_read', 'formalization_recipe', 'meaning_detail',
    'diagram', 'workspace_inspection', 'task_structure', 'capability-table-named-or-local',
    'positional-edit-decline', 'web-research-query', 'intent_web_search', 'code-search-fallback',
    'research-continuation', 'latest_turn_answer', 'note_composition', 'general-change-fallback',
    'web-research-final', 'capability-table-open-web'].map((arm) => ['plan_settled_routes', arm]),
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
  // A copy or move followed by edits of the file it makes is planned sentence
  // by sentence (PR #1188 G82).
  return (await requestSequence.planRequestSequenceStep(task, messages, toolNames, planChatStepResolved))
    ?? (await workspaceChange.planWorkspaceChangeStep(task, messages, toolNames))
    ?? (await moduleFunction.planModuleFunctionStep(task, messages, toolNames))
    // A bug report with a stated expectation is checked before anything is
    // rewritten (PR #1188 T93).
    ?? functionExpectation.planFunctionExpectationStep(task, messages, toolNames)
    // An assertion of a stated call and value is added in the test file's own
    // form, and the file is run (PR #1188 G13).
    ?? testAssertion.planTestAssertionStep(task, messages, toolNames)
    // A test asked for with no expected result is a question (PR #1188 G25).
    ?? functionExpectation.planTestExpectationQuestion(task);
}

/**
 * Mirrors `fn request_fault_answer`: the seeded answer declining a request
 * whose quotes do not pair (PR #1188 G71) or whose edit names several files
 * (G91), or null.
 * @param {string} task
 */
function requestFaultAnswer(task, allowMultipleFiles = false) {
  // Formal query escapes are validated by its own grammar before prose pairing.
  if (codeArtifact.explicitSubstitutionQuery(task) !== null
    || shellCommand.explicitPassthroughCommand(task) !== null) return null;
  const fault = quoteFault(task) ?? quoteNesting.nestedQuoteFault(task);
  if (fault !== null) {
    const answer = codeTask.renderSeededChange(fault.intent ?? `request_quote_${fault.kind}`, task, '', [['{fragment}', fault.fragment]]);
    return answer === null ? null : finalAnswer(answer);
  }
  // Steps joined by a sequence cue are each checked alone (PR #1188 G99).
  if (requestSequence.sequenceSteps(task) !== null) return null;
  const files = replaceList.severalEditTargets(task);
  if (files !== null && allowMultipleFiles) return null;
  const clause = files === null ? replaceList.unplannedEditClause(task) : null;
  const answer = files !== null
    ? codeTask.renderSeededListChange('request_several_edit_targets', task, '', '{files}', files)
    : clause !== null
      ? codeTask.renderSeededChange('request-edit-clause-unplanned', task, '', [['{clause}', clause]])
      : null;
  return answer === null ? null : finalAnswer(answer);
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
  return projectPlan(await planChatStepResolved(messages, toolNames));
}

/** Internal planner result; recursion retains disposition and origin. */
export async function planChatStepResolved(messages, toolNames) {
  const received = latestUserRequest(messages);
  if (received === null) return null;
  const summary = harnessEnvelope.summarizeRequest(received);
  if (summary !== null) return resolvedFinalAnswer(summary, FinalDisposition.Finding, 'harness_summary');
  const effective = continuedAgentTask(messages, received);
  const restart = await restartFeedback.planRestart(effective ?? received, messages, toolNames);
  const plan = restart ?? await planChatStepRoutes(messages, toolNames, received);
  if (plan === null) return null;
  return stopRepeatedFailure(stopRepeatedCall(plan, messages), messages);
}

/** Mirrors `fn stop_repeated_call` in rust/src/agentic_coding/planner/steps.rs. */
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

/** Mirrors `fn stop_repeated_failure` in rust/src/agentic_coding/planner/steps.rs. */
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
    || (!hasAuthoritativeLiteralWrite(task) && composeEditRequest(task) === null
      && looksLikeSkillDescription(positionalEdit.ownText(task)))) {
    return null;
  }
  const ownedGoals = await planGoalLedger(task, messages, toolNames, planChatStepResolved);
  if (ownedGoals !== null) return ownedGoals;
  // The computer_use arm. Ahead of it, quotes that do not pair leave no
  // telling the quoted text from the instruction, so the request is declined
  // before any arm reads its payload as words to act on (PR #1188 G71).
  const computerUse = requestFaultAnswer(task, toolFor(toolNames, Capability.MultiEdit) !== null) ?? computerUsePlanAgenticStep(messages, toolNames);
  if (computerUse !== null) return computerUse;
  if (hasAuthoritativeLiteralWrite(task) && capabilityRouter.workspaceCreationTool(toolNames) !== null) {
    const obligations = taskObligations.obligations(task);
    if (obligations !== null) return await planObligationsStep(task, messages, toolNames, obligations);
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
  const explicit = shellCommand.explicitPassthroughCommand(task);
  if (explicit !== null && toolFor(toolNames, Capability.Run) !== null) {
    return await shellFileFallback.planStep(task, messages, toolNames, explicit)
      ?? await mutatingAction.planStep(explicit, messages, toolNames, task)
      ?? planShellStep(messages, toolNames, explicit);
  }
  for (const arm of [gitCommit.planCommitStep, planWorkspaceChangeArm, codeTask.planGeneratedSourceStep, structuredEdit.planStructuredEditStep, structuredDocument.planStep]) {
    const plan = await arm(instructionView(task, literalWriteOwnership(task)), messages, toolNames);
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
    return await planObligationsStep(task, messages, toolNames, obligations);
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
  const artifact = await codeArtifact.planCodeArtifactStep(task, messages, toolNames)
    ?? missingSemanticImplementation(task);
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
  // The file-analysis arm. After its typed read, where a name or literal is
  // used inside the workspace is a content search (PR #1188 T90): grep it,
  // ahead of the file-name locate arm and web search.
  const analysis = fileReadTaskFor(task);
  const search = (analysis !== null && analysis.isAnalysis() ? await planFileReadStep(analysis, messages, toolNames) : null)
    ?? workspaceSearch.planWorkspaceSearchStep(task, messages, toolNames)
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
  if (clarification !== null) return resolvedFinalAnswer(clarification, FinalDisposition.Clarification, 'contextual_reference_clarification');
  if (webResearch.isDefinitionFollowup(task)) {
    const query = await webResearch.definitionFollowupTopic(messages, task);
    if (query === null) return resolvedFinalAnswer(webResearch.definitionFollowupClarification(task), FinalDisposition.Clarification, 'definition_followup_clarification');
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
    if (decline !== null) return resolvedFinalAnswer(decline, FinalDisposition.Gap, 'destructive_edit_declined');
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
  // An addition that quotes no text earns a question naming what is missing
  // (PR #1188 G69); any other unplanned local edit is declined.
  const unquoted = positionalEdit.unquotedAdditionPath(task);
  if (unquoted !== null || positionalEdit.namesLocalEdit(task)) {
    const question = unquoted === null ? null : codeTask.renderSeededChange('file_addition_unquoted', task, unquoted, []);
    return question === null ? null : resolvedFinalAnswer(question, FinalDisposition.Clarification, 'file_addition_unquoted');
  }
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
    const raw = outputs.length ? outputs[outputs.length - 1] : '';
    return resolvedFinalAnswer(toolResult.render(command, raw, latestUserRequest(messages) ?? ''),
      toolResult.stepOutcome(raw) === toolResult.StepOutcome.Failed
        ? FinalDisposition.Failure : FinalDisposition.Finding, 'shell_result_observed');
  }
  const tool = toolFor(toolNames, Capability.Run);
  if (tool !== null) return planOne(tool, jsonText({ command }));
  return finalAnswer(agenticMessage('shell_tool_missing', { command }));
}

function missingSemanticImplementation(task) {
  if (ownedSemanticAuthoringLead(task) && composeGeneralChangePlan(task) === null && composeEditRequest(task) === null) {
    const discovery = { reason: 'MissingContract', goal: task, authored: false, verified: false,
      missingContracts: ['source-bound-implementation-plan', 'independent-goal-validation'] };
    const plan = resolvedFinalAnswer(agenticMessage('callable-discovery-outcome', {
      reason: discovery.reason, discovery: jsonText(discovery),
    }), FinalDisposition.Gap, 'semantic-authoring-missing-contract');
    plan.result.discovery = discovery;
    return plan;
  }
  return null;
}
