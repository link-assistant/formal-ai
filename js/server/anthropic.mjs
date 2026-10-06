// Anthropic Messages -> OpenAI Chat Completions adapter (rust/src/anthropic.rs,
// and rust/src/server.rs `handle_anthropic_messages_request`).
//
// `claude` speaks `POST /v1/messages`; this module translates the envelope to
// the shared chat request (tool-aware: `tools`, assistant `tool_use` blocks
// and user `tool_result` blocks map onto the OpenAI function-tool shapes),
// solves it, and re-wraps the completion as an Anthropic message or SSE
// stream. Only the envelope is translated; the solver is the shared one.

import {
  RequestShapeError,
  defaultArray,
  defaultBool,
  optionalNumber,
  optionalString,
  parseJsonObject,
  plainText,
} from './chat-request.mjs';
import { stableId } from './ids.mjs';
import { sortedKeys, toCompactJson } from './json.mjs';
import { jsonResponse, messageError, sseResponse } from './response.mjs';
import { createChatCompletion, unsupportedModelResponse } from './openai.mjs';
import { advertisedContextCapacity, contextJson } from './gemini.mjs';
import { resolveModelId } from './seed.mjs';
import { renderThinkingSteps } from './thinking.mjs';

const U32_MAX = 0xffffffff;

function isMap(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function kindOf(value) {
  if (value === null) return 'null';
  return Array.isArray(value) ? 'sequence' : typeof value;
}

/** A serde `invalid type` rejection of `field`. */
function invalidType(field, value, expected) {
  return new RequestShapeError(`${field}: invalid type: ${kindOf(value)}, expected ${expected}`);
}

/** `Value::get`: a member of a JSON object, `undefined` otherwise. */
function member(value, key) {
  return isMap(value) && Object.prototype.hasOwnProperty.call(value, key) ? value[key] : undefined;
}

/** `Value::to_string` of a client value (a `serde_json::Map`, so keys sorted). */
function valueString(value) {
  return toCompactJson(sortedKeys(value));
}

/** `Option<Value>`: `null` deserializes as `None`. */
function optionalValue(object, field) {
  const value = object[field];
  return value === undefined || value === null ? null : value;
}

/** `Option<u32>`. */
function optionalU32(object, field) {
  const value = object[field];
  if (value === undefined || value === null) return null;
  if (!Number.isInteger(value) || value < 0 || value > U32_MAX) {
    throw new RequestShapeError(`${field}: invalid value: ${kindOf(value)} \`${String(value)}\`, expected u32`);
  }
  return value;
}

/** `AnthropicMessageInput`: `role` and `content` are both required. */
function parseMessageInput(value) {
  if (!isMap(value)) {
    throw invalidType('messages', value, 'struct AnthropicMessageInput');
  }
  for (const field of ['role', 'content']) {
    if (!Object.prototype.hasOwnProperty.call(value, field)) {
      throw new RequestShapeError(`missing field \`${field}\``);
    }
  }
  if (typeof value.role !== 'string') {
    throw invalidType('role', value.role, 'a string');
  }
  return { role: value.role, content: value.content };
}

/** `serde_json::from_str::<AnthropicMessagesRequest>` (throws `RequestShapeError`). */
export function parseAnthropicRequest(body) {
  const object = parseJsonObject(body);
  return {
    model: optionalString(object, 'model'),
    messages: defaultArray(object, 'messages').map(parseMessageInput),
    system: optionalValue(object, 'system'),
    max_tokens: optionalU32(object, 'max_tokens'),
    temperature: optionalNumber(object, 'temperature'),
    stream: defaultBool(object, 'stream'),
    tools: defaultArray(object, 'tools'),
    tool_choice: optionalValue(object, 'tool_choice'),
    thinking: optionalValue(object, 'thinking'),
  };
}

/** `AnthropicMessagesRequest::wants_thinking`. */
function wantsThinking(request) {
  return member(request.thinking, 'type') === 'enabled';
}

/** `anthropic_content_to_text`: flatten content to plain text. */
function anthropicContentToText(value) {
  if (typeof value === 'string') return value;
  if (Array.isArray(value)) {
    return value.map(anthropicContentToText).filter((text) => text.trim() !== '').join('\n');
  }
  if (isMap(value)) {
    const text = member(value, 'text');
    return typeof text === 'string' ? text : '';
  }
  return '';
}

function chatMessage(role, content, extra = {}) {
  return { role, content, tool_calls: [], tool_call_id: null, name: null, is_error: false, ...extra };
}

/** `append_anthropic_blocks`. */
function appendAnthropicBlocks(role, blocks, out, toolNamesById) {
  const isAssistant = role.toLowerCase() === 'assistant';
  let text = '';
  const toolCalls = [];
  const toolResults = [];
  for (const block of blocks) {
    const kind = member(block, 'type');
    if (kind === 'text') {
      const chunk = member(block, 'text');
      if (typeof chunk === 'string') {
        if (text) text += '\n';
        text += chunk;
      }
    } else if (kind === 'tool_use') {
      const id = typeof member(block, 'id') === 'string' ? block.id : '';
      const name = typeof member(block, 'name') === 'string' ? block.name : '';
      const input = member(block, 'input');
      const args = input === undefined ? '{}' : valueString(input);
      if (name) toolNamesById.set(id, name);
      toolCalls.push({ id, type: 'function', function: { name, arguments: args } });
    } else if (kind === 'tool_result') {
      const id = typeof member(block, 'tool_use_id') === 'string' ? block.tool_use_id : '';
      const content = member(block, 'content');
      toolResults.push({
        id,
        content: content === undefined ? '' : anthropicContentToText(content),
        isError: member(block, 'is_error') === true,
      });
    }
  }
  if (isAssistant) {
    if (toolCalls.length === 0) {
      if (text.trim()) out.push(chatMessage('assistant', text));
    } else {
      out.push(chatMessage('assistant', text.trim() ? text : '', { tool_calls: toolCalls }));
    }
    return;
  }
  if (text.trim()) out.push(chatMessage(role, text));
  for (const result of toolResults) {
    out.push(chatMessage('tool', result.content, {
      tool_call_id: result.id,
      name: toolNamesById.get(result.id) ?? null,
      is_error: result.isError,
    }));
  }
}

/** `append_anthropic_message`. */
function appendAnthropicMessage(input, out, toolNamesById) {
  const content = input.content;
  if (typeof content === 'string') {
    if (content.trim()) out.push(chatMessage(input.role, content));
  } else if (Array.isArray(content)) {
    appendAnthropicBlocks(input.role, content, out, toolNamesById);
  } else {
    appendAnthropicBlocks(input.role, [content], out, toolNamesById);
  }
}

/** `anthropic_tool_to_openai`. */
function anthropicToolToOpenai(tool) {
  const fn = {};
  for (const [from, to] of [['name', 'name'], ['description', 'description'], ['input_schema', 'parameters']]) {
    const value = member(tool, from);
    if (value !== undefined) fn[to] = value;
  }
  return sortedKeys({ type: 'function', function: fn });
}

/** `anthropic_tool_choice_to_openai`. */
function anthropicToolChoiceToOpenai(choice) {
  const kind = member(choice, 'type');
  if (kind === 'none') return 'none';
  if (kind === 'any') return 'required';
  if (kind === 'tool') {
    const name = member(choice, 'name');
    return { type: 'function', function: { name: name === undefined ? null : name } };
  }
  return 'auto';
}

/** `AnthropicMessagesRequest::to_chat_completion_request`. */
export function anthropicToChatCompletionRequest(request) {
  const messages = [];
  if (request.system !== null) {
    const text = anthropicContentToText(request.system);
    if (text.trim()) messages.push(chatMessage('system', text));
  }
  const toolNamesById = new Map();
  for (const message of request.messages) appendAnthropicMessage(message, messages, toolNamesById);
  return {
    model: request.model,
    messages,
    temperature: request.temperature,
    stream: false,
    tools: request.tools.map(anthropicToolToOpenai),
    tool_choice: request.tool_choice === null ? null : anthropicToolChoiceToOpenai(request.tool_choice),
    functions: [],
    function_call: null,
    stream_options: null,
  };
}

/** `tool_call_to_block`: arguments parsed back into the structured `input`. */
function toolCallToBlock(call) {
  let input;
  try {
    input = JSON.parse(call.function.arguments);
  } catch {
    input = {};
  }
  return { type: 'tool_use', id: call.id, name: call.function.name, input: sortedKeys(input) };
}

/** `anthropic_thinking_block`. */
function anthropicThinkingBlock(steps) {
  const text = renderThinkingSteps(steps);
  if (!text) return null;
  return { type: 'thinking', thinking: text, signature: stableId('thinking_signature', text) };
}

/** `anthropic_message_from_chat_completion`, in `AnthropicMessage` field order. */
export function anthropicMessageFromChatCompletion(request, completion) {
  const model = resolveModelId(request.model);
  const choice = completion.choices[0];
  let content;
  let stopReason;
  let seed;
  if (choice && choice.finish_reason === 'tool_calls') {
    const narration = plainText(choice.message.content);
    const calls = choice.message.tool_calls || [];
    seed = calls.map((call) => `${call.function.name}(${call.function.arguments})`).join('|');
    content = [];
    if (narration.trim()) content.push({ type: 'text', text: narration });
    content.push(...calls.map(toolCallToBlock));
    stopReason = 'tool_use';
  } else {
    const text = choice ? plainText(choice.message.content) : '';
    content = [{ type: 'text', text }];
    stopReason = 'end_turn';
    seed = text;
  }
  if (wantsThinking(request)) {
    const block = anthropicThinkingBlock(choice ? choice.message.thinking_steps || [] : []);
    if (block) content.unshift(block);
  }
  return {
    id: stableId('msg', seed),
    type: 'message',
    role: 'assistant',
    model,
    content,
    stop_reason: stopReason,
    usage: {
      input_tokens: completion.usage.prompt_tokens,
      output_tokens: completion.usage.completion_tokens,
    },
    context: contextJson(advertisedContextCapacity()),
  };
}

function sseEvent(event, data) {
  return `event: ${event}\ndata: ${toCompactJson(sortedKeys(data))}\n\n`;
}

/** The start / delta / stop events of one content block. */
function contentBlockEvents(index, block) {
  let body = '';
  const start = (contentBlock) => sseEvent('content_block_start', { type: 'content_block_start', index, content_block: contentBlock });
  const delta = (value) => sseEvent('content_block_delta', { type: 'content_block_delta', index, delta: value });
  if (block.type === 'thinking') {
    body += start({ type: 'thinking', thinking: '' });
    body += delta({ type: 'thinking_delta', thinking: block.thinking });
    body += delta({ type: 'signature_delta', signature: block.signature });
  } else if (block.type === 'text') {
    body += start({ type: 'text', text: '' });
    body += delta({ type: 'text_delta', text: block.text });
  } else {
    body += start({ type: 'tool_use', id: block.id, name: block.name, input: {} });
    body += delta({ type: 'input_json_delta', partial_json: toCompactJson(block.input) });
  }
  return body + sseEvent('content_block_stop', { type: 'content_block_stop', index });
}

/** `anthropic_message_sse`: the Anthropic Server-Sent-Events stream. */
export function anthropicMessageSse(message) {
  let body = sseEvent('message_start', {
    type: 'message_start',
    message: {
      id: message.id,
      type: 'message',
      role: 'assistant',
      model: message.model,
      content: [],
      stop_reason: null,
      stop_sequence: null,
      context: message.context,
      usage: { input_tokens: message.usage.input_tokens, output_tokens: 0 },
    },
  });
  message.content.forEach((block, index) => {
    body += contentBlockEvents(index, block);
  });
  body += sseEvent('message_delta', {
    type: 'message_delta',
    delta: { stop_reason: message.stop_reason, stop_sequence: null },
    usage: { output_tokens: message.usage.output_tokens },
  });
  body += sseEvent('message_stop', { type: 'message_stop' });
  return body;
}

/** `POST /v1/messages` (`handle_anthropic_messages_request`). */
export async function handleAnthropicMessages(ctx, request) {
  let parsed;
  try {
    parsed = parseAnthropicRequest(request.body);
  } catch (error) {
    if (!(error instanceof RequestShapeError)) throw error;
    return messageError(400, 'invalid_messages_request', { error: error.message });
  }
  const unsupported = unsupportedModelResponse(parsed.model);
  if (unsupported) return unsupported;
  const chatRequest = anthropicToChatCompletionRequest(parsed);
  const completion = await createChatCompletion(ctx, chatRequest);
  const message = anthropicMessageFromChatCompletion(parsed, completion);
  const answer = message.content
    .filter((block) => block.type === 'text')
    .map((block) => block.text)
    .join('\n');
  ctx.memory.recordExchange(chatRequest.messages, answer);
  if (parsed.stream) return sseResponse(anthropicMessageSse(message));
  return jsonResponse(200, message);
}
