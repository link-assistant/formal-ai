// Additive authoring over declared ordinary Read and replacement Write capabilities.
import { declaredAdditionContract, ownedAdditiveLiteralFrame } from './planner/owned_goals.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { Progress } from './progress.mjs';
import { readArguments } from './workspace_change.mjs';
import { writeArguments, planOne, resolvedFinalAnswer, FinalDisposition } from './plan.mjs';
import { renderSeededChange, renderSeededOutcome } from './code_task.mjs';

const refused = (task, target, disposition = FinalDisposition.Gap) => resolvedFinalAnswer(
  renderSeededOutcome('file-addition-unverified', task, target) ?? target,
  disposition, 'literal-addition-unverified'
);

/** Mirrors plan_literal_addition_step; caller metadata cannot manufacture an append primitive. */
export function planLiteralAdditionStep(task, messages, toolNames) {
  if (declaredAdditionContract(task) === null) return null;
  const contract = ownedAdditiveLiteralFrame(task);
  if (contract === null) return null;
  const { target, content, atEnd } = contract;
  if (atEnd === null) return resolvedFinalAnswer(
    renderSeededOutcome('file-addition-position-unknown', task, target) ?? target,
    FinalDisposition.Gap, 'literal-addition-position-unknown'
  );
  const read = toolFor(toolNames, Capability.Read), write = toolFor(toolNames, Capability.Write);
  if (read === null || write === null) return refused(task, target);
  const progress = Progress.scan(messages);
  const written = progress.successfulWriteContentFor(target);
  const before = written === null ? progress.sourceReadFor(target) : progress.sourceReadBeforeLatestWriteFor(target);
  if (before === null) return written === null ? planOne(read, readArguments(target)) : refused(task, target);
  const absent = before.absent === true;
  if (!absent && (before.error !== null || !before.complete || typeof before.source !== 'string')) return refused(task, target);
  const source = absent ? '' : before.source;
  const expected = source === '' ? content + '\n' : !atEnd ? content + '\n' + source
    : source + (source.endsWith('\n') ? '' : '\n') + content + '\n';
  if (written === null) {
    if (progress.attemptedWriteFor(target)) return refused(task, target, FinalDisposition.Failure);
    return planOne(write, writeArguments(target, expected));
  }
  if (written !== expected) return refused(task, target);
  const after = progress.sourceReadFor(target);
  if (!progress.attemptsAfterLatestWrite(target)?.some(attempt => attempt.capability === Capability.Read)) {
    return planOne(read, readArguments(target));
  }
  if (after === null || after.error !== null || !after.complete || after.source !== expected) return refused(task, target);
  const intent = atEnd ? 'file_edit_position_end' : 'file_edit_position_start';
  return resolvedFinalAnswer(renderSeededChange(intent, task, target, [['{new}', content]]) ?? target,
    FinalDisposition.Finding, 'literal-addition-observed');
}
