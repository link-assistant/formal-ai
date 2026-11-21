// OpenAI Chat Completions (rust/src/protocol.rs `create_chat_completion_*`,
// rust/src/server.rs `chat_completion_sse_response`) - the completion every
// other protocol adapter (Anthropic, Gemini, Vertex) translates through.

import {
  RequestShapeError,
  chatMessageJson,
  chatPromptAndHistory,
  parseChatRequest,
  plainText,
} from './chat-request.mjs';
import { logUnclassifiedTerminal, agenticOutcome, chatCompletionFromPlan, commandReroutePlan } from './agentic.mjs';
import { sortedKeys, toCompactJson } from './json.mjs';
import { jsonResponse, messageError, sseResponse } from './response.mjs';
import { learningTraceFromSymbolicAnswer } from './self-improvement.mjs';
import { canonicalModelId, resolveModelId, tryResolveModelId } from './seed.mjs';
import { answerFromMemory, estimateTokens, solveWithMemory, stableId, toolCallRefusalAnswer } from './solve.mjs';
import { renderThinkingSteps } from './thinking.mjs';
import { requestScope, runInRequestScope } from './standing-scope.mjs';
import { projectServerAnswer, requestOutputTurn } from './cnl-output.mjs';
import { formalizeRequest, contextualAnswer, composeRequestAnswer } from './request-context.mjs';
import { outputObligation, projectStructuredOutput, structuredAnswerFromSource, structuredOutputRequested, structuredThinkingStep } from './structured-output.mjs';
import { clientFunctionOutput, forcedFunction } from './client-functions.mjs';
import { recipePrompt } from './dynamic-meta/wire.mjs';

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
  const structured = structuredOutputRequested(request.response_format);
  if (!structured) symbolic = composeRequestAnswer(request.messages, symbolic);
  const promptTokens = messageInputTokens(request.messages);
  const completionTokens = estimateTokens(symbolic.answer);
  const reasoning = renderThinkingSteps(symbolic.thinking_steps);
  return {
    ...(symbolic.output_projection ? {output_projection:symbolic.output_projection} : {}),
    id: stableId('chatcmpl', prompt),
    object: 'chat.completion',
    created: responseTimestamp(),
    model: resolveModelId(request.model),
    choices: [{
      index: 0,
      message: {
        role: 'assistant',
        content: symbolic.answer,
        ...(structured && symbolic.intent === 'tool_call_refused' ? { refusal: symbolic.answer } : {}),
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
    learning_trace: learningTraceFromSymbolicAnswer(prompt, symbolic),
  };
}

/**
 * `create_chat_completion_with_solver_and_memory`.
 *
 * Agentic requests require agent mode. With agent mode on, the planner port
 * (js/agentic/, through js/server/agentic.mjs) emits the next tool calls.
 * @param {{worker: object, agentMode: boolean}} ctx
 * @param {object} request a parsed `ChatCompletionRequest`
 */
export async function createChatCompletion(ctx, request, options = {}) {
  const original = chatPromptAndHistory(request.messages);
  const turn = requestOutputTurn(original.prompt, original.history, request, ctx.outputConfiguration);
  const selected = {...ctx, outputConfiguration:turn.config};
  const context = formalizeRequest(request.messages);
  let completion;
  // A rewrite is resolved before memory lookup, planning or tool permission paths.
  if (turn.rewrite !== null) {
    const symbolic = await projectServerAnswer(selected, {intent:'cnl_paraphrase',answer:turn.rewrite,
      confidence:1,evidence_links:['cnl:paraphrase'],thinking_steps:[],links_notation:''}, turn.config, {language: context.language});
    completion = chatCompletionFromSymbolic(request, original.prompt, symbolic);
  } else {
    // Strip a recognized output directive only from the latest plain-text user
    // message. Preserve structured content, tool results and command arguments.
    let planningRequest = request;
    const latest = request.messages.findLastIndex(message => message.role.toLowerCase() === 'user');
    const message = request.messages[latest];
    if (turn.prompt !== original.prompt && typeof message?.content === 'string') {
      const start = message.content.indexOf(original.prompt);
      if (start >= 0) {
        const content = message.content.slice(0, start) + turn.prompt + message.content.slice(start + original.prompt.length);
        planningRequest = {...request, messages:request.messages.map((item,index) => index === latest ? {...item,content} : item)};
      }
    }
    completion = await createUnprojectedChatCompletion(selected, planningRequest, options);
  }
  const choice = completion.choices[0];
  const obligation = outputObligation(request.response_format, true);
  if (obligation) {
    completion.response_format = obligation.format;
    if (choice?.finish_reason !== 'stop' || choice.message.tool_calls?.length) return completion;
    const message = choice.message;
    const result = await projectStructuredOutput(ctx, obligation, original.prompt, plainText(message.content), message.refusal);
    message.content = result.content;
    if (result.refusal) message.refusal = result.refusal;
    const steps = [...(message.thinking_steps ?? [])];
    steps.push(structuredThinkingStep(steps, result));
    message.thinking_steps = steps;
    completion.derivation_id = result.id;
    completion.evidence_links = result.evidence;
    delete completion.output_projection;
    completion.usage.completion_tokens = estimateTokens(result.refusal ?? result.content);
    completion.usage.total_tokens = completion.usage.prompt_tokens + completion.usage.completion_tokens;
    return completion;
  }
  if (choice?.finish_reason === 'stop' && typeof choice.message.content === 'string' && choice.message.content !== '') {
    const projected = await projectServerAnswer(selected, {answer:choice.message.content,
      output_projection:completion.output_projection}, turn.config, {
      language: context.language,
      preserveAnswerBody: context.preserveAnswerBody,
      answerText: context.answerText(choice.message.content),
    });
    choice.message.content = projected.answer;
    completion.output_projection = projected.output_projection;
    completion.usage.completion_tokens = estimateTokens(projected.answer);
    completion.usage.total_tokens = completion.usage.prompt_tokens + completion.usage.completion_tokens;
  }
  return completion;
}

async function createUnprojectedChatCompletion(ctx, request, { forcedClientFunctions = true } = {}) {
  const { prompt, history } = chatPromptAndHistory(request.messages);
  const forced = forcedClientFunctions ? await forcedFunction(ctx, request, prompt) : null;
  if (forced) {
    const { callId, outputTokens } = clientFunctionOutput(forced, prompt, request.messages.length);
    const message = { role: 'assistant', content: '' };
    if (forced.refusal) message.refusal = forced.refusal;
    else message.tool_calls = [{ id: callId, type: 'function', function: { name: forced.name, arguments: forced.arguments } }];
    const promptTokens = messageInputTokens(request.messages);
    return {
      id: stableId('chatcmpl', prompt),
      object: 'chat.completion',
      created: responseTimestamp(),
      model: resolveModelId(request.model),
      choices: [{ index: 0, message, finish_reason: callId ? 'tool_calls' : 'stop' }],
      usage: { prompt_tokens: promptTokens, completion_tokens: outputTokens, total_tokens: promptTokens + outputTokens },
    };
  }
  const recipe = ctx.dynamicMeta && ctx.bearerToken && recipePrompt(prompt);
  if (recipe) {
    let answer;
    let evidence = [];
    try {
      const result = await ctx.dynamicMeta.act(recipe);
      answer = JSON.stringify(result);
      evidence = [`dynamic-meta:receipt:${result.revision}`];
    } catch (error) {
      if (error.code?.startsWith('ERR_SQLITE')) throw error;
      answer = error.message;
    }
    return chatCompletionFromSymbolic(request, prompt, {
      intent: 'dynamic_recipe', answer, evidence_links: evidence, thinking_steps: [], links_notation: '',
    });
  }
  const context = formalizeRequest(request.messages);
  const remembered = await answerFromMemory(ctx, prompt, history);
  if (remembered) {logUnclassifiedTerminal(ctx,'retained_answer');return chatCompletionFromSymbolic(request, prompt, remembered);}
  const outcome = await agenticOutcome(ctx, request, toolCallRefusalAnswer);
  if (outcome.kind === 'refused') return chatCompletionFromSymbolic(request, prompt, outcome.answer);
  if (outcome.kind === 'planned') return chatCompletionFromPlan(ctx, request, prompt, outcome.plan, responseTimestamp());
  const contextual = contextualAnswer(context);
  if (contextual) return chatCompletionFromSymbolic(request, prompt, contextual);
  const symbolic = await structuredAnswerFromSource(ctx, request.response_format, true, prompt)
    ?? await solveWithMemory(ctx, prompt, history, { forcedResponseLanguage: context.language });
  const reroute = await commandReroutePlan(ctx, request, symbolic);
  if (reroute) return chatCompletionFromPlan(ctx, request, prompt, reroute, responseTimestamp());
  logUnclassifiedTerminal(ctx,'symbolic_answer');
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
  if (completion.response_format !== undefined) out.response_format = completion.response_format;
  if (completion.derivation_id) {
    out.derivation_id = completion.derivation_id;
    out.evidence_links = completion.evidence_links;
  }
  if (completion.output_projection) out.output_projection = completion.output_projection;
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
    if (message.refusal) body += sseChunk(base, { index: 0, delta: { refusal: message.refusal }, finish_reason: null });
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
  const metadata = { ...base };
  let delta = {};
  if (completion.output_projection) metadata.output_projection = completion.output_projection;
  if (completion.response_format !== undefined) metadata.response_format = completion.response_format;
  if (completion.derivation_id) {
    metadata.derivation_id = completion.derivation_id;
    metadata.evidence_links = completion.evidence_links;
    delta = { thinking_steps: choice.message.thinking_steps };
  }
  body += sseChunk(metadata, { index: 0, delta, finish_reason: choice ? choice.finish_reason : 'stop' });
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
  const scope = requestScope({ body: request.body, messages: chat.messages, tools: chat.tools });
  const completion = await runInRequestScope(scope, () => createChatCompletion(ctx, chat));
  ctx.memory.recordExchange(chat.messages, plainText(completion.choices[0]?.message.content ?? ''), null, scope);
  if (chat.stream) return chatCompletionSse(completion, Boolean(chat.stream_options?.include_usage));
  return jsonResponse(200, chatCompletionJson(completion));
}
