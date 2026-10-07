// The Responses request model and its translation into the shared chat
// transcript: rust/src/protocol.rs `ResponsesRequest`
// (`to_chat_completion_request`), rust/src/protocol/responses_input.rs and
// rust/src/protocol/recording.rs (`response_prompt`, `value_to_prompt_text`).

import {
  defaultArray,
  defaultBool,
  optionalNumber,
  optionalString,
  parseJsonObject,
  userRequestText,
} from './chat-request.mjs';

const FAILED_STATUSES = ['failed', 'error', 'errored', 'cancelled', 'canceled'];

const isObject = (value) => Boolean(value) && typeof value === 'object' && !Array.isArray(value);

/** `ResponsesRequest` from a JSON body (throws `RequestShapeError`). */
export function parseResponsesRequest(body) {
  const object = parseJsonObject(body);
  return {
    model: optionalString(object, 'model'),
    input: object.input === undefined ? null : object.input,
    instructions: optionalString(object, 'instructions'),
    temperature: optionalNumber(object, 'temperature'),
    stream: defaultBool(object, 'stream'),
    tools: defaultArray(object, 'tools'),
    tool_choice: object.tool_choice === undefined ? null : object.tool_choice,
  };
}

/** `value_to_prompt_text`. */
export function valueToPromptText(value) {
  if (typeof value === 'string') return value;
  if (Array.isArray(value)) {
    return value.map(valueToPromptText).filter((text) => text.trim()).join('\n');
  }
  if (isObject(value)) {
    const inner = value.content !== undefined ? value.content : value.text;
    return inner === undefined ? '' : valueToPromptText(inner);
  }
  return '';
}

function chatMessage(role, content, extra = {}) {
  return {
    role,
    content,
    tool_calls: [],
    tool_call_id: null,
    name: null,
    is_error: false,
    ...extra,
  };
}

const stringField = (item, ...names) => {
  for (const name of names) {
    if (item[name] !== undefined) return typeof item[name] === 'string' ? item[name] : null;
  }
  return null;
};

/** `call_arguments_text`: a string as given, an object re-encoded (issue #1154). */
const callArgumentsText = (value) => {
  if (typeof value === 'string') return value;
  return isObject(value) ? JSON.stringify(value) : '{}';
};

function appendItem(item, out, toolNamesById) {
  const itemType = typeof item?.type === 'string' ? item.type : 'message';
  if (itemType === 'function_call' || itemType === 'custom_tool_call') {
    const callId = stringField(item, 'call_id', 'id') ?? '';
    const name = typeof item.name === 'string' ? item.name : '';
    const args = callArgumentsText(item.arguments !== undefined ? item.arguments : item.input);
    if (name) toolNamesById.set(callId, name);
    out.push(chatMessage('assistant', '', {
      tool_calls: [{ id: callId, type: 'function', function: { name, arguments: args } }],
    }));
    return;
  }
  if (itemType === 'function_call_output' || itemType === 'custom_tool_call_output') {
    const callId = typeof item.call_id === 'string' ? item.call_id : '';
    const output = item.output === undefined ? '' : valueToPromptText(item.output);
    const flag = item.is_error !== undefined ? item.is_error : item.isError;
    const isError = typeof flag === 'boolean'
      ? flag
      : typeof item.status === 'string' && FAILED_STATUSES.includes(item.status.toLowerCase());
    out.push(chatMessage('tool', output, {
      tool_call_id: callId,
      name: toolNamesById.has(callId) ? toolNamesById.get(callId) : null,
      is_error: isError,
    }));
    return;
  }
  const role = typeof item?.role === 'string' ? item.role : 'user';
  const content = item?.content === undefined ? '' : valueToPromptText(item.content);
  if (content.trim()) out.push(chatMessage(role, content));
}

/** `responses_input::messages`: a bare string, one item, or an item array. */
export function responsesInputMessages(input) {
  const out = [];
  const toolNamesById = new Map();
  if (typeof input === 'string') {
    if (input.trim()) out.push(chatMessage('user', input));
  } else if (Array.isArray(input)) {
    for (const item of input) appendItem(item, out, toolNamesById);
  } else if (isObject(input)) {
    appendItem(input, out, toolNamesById);
  }
  return out;
}

/** `ResponsesRequest::to_chat_completion_request`. */
export function toChatCompletionRequest(request) {
  const messages = responsesInputMessages(request.input);
  if (typeof request.instructions === 'string' && request.instructions.trim()) {
    messages.unshift(chatMessage('system', request.instructions.trim()));
  }
  return {
    model: request.model,
    messages,
    temperature: request.temperature,
    stream: false,
    tools: request.tools,
    tool_choice: request.tool_choice,
    functions: [],
    function_call: null,
    stream_options: null,
  };
}

function latestResponseUserText(input) {
  if (!Array.isArray(input)) return null;
  for (const item of [...input].reverse()) {
    if (!isObject(item) || typeof item.role !== 'string' || item.role.toLowerCase() !== 'user') continue;
    if (item.content === undefined) continue;
    const text = valueToPromptText(item.content);
    if (text.trim()) return text;
  }
  return null;
}

/** `response_prompt`: the latest user text, stripped of injected caller blocks. */
export function responsePrompt(request) {
  const text = latestResponseUserText(request.input) ?? valueToPromptText(request.input);
  return userRequestText(text);
}
