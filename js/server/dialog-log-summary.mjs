// The proxy-exchange summary a dialog-log record carries
// (rust/src/proxy.rs `summarize_proxy_exchange`, `model_from_path`,
// `collect_request_tool_names`; rust/src/proxy/summary.rs; and
// rust/src/protocol_policy.rs `tool_definition_names`).
//
// JSON values are parsed into plain objects (floats kept as float markers,
// js/server/json.mjs `parseJson`); the caller serializes them with sorted
// keys, as a serde_json `Value` (a B-tree map) prints.

import { isFloat, parseJson as parseJsonValue } from './json.mjs';

const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value) && !isFloat(value);
const get = (value, key) => (isObject(value) && Object.prototype.hasOwnProperty.call(value, key) ? value[key] : undefined);
const asString = (value) => (typeof value === 'string' ? value : null);

function parseJson(text) {
  try {
    return { ok: true, value: parseJsonValue(text) };
  } catch {
    return { ok: false, value: null };
  }
}

/** The first `count` Unicode scalars of `text`. */
function takeChars(text, count) {
  return Array.from(text).slice(0, count).join('');
}

function hostedToolTypeName(object) {
  const kind = asString(get(object, 'type'));
  if (kind === null) return null;
  if (kind === 'web_search' || kind === 'web_search_preview') return 'web_search';
  if (kind === 'file_search') return 'file_search';
  if (kind === 'computer_use' || kind === 'computer_use_preview') return 'computer_use';
  if (kind === 'code_interpreter') return 'code_interpreter';
  if (kind.startsWith('web_search_')) return 'web_search';
  return null;
}

/** `tool_definition_name`. */
function toolDefinitionName(value) {
  if (!isObject(value)) return null;
  const fnName = get(get(value, 'function'), 'name');
  const raw = fnName !== undefined ? fnName : get(value, 'name');
  const name = asString(raw);
  return name !== null ? name : hostedToolTypeName(value);
}

function qualifyToolName(prefix, name) {
  if (prefix === null) return name;
  const separator = prefix.endsWith('__') ? '' : '__';
  return name.startsWith(`${prefix}${separator}`) ? name : `${prefix}${separator}${name}`;
}

function appendQualifiedNames(value, prefix, names) {
  if (!isObject(value)) return;
  if (get(value, 'type') === 'namespace') {
    const namespace = asString(get(value, 'name'));
    if (namespace === null) return;
    const qualified = qualifyToolName(prefix, namespace);
    const children = get(value, 'tools');
    if (Array.isArray(children)) for (const child of children) appendQualifiedNames(child, qualified, names);
    return;
  }
  const name = toolDefinitionName(value);
  if (name !== null) names.push(qualifyToolName(prefix, name));
}

/** `collect_request_tool_names`: sorted, deduplicated. */
function collectRequestToolNames(value) {
  const names = [];
  const tools = get(value, 'tools');
  if (Array.isArray(tools)) {
    for (const tool of tools) {
      appendQualifiedNames(tool, null, names);
      const declarations = get(tool, 'functionDeclarations');
      if (Array.isArray(declarations)) {
        for (const declaration of declarations) {
          const name = asString(get(declaration, 'name'));
          if (name !== null) names.push(name);
        }
      }
    }
  }
  const functions = get(value, 'functions');
  if (Array.isArray(functions)) {
    for (const fn of functions) {
      const name = asString(get(fn, 'name'));
      if (name !== null) names.push(name);
    }
  }
  for (const key of ['tool_choice', 'function_call']) {
    const choice = get(value, key);
    if (choice === undefined) continue;
    const fn = get(choice, 'function');
    const name = asString(get(fn !== undefined ? fn : choice, 'name'));
    if (name !== null) names.push(name);
  }
  const sorted = names.sort(byteOrder);
  return sorted.filter((name, index) => index === 0 || name !== sorted[index - 1]);
}

/** Rust string ordering (UTF-8 bytes). */
function byteOrder(left, right) {
  return Buffer.compare(Buffer.from(left, 'utf8'), Buffer.from(right, 'utf8'));
}

/** `model_from_path`: the model a Gemini/Vertex path names. */
function modelFromPath(path) {
  const at = path.lastIndexOf('/models/');
  if (at < 0) return null;
  const after = path.slice(at + '/models/'.length);
  let model = after.split(/[?#]/)[0].split(':')[0];
  while (model.endsWith('/')) model = model.slice(0, -1);
  model = model.trim();
  return model ? model : null;
}

const emptySummary = () => ({ model: null, tool_calls: [], content: '' });

function argumentsFromStr(text) {
  const parsed = parseJson(text);
  return parsed.ok ? parsed.value : text;
}

function argumentsFromValue(value) {
  if (value === undefined) return null;
  return typeof value === 'string' ? argumentsFromStr(value) : value;
}

function setModel(summary, model) {
  if (summary.model === null && model !== null) summary.model = model;
}

function appendContentValue(value, summary) {
  if (typeof value === 'string') summary.content += value;
  else if (Array.isArray(value)) for (const part of value) appendContentValue(get(part, 'text'), summary);
}

function appendFunctionCall(fn, summary) {
  const name = asString(get(fn, 'name'));
  if (name !== null) summary.tool_calls.push({ name, arguments: argumentsFromValue(get(fn, 'arguments')) });
}

function applyChatMessage(message, summary) {
  const calls = get(message, 'tool_calls');
  if (Array.isArray(calls)) {
    for (const call of calls) {
      const fn = get(call, 'function');
      if (fn !== undefined) appendFunctionCall(fn, summary);
    }
  }
  const legacy = get(message, 'function_call');
  if (legacy !== undefined) appendFunctionCall(legacy, summary);
  appendContentValue(get(message, 'content'), summary);
}

function applyResponseItem(item, summary) {
  const kind = asString(get(item, 'type'));
  const name = asString(get(item, 'name'));
  if (kind === 'function_call' && name !== null) {
    summary.tool_calls.push({ name, arguments: argumentsFromValue(get(item, 'arguments')) });
  } else if (kind === 'custom_tool_call' && name !== null) {
    summary.tool_calls.push({ name, arguments: argumentsFromValue(get(item, 'input')) });
  } else if (kind === 'message') {
    const content = get(item, 'content');
    if (Array.isArray(content)) for (const part of content) appendContentValue(get(part, 'text'), summary);
  }
}

function applyGeminiPart(part, summary) {
  const call = get(part, 'functionCall');
  const name = asString(get(call, 'name'));
  if (call !== undefined && name !== null) {
    summary.tool_calls.push({ name, arguments: argumentsFromValue(get(call, 'args')) });
  }
  appendContentValue(get(part, 'text'), summary);
}

function applyAnthropicBlock(block, summary) {
  const kind = asString(get(block, 'type'));
  if (kind === 'tool_use') {
    const name = asString(get(block, 'name'));
    if (name !== null) summary.tool_calls.push({ name, arguments: argumentsFromValue(get(block, 'input')) });
  } else if (kind === 'text') {
    appendContentValue(get(block, 'text'), summary);
  }
}

function applyResponseValue(value, summary) {
  setModel(summary, asString(get(value, 'model')));
  setModel(summary, asString(get(value, 'modelVersion')));
  const response = get(value, 'response');
  if (response !== undefined) applyResponseValue(response, summary);
  const item = get(value, 'item');
  if (item !== undefined) applyResponseItem(item, summary);
  const choices = get(value, 'choices');
  if (Array.isArray(choices)) {
    for (const choice of choices) {
      const message = get(choice, 'message');
      if (message !== undefined) applyChatMessage(message, summary);
    }
  }
  const output = get(value, 'output');
  if (Array.isArray(output)) for (const entry of output) applyResponseItem(entry, summary);
  const kind = asString(get(value, 'type'));
  if (kind === 'function_call' || kind === 'custom_tool_call') applyResponseItem(value, summary);
  const candidates = get(value, 'candidates');
  if (Array.isArray(candidates)) {
    for (const candidate of candidates) {
      const parts = get(get(candidate, 'content'), 'parts');
      if (Array.isArray(parts)) for (const part of parts) applyGeminiPart(part, summary);
    }
  }
  const content = get(value, 'content');
  if (Array.isArray(content)) for (const block of content) applyAnthropicBlock(block, summary);
}

/** `summarize_response_value`. */
function summarizeResponseValue(value) {
  const summary = emptySummary();
  applyResponseValue(value, summary);
  return summary;
}

function mergeSummary(target, source) {
  if (target.model === null) target.model = source.model;
  target.tool_calls.push(...source.tool_calls);
  target.content += source.content;
}

/** Rust `str::lines`. */
function lines(text) {
  const out = text.split('\n');
  if (out.length > 0 && out[out.length - 1] === '') out.pop();
  return out.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
}

/** `parse_sse_events`: the `data:` payload of each blank-line-separated block. */
function parseSseEvents(body) {
  const events = [];
  for (const block of body.split('\r\n').join('\n').split('\n\n')) {
    let data = '';
    for (const line of lines(block)) {
      if (!line.startsWith('data:')) continue;
      if (data) data += '\n';
      data += line.slice('data:'.length).replace(/^\s+/u, '');
    }
    if (data) events.push(data);
  }
  return events;
}

function accumulatedCalls(calls) {
  return [...calls.keys()].sort((a, b) => a - b)
    .map((key) => calls.get(key))
    .filter((call) => call.name)
    .map((call) => ({ name: call.name, arguments: argumentsFromStr(call.arguments) }));
}

const nonNegativeInteger = (value) => (Number.isInteger(value) && value >= 0 ? value : null);

function callAt(calls, index) {
  if (!calls.has(index)) calls.set(index, { name: '', arguments: '' });
  return calls.get(index);
}

/** `StreamingChatAccumulator::apply_chunk`. */
function applyChatChunk(chat, value) {
  const choices = get(value, 'choices');
  if (!Array.isArray(choices)) return false;
  if (chat.model === null) chat.model = asString(get(value, 'model'));
  for (const choice of choices) {
    const delta = get(choice, 'delta');
    if (delta === undefined) continue;
    const content = asString(get(delta, 'content'));
    if (content !== null) chat.content += content;
    const calls = get(delta, 'tool_calls');
    if (!Array.isArray(calls)) continue;
    for (const call of calls) {
      const entry = callAt(chat.calls, nonNegativeInteger(get(call, 'index')) ?? 0);
      const fn = get(call, 'function');
      const name = asString(get(fn, 'name'));
      if (name !== null) entry.name += name;
      const args = asString(get(fn, 'arguments'));
      if (args !== null) entry.arguments += args;
    }
  }
  return true;
}

const ANTHROPIC_PASSIVE = ['content_block_stop', 'message_delta', 'message_stop', 'ping'];

/** `StreamingAnthropicAccumulator::apply_event`. */
function applyAnthropicEvent(acc, value) {
  const kind = asString(get(value, 'type'));
  if (kind === null) return false;
  const index = nonNegativeInteger(get(value, 'index')) ?? 0;
  if (kind === 'message_start') {
    if (acc.model === null) acc.model = asString(get(get(value, 'message'), 'model'));
  } else if (kind === 'content_block_start') {
    const block = get(value, 'content_block');
    const name = asString(get(block, 'name'));
    if (get(block, 'type') === 'tool_use' && name !== null) callAt(acc.calls, index).name += name;
  } else if (kind === 'content_block_delta') {
    const delta = get(value, 'delta');
    const text = asString(get(delta, 'text'));
    if (text !== null) acc.content += text;
    const partial = asString(get(delta, 'partial_json'));
    if (partial !== null) callAt(acc.calls, index).arguments += partial;
  } else if (!ANTHROPIC_PASSIVE.includes(kind)) {
    return false;
  }
  return true;
}

/** `summarize_sse_response`. */
function summarizeSseResponse(body) {
  const events = parseSseEvents(body)
    .map((data) => data.trim())
    .filter((data) => data && data !== '[DONE]')
    .map(parseJson)
    .filter((parsed) => parsed.ok)
    .map((parsed) => parsed.value);
  for (const value of events) {
    const response = get(value, 'response');
    if (get(value, 'type') === 'response.completed' && response !== undefined) return summarizeResponseValue(response);
  }
  const chat = { model: null, content: '', calls: new Map() };
  const anthropic = { model: null, content: '', calls: new Map() };
  const summary = emptySummary();
  for (const value of events) {
    if (applyChatChunk(chat, value) || applyAnthropicEvent(anthropic, value)) continue;
    mergeSummary(summary, summarizeResponseValue(value));
  }
  mergeSummary(summary, { model: anthropic.model, tool_calls: accumulatedCalls(anthropic.calls), content: anthropic.content });
  mergeSummary(summary, { model: chat.model, tool_calls: accumulatedCalls(chat.calls), content: chat.content });
  return summary;
}

/**
 * `summarize_proxy_exchange` with `log_bodies = true`, as a `ProxyExchangeLog`
 * in field declaration order.
 */
export function summarizeProxyExchange(method, path, requestBody, status, responseContentType, responseBody) {
  const request = parseJson(requestBody);
  const isSse = responseContentType.split(';')[0].trim().toLowerCase() === 'text/event-stream';
  let summary;
  if (isSse) {
    summary = summarizeSseResponse(responseBody);
  } else {
    const parsed = parseJson(responseBody);
    summary = parsed.ok ? summarizeResponseValue(parsed.value) : { ...emptySummary(), content: takeChars(responseBody, 160) };
  }
  const requestModel = request.ok ? asString(get(request.value, 'model')) : null;
  return {
    method,
    path,
    request_model: requestModel !== null ? requestModel : modelFromPath(path),
    request_tools: request.ok ? collectRequestToolNames(request.value) : [],
    status,
    response_model: summary.model,
    response_tool_calls: summary.tool_calls,
    response_content_preview: takeChars(summary.content, 160),
    request_body: requestBody,
    response_body: responseBody,
  };
}
