// The agentic loop on the JavaScript server: rust/src/protocol.rs
// `agentic_outcome`, `command_reroute_plan`, `groundable_tool_names`,
// `chat_completion_from_plan` and `response_from_plan`, with
// rust/src/protocol_policy.rs `tool_permission_refusal_answer`.
//
// The deterministic planner itself is the port in js/agentic/ (R1015); this
// module installs its host over the booted worker and projects its plans onto
// the Chat Completions and Responses wire shapes exactly as Rust does.

import { plainText, requestsToolExecution } from './chat-request.mjs';
import { estimateTokens, stableId } from './ids.mjs';
import { resolveModelId } from './seed.mjs';
import { applyRetainedAmendments } from './standing-requirements.mjs';
import { thinkingStep } from './thinking.mjs';
import { clientWorkingDirectory } from '../agentic/content.mjs';
import { toolActionNarration } from '../agentic/narration.mjs';
import { ensureNodeHost } from './node-host-install.mjs';
import { agenticMessage } from '../agentic/messages.mjs';
import { planSymbolicCommandReroute } from '../agentic/command_reroute.mjs';
import { suppliedFileAnswer } from '../agentic/file_read/supplied.mjs';
import { planChatStep, toolCapability } from '../agentic/planner.mjs';
import { projectDeclaredCommandKeys } from '../agentic/tool_result.mjs';
import {
  agenticToolPermissionDenial,
  findToolDefinition,
  isHostedToolDefinition,
  requestedToolNames,
  responseToolCallIdentity,
} from '../agentic/protocol_policy.mjs';
import {
  customResponseToolInput,
  isCustomResponseTool,
  responseArgumentsForTool,
} from '../agentic/crate/protocol_responses.mjs';
import { projectAppendContracts } from '../agentic/append_contract.mjs';
import { ungroundedIdentityArguments } from '../agentic/crate/tool_scope.mjs';

/** Install the planner host over `ctx.worker` once (shared with js/server/solve.mjs). */
async function ensureHost(ctx) {
  await ensureNodeHost(ctx);
}

function memoryEvents(ctx) {
  return ctx.memory && typeof ctx.memory.events === 'function' ? ctx.memory.events() : [];
}

/** `dreaming_application::amended_answer`. */
function amendedAnswer(ctx, prompt, answer) {
  return applyRetainedAmendments(prompt, { answer, evidence_links: [] }, memoryEvents(ctx)).answer;
}

/** `tool_permission_refusal_answer`. */
export function toolPermissionRefusalAnswer(denial) {
  const { capability } = denial;
  const detail = `package_permission_required:${capability}`;
  return {
    intent: 'tool_call_refused',
    answer: agenticMessage('tool_permission_refused', { capability, reason: denial.reason }),
    confidence: 1,
    evidence_links: [`policy:${detail}`],
    thinking_steps: [thinkingStep(0, 'policy_refusal', detail, 'high', 'policy')],
    links_notation: `tool_call_refusal\n  policy "package_permission_required"\n  capability "${capability}"\n  thinking_step "policy_refusal ${detail}"\n`,
  };
}

/** `groundable_tool_names`: the advertised tools this request can address. */
function groundableToolNames(request, names) {
  const context = request.messages.map((message) => plainText(message.content)).join('\n');
  return names.filter((name) => {
    const definition = findToolDefinition(request.tools, name);
    return !definition || ungroundedIdentityArguments(definition, {}, context).length === 0;
  });
}

/**
 * `agentic_outcome`: `{kind: 'refused', answer}`, `{kind: 'planned', plan}`
 * or `{kind: 'fallthrough'}`. The caller has already refused a tool-bearing
 * request without agent mode only when this says so.
 * @param {{worker: object, agentMode: boolean}} ctx
 * @param {object} request a parsed `ChatCompletionRequest`
 * @param {() => object} refusal `tool_call_refusal_answer`
 */
export async function agenticOutcome(ctx, request, refusal) {
  if (!requestsToolExecution(request)) {
    if (ctx.agentMode) {
      await ensureHost(ctx);
      const answer = suppliedFileAnswer(request.messages);
      if (answer !== null) return { kind: 'planned', plan: { kind: 'final', answer } };
    }
    return { kind: 'fallthrough' };
  }
  if (!ctx.agentMode) return { kind: 'refused', answer: refusal() };
  await ensureHost(ctx);
  const owned = requestedToolNames(request);
  const denial = agenticToolPermissionDenial(owned, toolCapability);
  if (denial) return { kind: 'refused', answer: toolPermissionRefusalAnswer(denial) };
  // R1154-1: a shell tool whose schema names its command property something
  // else than `command`/`cmd`/`script` is read through that declared key.
  const messages = projectDeclaredCommandKeys(request.messages, request.tools);
  const scoped = projectAppendContracts(messages, request.tools);
  const plan = await planChatStep(scoped, groundableToolNames(request, owned));
  return plan === null ? { kind: 'fallthrough' } : { kind: 'planned', plan };
}

/** `command_reroute_plan`: a plan that reroutes the solver's command, or null. */
export async function commandReroutePlan(ctx, request, symbolic) {
  if (!ctx.agentMode || !requestsToolExecution(request)) return null;
  await ensureHost(ctx);
  return planSymbolicCommandReroute(request.messages, requestedToolNames(request), symbolic);
}

function messageTokens(messages) {
  return messages.reduce((total, message) => {
    if (typeof message.content === 'string') return total + estimateTokens(message.content);
    return total + message.content.reduce((sum, part) => sum + (typeof part.text === 'string' ? estimateTokens(part.text) : 0), 0);
  }, 0);
}

/**
 * `chat_completion_from_plan`.
 * @param {object} ctx
 * @param {object} request a parsed `ChatCompletionRequest`
 * @param {string} prompt
 * @param {object} plan an `AgenticPlan`
 * @param {number} created the response timestamp
 */
export function chatCompletionFromPlan(ctx, request, prompt, plan, created) {
  const promptTokens = messageTokens(request.messages);
  const workspace = clientWorkingDirectory(request.messages);
  let message;
  let finishReason;
  let completionTokens;
  if (plan.kind === 'tool_calls') {
    const narration = toolActionNarration(prompt, plan.calls);
    const toolCalls = plan.calls.map((call, index) => ({
      id: stableId('call', `${prompt}|${index}|${call.tool}|${call.arguments}`),
      type: 'function',
      function: {
        name: call.tool,
        arguments: responseArgumentsForTool(request.tools, call.tool, call.arguments, prompt, workspace),
      },
    }));
    completionTokens = estimateTokens(narration) + toolCalls.reduce((total, call) =>
      total + estimateTokens(call.function.name) + estimateTokens(call.function.arguments), 0);
    message = { role: 'assistant', content: narration, tool_calls: toolCalls };
    finishReason = 'tool_calls';
  } else {
    const answer = amendedAnswer(ctx, prompt, plan.answer);
    completionTokens = estimateTokens(answer);
    message = { role: 'assistant', content: answer, tool_calls: [] };
    finishReason = 'stop';
  }
  return {
    id: stableId('chatcmpl', prompt),
    object: 'chat.completion',
    created,
    model: resolveModelId(request.model),
    choices: [{ index: 0, message, finish_reason: finishReason }],
    usage: { prompt_tokens: promptTokens, completion_tokens: completionTokens, total_tokens: promptTokens + completionTokens },
  };
}

function outputMessage(text) {
  return { id: stableId('msg', text), type: 'message', role: 'assistant', content: [{ type: 'output_text', text }] };
}

/** The output item one planned call becomes on the Responses surface. */
function responseCallItem(request, prompt, workspace, call, index) {
  const { tool } = call;
  const seed = `${prompt}|${index}|${tool}|${call.arguments}`;
  const args = responseArgumentsForTool(request.tools, tool, call.arguments, prompt, workspace);
  if (tool === 'web_search' && request.tools.some((definition) => isHostedToolDefinition(definition, tool))) {
    let query = prompt;
    try {
      const value = JSON.parse(args);
      if (value && typeof value.query === 'string') query = value.query;
    } catch {
      // `serde_json::from_str(..).ok()`: an unparsable argument keeps the prompt.
    }
    const item = {
      id: stableId('ws', seed),
      type: 'web_search_call',
      status: 'completed',
      action: { type: 'search', query, queries: [query] },
    };
    return [item, estimateTokens(tool) + estimateTokens(args)];
  }
  if (isCustomResponseTool(request.tools, tool)) {
    const input = customResponseToolInput(request.tools, tool, call.arguments);
    const item = { id: stableId('ctc', seed), type: 'custom_tool_call', call_id: stableId('call', seed), name: tool, input };
    return [item, estimateTokens(tool) + estimateTokens(input)];
  }
  const [name, namespace] = responseToolCallIdentity(request.tools, tool);
  const item = { id: stableId('fc', seed), type: 'function_call', call_id: stableId('call', seed), name };
  if (namespace !== null) item.namespace = namespace;
  item.arguments = args;
  item.status = 'completed';
  return [item, estimateTokens(tool) + estimateTokens(args)];
}

/**
 * `response_from_plan`.
 * @param {object} ctx
 * @param {object} request a parsed `ResponsesRequest`
 * @param {object} chatRequest its chat projection
 * @param {string} prompt
 * @param {object} plan an `AgenticPlan`
 * @param {number} created the response timestamp
 */
export function responseFromPlan(ctx, request, chatRequest, prompt, plan, created) {
  const inputTokens = messageTokens(chatRequest.messages);
  const workspace = clientWorkingDirectory(chatRequest.messages);
  let output;
  let outputTokens;
  if (plan.kind === 'tool_calls') {
    const narration = toolActionNarration(prompt, plan.calls);
    output = [outputMessage(narration)];
    outputTokens = estimateTokens(narration);
    plan.calls.forEach((call, index) => {
      const [item, tokens] = responseCallItem(chatRequest, prompt, workspace, call, index);
      output.push(item);
      outputTokens += tokens;
    });
  } else {
    const answer = amendedAnswer(ctx, prompt, plan.answer);
    output = [outputMessage(answer)];
    outputTokens = estimateTokens(answer);
  }
  return {
    id: stableId('resp', prompt),
    object: 'response',
    created_at: created,
    status: 'completed',
    model: resolveModelId(request.model),
    output,
    usage: { input_tokens: inputTokens, output_tokens: outputTokens, total_tokens: inputTokens + outputTokens },
    evidence_links: [],
  };
}
