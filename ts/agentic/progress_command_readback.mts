// Exact commanded readback custody. The caller constructs the owned read-only command.
import {Capability} from './capability.mjs';
import {commandArgument} from './tool_result.mjs';
import {agenticMessage} from './messages.mjs';
import {quoteShellWord} from './portable/string_contracts.mjs';

/** One seeded command builder shared by execution and freshness classification. */
export function ownedCreationReadbackCommand(target) {
  return agenticMessage('literal_create_shell_readback', {path: quoteShellWord(target)});
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
