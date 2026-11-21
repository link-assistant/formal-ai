// OpenAI Responses API (`POST /v1/responses`): rust/src/server.rs (the
// `/v1/responses` arm), rust/src/protocol.rs
// (`create_response_with_solver_and_memory`, `response_from_symbolic`,
// `response_reasoning_item`, `responses_input_tokens`) and the
// `ResponseObject` wire shape of rust/src/protocol/output.rs.
//
// Agentic requests go through the shared planner and its permission checks.

import { logUnclassifiedTerminal, agenticOutcome, commandReroutePlan, responseFromPlan } from './agentic.mjs';
import { RequestShapeError, chatPromptAndHistory } from './chat-request.mjs';
import { messageInputTokens, responseTimestamp, unsupportedModelResponse } from './openai.mjs';
import { jsonResponse, messageError } from './response.mjs';
import { parseResponsesRequest, responsePrompt, toChatCompletionRequest } from './responses-input.mjs';
import { responsesSse } from './responses-stream.mjs';
import { resolveModelId } from './seed.mjs';
import { answerFromMemory, estimateTokens, solveWithMemory, stableId, toolCallRefusalAnswer } from './solve.mjs';
import { renderThinkingSteps } from './thinking.mjs';
import { requestScope, runInRequestScope } from './standing-scope.mjs';
import { formalizeRequest, contextualAnswer, composeRequestAnswer } from './request-context.mjs';
import { ensureNodeHost } from './node-host-install.mjs';
import { outputObligation, projectStructuredOutput, structuredAnswerFromSource, structuredOutputRequested, structuredThinkingStep } from './structured-output.mjs';
import { clientFunctionOutput, forcedFunction } from './client-functions.mjs';

/** `response_reasoning_item`: the rendered trace as a `reasoning` output item. */
function responseReasoningItem(prompt, thinkingSteps) {
  const text = renderThinkingSteps(thinkingSteps);
  if (!text) return null;
  return {
    id: stableId('rs', prompt),
    type: 'reasoning',
    summary: [{ type: 'summary_text', text }],
  };
}

/**
 * `response_from_symbolic`, as the wire JSON of `ResponseObject` (struct field
 * order; empty `thinking_steps` skipped).
 */
export function responseFromSymbolic(request, chatRequest, prompt, symbolic) {
  const structured = structuredOutputRequested(request.text?.format);
  if (!structured) symbolic = composeRequestAnswer(chatRequest.messages, symbolic);
  const inputTokens = messageInputTokens(chatRequest.messages);
  const outputTokens = estimateTokens(symbolic.answer);
  const steps = symbolic.thinking_steps;
  const message = {
    id: stableId('msg', symbolic.answer),
    type: 'message',
    role: 'assistant',
    content: structured && symbolic.intent === 'tool_call_refused'
      ? [{ type: 'refusal', refusal: symbolic.answer }]
      : [{ type: 'output_text', text: symbolic.answer }],
  };
  if (steps.length) message.thinking_steps = steps;
  const output = [message];
  const reasoning = responseReasoningItem(prompt, steps);
  if (reasoning) output.push(reasoning);
  const response = {
    id: stableId('resp', prompt),
    object: 'response',
    created_at: responseTimestamp(),
    status: 'completed',
    model: resolveModelId(request.model),
    output,
    usage: {
      input_tokens: inputTokens,
      output_tokens: outputTokens,
      total_tokens: inputTokens + outputTokens,
    },
    evidence_links: symbolic.evidence_links,
    // Issue #1184 R8: the answer's content-addressed derivation id
    // (`SymbolicAnswer::derivation_id`), the key `formal-ai explain` reads.
    derivation_id: stableId('answer', symbolic.answer),
  };
  if (steps.length) response.thinking_steps = steps;
  return response;
}

/** `create_response_with_solver_and_memory`. */
export async function createResponse(ctx, request) {
  const response = await createUnstructuredResponse(ctx, request);
  const obligation = outputObligation(request.text?.format, false);
  if (!obligation) return response;
  response.text = { format: obligation.format };
  const message = response.output.find(item => item.type === 'message');
  if (!message) return response;
  const source = chatPromptAndHistory(toChatCompletionRequest(request).messages).prompt;
  const answer = message.content.filter(part => part.type === 'output_text').map(part => part.text).join('\n');
  const result = await projectStructuredOutput(ctx, obligation, source, answer,
    message.content.find(part => part.type === 'refusal')?.refusal);
  message.content = result.refusal ? [{ type: 'refusal', refusal: result.refusal }]
    : [{ type: 'output_text', text: result.content }];
  message.id = stableId('msg', result.refusal ?? result.content);
  const steps = [...(message.thinking_steps ?? [])];
  steps.push(structuredThinkingStep(steps, result));
  message.thinking_steps = steps;
  response.thinking_steps = steps;
  response.derivation_id = result.id;
  response.evidence_links = [...result.evidence, ...response.evidence_links];
  response.usage.output_tokens = estimateTokens(result.refusal ?? result.content);
  response.usage.total_tokens = response.usage.input_tokens + response.usage.output_tokens;
  return response;
}

async function createUnstructuredResponse(ctx, request) {
  const prompt = responsePrompt(request);
  const chatRequest = toChatCompletionRequest(request);
  const { prompt: chatPrompt, history } = chatPromptAndHistory(chatRequest.messages);
  const memoryPrompt = chatPrompt.trim() ? chatPrompt : prompt;
  const forced = await forcedFunction(ctx, chatRequest, chatPrompt);
  if (forced) {
    const { callId, outputTokens } = clientFunctionOutput(forced, prompt, chatRequest.messages.length);
    const item = forced.refusal ? {
      id: stableId('msg', forced.refusal), type: 'message', role: 'assistant',
      content: [{ type: 'refusal', refusal: forced.refusal }],
    } : {
      id: stableId('fc', callId), type: 'function_call', call_id: callId,
      name: forced.name, arguments: forced.arguments, status: 'completed',
    };
    const inputTokens = messageInputTokens(chatRequest.messages);
    return {
      id: stableId('resp', prompt), object: 'response', created_at: responseTimestamp(),
      status: 'completed', model: resolveModelId(request.model), output: [item], evidence_links: [],
      usage: { input_tokens: inputTokens, output_tokens: outputTokens, total_tokens: inputTokens + outputTokens },
    };
  }
  const context = formalizeRequest(chatRequest.messages);
  const remembered = await answerFromMemory(ctx, memoryPrompt, history, prompt);
  if (remembered) {logUnclassifiedTerminal(ctx,'retained_answer');return responseFromSymbolic(request, chatRequest, prompt, remembered);}
  const outcome = await agenticOutcome(ctx, chatRequest, toolCallRefusalAnswer);
  if (outcome.kind === 'refused') return responseFromSymbolic(request, chatRequest, prompt, outcome.answer);
  if (outcome.kind === 'planned') return responseFromPlan(ctx, request, chatRequest, prompt, outcome.plan, responseTimestamp());
  if (context.titleTask) await ensureNodeHost(ctx);
  const contextual = contextualAnswer(context);
  if (contextual) return responseFromSymbolic(request, chatRequest, prompt, contextual);
  const symbolic = await structuredAnswerFromSource(ctx, request.text?.format, false, memoryPrompt)
    ?? await solveWithMemory(ctx, memoryPrompt, history, { forcedResponseLanguage: context.language });
  const reroute = await commandReroutePlan(ctx, chatRequest, symbolic);
  if (reroute) return responseFromPlan(ctx, request, chatRequest, prompt, reroute, responseTimestamp());
  logUnclassifiedTerminal(ctx,'symbolic_answer');
  return responseFromSymbolic(request, chatRequest, prompt, symbolic);
}

/** The text of every assistant message item (`output_messages`), joined. */
function responseAnswerText(response) {
  return response.output
    .filter((item) => item.type === 'message')
    .flatMap((item) => item.content.filter((content) => content.type === 'output_text').map((content) => content.text))
    .join('\n');
}

/** `POST /v1/responses`. */
export async function handleResponses(ctx, request) {
  let parsed;
  try {
    parsed = parseResponsesRequest(request.body);
  } catch (error) {
    if (!(error instanceof RequestShapeError)) throw error;
    return messageError(400, 'invalid_responses_request', { error: error.message });
  }
  const unsupported = unsupportedModelResponse(parsed.model);
  if (unsupported) return unsupported;
  const chat = toChatCompletionRequest(parsed);
  const scope = requestScope({ body: request.body, messages: chat.messages, tools: chat.tools });
  const response = await runInRequestScope(scope, () => createResponse(ctx, parsed));
  ctx.memory.recordExchange(chat.messages, responseAnswerText(response), null, scope);
  if (parsed.stream) return responsesSse(response);
  return jsonResponse(200, response);
}
