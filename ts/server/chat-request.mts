// The OpenAI Chat Completions request model (rust/src/protocol.rs
// `ChatCompletionRequest`, `ChatMessage`, `MessageContent`) and the prompt /
// history extraction every adapter shares (rust/src/protocol/recording.rs,
// rust/src/protocol/content.rs).
//
// Validation mirrors what serde accepts: a field of the wrong JSON type is a
// 400, unknown fields are ignored, `content: null` is empty text. The error
// detail after the `invalid ... request:` prefix is the parser's own and is
// not part of the parity contract.

import { childrenNamed, parseLino, readRepoFile } from './lino.mjs';
import { serverMessage } from './messages.mjs';

const byteLength = (text) => Buffer.byteLength(text, 'utf8');

/** A request rejected by validation. */
export class RequestShapeError extends Error {}

function typeName(value) {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'sequence';
  if (typeof value === 'object') return 'map';
  if (typeof value === 'number') return Number.isInteger(value) ? 'integer' : 'floating point';
  return typeof value;
}

function fail(field, expected, value) {
  throw new RequestShapeError(`${field}: invalid type: ${typeName(value)} \`${String(value)}\`, expected ${expected}`);
}

/** Parse a JSON body into an object, or throw `RequestShapeError`. */
export function parseJsonObject(body) {
  let value;
  try {
    value = JSON.parse(body);
  } catch (error) {
    throw new RequestShapeError(error.message);
  }
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    fail('request', 'a map', value);
  }
  return value;
}

export function optionalString(object, field) {
  const value = object[field];
  if (value === undefined || value === null) return null;
  if (typeof value !== 'string') fail(field, 'a string', value);
  return value;
}

export function defaultBool(object, field) {
  const value = object[field];
  if (value === undefined) return false;
  if (typeof value !== 'boolean') fail(field, 'a boolean', value);
  return value;
}

export function optionalNumber(object, field) {
  const value = object[field];
  if (value === undefined || value === null) return null;
  if (typeof value !== 'number') fail(field, 'f32', value);
  return value;
}

export function defaultArray(object, field) {
  const value = object[field];
  if (value === undefined) return [];
  if (!Array.isArray(value)) fail(field, 'a sequence', value);
  return value;
}

function untaggedContent(field) {
  return new RequestShapeError(serverMessage('serde_untagged_message_content', { field }));
}

/** `MessageContent`: a string, or an array of `{type, text?}` parts. */
export function parseContent(value, field = 'content') {
  if (value === undefined || value === null) return '';
  if (typeof value === 'string') return value;
  if (Array.isArray(value)) {
    return value.map((part) => {
      if (!part || typeof part !== 'object' || Array.isArray(part) || typeof part.type !== 'string') {
        throw untaggedContent(field);
      }
      const text = part.text === undefined || part.text === null ? null : part.text;
      if (text !== null && typeof text !== 'string') {
        throw untaggedContent(field);
      }
      return { type: part.type, text };
    });
  }
  throw untaggedContent(field);
}

function parseToolCall(value) {
  if (!value || typeof value !== 'object' || typeof value.id !== 'string') fail('tool_calls', 'struct ToolCall', value);
  const fn = value.function;
  if (!fn || typeof fn.name !== 'string' || typeof fn.arguments !== 'string') {
    fail('function', 'struct FunctionCall', fn);
  }
  return { id: value.id, type: typeof value.type === 'string' ? value.type : 'function', function: { name: fn.name, arguments: fn.arguments } };
}

/** `ChatMessage` from JSON. */
export function parseChatMessage(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) fail('messages', 'struct ChatMessage', value);
  if (typeof value.role !== 'string') {
    if (value.role === undefined) throw new RequestShapeError(serverMessage('serde_missing_field', { field: 'role' }));
    fail('role', 'a string', value.role);
  }
  const isError = value.is_error ?? value.isError ?? false;
  return {
    role: value.role,
    content: parseContent(value.content),
    tool_calls: defaultArray(value, 'tool_calls').map(parseToolCall),
    tool_call_id: optionalString(value, 'tool_call_id'),
    name: optionalString(value, 'name'),
    is_error: Boolean(isError),
  };
}

/** `ChatCompletionRequest` from a JSON body (throws `RequestShapeError`). */
export function parseChatRequest(body) {
  const object = parseJsonObject(body);
  const streamOptions = object.stream_options;
  let includeUsage = false;
  if (streamOptions !== undefined && streamOptions !== null) {
    if (typeof streamOptions !== 'object' || Array.isArray(streamOptions)) fail('stream_options', 'struct StreamOptions', streamOptions);
    includeUsage = defaultBool(streamOptions, 'include_usage');
  }
  return {
    model: optionalString(object, 'model'),
    messages: defaultArray(object, 'messages').map(parseChatMessage),
    temperature: optionalNumber(object, 'temperature'),
    stream: defaultBool(object, 'stream'),
    tools: defaultArray(object, 'tools'),
    tool_choice: object.tool_choice ?? null,
    functions: defaultArray(object, 'functions'),
    function_call: object.function_call ?? null,
    stream_options: streamOptions === undefined || streamOptions === null ? null : { include_usage: includeUsage },
  };
}

/** `MessageContent::plain_text`. */
export function plainText(content) {
  if (typeof content === 'string') return content;
  return content.filter((part) => typeof part.text === 'string').map((part) => part.text).join('\n');
}

let injectedBlocks = null;

function callerContextBlocks() {
  if (!injectedBlocks) {
    const root = parseLino(readRepoFile('data/seed/caller-context.lino'));
    const group = childrenNamed(root, 'injected_blocks')[0];
    injectedBlocks = childrenNamed(group, 'block').map((block) => block.value);
  }
  return injectedBlocks;
}

function stripBlock(text, open, close) {
  let remaining = text;
  let request = '';
  for (let start = remaining.indexOf(open); start >= 0; start = remaining.indexOf(open)) {
    request += remaining.slice(0, start);
    const afterOpen = remaining.slice(start + open.length);
    const end = afterOpen.indexOf(close);
    if (end < 0) {
      remaining = '';
      break;
    }
    remaining = afterOpen.slice(end + close.length);
  }
  return request + remaining;
}

/** `MessageContent::user_request_text`. */
export function userRequestText(content) {
  return callerContextBlocks()
    .reduce((text, tag) => stripBlock(text, `<${tag}>`, `</${tag}>`), plainText(content))
    .trim();
}

function stripSystemEcho(request, system) {
  const MIN_ECHO = 40;
  if (!system.trim()) return request;
  const lines = request.split(/\r?\n/);
  for (let start = 1; start < lines.length; start += 1) {
    const tail = lines.slice(start).join('\n').trim();
    if (byteLength(tail) < MIN_ECHO || !system.includes(tail)) continue;
    const head = lines.slice(0, start).join('\n').trim();
    if (head) return head;
  }
  return request;
}

/** `system_prompt_text`. */
export function systemPromptText(messages) {
  return messages
    .filter((message) => message.role.toLowerCase() === 'system')
    .map((message) => plainText(message.content))
    .join('\n');
}

/** `chat_message_to_turn`. */
export function chatMessageToTurn(message) {
  const role = message.role.toLowerCase();
  const content = role === 'user' ? userRequestText(message.content) : plainText(message.content);
  if (!content.trim()) return null;
  if (role === 'user' || role === 'assistant') return { role, content };
  return null;
}

/** `chat_prompt_and_history`: the latest user request and the turns before it. */
export function chatPromptAndHistory(messages) {
  let latest = -1;
  messages.forEach((message, index) => {
    if (message.role.toLowerCase() === 'user') latest = index;
  });
  if (latest < 0) return { prompt: '', history: [] };
  const prompt = stripSystemEcho(userRequestText(messages[latest].content), systemPromptText(messages));
  const history = messages.slice(0, latest).map(chatMessageToTurn).filter(Boolean);
  return { prompt, history };
}

/** `matches_tool_choice_none`. */
export function isToolChoiceNone(value) {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string') return value.toLowerCase() === 'none';
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return typeof value.type === 'string' && value.type.toLowerCase() === 'none';
  }
  return false;
}

/** `is_tool_choice_request`. */
export function isToolChoiceRequest(value) {
  return !isToolChoiceNone(value);
}

/** `ChatCompletionRequest::requests_tool_execution`. */
export function requestsToolExecution(request) {
  if ((request.tool_choice !== null && isToolChoiceRequest(request.tool_choice))
    || (request.function_call !== null && isToolChoiceRequest(request.function_call))) {
    return true;
  }
  const toolsOff = request.tool_choice !== null && isToolChoiceNone(request.tool_choice);
  const functionsOff = request.function_call !== null && isToolChoiceNone(request.function_call);
  return (request.tools.length > 0 && !toolsOff) || (request.functions.length > 0 && !functionsOff);
}

/** `chat_tool_executions`: completed client-side tool runs after the last user turn. */
export function chatToolExecutions(messages) {
  let start = 0;
  messages.forEach((message, index) => {
    if (message.role.toLowerCase() === 'user') start = index;
  });
  const calls = new Map();
  const out = [];
  for (const message of messages.slice(start)) {
    for (const call of message.tool_calls) calls.set(call.id, [call.function.name, call.function.arguments]);
    if (message.role.toLowerCase() !== 'tool' || !message.tool_call_id) continue;
    const called = calls.get(message.tool_call_id);
    if (!called) continue;
    const tool = message.name && message.name.trim() ? message.name : called[0];
    out.push({ tool, inputs: called[1], outputs: plainText(message.content) });
  }
  return out;
}

/** Serialize a `ChatMessage` in struct field order, skipping empty fields. */
export function chatMessageJson(message) {
  const out = { role: message.role, content: message.content };
  if (message.tool_calls?.length) out.tool_calls = message.tool_calls;
  if (message.tool_call_id) out.tool_call_id = message.tool_call_id;
  if (message.name) out.name = message.name;
  if (message.is_error) out.is_error = true;
  if (message.thinking_steps?.length) out.thinking_steps = message.thinking_steps;
  if (message.reasoning_content) out.reasoning_content = message.reasoning_content;
  if (message.reasoning) out.reasoning = message.reasoning;
  return out;
}
