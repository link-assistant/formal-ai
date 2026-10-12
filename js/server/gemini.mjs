// Google Gemini / Vertex `generateContent` adapters over the shared solver
// (rust/src/gemini.rs, and rust/src/server.rs
// `handle_gemini_generate_content_request`, `normalize_protocol_model_id`,
// `is_vendor_hardcoded_gemini_model`).
//
// Gemini content parts become the same chat request the OpenAI and Anthropic
// envelopes use; the completion is wrapped back into a
// `GenerateContentResponse`. Every response value here is a `json!` value in
// Rust, so its keys come out sorted.

import {
  RequestShapeError,
  defaultArray,
  optionalNumber,
  optionalString,
  parseJsonObject,
  plainText,
} from './chat-request.mjs';
import { stableId } from './ids.mjs';
import { f64, sortedKeys, toCompactJson } from './json.mjs';
import { contextCapacity } from './memory.mjs';
import { serverMessage } from './messages.mjs';
import { jsonResponse, messageError, sseResponse } from './response.mjs';
import { createChatCompletion, unsupportedModelResponse } from './openai.mjs';
import { canonicalModelId, resolveModelId } from './seed.mjs';

const OUTPUT_TOKEN_LIMIT = 8192;
const DEFAULT_AVG_UTF8_BYTES_PER_CHAR = 2;

/** `avg_utf8_bytes_per_char`. */
function avgUtf8BytesPerChar() {
  const raw = process.env.FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR;
  const value = raw !== undefined && /^\d+$/.test(raw) ? Number(raw) : 0;
  return value > 0 ? value : DEFAULT_AVG_UTF8_BYTES_PER_CHAR;
}

/**
 * `ContextCapacity::current()` falling back to `from_bytes(0, 0, avg)`, as
 * the plain capacity record.
 */
export function advertisedContextCapacity() {
  try {
    return contextCapacity();
  } catch {
    return {
      context_window_tokens: 0,
      context_used_tokens: 0,
      context_used_fraction: 0,
      disk_free_bytes: 0,
      memory_used_bytes: 0,
      avg_utf8_bytes_per_char: avgUtf8BytesPerChar(),
    };
  }
}

/** `json!(context)`: the capacity as a `Value` (keys sorted, fraction a float). */
export function contextJson(capacity) {
  return sortedKeys({
    context_window_tokens: capacity.context_window_tokens,
    context_used_tokens: capacity.context_used_tokens,
    context_used_fraction: f64(capacity.context_used_fraction),
    disk_free_bytes: capacity.disk_free_bytes,
    memory_used_bytes: capacity.memory_used_bytes,
    avg_utf8_bytes_per_char: capacity.avg_utf8_bytes_per_char,
  });
}

/** `normalize_protocol_model_id`. @param {string} model */
export function normalizeProtocolModelId(model) {
  const text = String(model ?? '');
  return (text.startsWith('models/') ? text.slice('models/'.length) : text).trim();
}

/** `is_vendor_hardcoded_gemini_model`. @param {string} model */
export function isVendorHardcodedGeminiModel(model) {
  return model.trim().toLowerCase().startsWith('gemini-');
}

// ---- request parsing (serde shape of `GeminiGenerateContentRequest`) -------

function isMap(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function expectMap(field, value, struct) {
  if (!isMap(value)) {
    throw new RequestShapeError(`${field}: invalid type: ${Array.isArray(value) ? 'sequence' : typeof value}, expected struct ${struct}`);
  }
}

/** A JSON value that `Option<Value>` deserializes: `null` is `None`. */
function optionalValue(object, field) {
  const value = object[field];
  return value === undefined || value === null ? null : value;
}

function parsePart(value) {
  expectMap('parts', value, 'GeminiPart');
  return {
    text: optionalString(value, 'text'),
    functionCall: optionalValue(value, 'functionCall'),
    functionResponse: optionalValue(value, 'functionResponse'),
  };
}

function parseContentEntry(value, field) {
  expectMap(field, value, 'GeminiContent');
  return {
    role: optionalString(value, 'role'),
    parts: defaultArray(value, 'parts').map(parsePart),
  };
}

/** `serde_json::from_str::<GeminiGenerateContentRequest>` (throws `RequestShapeError`). */
export function parseGeminiRequest(body) {
  const object = parseJsonObject(body);
  const system = optionalValue(object, 'systemInstruction');
  const config = optionalValue(object, 'generationConfig');
  if (config !== null) expectMap('generationConfig', config, 'GeminiGenerationConfig');
  return {
    contents: defaultArray(object, 'contents').map((entry) => parseContentEntry(entry, 'contents')),
    systemInstruction: system === null ? null : parseContentEntry(system, 'systemInstruction'),
    generationConfig: config === null ? null : { temperature: optionalNumber(config, 'temperature') },
    tools: defaultArray(object, 'tools'),
  };
}

// ---- translation to a chat request -----------------------------------------

/** `GeminiContent::text`. */
function contentText(content) {
  return content.parts
    .map((part) => part.text)
    .filter((text) => text !== null && text.trim() !== '')
    .join('\n');
}

/** `gemini_role_to_chat_role`. */
function geminiRoleToChatRole(role) {
  const lower = typeof role === 'string' ? role.toLowerCase() : '';
  if (lower === 'model' || lower === 'assistant') return 'assistant';
  if (lower === 'system') return 'system';
  return 'user';
}

function chatMessage(role, content, extra = {}) {
  return { role, content, tool_calls: [], tool_call_id: null, name: null, is_error: false, ...extra };
}

/** `ToolCall::function`. */
function toolCall(id, name, args) {
  return { id, type: 'function', function: { name, arguments: args } };
}

/** `Value::get`: a member of a JSON object, `undefined` otherwise. */
function member(value, key) {
  return isMap(value) && Object.prototype.hasOwnProperty.call(value, key) ? value[key] : undefined;
}

/** `Value::to_string` of a client value (a `serde_json::Map`, so keys sorted). */
function valueString(value) {
  return toCompactJson(sortedKeys(value));
}

function hostedGeminiTool(name) {
  return { type: 'function', function: { name, parameters: { type: 'object' } } };
}

/** `gemini_function_declaration_to_openai`. */
function functionDeclarationToOpenai(declaration) {
  const name = member(declaration, 'name');
  if (name === undefined) return null;
  const fn = { name };
  const description = member(declaration, 'description');
  if (description !== undefined) fn.description = description;
  const parameters = member(declaration, 'parameters');
  if (parameters !== undefined) fn.parameters = parameters;
  return sortedKeys({ type: 'function', function: fn });
}

/** `gemini_tools_to_openai`. */
function geminiToolsToOpenai(tools) {
  const definitions = [];
  for (const tool of tools) {
    const declarations = member(tool, 'functionDeclarations');
    if (Array.isArray(declarations)) {
      for (const declaration of declarations) {
        const definition = functionDeclarationToOpenai(declaration);
        if (definition) definitions.push(definition);
      }
    }
    if (member(tool, 'google_search') !== undefined || member(tool, 'googleSearch') !== undefined) {
      definitions.push(hostedGeminiTool('web_search'));
    }
    if (member(tool, 'url_context') !== undefined || member(tool, 'urlContext') !== undefined) {
      definitions.push(hostedGeminiTool('web_fetch'));
    }
  }
  return definitions;
}

/** `GeminiGenerateContentRequest::to_chat_completion_request`. */
export function geminiToChatCompletionRequest(request, model) {
  const messages = [];
  if (request.systemInstruction) {
    const text = contentText(request.systemInstruction);
    if (text.trim()) messages.push(chatMessage('system', text));
  }
  const callsByName = new Map();
  request.contents.forEach((content, contentIndex) => {
    const text = contentText(content);
    if (text.trim()) messages.push(chatMessage(geminiRoleToChatRole(content.role), text));
    const calls = [];
    content.parts.forEach((part, partIndex) => {
      const call = part.functionCall;
      const name = member(call, 'name');
      if (typeof name !== 'string') return;
      const args = member(call, 'args');
      const argumentsText = valueString(args === undefined ? {} : args);
      const givenId = member(call, 'id');
      const id = typeof givenId === 'string'
        ? givenId
        : stableId('gemini_call', `${contentIndex}:${partIndex}:${name}:${argumentsText}`);
      callsByName.set(name, id);
      calls.push(toolCall(id, name, argumentsText));
    });
    if (calls.length) messages.push(chatMessage('assistant', '', { tool_calls: calls }));
    for (const part of content.parts) {
      const response = part.functionResponse;
      const name = member(response, 'name');
      if (typeof name !== 'string') continue;
      const givenId = member(response, 'id');
      const id = typeof givenId === 'string'
        ? givenId
        : (callsByName.get(name) ?? stableId('gemini_call', name));
      const output = member(response, 'response') ?? null;
      const result = typeof output === 'string' ? output : valueString(output);
      messages.push(chatMessage('tool', result, { tool_call_id: id, name }));
    }
  });
  return {
    model: resolveModelId(model),
    messages,
    temperature: request.generationConfig ? request.generationConfig.temperature : null,
    stream: false,
    tools: geminiToolsToOpenai(request.tools),
    tool_choice: null,
    functions: [],
    function_call: null,
    stream_options: null,
  };
}

// ---- responses ---------------------------------------------------------------

/** Parse tool-call arguments back into a value, `{}` when they are not JSON. */
function parseArguments(text) {
  try {
    return JSON.parse(text);
  } catch {
    return {};
  }
}

/** `gemini_response_from_chat_completion`. */
export function geminiResponseFromChatCompletion(completion) {
  const choice = completion.choices[0];
  let parts = [];
  if (choice) {
    const calls = choice.message.tool_calls || [];
    if (calls.length === 0) {
      const text = plainText(choice.message.content);
      if (text) parts = [{ text }];
    } else {
      const narration = plainText(choice.message.content);
      if (narration.trim()) parts.push({ text: narration });
      for (const call of calls) {
        parts.push({ functionCall: { id: call.id, name: call.function.name, args: parseArguments(call.function.arguments) } });
      }
    }
  }
  return sortedKeys({
    candidates: [{
      content: { role: 'model', parts },
      finishReason: 'STOP',
      index: 0,
    }],
    usageMetadata: {
      promptTokenCount: completion.usage.prompt_tokens,
      candidatesTokenCount: completion.usage.completion_tokens,
      totalTokenCount: completion.usage.total_tokens,
    },
    modelVersion: completion.model,
  });
}

/** `gemini_response_sse`. */
export function geminiResponseSse(response) {
  return `data: ${toCompactJson(response)}\n\n`;
}

/** `gemini_model_metadata`. */
export function geminiModelMetadata(name) {
  const capacity = advertisedContextCapacity();
  return sortedKeys({
    name,
    version: '001',
    displayName: canonicalModelId(),
    description: serverMessage('gemini_model_description'),
    inputTokenLimit: capacity.context_window_tokens,
    outputTokenLimit: OUTPUT_TOKEN_LIMIT,
    context: contextJson(capacity),
    supportedGenerationMethods: ['generateContent', 'streamGenerateContent'],
  });
}

/** `gemini_model_list`. */
export function geminiModelList() {
  return { models: [geminiModelMetadata(`models/${canonicalModelId()}`)] };
}

/** `vertex_model_metadata`. */
function vertexModelMetadata(project, location) {
  const capacity = advertisedContextCapacity();
  return sortedKeys({
    name: `projects/${project}/locations/${location}/publishers/google/models/${canonicalModelId()}`,
    versionId: '001',
    displayName: canonicalModelId(),
    description: serverMessage('gemini_vertex_model_description'),
    inputTokenLimit: capacity.context_window_tokens,
    outputTokenLimit: OUTPUT_TOKEN_LIMIT,
    context: contextJson(capacity),
    supportedActions: { generateContent: {}, streamGenerateContent: {} },
  });
}

/** `vertex_model_list`. */
export function vertexModelList(project, location) {
  return { publisherModels: [vertexModelMetadata(project, location)] };
}

// ---- handlers ----------------------------------------------------------------

/** `GET /api/gemini/v1beta/models`. */
export function handleGeminiModels() {
  return jsonResponse(200, geminiModelList());
}

/** `GET /api/gemini/v1beta/models/{model}`. */
export function handleGeminiModel(_ctx, request) {
  const model = normalizeProtocolModelId(request.params.model);
  return jsonResponse(200, geminiModelMetadata(`models/${model}`));
}

/** `GET /api/vertex/v1/projects/{project}/locations/{location}/publishers/google/models`. */
export function handleVertexModels(_ctx, request) {
  return jsonResponse(200, vertexModelList(request.params.project, request.params.location));
}

/**
 * `handle_gemini_generate_content_request`, for both the Gemini and the Vertex
 * routes. The route already normalized the path model once; the handler
 * normalizes it again, exactly as natively.
 */
export async function handleGeminiGenerateContent(ctx, request, stream) {
  let model = normalizeProtocolModelId(normalizeProtocolModelId(request.params.model));
  if (isVendorHardcodedGeminiModel(model)) {
    // Answer as ourselves, never as the vendor model the CLI hardcodes.
    model = canonicalModelId();
  } else {
    const unsupported = unsupportedModelResponse(model);
    if (unsupported) return unsupported;
  }
  let parsed;
  try {
    parsed = parseGeminiRequest(request.body);
  } catch (error) {
    if (!(error instanceof RequestShapeError)) throw error;
    return messageError(400, 'invalid_generate_content_request', { error: error.message });
  }
  const chatRequest = geminiToChatCompletionRequest(parsed, model);
  const completion = await createChatCompletion(ctx, chatRequest);
  const response = geminiResponseFromChatCompletion(completion);
  const answer = response.candidates[0].content.parts
    .filter((part) => typeof part.text === 'string')
    .map((part) => part.text)
    .join('\n');
  ctx.memory.recordExchange(chatRequest.messages, answer);
  return stream ? sseResponse(geminiResponseSse(response)) : jsonResponse(200, response);
}
