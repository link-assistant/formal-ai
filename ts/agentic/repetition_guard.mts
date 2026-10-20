// Finite per-action budgets from the transcript, including unrecognized tools/errors.
import { jsonText } from './plan.mjs';
import { plainText } from './content.mjs';
import { fileReadEvidence, fileReadTaskFor } from './file_read.mjs';
import { latestUserRequest } from './content.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';

export const MAX_IDENTICAL_ACTION_ATTEMPTS = 3;
export const MAX_ACTIONS_PER_OBJECTIVE = 128;
function actionKey(call) {
  let args = call.function.arguments;
  try { args = jsonText(JSON.parse(args)); } catch { /* Preserve invalid input exactly. */ }
  return JSON.stringify([call.function.name.toLowerCase(), args]);
}
/** No result-shape inference, capability recognition or process-local counter is required. */
export function repeatedActionStop(messages) {
  if (messages.reduce((sum, message) => sum + (message.role.toLowerCase() === 'assistant' ? (message.tool_calls?.length ?? 0) : 0), 0) < MAX_IDENTICAL_ACTION_ATTEMPTS) return null;
  if (fileReadTaskFor(latestUserRequest(messages) ?? '') !== null) messages = fileReadEvidence(messages);
  const counts = new Map();
  let repeated = null;
  let total = 0;
  for (const message of messages.slice(evidenceWindowStart(messages))) {
    if (message.role.toLowerCase() !== 'assistant') continue;
    for (const call of message.tool_calls ?? []) {
      total += 1;
      const key = actionKey(call), attempts = (counts.get(key) ?? 0) + 1;
      counts.set(key, attempts);
      if (attempts >= MAX_IDENTICAL_ACTION_ATTEMPTS) repeated = {call, attempts};
    }
  }
  if (repeated === null && total >= MAX_ACTIONS_PER_OBJECTIVE) return `Action budget exhausted after ${total} tool declarations (limit ${MAX_ACTIONS_PER_OBJECTIVE}). The task is incomplete; inspect progress before starting a new request.`;
  if (repeated === null) return null;
  const {call, attempts} = repeated;
  const receipt = messages.findLast(message => message.role.toLowerCase() === 'tool'
    && message.tool_call_id === call.id);
  const result = receipt ? plainText(receipt.content) : 'No bound tool result was received.';
  return `Repeated action stopped after ${attempts} attempts (limit ${MAX_IDENTICAL_ACTION_ATTEMPTS}): ${call.function.name} ${call.function.arguments}. Last result: ${result.slice(0, 2048)}. The task is incomplete; inspect the tool contract or change strategy.`;
}
