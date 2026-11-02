// Exact commanded readback custody. The caller constructs the owned read-only command.
import {Capability} from './capability.mjs';
import {commandArgument} from './tool_result.mjs';
import {argumentValuesEqual} from './qualified_tool_observation.mjs';
import {agenticMessage} from './messages.mjs';
import {quoteShellWord} from './portable/string_contracts.mjs';

/** One seeded command builder shared by execution and freshness classification. */
export function ownedCreationReadbackCommand(target) {
  return agenticMessage('literal-create-shell-readback', {path: quoteShellWord(target)});
}
export function readOnlyCapability(capability) {
  return [Capability.Read, Capability.ReadMany, Capability.Search, Capability.Fetch,
    Capability.Grep, Capability.Glob, Capability.ListDir, Capability.AskUser].includes(capability);
}
function qualifiedReceipt(progress, attempt, command, qualify) {
  const raw = attempt.raw_receipt ?? null, frame = qualify(progress, command);
  const qualified = typeof raw === 'string' && frame?.binding === 'exact'
    && frame.receiptPresent === true && frame.detail === raw && frame.declaredTool === attempt.tool;
  return {raw, qualified, succeeded: qualified && frame.succeeded === true && attempt.succeeded === true};
}
function scalarCommand(argumentsText) {
  try {
    const parsedArguments = JSON.parse(argumentsText);
    return parsedArguments !== null && typeof parsedArguments === 'object' && !Array.isArray(parsedArguments)
      && Object.keys(parsedArguments).length === 1 && typeof parsedArguments.command === 'string'
      ? parsedArguments.command : null;
  } catch { return null; }
}
/** Known framed reads may coexist; unknown or unbound shell actions still invalidate proof. */
export function readOnlyAttempt(progress, attempt, qualify, pathFromArguments) {
  if (readOnlyCapability(attempt.capability)) return true;
  if (attempt.capability !== Capability.Run || typeof qualify !== 'function'
    || typeof pathFromArguments !== 'function') return false;
  const command = scalarCommand(attempt.arguments);
  if (command === null) return false;
  const known = progress.attempts.some(write => {
    if (write.capability !== Capability.Write || !write.succeeded || write.arguments === null) return false;
    const path = pathFromArguments(write.arguments);
    return path !== null && command === ownedCreationReadbackCommand(path);
  });
  // A failed qualified cat is still read-only; it never becomes successful evidence.
  return known && qualifiedReceipt(progress, attempt, command, qualify).qualified;
}
export function postWriteCommandReceiptFor(progress, path, command, qualify, pathFromArguments) {
  const attempts = progress.attemptsAfterLatestWrite(path) ?? [];
  for (let index = attempts.length - 1; index >= 0; index--) {
    const attempt = attempts[index];
    if (attempt.capability === Capability.Run && attempt.arguments !== null
      && commandArgument(attempt.arguments) === command) {
      return qualifiedReceipt(progress, attempt, command, qualify);
    }
    if (!readOnlyAttempt(progress, attempt, qualify, pathFromArguments)) return null;
  }
  return null;
}

/** Only the exact successful latest mutation may lend its subsequent complete Read. */
export function postMutationReadStateFor(progress, path, capability, argumentsValue, qualifyTool, qualifyCommand, pathFromArguments) {
  let read = null;
  for (let index = progress.attempts.length - 1; index >= 0; index--) {
    const attempt = progress.attempts[index];
    if (readOnlyAttempt(progress, attempt, qualifyCommand, pathFromArguments)) {
      if (read === null && attempt.capability === Capability.Read && attempt.arguments !== null
        && pathFromArguments(attempt.arguments) === path) read = attempt;
      continue;
    }
    let actual; try { actual = JSON.parse(attempt.arguments); } catch { return {current:false,attempt:null}; }
    const frame = qualifyTool(progress, capability, argumentsValue);
    const current = attempt.capability === capability && argumentValuesEqual(actual, argumentsValue)
      && attempt.succeeded && frame?.binding === 'exact' && frame.receiptPresent
      && frame.declaredTool === attempt.tool && frame.succeeded && frame.detail === attempt.detail;
    return {current,attempt:current ? read : null};
  }
  return {current:false,attempt:null};
}

/** The actual complete preimage before the latest target mutation, without intervening effects. */
export function sourceReadBeforeLatestMutationFor(progress, path, capability, qualifyCommand, pathFromArguments) {
  const index = progress.attempts.findLastIndex(attempt => attempt.capability === capability
    && attempt.arguments !== null && pathFromArguments(attempt.arguments) === path);
  for (let prior = index - 1; prior >= 0; prior--) {
    const attempt = progress.attempts[prior];
    if (!readOnlyAttempt(progress, attempt, qualifyCommand, pathFromArguments)) return null;
    if (attempt.capability === Capability.Read && attempt.arguments !== null
      && pathFromArguments(attempt.arguments) === path) {
      const observed = attempt.source_read;
      return attempt.succeeded && observed?.error === null && observed.complete ? attempt : null;
    }
  }
  return null;
}

/** Current target Read, stopping at any observed potential effect. */
export function freshSourceReadAttemptFor(progress, path, qualifyCommand, pathFromArguments) {
  for (let index = progress.attempts.length - 1; index >= 0; index--) {
    const attempt = progress.attempts[index];
    if (!readOnlyAttempt(progress, attempt, qualifyCommand, pathFromArguments)) return null;
    if (attempt.capability === Capability.Read && attempt.arguments !== null
      && pathFromArguments(attempt.arguments) === path) return attempt;
  }
  return null;
}
