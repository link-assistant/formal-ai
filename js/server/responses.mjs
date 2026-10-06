// OpenAI Responses API (`POST /v1/responses`): rust/src/server.rs (the
// `/v1/responses` arm), rust/src/protocol.rs
// (`create_response_with_solver_and_memory`, `response_from_symbolic`,
// `response_reasoning_item`, `responses_input_tokens`) and the
// `ResponseObject` wire shape of rust/src/protocol/output.rs.
//
// The deterministic agentic planner is native-only: a tool-bearing request
// without agent mode is refused exactly as natively (`AgenticOutcome::Refused`);
// with agent mode on the request is answered symbolically, as the Chat
// Completions port does.

import { RequestShapeError, chatPromptAndHistory, requestsToolExecution } from './chat-request.mjs';
import { messageInputTokens, responseTimestamp, unsupportedModelResponse } from './openai.mjs';
import { jsonResponse, messageError } from './response.mjs';
import { parseResponsesRequest, responsePrompt, toChatCompletionRequest } from './responses-input.mjs';
import { responsesSse } from './responses-stream.mjs';
import { resolveModelId } from './seed.mjs';
import { estimateTokens, solveSymbolic, stableId, toolCallRefusalAnswer } from './solve.mjs';
import { renderThinkingSteps } from './thinking.mjs';

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
  const inputTokens = messageInputTokens(chatRequest.messages);
  const outputTokens = estimateTokens(symbolic.answer);
  const steps = symbolic.thinking_steps;
  const message = {
    id: stableId('msg', symbolic.answer),
    type: 'message',
    role: 'assistant',
    content: [{ type: 'output_text', text: symbolic.answer }],
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
  };
  if (steps.length) response.thinking_steps = steps;
  return response;
}

/** `create_response_with_solver_and_memory`. */
export async function createResponse(ctx, request) {
  const prompt = responsePrompt(request);
  const chatRequest = toChatCompletionRequest(request);
  const { prompt: chatPrompt, history } = chatPromptAndHistory(chatRequest.messages);
  const memoryPrompt = chatPrompt.trim() ? chatPrompt : prompt;
  if (requestsToolExecution(chatRequest) && !ctx.agentMode) {
    return responseFromSymbolic(request, chatRequest, prompt, toolCallRefusalAnswer());
  }
  const symbolic = await solveSymbolic(ctx, memoryPrompt, history);
  return responseFromSymbolic(request, chatRequest, prompt, symbolic);
}

/** The text of every assistant message item (`output_messages`), joined. */
function responseAnswerText(response) {
  return response.output
    .filter((item) => item.type === 'message')
    .flatMap((item) => item.content.map((content) => content.text))
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
  const response = await createResponse(ctx, parsed);
  ctx.memory.recordExchange(toChatCompletionRequest(parsed).messages, responseAnswerText(response));
  if (parsed.stream) return responsesSse(response);
  return jsonResponse(200, response);
}
