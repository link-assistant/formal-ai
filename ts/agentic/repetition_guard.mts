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
function canonicalActionKey(tool, argumentsValue) {
  let argumentsText = argumentsValue;
  try { argumentsText = jsonText(JSON.parse(argumentsText)); } catch { /* Preserve invalid input exactly. */ }
  return JSON.stringify([tool.toLowerCase(), argumentsText]);
}
function actionKey(call) {
  return canonicalActionKey(call.function.name, call.function.arguments);
}
function observationDetail(value) {
  return [...value.slice(0, 4096)].slice(0, 2048).join('');
}
/** No result-shape inference, capability recognition or process-local counter is required.
 * Optional proposed calls filter only the recognized two-failure stop; global caps remain unconditional. */
export function repeatedActionStop(messages, candidates = null) {
  if (messages.reduce((sum, message) => sum + (message.role.toLowerCase() === 'assistant' ? (message.tool_calls?.length ?? 0) : 0), 0) < MAXIMUM_IDENTICAL_FAILED_ACTIONS) return null;
  const candidateKeys = candidates === null ? null
    : new Set(candidates.map(call => canonicalActionKey(call.tool, call.arguments)));
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
  let latestReceipt = null;
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
      latestReceipt = {call, result};
      if (failureMessage(result, message.is_error === true || message.isError === true, false) === null) continue;
      const key = actionKey(call);
      const attempts = (failedCounts.get(key) ?? 0) + 1;
      failedCounts.set(key, attempts);
      if (attempts >= MAXIMUM_IDENTICAL_FAILED_ACTIONS
          && (candidateKeys === null || candidateKeys.has(key))) failed = {call, attempts, result};
    }
  }
  if (failed !== null) {
    let detail = observationDetail(failed.result);
    // A later bound observation must survive the stop, including a verification's exit.
    if (latestReceipt !== null && latestReceipt.call !== failed.call) {
      detail += '\n' + agenticMessage('action-repetition-observation', {
        tool: latestReceipt.call.function.name,
        arguments: observationDetail(latestReceipt.call.function.arguments),
        result: observationDetail(latestReceipt.result),
      });
    }
    return agenticMessage('action-repetition-stopped', {
      attempts: failed.attempts, maximum: MAXIMUM_IDENTICAL_FAILED_ACTIONS,
      tool: failed.call.function.name, arguments: failed.call.function.arguments, result: detail,
    });
  }
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
