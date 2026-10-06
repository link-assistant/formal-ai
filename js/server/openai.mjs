// OpenAI Chat Completions (rust/src/protocol.rs `create_chat_completion_*`,
// rust/src/server.rs `chat_completion_sse_response`) - the completion every
// other protocol adapter (Anthropic, Gemini, Vertex) translates through.

import {
  RequestShapeError,
  chatMessageJson,
  chatPromptAndHistory,
  parseChatRequest,
  plainText,
  requestsToolExecution,
} from './chat-request.mjs';
import { sortedKeys, toCompactJson } from './json.mjs';
import { jsonResponse, messageError, sseResponse } from './response.mjs';
import { canonicalModelId, resolveModelId, tryResolveModelId } from './seed.mjs';
import { estimateTokens, solveSymbolic, stableId, toolCallRefusalAnswer } from './solve.mjs';
import { renderThinkingSteps } from './thinking.mjs';

/** Seconds since the epoch, never 0 (`response_timestamp`). */
export function responseTimestamp() {
  return Math.max(1, Math.floor(Date.now() / 1000));
}

function contentTokens(content) {
  if (typeof content === 'string') return estimateTokens(content);
  return content.reduce((total, part) => total + (typeof part.text === 'string' ? estimateTokens(part.text) : 0), 0);
}

/** `message_input_tokens`. */
export function messageInputTokens(messages) {
  return messages.reduce((total, message) => total + contentTokens(message.content), 0);
}

/**
 * `unsupported_model_response`: a 400 for a model id no alias accepts.
 * @param {string|null} model
 */
export function unsupportedModelResponse(model) {
  const trimmed = typeof model === 'string' ? model.trim() : '';
  if (!trimmed || tryResolveModelId(trimmed)) return null;
  return messageError(400, 'unsupported_model', { model: trimmed, canonical: canonicalModelId() });
}

/** `chat_completion_from_symbolic`. */
export function chatCompletionFromSymbolic(request, prompt, symbolic) {
  const promptTokens = messageInputTokens(request.messages);
  const completionTokens = estimateTokens(symbolic.answer);
  const reasoning = renderThinkingSteps(symbolic.thinking_steps);
  return {
    id: stableId('chatcmpl', prompt),
    object: 'chat.completion',
    created: responseTimestamp(),
    model: resolveModelId(request.model),
    choices: [{
      index: 0,
      message: {
        role: 'assistant',
        content: symbolic.answer,
        tool_calls: [],
        thinking_steps: symbolic.thinking_steps,
        reasoning_content: reasoning,
        reasoning,
      },
      finish_reason: 'stop',
    }],
    usage: {
      prompt_tokens: promptTokens,
      completion_tokens: completionTokens,
      total_tokens: promptTokens + completionTokens,
    },
  };
}

/**
 * `create_chat_completion_with_solver_and_memory`.
 *
 * A tool-bearing request without agent mode is refused by policy, exactly as
 * natively. With agent mode on, the JavaScript server answers symbolically:
 * the deterministic agentic planner is native-only today, which the parity
 * ratchet counts.
 * @param {{worker: object, agentMode: boolean}} ctx
 * @param {object} request a parsed `ChatCompletionRequest`
 */
export async function createChatCompletion(ctx, request) {
  const { prompt, history } = chatPromptAndHistory(request.messages);
  if (requestsToolExecution(request) && !ctx.agentMode) {
    return chatCompletionFromSymbolic(request, prompt, toolCallRefusalAnswer());
  }
  const symbolic = await solveSymbolic(ctx, prompt, history);
  return chatCompletionFromSymbolic(request, prompt, symbolic);
}

/** A completion as its wire JSON, in struct field order. */
export function chatCompletionJson(completion) {
  const out = {
    id: completion.id,
    object: completion.object,
    created: completion.created,
    model: completion.model,
    choices: completion.choices.map((choice) => ({
      index: choice.index,
      message: chatMessageJson(choice.message),
      finish_reason: choice.finish_reason,
    })),
    usage: completion.usage,
  };
  if (completion.learning_trace) out.learning_trace = completion.learning_trace;
  return out;
}

function sseChunk(base, choice) {
  return `data: ${toCompactJson(sortedKeys({ ...base, choices: [choice] }))}\n\n`;
}

/** `chat_completion_sse_response`: the `chat.completion.chunk` stream. */
export function chatCompletionSse(completion, includeUsage) {
  const base = {
    id: completion.id,
    object: 'chat.completion.chunk',
    created: completion.created,
    model: completion.model,
  };
  let body = sseChunk(base, { index: 0, delta: { role: 'assistant' }, finish_reason: null });
  const choice = completion.choices[0];
  if (choice) {
    const message = choice.message;
    const reasoning = message.reasoning_content || renderThinkingSteps(message.thinking_steps || []);
    if (reasoning) {
      body += sseChunk(base, { index: 0, delta: { reasoning_content: reasoning, reasoning }, finish_reason: null });
    }
    const text = plainText(message.content);
    if (text) body += sseChunk(base, { index: 0, delta: { content: text }, finish_reason: null });
    (message.tool_calls || []).forEach((call, index) => {
      body += sseChunk(base, {
        index: 0,
        delta: {
          tool_calls: [{
            index,
            id: call.id,
            type: call.type,
            function: { name: call.function.name, arguments: call.function.arguments },
          }],
        },
        finish_reason: null,
      });
    });
  }
  body += sseChunk(base, { index: 0, delta: {}, finish_reason: choice ? choice.finish_reason : 'stop' });
  if (includeUsage) {
    const usage = sortedKeys({
      ...base,
      choices: [],
      usage: {
        prompt_tokens: completion.usage.prompt_tokens,
        completion_tokens: completion.usage.completion_tokens,
        total_tokens: completion.usage.total_tokens,
      },
    });
    body += `data: ${toCompactJson(usage)}\n\n`;
  }
  body += 'data: [DONE]\n\n';
  return sseResponse(body);
}

/** `POST /v1/chat/completions`. */
export async function handleChatCompletions(ctx, request) {
  let chat;
  try {
    chat = parseChatRequest(request.body);
  } catch (error) {
    if (!(error instanceof RequestShapeError)) throw error;
    return messageError(400, 'invalid_chat_request', { error: error.message });
  }
  const unsupported = unsupportedModelResponse(chat.model);
  if (unsupported) return unsupported;
  const completion = await createChatCompletion(ctx, chat);
  ctx.memory.recordExchange(chat.messages, plainText(completion.choices[0]?.message.content ?? ''));
  if (chat.stream) return chatCompletionSse(completion, Boolean(chat.stream_options?.include_usage));
  return jsonResponse(200, chatCompletionJson(completion));
}
