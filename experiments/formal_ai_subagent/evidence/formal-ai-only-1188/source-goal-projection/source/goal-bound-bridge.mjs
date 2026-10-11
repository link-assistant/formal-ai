// Reviewed generic scratch safety prototype, not source-plan synthesis.
import { latestUserRequest } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/content.mjs';
import { sentences } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/shell_command_policy.mjs';
import { quotedSegmentSpans } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/normal_markov.mjs';
import { roleWordForms } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/write_lexicon.mjs';
import { fileReadTaskFor, readArguments } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/file_read.mjs';
import { shellCommandForTask } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/shell_command.mjs';
import { Progress } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/progress.mjs';
import { SourceReadStatus } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/tool_result.mjs';
import { FinalDisposition, resolvedFinalAnswer } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/final_result.mjs';
import { toolCalls, plannedCall } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/plan.mjs';
import { Capability } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/capability.mjs';
import { toolFor } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/capability_router.mjs';
import { agenticMessage } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/messages.mjs';
import { sha256Hex } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/source_fetch.mjs';

const encoder = new TextEncoder();
function ownedClauses(request) {
  const units = request.split('');
  for (const span of quotedSegmentSpans(request)) {
    for (let index = span.start; index < span.end; index += 1) {
      if (/[.!?;\n。！？；।]/u.test(units[index])) units[index] = ' ';
    }
  }
  return sentences(units.join('')).map(({ span }) => ({
    text: request.slice(span.start, span.end).trim(),
    span: [encoder.encode(request.slice(0, span.start)).length,
      encoder.encode(request.slice(0, span.end)).length],
  }));
}
function ownsReadLead(clause) {
  const lowered = clause.trimStart().toLowerCase();
  return roleWordForms('file_read_action_cue').some((form) => {
    if (form.slot !== 'bare') return false;
    const cue = form.text.toLowerCase();
    if (!lowered.startsWith(cue)) return false;
    const next = Array.from(lowered.slice(cue.length))[0];
    return next === undefined || !/[\p{L}\p{N}\p{M}_-]/u.test(next)
      || /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]/u.test(cue);
  });
}
export function requestGraph(request) {
  const clauses = ownedClauses(request);
  const dependencies = [];
  const goals = [];
  const inputGaps = [];
  for (const clause of clauses) {
    const task = ownsReadLead(clause.text) ? fileReadTaskFor(clause.text) : null;
    if (task && ['direct', 'direct_many'].includes(task.kind) && task.mode.kind === 'full') {
      for (const path of task.kind === 'direct' ? [task.path] : task.paths) {
        if (!dependencies.some((dependency) => dependency.path === path)) {
          dependencies.push({ kind: 'source-input', path, span: clause.span });
        }
      }
    } else if (ownsReadLead(clause.text)) {
      inputGaps.push({ span: clause.span, missing: 'source-input-contract' });
    } else {
      const command = shellCommandForTask(clause.text);
      goals.push(command === null
        ? { kind: 'unsupported-source-goal', span: clause.span, missing: 'goal-composition-contract' }
        : { kind: 'verified-run', span: clause.span,
          expectation: { kind: 'command_exit', command, expected_exit: 0 },
          missing: 'owned-root-command-evidence' });
    }
  }
  return { startsWithOwnedRead: Boolean(clauses[0] && ownsReadLead(clauses[0].text)), dependencies, goals, inputGaps };
}
function gap(disposition, missingContracts, observations, goals) {
  const discovery = { reason: 'MissingContract', authored: false, verified: false,
    missingContracts, observations, goals };
  const result = resolvedFinalAnswer(agenticMessage('callable-discovery-outcome', {
    reason: discovery.reason, discovery: JSON.stringify(discovery),
  }), disposition, 'goal-bound-source-observation');
  result.result.discovery = discovery;
  return result;
}
export async function planGoalBound(messages, tools, next) {
  const graph = requestGraph(latestUserRequest(messages) ?? '');
  if (!graph.startsWithOwnedRead || !graph.goals.length) {
    return next(messages, tools);
  }
  if (graph.inputGaps.length) return gap(FinalDisposition.Gap, ['source-input-contract'], [], graph.goals);
  const progress = Progress.scan(messages);
  const observations = graph.dependencies.map(({ path, span }) => {
    const read = progress.sourceReadFor(path);
    return { path, span, status: read?.status ?? SourceReadStatus.Unknown,
      complete: read?.complete ?? false,
      utf8Bytes: read?.source === null || read?.source === undefined ? null : encoder.encode(read.source).length,
      contentIdentity: read?.source === null || read?.source === undefined ? null
        : sha256Hex(encoder.encode(read.source)) };
  });
  if (observations.some((observation) => observation.status === SourceReadStatus.Failure)) {
    return gap(FinalDisposition.Failure, ['failed-source-provider'], observations, graph.goals);
  }
  const missing = graph.dependencies.filter(({ path }) => progress.sourceReadFor(path) === null);
  if (missing.length) {
    const readTool = toolFor(tools, Capability.Read);
    return readTool === null
      ? gap(FinalDisposition.Gap, ['source-provider-unavailable'], observations, graph.goals)
      : toolCalls(missing.map(({ path }) => plannedCall(readTool, readArguments(path, { kind: 'full' }))));
  }
  if (observations.some((observation) => !observation.complete
    || observation.status !== SourceReadStatus.Success)) {
    return gap(FinalDisposition.Gap, ['complete-bound-source-observation'], observations, graph.goals);
  }
  return gap(FinalDisposition.Gap, [...new Set(graph.goals.map((goal) => goal.missing))], observations, graph.goals);
}
