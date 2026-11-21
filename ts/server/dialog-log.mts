import { nonNullObject as isObject } from '../logging/object-values.js';
// Per-dialog HTTP exchange logs (rust/src/dialog_log.rs
// `record_api_exchange_if_enabled`, `write_dialog_exchange`, `dialog_id`,
// `conversation_epoch`, `explicit_dialog_id`, `first_user_prompt`, and
// rust/src/dialog_conversation.rs `write_conversation_record`).
//
// Every authorized exchange outside `/conversations/` appends one typed Lino trace
// record to `<dir>/<dialog_id>.lino` and, when it carried turns, one
// conversation record to `<dir>/<dialog_id>.conversation.lino`. Logging is
// best-effort: failures go to stderr and never change the response.
//
// `DialogScope` (the thread-local the Rust server sets around one request) is
// an `AsyncLocalStorage` scope here: `withDialogScope` enters it, and the
// agentic planner reads it back through `currentDialogId` when it writes a
// report command (rust/src/agentic_coding/report_issue.rs).

import { AsyncLocalStorage } from 'node:async_hooks';
import fs from 'node:fs';
import path from 'node:path';
import { encodeDialogRecord } from './dialog-codec.mjs';
import { makeRedactor, redactedBody } from './structured-log.mjs';

import { configuredDialogLogDirectory, turnsInExchange } from './conversations.mjs';
import { summarizeProxyExchange } from './dialog-log-summary.mjs';
import { stableId } from './ids.mjs';
import { sortedKeys, toCompactJson } from './json.mjs';
import { serverMessage } from './messages.mjs';

export const DIALOG_ID_HEADER = 'x-formal-ai-dialog-id';
const CONVERSATION_LOG_SUFFIX = '.conversation.lino';
const UNIT_SEPARATOR = '\u001f';

let requestSequence = 1n;
let arrivals = 0n;
const epochs = new Map();



/** `explicit_dialog_id`: the caller's session id, when it is a safe path component. */
export function explicitDialogId(headers) {
  return sessionHeader(headers, DIALOG_ID_HEADER);
}

function sessionHeader(headers, expected) {
  const value = headerValue(headers, expected);
  return /^[A-Za-z0-9_-]{1,128}$/u.test(value ?? '') ? value : null;
}

function headerValue(headers, expected) {
  const found = (headers || []).find(([name, value]) =>
    name.toLowerCase() === expected && String(value).trim().length > 0);
  return found ? String(found[1]).trim() : null;
}

function requestSessionIdentifier(headers, requestPath, requestBody) {
  const identifier = explicitDialogId(headers) ?? sessionHeader(headers, 'x-opencode-session-id')
    ?? sessionHeader(headers, 'x-opencode-session');
  if (identifier !== null) return identifier;
  const endpoint = requestPath?.split('?')[0];
  let session = null;
  if (['/v1/messages', '/api/anthropic/v1/messages'].includes(endpoint)) {
    try { session = JSON.parse(JSON.parse(requestBody).metadata?.user_id)?.session_id; } catch {}
  } else if (['/v1/responses', '/api/openai/v1/responses'].includes(endpoint)) {
    try { session = JSON.parse(headerValue(headers, 'x-codex-turn-metadata'))?.thread_id; } catch {}
    session ??= sessionHeader(headers, 'session_id');
  }
  return typeof session === 'string' && /^[A-Za-z0-9_-]{36}$/u.test(session) ? session : null;
}

const dialogScope = new AsyncLocalStorage();

/**
 * Mirrors rust/src/dialog_log.rs `DialogScope::begin` (and its `Drop`): run
 * `body` with the caller's declared session id in scope; the scope ends, and
 * the enclosing one is current again, when `body` settles.
 * @template T
 * @param {Array<[string, string]>} headers
 * @param {() => T} body
 * @param {{path: string, body: string}} [request]
 * @returns {T}
 */
export function withDialogScope(headers, body, request) {
  return dialogScope.run({ dialogId: requestSessionIdentifier(headers, request?.path, request?.body) }, body);
}

/**
 * Mirrors rust/src/dialog_log.rs `current_dialog_id`: the session id the
 * request being served declared, or null.
 * @returns {string|null}
 */
export function currentDialogId() {
  return dialogScope.getStore()?.dialogId ?? null;
}

/** `first_user_prompt`: the first user message's text, for the fallback dialog id. */
function firstUserPrompt(body) {
  const text = (value) => {
    if (typeof value === 'string') return value.trim() ? value.trim() : null;
    if (Array.isArray(value)) {
      for (const item of value) {
        const found = text(item);
        if (found !== null) return found;
      }
      return null;
    }
    if (isObject(value)) {
      for (const key of ['text', 'content', 'input']) {
        if (!(key in value)) continue;
        const found = text(value[key]);
        if (found !== null) return found;
      }
    }
    return null;
  };
  const userContent = (value) => {
    if (Array.isArray(value)) {
      for (const item of value) {
        const found = userContent(item);
        if (found !== null) return found;
      }
      return null;
    }
    if (!isObject(value)) return null;
    if (typeof value.role === 'string' && value.role.toLowerCase() === 'user') {
      const content = 'content' in value ? value.content : value.parts;
      return content === undefined ? null : text(content);
    }
    for (const key of Object.keys(value)) {
      const found = userContent(value[key]);
      if (found !== null) return found;
    }
    return null;
  };
  let value;
  try {
    value = sortedKeys(JSON.parse(body));
  } catch {
    return null;
  }
  const found = userContent(value);
  if (found !== null) return found;
  return isObject(value) && 'input' in value ? text(value.input) : null;
}

/** `conversation_epoch`: a marker fixed for one conversation, distinct across two. */
function conversationEpoch(scope) {
  const marker = (BigInt(Date.now()) << 20n) | (arrivals & 0xfffffn);
  arrivals += 1n;
  if (!epochs.has(scope)) epochs.set(scope, marker);
  return epochs.get(scope);
}

/** `dialog_id`. */
function dialogId(directory, headers, requestBody, requestPath) {
  const declared = requestSessionIdentifier(headers, requestPath, requestBody);
  if (declared !== null) return declared;
  const basis = firstUserPrompt(requestBody) ?? requestPath;
  const scope = `${directory}${UNIT_SEPARATOR}${basis}`;
  return stableId('dialog', `${conversationEpoch(scope)}${UNIT_SEPARATOR}${scope}`);
}

/** `write_conversation_record`: skipped when the exchange carried no turns. */
function writeConversationRecord(directory, record) {
  if (record.messages.length === 0) return null;
  fs.mkdirSync(directory, { recursive: true });
  const file = path.join(directory, `${record.dialog_id}${CONVERSATION_LOG_SUFFIX}`);
  fs.appendFileSync(file, `${encodeDialogRecord(makeRedactor(process.env,{preserveJsonNumbers:true})(record))}\n`, { mode: 0o600 });
  return file;
}

/**
 * `write_dialog_exchange`: append one exchange; returns the trace path.
 * @returns {string}
 */
export function writeDialogExchange(directory, method, requestPath, headers, requestBody, status, contentType, responseBody) {
  const timestamp = Date.now();
  const id = dialogId(directory, headers, requestBody, requestPath);
  const sequence = requestSequence;
  requestSequence += 1n;
  const requestId = stableId('request', `${timestamp}|${sequence}|${id}|${requestPath}`);
  const exchange = summarizeProxyExchange(method, requestPath, requestBody, status, contentType, responseBody);
  writeConversationRecord(directory, {
    timestamp_unix_ms: timestamp,
    dialog_id: id,
    request_id: requestId,
    messages: sortedKeys(turnsInExchange(requestBody, responseBody)),
  });
  const record = {
    timestamp_unix_ms: timestamp,
    dialog_id: id,
    request_id: requestId,
    ...exchange,
    response_tool_calls: exchange.response_tool_calls.map((call) => ({
      name: call.name,
      arguments: sortedKeys(call.arguments),
    })),
  };
  fs.mkdirSync(directory, { recursive: true });
  const file = path.join(directory, `${id}.lino`);
  fs.appendFileSync(file, `${encodeDialogRecord(makeRedactor(process.env,{preserveJsonNumbers:true})(record))}\n`, { mode: 0o600 });
  return file;
}

/**
 * `record_api_exchange_if_enabled`: log an authorized exchange; never throws.
 * @param {{method: string, path: string, headers: Array<[string, string]>, body: string}} request
 * @param {{statusCode: number, contentType: string, body: string}} response
 * @param {boolean} authorized
 * @param {object} [env]
 */
export function recordApiExchangeIfEnabled(request, response, authorized, env = process.env, logger = null) {
  try {
    if (!authorized || logger?.level === 'silent') return;
    const directory = configuredDialogLogDirectory(env);
    if (!directory || String(request.path).split('?')[0].includes('/conversations/')) return;
    {
      const redact=makeRedactor(env,{preserveJsonNumbers:true});
      const safe=redact({...request,headers:Object.fromEntries(request.headers??[]),body:redactedBody(String(request.body??''),env)});
      request={...safe,headers:Object.entries(safe.headers)};
      response=redact({...response,body:redactedBody(String(response.body??''),env)});
    }
    const file = writeDialogExchange(
      directory,
      request.method,
      String(request.path),
      request.headers,
      String(request.body ?? ''),
      response.statusCode,
      response.contentType,
      String(response.body ?? ''),
    );
    if(logger)logger.log('debug','dialog.appended',{component:'server',path:file});
    else process.stderr.write(`${serverMessage('dialog_log_appended', { path: file })}\n`);
  } catch (error) {
    if(logger)logger.log('error','dialog.failed',{component:'server',error});
    else process.stderr.write(`${serverMessage('dialog_log_failed', { error: error?.message || error })}\n`);
  }
}
