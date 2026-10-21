// Finite per-action budgets from the transcript, including unrecognized tools/errors.
import { failureMessage } from './tool_result.mjs';
import { jsonText } from './plan.mjs';
import { agenticMessage } from './messages.mjs';
import { plainText } from './content.mjs';
import { fileReadEvidence, fileReadTaskFor } from './file_read.mjs';
import { latestUserRequest } from './content.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';

export const MAXIMUM_IDENTICAL_ACTION_ATTEMPTS = 3;
export const MAXIMUM_IDENTICAL_FAILED_ACTIONS = 2;
export const MAXIMUM_ACTIONS_PER_OBJECTIVE = 128;
function actionKey(call) {
  let argumentsText = call.function.arguments;
  try { argumentsText = jsonText(JSON.parse(argumentsText)); } catch { /* Preserve invalid input exactly. */ }
  return JSON.stringify([call.function.name.toLowerCase(), argumentsText]);
}
/** No result-shape inference, capability recognition or process-local counter is required. */
export function repeatedActionStop(messages) {
  if (messages.reduce((sum, message) => sum + (message.role.toLowerCase() === 'assistant' ? (message.tool_calls?.length ?? 0) : 0), 0) < MAXIMUM_IDENTICAL_FAILED_ACTIONS) return null;
  const lastUser = messages.findLastIndex(message => message.role.toLowerCase() === 'user');
  const priorCalls = messages.slice(0, lastUser).some(message => message.role.toLowerCase() === 'assistant' && message.tool_calls?.length);
  let window = lastUser + 1;
  if (priorCalls) {
    if (fileReadTaskFor(latestUserRequest(messages) ?? '') !== null) messages = fileReadEvidence(messages);
    window = evidenceWindowStart(messages);
  }
  // Only a receipt paired to a preceding actual call spends the failure budget.
  // Consuming IDs permits sequential client ID reuse without counting duplicate receipts.
  const pending = new Map();
  const failedCounts = new Map();
  let failed = null;
  for (const message of messages.slice(window)) {
    const role = message.role.toLowerCase();
    if (role === 'assistant') {
      const calls = message.tool_calls ?? [];
      const identifiers = new Set();
      for (const call of calls) {
        if (typeof call.id !== 'string' || call.id === '') continue;
        if (identifiers.has(call.id)) pending.set(call.id, null);
        else { identifiers.add(call.id); pending.set(call.id, call); }
      }
    } else if (role === 'tool') {
      const call = pending.get(message.tool_call_id);
      pending.delete(message.tool_call_id);
      if (!call || (message.name != null && (typeof message.name !== 'string'
        || message.name.toLowerCase() !== call.function.name.toLowerCase()))) continue;
      const result = plainText(message.content);
      if (failureMessage(result, message.is_error === true || message.isError === true, false) === null) continue;
      const key = actionKey(call);
      const attempts = (failedCounts.get(key) ?? 0) + 1;
      failedCounts.set(key, attempts);
      if (attempts >= MAXIMUM_IDENTICAL_FAILED_ACTIONS) failed = {call, attempts, result};
    }
  }
  if (failed !== null) return agenticMessage('action-repetition-stopped', {
    attempts: failed.attempts, maximum: MAXIMUM_IDENTICAL_FAILED_ACTIONS,
    tool: failed.call.function.name, arguments: failed.call.function.arguments,
    result: failed.result.slice(0, 2048),
  });
  const counts = new Map();
  let repeated = null;
  let total = 0;
  for (const message of messages.slice(window)) {
    if (message.role.toLowerCase() !== 'assistant') continue;
    for (const call of message.tool_calls ?? []) {
      total += 1;
      const key = actionKey(call), attempts = (counts.get(key) ?? 0) + 1;
      counts.set(key, attempts);
      if (attempts >= MAXIMUM_IDENTICAL_ACTION_ATTEMPTS) repeated = {call, attempts};
    }
  }
  if (repeated === null && total >= MAXIMUM_ACTIONS_PER_OBJECTIVE) return agenticMessage('action-budget-exhausted', {total, maximum: MAXIMUM_ACTIONS_PER_OBJECTIVE});
  if (repeated === null) return null;
  const {call, attempts} = repeated;
  const receipt = messages.findLast(message => message.role.toLowerCase() === 'tool'
    && message.tool_call_id === call.id);
  const result = receipt ? plainText(receipt.content) : agenticMessage('action-receipt-missing');
  return agenticMessage('action-repetition-stopped', {attempts, maximum: MAXIMUM_IDENTICAL_ACTION_ATTEMPTS, tool: call.function.name, arguments: call.function.arguments, result: result.slice(0, 2048)});
}
