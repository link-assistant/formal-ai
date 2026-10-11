// A record's persisted bytes require observation after its current Write.
import { Capability } from '../capability.mjs';
import { toolFor } from '../capability_router.mjs';
import { sourceFromReadResult } from '../code_artifact.mjs';
import { renderSeededOutcome } from '../code_task.mjs';
import { shellQuote } from '../general_planner.mjs';
import { FinalDisposition, jsonText, planOne, resolvedFinalAnswer } from '../plan.mjs';
import { commandArgument, harnessReportedFailure, observedBytesMatch, renderFailure, reportedExitCode } from '../tool_result.mjs';

/** Mirrors `fn record_readback_command`: quote unsafe path bytes as one operand. */
export function recordReadbackCommand(target) {
  return /^[A-Za-z0-9_./-]+$/u.test(target) && !target.startsWith('-')
    ? 'cat ' + target : 'cat -- ' + shellQuote(target);
}

/** Mirrors `fn attempt_targets`: exact authoritative path, never a filename suffix. */
function attemptTargets(attempt, target) {
  try {
    const value = JSON.parse(attempt.arguments);
    const key = ['path', 'filePath', 'file_path'].find((name) => typeof value?.[name] === 'string');
    return key !== undefined && value[key] === target;
  } catch { return false; }
}

/** Mirrors `fn record_observation_outcome`: acknowledgements and prior reads remain pending. */
export function recordObservationOutcome(progress, target, content) {
  const attempts = progress.attemptsAfterLatestWrite(target);
  if (content === null || attempts === null) return { kind: 'pending' };
  const command = recordReadbackCommand(target);
  for (const attempt of [...attempts].reverse()) {
    const reads = attempt.capability === Capability.Read && attemptTargets(attempt, target);
    const runs = attempt.capability === Capability.Run && commandArgument(attempt.arguments ?? '') === command;
    const writes = attempt.capability === Capability.Write && attemptTargets(attempt, target);
    if (!reads && !runs && !writes) continue;
    if (!attempt.succeeded || writes || harnessReportedFailure(attempt.detail)
      || (reportedExitCode(attempt.detail) !== null && reportedExitCode(attempt.detail) !== 0)) {
      return { kind: 'failed', label: runs ? command : attempt.tool ?? target, detail: attempt.detail };
    }
    const exact = runs ? observedBytesMatch(attempt.detail, content)
      : sourceFromReadResult(attempt.detail) === content || observedBytesMatch(attempt.detail, content);
    return { kind: exact ? 'verified' : 'mismatch' };
  }
  return { kind: 'pending' };
}

/** Mirrors `fn plan_record_readback_step`: null only when the current record bytes are verified. */
export function planRecordReadbackStep(task, target, content, progress, toolNames) {
  const outcome = recordObservationOutcome(progress, target, content);
  if (outcome.kind === 'verified') return null;
  if (outcome.kind === 'failed') return resolvedFinalAnswer(renderFailure(outcome.label, outcome.detail, task),
    FinalDisposition.Failure, 'record_readback_failed');
  if (outcome.kind === 'mismatch') return resolvedFinalAnswer(renderSeededOutcome('coding_workspace_verification_failed', task, target),
    FinalDisposition.Failure, 'record_readback_mismatch');
  if (content !== null) {
    const runTool = toolFor(toolNames, Capability.Run);
    if (runTool !== null) return planOne(runTool, jsonText({ command: recordReadbackCommand(target) }));
    const readTool = toolFor(toolNames, Capability.Read);
    if (readTool !== null) return planOne(readTool, jsonText({ path: target, filePath: target, file_path: target }));
  }
  return resolvedFinalAnswer(renderSeededOutcome('coding_workspace_written_unverified', task, target),
    FinalDisposition.Gap, 'record_readback_unavailable');
}
