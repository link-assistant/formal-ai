// Expected source bytes, verified by a command receipt bound to the current mutation.
import { Capability } from '../capability.mjs';
import { classifyTool, toolFor } from '../capability_router.mjs';
import { resultForCommand } from '../code_artifact.mjs';
import { renderSeededChange, renderSeededOutcome } from '../code_task.mjs';
import { plainText } from '../content.mjs';
import { FinalDisposition, jsonText, planOne, resolvedFinalAnswer } from '../plan.mjs';
import { sha256Hex } from '../crate/source_fetch.mjs';
import { commandArgument, observedDigestMatches, StepOutcome, stepOutcome } from '../tool_result.mjs';

const finish = (text, disposition, origin) => text === null ? null : resolvedFinalAnswer(text, disposition, origin);
const failed = (task, target) => finish(renderSeededOutcome('coding_workspace_verification_failed', task, target),
  FinalDisposition.Failure, 'coding_workspace_verification_failed');

/** Mirrors `fn plan_digest_verification`: keep the existing bare digest observation contract. */
export function planDigestVerification(task, currentTurn, toolNames, change) {
  const command = 'sha256sum -- ' + change.target;
  const observed = resultForCommand(currentTurn, command);
  if (observed === null) {
    const tool = toolFor(toolNames, Capability.Run);
    return tool === null ? null : planOne(tool, jsonText({ command }));
  }
  if (!observedDigestMatches(observed, sha256Hex(change.expected))) return failed(task, change.target);
  return finish(renderSeededChange(change.intent, task, change.target, change.slots), FinalDisposition.Finding, change.intent);
}

/** Mirrors `fn receipt_call`: associate receipt and call without discarding the client failure flag. */
function receiptCall(messages, index) {
  const message = messages[index];
  if (message.role.toLowerCase() !== 'tool' || message.tool_call_id == null) return null;
  for (let prior = index - 1; prior >= 0; prior -= 1) {
    const call = (messages[prior].tool_calls ?? []).findLast((candidate) => candidate.id === message.tool_call_id);
    if (call) return call;
  }
  return null;
}

/** Mirrors `fn argument_matches_path`: only an absolute client path may qualify a relative target. */
function targets(argumentsText, target) {
  let argumentsValue;
  try { argumentsValue = JSON.parse(argumentsText); } catch { return false; }
  const components = (path) => path.split('/').filter((part, index) => part !== '' && (part !== '.' || index === 0));
  return ['path', 'filePath', 'file_path'].some((key) => {
    const observed = argumentsValue?.[key];
    if (typeof observed !== 'string') return false;
    if (observed === target) return true;
    if (target.startsWith('/') || !observed.startsWith('/')) return false;
    const wanted = components(target), actual = components(observed);
    return wanted.length <= actual.length && wanted.every((part, index) => actual[actual.length - wanted.length + index] === part);
  });
}

/** Mirrors `fn write_window`: later writes own the mutation, even when they failed. */
function writeWindow(messages, target, expected) {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const call = receiptCall(messages, index);
    if (call === null || classifyTool(call.function.name) !== Capability.Write || !targets(call.function.arguments, target)) continue;
    const message = messages[index], argumentsValue = JSON.parse(call.function.arguments);
    if (argumentsValue.content !== expected || message.is_error === true || message.isError === true
      || stepOutcome(plainText(message.content)) === StepOutcome.Failed) return null;
    return messages.slice(index + 1);
  }
  return null;
}

/** Mirrors `fn plan_write_digest_verification`: freshness and outer failures precede digest matching. */
export function planWriteDigestVerification(task, messages, toolNames, change) {
  const current = writeWindow(messages, change.target, change.expected);
  if (current === null) return failed(task, change.target);
  const command = 'sha256sum -- ' + change.target;
  for (let index = current.length - 1; index >= 0; index -= 1) {
    const call = receiptCall(current, index);
    if (call === null || classifyTool(call.function.name) !== Capability.Run || commandArgument(call.function.arguments) !== command) continue;
    if (current[index].is_error === true || current[index].isError === true) return failed(task, change.target);
    break;
  }
  return planDigestVerification(task, current, toolNames, change);
}
