// Source-owned mixed actions share the existing obligation nodes and request replay.
import { composeEditClauses } from '../write_request.mjs';
import { literalWriteOwnership, instructionView } from '../general_planner.mjs';
import { sentences } from '../shell_command_policy.mjs';
import { quotedSegmentSpans, quoteFault } from '../crate/normal_markov.mjs';
import { nestedQuoteFault } from '../quote_nesting.mjs';
import { leafNode } from '../crate/obligation_ledger.mjs';
import { gapAnswer } from '../task_obligations.mjs';
import { planBoundRequestSteps } from '../request_sequence.mjs';
import { FinalDisposition, canDeliverFinal, resolvedFinalAnswer } from '../plan.mjs';

const encoder = new TextEncoder();
const grammarTail = (text) => /^[\s.!?。！？।;；]*$/u.test(text);

function closedPayload(request, contract) {
  return contract === null ? null : quotedSegmentSpans(request).find((span) =>
    span.start >= contract.payload.start && /^[\s:]*$/u.test(request.slice(contract.payload.start, span.start))) ?? null;
}

/** Mirrors fn goal_ledger: preserve raw UTF16 positions and the existing node's UTF8 source span. */
export function goalLedger(request) {
  const contract = literalWriteOwnership(request);
  let view = closedPayload(request, contract) === null ? instructionView(request, contract) : request;
  if (view === null) return null;
  for (const span of quotedSegmentSpans(request)) view = view.slice(0, span.start) + ' '.repeat(span.end - span.start) + view.slice(span.end);
  const goals = sentences(view).map((sentence) => {
    const raw = request.slice(sentence.span.start, sentence.span.end);
    const clause = raw.trim();
    const start = sentence.span.start + raw.indexOf(clause);
    const span = { start, end: start + clause.length };
    const byteSpan = [encoder.encode(request.slice(0, span.start)).length, encoder.encode(request.slice(0, span.end)).length];
    const literal = literalWriteOwnership(clause);
    const edit = literal === null ? composeEditClauses(clause) : null;
    const payloadEnd = closedPayload(clause, literal)?.end ?? literal?.payload.end;
    const completeLiteral = literal !== null && grammarTail(clause.slice(payloadEnd));
    const kind = completeLiteral ? 'literal_file' : edit !== null && edit.spans !== null ? 'source_edit' : 'unsupported';
    const target = literal?.target ?? edit?.edit[0] ?? null;
    const expectation = kind === 'literal_file' ? { kind: 'file_bytes', path: target, sha256: null }
      : { kind: 'underivable', reason: kind === 'source_edit' ? 'source-edit-preimage-required' : 'no_artifact_in_clause' };
    return { clause, span, byteSpan, sourceUnit: 'utf16', kind, target, expected: literal?.content ?? null,
      node: leafNode(null, clause, byteSpan, 0, expectation) };
  });
  if (goals.length === 0 || goals.length === 1 && goals[0].kind !== 'unsupported') return null;
  return goals.some((goal) => literalWriteOwnership(goal.clause) !== null) ? goals : null;
}

function goalGap(goal) {
  return resolvedFinalAnswer(gapAnswer(goal.node.node_id, goal.clause, goal.byteSpan, 'no_artifact_in_clause'),
    FinalDisposition.Gap, 'owned-goal-missing-contract');
}

/** Mirrors `fn plan_owned_goal_step`: existing literal-only scheduling remains authoritative. */
export async function planGoalLedger(request, messages, toolNames, planFor) {
  const goals = goalLedger(request);
  if (goals === null || quoteFault(request) !== null || nestedQuoteFault(request) !== null) return null;
  const missingIndex = goals.findIndex((goal) => goal.kind === 'unsupported');
  if (missingIndex >= 0) {
    const missing = goals[missingIndex];
    if (literalWriteOwnership(missing.clause) !== null) return goalGap(missing);
    if (missingIndex === 0) return null;
    const plan = await planBoundRequestSteps(goals.slice(0, missingIndex).map((goal) => goal.clause), messages, toolNames, planFor);
    return plan?.kind === 'final' && canDeliverFinal(plan) ? goalGap(missing) : plan;
  }
  if (!goals.some((goal) => goal.kind === 'source_edit')) return null;
  return planBoundRequestSteps(goals.map((goal) => goal.clause), messages, toolNames, planFor);
}
