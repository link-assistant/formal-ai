// Complete agentic conversation export and learning endpoints
// (rust/src/server/conversation_reports.rs `handle_context_request`,
// `handle_learning_request`), with the readers they call:
// rust/src/conversation_context.rs (`load_conversation_context`,
// `conversation_context_to_lino`, `learn_from_conversation`),
// rust/src/dialog_log.rs (`configured_directory`, the `<id>.jsonl` trace),
// rust/src/dialog_conversation.rs (`<id>.conversation.jsonl` records,
// `append_with_overlap`, `turns_in_exchange`), rust/src/json_lino.rs
// (`json_to_lino`) and rust/src/self_improvement.rs
// (`learn_from_reported_conversation`, ported in ./self-improvement.mjs).

import fs from 'node:fs';
import path from 'node:path';

import { isFloat, parseJson as parseJsonValue, renderFloat, sortedKeys } from './json.mjs';
import { queryParam } from './memory.mjs';
import { serverMessage } from './messages.mjs';
import { SyncStore, parseBoolEnv, rustLines, sharedMemoryPath } from './memory-store.mjs';
import { errorResponse, jsonResponse, linksNotationResponse, messageError } from './response.mjs';
import { agentInfo } from './seed.mjs';
import { learnFromReportedConversation } from './self-improvement.mjs';

const DIALOG_LOG_DIRECTORY_ENV = 'FORMAL_AI_DIALOG_LOG_DIR';
const CONVERSATION_LOG_SUFFIX = '.conversation.jsonl';

/** An `io::Error` with the kind the HTTP layer maps to a status. */
class ConversationError extends Error {
  constructor(kind, message) {
    super(message);
    this.kind = kind;
  }
}

const STATUS_BY_KIND = { invalid: 400, not_found: 404 };

/** `config(key)`: an agent-info value, or the key itself. */
function config(key) {
  return agentInfo().has(key) ? agentInfo().get(key) : key;
}

/** `dialog_log::configured_directory`. */
export function configuredDialogLogDirectory(env = process.env) {
  const explicit = env[DIALOG_LOG_DIRECTORY_ENV];
  if (explicit) return explicit;
  if (parseBoolEnv(env.FORMAL_AI_SILENT) === true) return null;
  return path.join(path.dirname(sharedMemoryPath(env)), 'dialog-logs');
}

function readJsonLines(file) {
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (error) {
    if (error.code === 'ENOENT') return [];
    throw new ConversationError('other', error.message);
  }
  return rustLines(text)
    .filter((line) => line.trim())
    .map((line) => {
      try {
        return parseJsonValue(line);
      } catch (error) {
        throw new ConversationError('other', error.message);
      }
    });
}

function requireType(record, field, check) {
  if (!check(record[field])) throw new ConversationError('other', field);
  return record[field];
}

const isString = (value) => typeof value === 'string';
const isNumber = (value) => typeof value === 'number';
const isOptionalString = (value) => value === undefined || value === null || typeof value === 'string';
const isStringArray = (value) => Array.isArray(value) && value.every(isString);

/** `DialogExchangeLog` deserialized and re-serialized (`serde_json::to_value`). */
function exchangeRecord(record) {
  if (!record || typeof record !== 'object' || Array.isArray(record)) throw new ConversationError('other', 'record');
  const toolCalls = requireType(record, 'response_tool_calls', Array.isArray).map((call) => {
    if (!call || typeof call !== 'object' || !isString(call.name) || !('arguments' in call)) {
      throw new ConversationError('other', 'response_tool_calls');
    }
    return { name: call.name, arguments: call.arguments };
  });
  const out = {
    timestamp_unix_ms: requireType(record, 'timestamp_unix_ms', isNumber),
    dialog_id: requireType(record, 'dialog_id', isString),
    request_id: requireType(record, 'request_id', isString),
    method: requireType(record, 'method', isString),
    path: requireType(record, 'path', isString),
    request_model: requireType(record, 'request_model', isOptionalString) ?? null,
    request_tools: requireType(record, 'request_tools', isStringArray),
    status: requireType(record, 'status', isNumber),
    response_model: requireType(record, 'response_model', isOptionalString) ?? null,
    response_tool_calls: toolCalls,
    response_content_preview: requireType(record, 'response_content_preview', isString),
  };
  const requestBody = requireType(record, 'request_body', isOptionalString);
  const responseBody = requireType(record, 'response_body', isOptionalString);
  if (requestBody !== undefined && requestBody !== null) out.request_body = requestBody;
  if (responseBody !== undefined && responseBody !== null) out.response_body = responseBody;
  return out;
}

/** `DialogConversationLog`. */
function conversationRecord(record) {
  if (!record || typeof record !== 'object' || Array.isArray(record)) throw new ConversationError('other', 'record');
  return {
    timestamp_unix_ms: requireType(record, 'timestamp_unix_ms', isNumber),
    dialog_id: requireType(record, 'dialog_id', isString),
    request_id: requireType(record, 'request_id', isString),
    messages: requireType(record, 'messages', Array.isArray),
  };
}

/** `serde_json::Value` equality (objects compare by key set). */
function jsonEqual(left, right) {
  if (left === right) return true;
  if (isFloat(left) || isFloat(right)) return isFloat(left) && isFloat(right) && left.value === right.value;
  if (Array.isArray(left) || Array.isArray(right)) {
    return Array.isArray(left) && Array.isArray(right) && left.length === right.length
      && left.every((item, index) => jsonEqual(item, right[index]));
  }
  if (!left || !right || typeof left !== 'object' || typeof right !== 'object') return false;
  const keys = Object.keys(left);
  return keys.length === Object.keys(right).length
    && keys.every((key) => Object.prototype.hasOwnProperty.call(right, key) && jsonEqual(left[key], right[key]));
}

/** `append_with_overlap`. */
function appendWithOverlap(transcript, incoming) {
  const limit = Math.min(transcript.length, incoming.length);
  let overlap = 0;
  for (let size = limit; size >= 1; size -= 1) {
    const tail = transcript.slice(transcript.length - size);
    if (tail.every((item, index) => jsonEqual(item, incoming[index]))) {
      overlap = size;
      break;
    }
  }
  transcript.push(...incoming.slice(overlap));
}

function parseJson(text) {
  try {
    return { ok: true, value: parseJsonValue(text) };
  } catch {
    return { ok: false, value: null };
  }
}

/** `extract_request_messages`. */
function extractRequestMessages(body) {
  const { ok, value: root } = parseJson(body);
  if (!ok) return null;
  const isObject = root && typeof root === 'object' && !Array.isArray(root);
  for (const key of ['messages', 'input', 'contents']) {
    if (isObject && Array.isArray(root[key])) return root[key].slice();
  }
  if (isObject && root.input !== undefined && root.input !== null) return [{ role: 'user', content: root.input }];
  return null;
}

/** `response_messages_from_value`. */
function responseMessagesFromValue(root) {
  const isObject = root && typeof root === 'object' && !Array.isArray(root);
  if (!isObject) return [];
  if (Array.isArray(root.choices)) {
    return root.choices.filter((choice) => choice && typeof choice === 'object' && 'message' in choice).map((choice) => choice.message);
  }
  if (Array.isArray(root.output)) {
    return root.output.filter((item) => !(item && typeof item === 'object' && !Array.isArray(item) && 'type' in item)
      || typeof item.type !== 'string' || item.type === 'message');
  }
  if (Array.isArray(root.candidates)) {
    return root.candidates.filter((candidate) => candidate && typeof candidate === 'object' && 'content' in candidate)
      .map((candidate) => candidate.content);
  }
  if ('role' in root && 'content' in root) return [root];
  return [];
}

/** `extract_sse_message`. */
function extractSseMessage(body) {
  let role = null;
  let content = '';
  const toolCalls = new Map();
  for (const line of rustLines(body)) {
    const trimmed = line.trim();
    if (!trimmed.startsWith('data:')) continue;
    const data = trimmed.slice('data:'.length).trim();
    if (!data || data === '[DONE]') continue;
    const { ok, value: event } = parseJson(data);
    const delta = ok && Array.isArray(event?.choices) ? event.choices[0]?.delta : null;
    if (!delta || typeof delta !== 'object' || Array.isArray(delta)) continue;
    if (typeof delta.role === 'string') role = delta.role;
    if (typeof delta.content === 'string') content += delta.content;
    if (!Array.isArray(delta.tool_calls)) continue;
    delta.tool_calls.forEach((call, position) => {
      const index = Number.isInteger(call?.index) && call.index >= 0 ? call.index : position;
      if (!toolCalls.has(index)) toolCalls.set(index, { id: '', kind: '', name: '', arguments: '' });
      const accumulated = toolCalls.get(index);
      if (typeof call?.id === 'string') accumulated.id += call.id;
      if (typeof call?.type === 'string') accumulated.kind += call.type;
      const fn = call?.function;
      if (fn && typeof fn === 'object' && !Array.isArray(fn)) {
        if (typeof fn.name === 'string') accumulated.name += fn.name;
        if (typeof fn.arguments === 'string') accumulated.arguments += fn.arguments;
      }
    });
  }
  if (role === null && !content && toolCalls.size === 0) return null;
  const message = { content: content || null, role: role ?? 'assistant' };
  if (toolCalls.size > 0) {
    message.tool_calls = [...toolCalls.keys()].sort((a, b) => a - b).map((key) => {
      const call = toolCalls.get(key);
      return { function: { arguments: call.arguments, name: call.name }, id: call.id, type: call.kind || 'function' };
    });
  }
  return message;
}

/** `turns_in_exchange`. */
export function turnsInExchange(requestBody, responseBody) {
  const turns = (requestBody === null || requestBody === undefined ? null : extractRequestMessages(requestBody)) || [];
  if (responseBody !== null && responseBody !== undefined) {
    const parsed = parseJson(responseBody);
    const extracted = parsed.ok ? responseMessagesFromValue(parsed.value) : [extractSseMessage(responseBody)].filter(Boolean);
    appendWithOverlap(turns, extracted);
  }
  return turns;
}

/** `load_conversation_context_from`. */
function loadConversationContextFrom(directory, dialogId) {
  if (!dialogId || !/^[A-Za-z0-9_-]+$/.test(dialogId)) {
    throw new ConversationError('invalid', serverMessage('conversation_unsafe_id'));
  }
  const exchanges = readJsonLines(path.join(directory, `${dialogId}.jsonl`)).map(exchangeRecord);
  const records = readJsonLines(path.join(directory, `${dialogId}${CONVERSATION_LOG_SUFFIX}`)).map(conversationRecord);
  if (exchanges.length === 0 && records.length === 0) {
    throw new ConversationError('not_found', serverMessage('conversation_no_exchanges'));
  }
  const messages = [];
  let messagesSource;
  if (records.length === 0) {
    for (const record of exchanges) appendWithOverlap(messages, turnsInExchange(record.request_body, record.response_body));
    messagesSource = 'http-proxy-trace';
  } else {
    for (const record of records) appendWithOverlap(messages, record.messages.slice());
    messagesSource = 'conversation-record';
  }
  const first = exchanges.length > 0 ? exchanges[0] : records[0];
  const last = exchanges.length > 0 ? exchanges[exchanges.length - 1] : records[records.length - 1];
  return sortedKeys({
    metadata: {
      dialog_id: dialogId,
      source: 'formal-ai-server-dialog-log',
      format: 'complete-agentic-conversation',
      messages_source: messagesSource,
      message_count: messages.length,
      exchange_count: exchanges.length,
      first_timestamp_unix_ms: first ? first.timestamp_unix_ms : 0,
      last_timestamp_unix_ms: last ? last.timestamp_unix_ms : 0,
    },
    messages,
    server_logs: exchanges,
  });
}

/** `load_conversation_context`. */
export function loadConversationContext(dialogId, env = process.env) {
  const directory = configuredDialogLogDirectory(env);
  if (!directory) {
    throw new ConversationError('not_found', config('context_dialog_log_unavailable').split('{variable}').join(DIALOG_LOG_DIRECTORY_ENV));
  }
  return loadConversationContextFrom(directory, dialogId);
}

// ---- json_lino::json_to_lino (the native document form) -------------------

const isObjectValue = (value) => value !== null && typeof value === 'object' && !Array.isArray(value) && !isFloat(value);
const isScalar = (value) => !Array.isArray(value) && !isObjectValue(value);

function nativeBareReference(value) {
  return value.length > 0 && /^[A-Za-z0-9_.\-/]+$/.test(value);
}

function isScalarLiteral(value) {
  return ['null', 'true', 'false'].includes(value) || /^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?$/.test(value);
}

function numberToken(value) {
  return Number.isInteger(value) && Math.abs(value) < 2 ** 63 ? BigInt(value).toString() : String(value);
}

function nativeStringToken(value) {
  const flat = value.replace(/\r/g, '\\r').replace(/\n/g, '\\n').replace(/\t/g, '\\t');
  if (nativeBareReference(flat) && !isScalarLiteral(flat)) return flat;
  if (!flat.includes('"')) return `"${flat}"`;
  if (!flat.includes("'")) return `'${flat}'`;
  return `"b64:${Buffer.from(value, 'utf8').toString('base64')}"`;
}

function nativeScalarToken(value) {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'boolean') return String(value);
  if (isFloat(value)) return renderFloat(value.value);
  if (typeof value === 'number') return numberToken(value);
  if (typeof value === 'bigint') return value.toString();
  if (typeof value === 'string') return nativeStringToken(value);
  return '';
}

function writeLine(out, indent, name, value) {
  out.push(`${' '.repeat(indent)}${name}${value === null ? '' : ` ${value}`}`);
}

function inlineScalars(items) {
  return `(${items.map(nativeScalarToken).join(' ')})`;
}

function singularItemName(name) {
  if (name === 'messages') return 'message';
  if (name === 'parts') return 'part';
  if (name.endsWith('ies') && Buffer.byteLength(name) > 3) return `${name.slice(0, -3)}y`;
  if (name.endsWith('s') && Buffer.byteLength(name) > 1) return name.slice(0, -1);
  return `${name}_item`;
}

function writeNativeObject(out, indent, object) {
  for (const key of Object.keys(object)) {
    if (nativeBareReference(key)) {
      writeNativeNamedValue(out, indent, key, object[key]);
    } else {
      writeLine(out, indent, 'field', null);
      writeLine(out, indent + 2, 'name', nativeStringToken(key));
      writeNativeNamedValue(out, indent + 2, 'value', object[key]);
    }
  }
}

function writeNativeNamedValue(out, indent, name, value) {
  if (isObjectValue(value)) {
    writeLine(out, indent, name, null);
    writeNativeObject(out, indent + 2, value);
  } else if (Array.isArray(value) && value.every(isScalar)) {
    writeLine(out, indent, name, inlineScalars(value));
  } else if (Array.isArray(value)) {
    writeNativeSequence(out, indent, name, value);
  } else {
    writeLine(out, indent, name, nativeScalarToken(value));
  }
}

function writeNativeSequence(out, indent, name, items) {
  writeLine(out, indent, name, null);
  const itemName = singularItemName(name);
  for (const item of items) {
    if (isObjectValue(item)) {
      writeLine(out, indent + 2, itemName, null);
      writeNativeObject(out, indent + 4, item);
    } else if (Array.isArray(item) && item.every(isScalar)) {
      writeLine(out, indent + 2, itemName, inlineScalars(item));
    } else if (Array.isArray(item)) {
      writeNativeSequence(out, indent + 2, itemName, item);
    } else {
      writeLine(out, indent + 2, itemName, nativeScalarToken(item));
    }
  }
}

/** `json_to_lino`. */
export function jsonToLino(value) {
  const out = [];
  if (isObjectValue(value)) writeNativeObject(out, 0, value);
  else if (Array.isArray(value)) writeNativeSequence(out, 0, 'items', value);
  else writeLine(out, 0, 'value', nativeScalarToken(value));
  return out.map((line) => `${line}\n`).join('');
}

/** `conversation_context_to_lino`. */
export function conversationContextToLino(dialogId, context) {
  let output = `conversation ${dialogId}\n`;
  for (const line of rustLines(jsonToLino(context))) output += `  ${line}\n`;
  return output;
}

/** `learn_from_conversation`. */
export function learnFromConversation(dialogId, env = process.env) {
  const context = loadConversationContext(dialogId, env);
  const document = conversationContextToLino(dialogId, context);
  const staged = learnFromReportedConversation(context);
  const store = SyncStore.open(env);
  let eventsRecorded;
  try {
    eventsRecorded = store.recordChatExchangeWithTools(`agentic_report_${dialogId}`, document, []);
  } catch (error) {
    throw new ConversationError('other', error.message);
  }
  return sortedKeys({
    dialog_id: dialogId,
    learned: true,
    events_recorded: eventsRecorded,
    learning_trace_found: staged !== null,
    rule_proposals: staged ? staged.learning.proposals.length : 0,
    awaiting_human_review: staged ? staged.awaiting_human_review : false,
    promoted: false,
  });
}

function errorFor(error, otherResponse) {
  if (!(error instanceof ConversationError)) throw error;
  const status = STATUS_BY_KIND[error.kind];
  return status ? errorResponse(status, error.message) : otherResponse(error);
}

/** `GET /v1/conversations/{dialog_id}`. */
export function handleConversationContext(ctx, request) {
  const env = ctx?.env || process.env;
  const dialogId = request.params?.dialog_id ?? '';
  let context;
  try {
    context = loadConversationContext(dialogId, env);
  } catch (error) {
    return errorFor(error, (failure) => errorResponse(500, failure.message));
  }
  const include = queryParam(request.query, 'include');
  if (include === 'harness') delete context.server_logs;
  else if (include === 'server') delete context.messages;
  else if (include !== null && include !== 'both') return messageError(400, 'conversation_include_invalid');
  if (queryParam(request.query, 'format') === 'json') return jsonResponse(200, context);
  return linksNotationResponse(200, conversationContextToLino(dialogId, context));
}

/** `POST /v1/conversations/{dialog_id}/learn`. */
export function handleConversationLearn(ctx, request) {
  const env = ctx?.env || process.env;
  const dialogId = request.params?.dialog_id ?? '';
  try {
    return jsonResponse(200, learnFromConversation(dialogId, env));
  } catch (error) {
    return errorFor(error, (failure) =>
      errorResponse(500, config('context_learning_failed').split('{error}').join(failure.message)));
  }
}
