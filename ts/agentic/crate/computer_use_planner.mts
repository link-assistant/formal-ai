// The computer-use agentic step planner (issue #707): a port of
// rust/src/computer_use/planner.rs (`plan_agentic_step`, `tool_for_primitive`).
//
// Plans the latest user request, then walks the tool results that follow it:
// each must come from the expected primitive and report `"verified": true`,
// otherwise the turn ends with the seeded verification-failure message. The
// next unexecuted step becomes one tool call whose arguments carry the plan
// and step ids and the step's pre- and postcondition.

import { finalAnswer, jsonText, plannedCall, toolCalls } from '../plan.mjs';
import { plainText } from '../content.mjs';
import { computerPrimitiveFromToolName } from '../protocol_policy.mjs';
import { capabilityGapForRequest, planRequest } from './computer_use.mjs';
import { completionMessage, missingPrimitiveMessage, verificationFailedMessage } from './computer_use_seed.mjs';

const UNEXPECTED_RESULT = 'unexpected-result';
const UNKNOWN_TOOL = 'unknown';

/**
 * Mirrors `fn tool_for_primitive`: the first advertised tool name that maps to
 * `primitive`, or null.
 * @param {Array<string>} toolNames
 * @param {string} primitive the dotted primitive name
 */
export function toolForPrimitive(toolNames, primitive) {
  return toolNames.find((name) => computerPrimitiveFromToolName(name) === primitive) ?? null;
}

/**
 * Mirrors `fn plan_agentic_step` in rust/src/computer_use/planner.rs.
 * @param {Array<object>} messages parsed chat messages
 * @param {Array<string>} toolNames advertised tool names
 * @returns {object|null} an `AgenticPlan` or null
 */
export function computerUsePlanAgenticStep(messages, toolNames) {
  let userIndex = -1;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role === 'user') {
      userIndex = index;
      break;
    }
  }
  if (userIndex < 0) return null;
  const task = plainText(messages[userIndex].content);
  const gap = capabilityGapForRequest(task);
  if (gap !== null) return finalAnswer(gap.response);
  const plan = planRequest(task);
  if (plan === null) return null;

  const toolResults = [];
  for (let index = userIndex + 1; index < messages.length; index += 1) {
    if (messages[index].role === 'tool') toolResults.push(index);
  }
  for (let stepIndex = 0; stepIndex < toolResults.length; stepIndex += 1) {
    const messageIndex = toolResults[stepIndex];
    const message = messages[messageIndex];
    const step = plan.steps[stepIndex];
    if (!step) {
      return finalAnswer(verificationFailedMessage(plan.locale, plan.id, UNEXPECTED_RESULT,
        message.name ?? UNKNOWN_TOOL));
    }
    const toolName = message.name ?? toolNameForResult(messages, userIndex, messageIndex);
    const matchesPrimitive = toolName !== null && computerPrimitiveFromToolName(toolName) === step.primitive;
    if (!matchesPrimitive || !reportsVerified(plainText(message.content))) {
      return finalAnswer(verificationFailedMessage(plan.locale, plan.id, step.id, step.primitive));
    }
  }

  const completed = toolResults.length;
  const step = plan.steps[completed];
  if (!step) return finalAnswer(completionMessage(plan.locale, plan.id, plan.steps.length));
  const tool = toolForPrimitive(toolNames, step.primitive);
  if (tool === null) {
    // A client that advertises no computer-use primitive is not this route's
    // client (issue #1066); one that is missing a single primitive is told which.
    if (!advertisesComputerUse(toolNames)) return null;
    return finalAnswer(missingPrimitiveMessage(plan.locale, plan.id, step.id, step.primitive));
  }
  const args = structuredClone(step.arguments);
  if (args && typeof args === 'object' && !Array.isArray(args)) {
    args.plan_id = plan.id;
    args.step_id = step.id;
    args.precondition = step.precondition;
    args.postcondition = step.postcondition;
  }
  return toolCalls([plannedCall(tool, jsonText(args))]);
}

/** `serde_json::from_str::<Value>(text)?.get("verified")?.as_bool() == Some(true)`. */
function reportsVerified(text) {
  try {
    const value = JSON.parse(text);
    return Boolean(value) && typeof value === 'object' && !Array.isArray(value) && value.verified === true;
  } catch {
    return false;
  }
}

/** Mirrors `fn advertises_computer_use`. */
function advertisesComputerUse(toolNames) {
  return toolNames.some((name) => computerPrimitiveFromToolName(name) !== null);
}

/** Mirrors `fn tool_name_for_result`: the name of the call a result answers, by call id. */
function toolNameForResult(messages, userIndex, resultIndex) {
  const callId = messages[resultIndex]?.tool_call_id;
  if (callId === null || callId === undefined) return null;
  for (let index = resultIndex - 1; index > userIndex; index -= 1) {
    const calls = messages[index].tool_calls || [];
    for (let at = calls.length - 1; at >= 0; at -= 1) {
      if (calls[at].id === callId) return calls[at].function.name;
    }
  }
  return null;
}
