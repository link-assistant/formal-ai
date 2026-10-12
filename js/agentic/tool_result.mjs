// Friendly, lossless presentation of client-owned tool results (issue #750):
// a port of rust/src/agentic_coding/tool_result.rs.

import { fill as fillWorkItemStep } from './work_item_steps.mjs';
import { plainText, rustLines } from './content.mjs';
import { sourceFromAgentReadResult } from './code_artifact.mjs';
import { agenticMessage } from './messages.mjs';
import { classifyTool, isWorkspaceCreationTool } from './capability_router.mjs';
import { Capability } from './capability.mjs';
import { findToolDefinition } from './protocol_policy.mjs';
import { requestFor as localSearchRequestFor } from './local_search.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { detect, fallbackLanguage } from './crate/language.mjs';
import { cached, childValue, childrenNamed, readText } from './host.mjs';
import { parseRoot } from './crate/seed_parser.mjs';
import { textOutsideQuotedSegments } from './crate/coding_program_contract.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { appendInvitation } from './crate/failure_reporting.mjs';
import { lexicon, mentionsRole } from './crate/seed_meanings.mjs';
import {
  asI64, asStr, compactJson, isObject, isWhitespace, jsonGet, parseI64, parseJson, parseUsize, prettyJson,
  replaceAllLiteral, splitInclusiveNewline, splitOnce, splitWhitespace, toAsciiLowercase, trim, trimEndMatches,
  trimStart, utf8Len,
} from './crate/rust_str.mjs';

const SHELL_INTENTS_FILE = 'data/seed/shell-intents.lino';

const ROLE_TOOL_RESULT_DETAIL_REQUEST = 'tool_result_detail_request';
const ROLE_TOOL_RESULT_FAILURE_SIGNAL = 'tool_result_failure_signal';
const ROLE_TOOL_RESULT_FIRST_REFERENCE = 'tool_result_first_reference';
const ROLE_TOOL_RESULT_LINE_REQUEST = 'tool_result_line_request';
const ROLE_TOOL_RESULT_SECOND_REFERENCE = 'tool_result_second_reference';
const ROLE_TOOL_RESULT_URL_REQUEST = 'tool_result_url_request';

/** `StepOutcome` variants. */
export const StepOutcome = Object.freeze({ Succeeded: 'succeeded', Failed: 'failed', Unreported: 'unreported' });

const isUser = (message) => message.role.toLowerCase() === 'user';
const isTool = (message) => message.role.toLowerCase() === 'tool';

/** Mirrors `fn step_outcome`. */
export function stepOutcome(raw) {
  const result = normalize(raw);
  if (result.error !== null) return StepOutcome.Failed;
  if (result.exit_code === 0) return StepOutcome.Succeeded;
  if (result.exit_code !== null) return StepOutcome.Failed;
  return looksLikeError(result.payload) ? StepOutcome.Failed : StepOutcome.Unreported;
}

/** Mirrors `fn observed_bytes_match`: an explicit successful exact byte receipt. */
export function observedBytesMatch(raw, expected) {
  const result = normalize(raw);
  if (result.exit_code !== 0 || result.error !== null || incompleteReceipt(raw)) return false;
  const exact = /^(?:Command: [^\n]*\n)?Output: ([\s\S]*)\nExit Code: 0$/u.exec(untrustedInner(raw));
  if (exact !== null) return exact[1] === expected;
  const value = parseJson(trim(raw));
  if (!isObject(value)) return false;
  if (['is_error', 'isError'].some((name) => value[name] === true)
    || ['ok', 'success'].some((name) => value[name] === false)
    || ['error', 'stderr', 'failure'].some((name) => nonemptyText(value[name]) !== null)) return false;
  const key = ['output', 'stdout', 'content', 'result'].find((name) => has(value, name));
  return key !== undefined && typeof value[key] === 'string' && value[key] === expected;
}

/** Mirrors `fn observed_digest_matches`. */
export function observedDigestMatches(raw, expected) {
  return !incompleteReceipt(raw) && stepOutcome(raw) !== StepOutcome.Failed
    && splitWhitespace(observedPayload(raw) ?? '')[0] === expected;
}

/** Mirrors `fn failed_verification`. */
export function failedVerification(runOutputs, verificationCommand, prompt) {
  const output = runOutputs[runOutputs.length - 1];
  if (output === undefined || stepOutcome(output) !== StepOutcome.Failed) return null;
  return render(verificationCommand, output, prompt);
}

/** Mirrors `fn reported_exit_code`. */
export function reportedExitCode(raw) {
  return normalize(raw).exit_code;
}

/** Mirrors `fn harness_reported_failure`. */
export function harnessReportedFailure(raw) {
  return normalize(raw).error !== null;
}

/** Mirrors `fn shell_step`: `{text}` or null. */
export function shellStep(raw) {
  const envelope = parseShellEnvelope(trim(raw));
  if (!envelope) return null;
  const result = envelopeResult(envelope);
  return { text: result.error ?? result.payload };
}

/** Mirrors `fn normalized_payload`. */
export function normalizedPayload(raw) {
  const result = normalize(raw);
  const succeeded = result.exit_code === 0;
  return result.error === null && (succeeded || !looksLikeError(result.payload)) ? result.payload : null;
}

/** Mirrors `fn observed_payload`. */
export function observedPayload(raw) {
  const envelope = parseShellEnvelope(raw);
  if (envelope?.exit_code === 0) {
    // The explicit stdout-only receipt keeps authored whitespace and JSON bytes.
    const exact = /^Output: ([\s\S]*)\nExit Code: 0$/u.exec(untrustedInner(raw));
    return exact === null ? envelope.output : exact[1];
  }
  const result = normalize(raw);
  if (result.error !== null) return null;
  if (result.exit_code === 0) {
    const value = parseJson(trim(raw));
    if (isObject(value)) {
      const key = ['output', 'stdout', 'content', 'result'].find((name) => has(value, name));
      if (key !== undefined && typeof value[key] === 'string') return value[key];
    }
  }
  return result.payload;
}

/** Mirrors `fn failure_message`. */
export function failureMessage(raw, explicitlyFailed, inferFromProse) {
  const result = normalize(raw);
  if (result.error !== null) return result.error;
  if (explicitlyFailed || (inferFromProse && result.exit_code !== 0 && looksLikeError(result.payload))) {
    return trim(result.payload) ? result.payload : trim(raw);
  }
  return null;
}


/** Mirrors fn incomplete_receipt: declared partial observations cannot certify bytes. */
export function incompleteReceipt(raw) {
  const value = parseJson(trim(raw));
  return isObject(value) && (value.complete === false || value.stream_complete === false
    || value.truncated === true || value.timed_out === true || value.aborted === true
    || (typeof value.signal === 'string' && value.signal.length > 0));
}

/** Mirrors SourceReadStatus: source observation status is distinct from process success. */
export const SourceReadStatus = Object.freeze({ Success: 'reported-success', Failure: 'reported-failure', Unknown: 'unknown' });

/** Mirrors fn source_read_observation: caller must bind the current Read and exact path. */
/** Project only a provider-owned complete value; arbitrary callback strings remain opaque. */
export function completeOwnedSourceReadFrame(source) {
  const rows = source.split('\n');
  const body = rows.map((line, index) => String(index + 1).padStart(5, '0') + '| ' + line).join('\n');
  // Fill source last so authored placeholder-looking bytes are never substituted.
  return fillWorkItemStep('agent-file-read-frame', [['{lines}', String(rows.length)], ['{body}', body]]);
}

export function sourceReadObservation(raw, explicitlyFailed, metadata = null, expectedPath = null) {
  // The file body never establishes its own provider status or metadata.
  const bound = isObject(metadata) && typeof expectedPath === 'string' && metadata.path === expectedPath;
  const receipt = bound ? JSON.stringify(metadata) : null;
  const exit = receipt === null ? null : reportedExitCode(receipt);
  const error = explicitlyFailed ? raw : receipt === null ? null
    : failureMessage(receipt, false, false) ?? (exit !== null && exit !== 0 ? receipt : null);
  const status = error !== null ? SourceReadStatus.Failure
    : bound && metadata.success === true ? SourceReadStatus.Success : SourceReadStatus.Unknown;
  const framed = metadata === null ? sourceFromAgentReadResult(raw) : null;
  const boundary = raw.lastIndexOf('\n\n(');
  const numbered = boundary < 0 ? [] : raw.slice('<file>\n'.length, boundary).split('\n');
  const count = boundary < 0 ? NaN : Number(raw.slice(boundary + 3).split(/\s+/u)[5]);
  const wholeFrame = framed !== null && count === numbered.length
    && numbered.every((line, index) => Number(line.slice(0, line.indexOf('| '))) === index + 1);
  const complete = error === null && (wholeFrame || (bound && status === SourceReadStatus.Success
    && metadata.format === 'raw' && metadata.complete === true && !incompleteReceipt(receipt)));
  const absent = bound && status === SourceReadStatus.Failure && metadata.success === false
    && metadata.complete === false && metadata.format === 'raw' && metadata.error_code === 'ENOENT'
    && ['truncated', 'timed_out', 'timedOut', 'interrupted', 'canceled', 'cancelled', 'aborted']
      .every(key => metadata[key] === undefined || metadata[key] === false)
    && (metadata.signal === undefined || metadata.signal === null || metadata.signal === '')
    && (metadata.stream_complete === undefined || metadata.stream_complete === true);
  return { status, complete, absent, source: error === null ? framed ?? raw : null, error };
}

const PROSE_FAILURE_PREFIX_CHARS = 512;

/** Mirrors `fn looks_like_error`. */
function looksLikeError(text) {
  const prefix = Array.from(ownWords(text)).slice(0, PROSE_FAILURE_PREFIX_CHARS).join('');
  return mentionsRole(ROLE_TOOL_RESULT_FAILURE_SIGNAL, normalizePrompt(prefix));
}

const CITED_LINES_THAT_MAKE_A_QUOTATION = 2;

/** Mirrors `fn own_words`. */
function ownWords(text) {
  let consumed = 0;
  let firstCitation = null;
  let cited = 0;
  for (const line of splitInclusiveNewline(text)) {
    const offset = citationOffset(line);
    if (offset !== null) {
      cited += 1;
      if (firstCitation === null) firstCitation = consumed + offset;
    }
    consumed += line.length;
  }
  return firstCitation !== null && cited >= CITED_LINES_THAT_MAKE_A_QUOTATION ? text.slice(0, firstCitation) : text;
}

/** Mirrors `fn citation_offset`. */
function citationOffset(line) {
  for (let colon = line.indexOf(':'); colon >= 0; colon = line.indexOf(':', colon + 1)) {
    const before = line.slice(0, colon);
    const stem = trimEndMatches(before, (character) => /^[0-9]$/.test(character));
    const last = Array.from(stem).pop();
    const cites = stem.length < before.length
      && (last === undefined || last === ':' || last === '/' || last === '.' || isWhitespace(last));
    if (cites) return wordStart(before);
  }
  return null;
}

/** Mirrors `fn word_start`. */
function wordStart(text) {
  const chars = Array.from(text);
  let offset = text.length;
  for (let index = chars.length - 1; index >= 0; index -= 1) {
    offset -= chars[index].length;
    if (isWhitespace(chars[index])) return offset + chars[index].length;
  }
  return 0;
}

/** Mirrors `fn render`. */
export function render(label, raw, prompt) {
  const result = normalize(raw);
  const language = responseLanguage(prompt);
  if (result.error !== null) return failureReport(label, result.error, result.exit_code, language);
  if (result.exit_code !== 0 && looksLikeError(result.payload)) {
    return failureReport(label, result.payload, result.exit_code, language);
  }
  if (!trim(result.payload)) {
    let intent = 'tool_result_empty_generic';
    if (localSearchRequestFor(prompt) !== null) intent = 'tool_result_empty_local_path_search';
    else if (isListing(label)) intent = 'tool_result_empty_list';
    else if (isSearch(label)) intent = 'tool_result_empty_search';
    return fill(intent, language, [['{tool}', label]]);
  }
  return fill('tool_result_completed', language, [
    ['{tool}', label],
    ['{format}', result.format],
    ['{payload}', result.payload],
  ]);
}

/** Mirrors `fn render_failure`. */
export function renderFailure(label, detail, prompt) {
  const result = normalize(detail);
  return failureReport(label, result.error ?? result.payload, result.exit_code, responseLanguage(prompt));
}

/** Mirrors `fn failure_report`. */
function failureReport(label, error, exitCode, language) {
  const failure = exitCode === null
    ? fill('tool_result_failed', language, [['{tool}', label], ['{error}', trim(error)]])
    : fill('tool_result_failed_exit_code', language, [
      ['{tool}', label],
      ['{exit_code}', String(exitCode)],
      ['{error}', trim(error)],
    ]);
  return appendInvitation(failure, language);
}

function lastIndexWhere(items, predicate, from = 0) {
  for (let index = items.length - 1; index >= from; index -= 1) if (predicate(items[index])) return index;
  return -1;
}

/** Mirrors `fn latest_turn_answer`. */
export function latestTurnAnswer(messages, toolNames, prompt) {
  const start = lastIndexWhere(messages, isUser);
  if (start < 0) return null;
  const index = lastIndexWhere(messages, isTool, start + 1);
  if (index < 0) return null;
  if (isWriteRunRecipe(messages, toolNames)) return null;
  return render(resultLabel(messages, index), plainText(messages[index].content), prompt);
}

/**
 * Mirrors `fn latest_turn_quiet_success`: whether the latest tool result of
 * this turn succeeded with no output, as an edit tool's empty reply does
 * (PR #1188 T181).
 */
export function latestTurnQuietSuccess(messages) {
  const start = lastIndexWhere(messages, isUser);
  if (start < 0) return false;
  const index = lastIndexWhere(messages, isTool, start + 1);
  if (index < 0) return false;
  const result = normalize(plainText(messages[index].content));
  return result.error === null && (result.exit_code === null || result.exit_code === 0) && !trim(result.payload);
}

/** Mirrors `fn has_latest_turn_result`. */
export function hasLatestTurnResult(messages) {
  const start = lastIndexWhere(messages, isUser);
  if (start < 0) return false;
  return messages.slice(start + 1).some(isTool);
}

/** Mirrors `fn is_write_run_recipe`. */
function isWriteRunRecipe(messages, toolNames) {
  return toolNames.some(isWorkspaceCreationTool)
    && toolNames.some((name) => classifyTool(name) === Capability.Run)
    && messages.some((message) => (message.tool_calls || []).some((call) => isWorkspaceCreationTool(call.function.name)));
}

/** Mirrors `fn follow_up_answer`. */
export function followUpAnswer(messages, prompt) {
  const normalizedPrompt = normalizePrompt(prompt);
  const wantsUrl = mentionsRole(ROLE_TOOL_RESULT_URL_REQUEST, normalizedPrompt);
  const wantsLine = mentionsRole(ROLE_TOOL_RESULT_LINE_REQUEST, normalizedPrompt);
  const wantsDetail = mentionsRole(ROLE_TOOL_RESULT_DETAIL_REQUEST, normalizedPrompt);
  if (!wantsUrl && !wantsLine && !wantsDetail) return null;
  const latestUser = lastIndexWhere(messages, isUser);
  if (latestUser < 0) return null;
  const toolIndex = lastIndexWhere(messages.slice(0, latestUser), isTool);
  if (toolIndex < 0) return null;
  const result = normalize(plainText(messages[toolIndex].content));
  if (wantsUrl) return extractUrls(result.payload)[requestedIndex(normalizedPrompt)] ?? null;
  if (wantsLine) return rustLines(result.payload)[requestedIndex(normalizedPrompt)] ?? null;
  return result.payload;
}

/** Mirrors `fn requested_index`. */
function requestedIndex(prompt) {
  if (mentionsRole(ROLE_TOOL_RESULT_SECOND_REFERENCE, prompt)) return 1;
  if (mentionsRole(ROLE_TOOL_RESULT_FIRST_REFERENCE, prompt)) return 0;
  let found = null;
  for (const digits of prompt.split(/[^0-9]/)) {
    const parsed = parseUsize(digits);
    if (parsed !== null && Number.isSafeInteger(parsed)) {
      found = parsed;
      break;
    }
  }
  return Math.max((found ?? 1) - 1, 0);
}

function result(payload, error, format, exitCode) {
  return { payload, error, format, exit_code: exitCode };
}

const has = (object, key) => Object.prototype.hasOwnProperty.call(object, key);

/** Mirrors `fn normalize`: `{payload, error, format, exit_code}`. */
function normalize(raw) {
  const trimmed = trim(raw);
  const value = parseJson(trimmed);
  if (value === undefined) {
    const envelope = parseShellEnvelope(trimmed);
    if (envelope) return envelopeResult(envelope);
    return fromPayload(stripTransportEnvelope(trimmed), null);
  }
  if (!isObject(value)) return fromPayload(prettyJson(value), null);
  const object = value;
  const present = (keys) => keys.filter((key) => has(object, key)).map((key) => object[key]);
  const nonzeroExit = present(['exit_code', 'exitCode']).some(nonzeroStatus);
  const statusCode = object.status_code;
  const failedHttp = has(object, 'status_code') && Number.isInteger(statusCode) && statusCode >= 400;
  const statusValues = present(['status', 'state', 'outcome']);
  const status = statusValues.length ? statusValues[0] : undefined;
  const expectedStop = status !== undefined && expectedStopStatus(status);
  const failed = status !== undefined && failedStatus(status);
  const explicitlyUnsuccessful = present(['ok', 'success']).some((entry) => entry === false);
  const explicitlyFailed = present(['is_error', 'isError']).some((entry) => entry === true);
  const stopped = object.timed_out === true || object.aborted === true
    || (typeof object.signal === 'string' && object.signal.length > 0);
  let explicitError = null;
  for (const entry of present(['error', 'stderr', 'failure'])) {
    explicitError = nonemptyText(entry);
    if (explicitError !== null) break;
  }
  let reportedExit = null;
  for (const entry of present(['exit_code', 'exitCode'])) {
    reportedExit = asI64(entry) ?? (typeof entry === 'string' ? parseI64(entry) : null);
    if (reportedExit !== null) break;
  }
  if (stopped || (!expectedStop && (nonzeroExit || failedHttp || failed || explicitlyUnsuccessful || explicitlyFailed
    || explicitError !== null))) {
    let error = explicitError;
    for (const key of ['output', 'content', 'result']) {
      if (error !== null) break;
      if (has(object, key)) error = nonemptyText(object[key]);
    }
    if (error === null) {
      const key = ['exit_code', 'exitCode', 'status_code', 'status', 'state', 'outcome', 'timed_out', 'aborted', 'signal'].find((name) => has(object, name));
      error = key === undefined ? '' : `${key}=${compactJson(object[key])}`;
    }
    return { ...fromPayload('', error), exit_code: reportedExit !== null && reportedExit !== 0 ? reportedExit : null };
  }
  const text = mcpTextContent(value);
  if (text !== null) return { ...fromPayload(text, null), exit_code: reportedExit };
  const payloadKey = ['output', 'stdout', 'content', 'result'].find((key) => has(object, key));
  if (payloadKey !== undefined) {
    const payload = object[payloadKey];
    const textPayload = typeof payload === 'string' ? stripTransportEnvelope(payload) : prettyJson(payload);
    return { ...fromPayload(textPayload, null), exit_code: reportedExit };
  }
  return { ...fromPayload(prettyJson(value), null), exit_code: reportedExit };
}

/** Mirrors `ShellEnvelope::into_result`. */
function envelopeResult(envelope) {
  if (envelope.exit_code === 0) {
    return { ...fromPayload(envelope.output || envelope.error, null), exit_code: 0 };
  }
  return result('', envelope.error || envelope.output, 'text', envelope.exit_code);
}

function untrustedInner(text) {
  const open = splitOnce(text, '<untrusted_context>');
  const close = open && splitOnce(open[1], '</untrusted_context>');
  return close ? close[0] : text;
}

/** Mirrors `fn parse_shell_envelope`: `{output, error, exit_code}` or null. */
function parseShellEnvelope(text) {
  const lines = rustLines(untrustedInner(text));
  let exitCode = null;
  // A call the harness killed (a timeout) names its signal and no exit code
  // (PR #1188 G77): the kill is the error, and no exit code is invented.
  const killed = [];
  let end = lines.length;
  while (end > 0) {
    const split = splitOnce(trim(lines[end - 1]), ':');
    if (!split) break;
    const field = trim(split[0]);
    if (field === 'Exit Code') {
      const code = parseI64(trim(split[1]));
      if (code === null) return null;
      exitCode = code;
    } else if (field === 'Signal' || field === 'Timeout') {
      killed.unshift(trim(lines[end - 1]));
    } else if (field !== agenticMessage('tool_result_process_group_field')) {
      break;
    }
    end -= 1;
  }
  if (exitCode === null && killed.length === 0) return null;
  const head = lines.slice(0, end);
  const outputAt = head.findIndex((line) => isField(line, 'Output'));
  if (outputAt < 0) return null;
  const errorAt = lastIndexWhere(head, (line) => isField(line, 'Error'), outputAt + 1);
  const output = envelopeSection(lines.slice(outputAt, errorAt < 0 ? end : errorAt), 'Output:');
  const stated = errorAt < 0 ? '' : envelopeSection(lines.slice(errorAt, end), 'Error:');
  const error = exitCode === null ? [stated, ...killed].filter((part) => part !== '').join('\n') : stated;
  return { output, error, exit_code: exitCode };
}

/** Mirrors `fn is_field`. */
function isField(line, field) {
  const trimmed = trim(line);
  if (!trimmed.startsWith(field)) return false;
  const rest = trimmed.slice(field.length);
  if (!rest.startsWith(':')) return false;
  const after = rest.slice(1);
  return after === '' || after.startsWith(' ');
}

const ABSENT_SECTIONS = ['', '(empty)', '(none)', '(no output)', '(no error)', 'none'];

/** Mirrors `fn envelope_section`. */
function envelopeSection(lines, field) {
  let section = '';
  lines.forEach((line, index) => {
    if (index > 0) section += '\n';
    if (index === 0) {
      const trimmedLine = trim(line);
      section += trimStart(trimmedLine.startsWith(field) ? trimmedLine.slice(field.length) : line);
    } else {
      section += line;
    }
  });
  const trimmed = trim(section);
  return ABSENT_SECTIONS.includes(toAsciiLowercase(trimmed)) ? '' : trimmed;
}

function noOutputMarkers() {
  return agenticMessage('tool_result_no_output_markers').split('|');
}

/** Mirrors `fn from_payload`. */
function fromPayload(payload, error) {
  let trimmed = trim(payload);
  if (noOutputMarkers().includes(toAsciiLowercase(trimmed))) trimmed = '';
  const json = parseJson(trimmed);
  if (json !== undefined) {
    const text = mcpTextContent(json);
    if (text !== null) return result(text, error, 'text', null);
    return result(prettyJson(json), error, 'json', null);
  }
  let format = 'text';
  if (trimmed.startsWith('#!/bin/bash') || trimmed.startsWith(agenticMessage('tool_result_env_bash_shebang'))) format = 'bash';
  else if (trimmed.startsWith(agenticMessage('tool_result_env_python_shebang'))) format = 'python';
  return result(trimmed, error, format, null);
}

/** Mirrors `fn mcp_text_content`. */
function mcpTextContent(value) {
  const content = isObject(value) && has(value, 'content') ? value.content : value;
  if (!Array.isArray(content) || content.length === 0) return null;
  const parts = [];
  for (const item of content) {
    if (!isObject(item) || item.type !== 'text' || typeof item.text !== 'string') return null;
    parts.push(item.text);
  }
  const text = parts.join('\n');
  if (isObject(value) && has(value, 'structuredContent')) {
    const structured = structuredContentText(value.structuredContent);
    if (utf8Len(structured) > utf8Len(text)) return structured;
  }
  return text;
}

/** Mirrors `fn structured_content_text`. */
function structuredContentText(structured) {
  const inner = isObject(structured) && has(structured, 'content') ? structured.content : structured;
  let record = inner;
  if (typeof inner === 'string') {
    const parsed = parseJson(inner);
    record = parsed === undefined ? inner : parsed;
  }
  if (typeof record === 'string') return record;
  if (isObject(record)) {
    const title = asStr(jsonGet(record, 'title'));
    const body = asStr(jsonGet(record, 'body'));
    if (title !== null && body !== null) return [title, body].join('\n\n');
    if (title !== null) return title;
    if (body !== null) return body;
  }
  return prettyJson(record);
}

/** Mirrors `fn strip_transport_envelope`. */
export function stripTransportEnvelope(text) {
  const lines = rustLines(untrustedInner(text));
  const output = lines.findIndex((line) => {
    const trimmed = trim(line);
    return trimmed === 'Output:' || trimmed.startsWith('Output: ');
  });
  const kept = [];
  lines.forEach((line, index) => {
    if (output >= 0 && index < output) return;
    const trimmed = trim(line);
    if (trimmed.startsWith(`${agenticMessage('tool_result_process_group_field')}:`)) return;
    if (trimmed.startsWith('Output:')) {
      const rest = trim(trimmed.slice('Output:'.length));
      if (rest !== '(empty)' && rest !== '(no output)') kept.push(rest);
      return;
    }
    kept.push(line);
  });
  return trim(kept.join('\n'));
}

/** Mirrors `fn nonzero_status`. */
function nonzeroStatus(value) {
  const integer = asI64(value);
  if (integer !== null && integer !== 0) return true;
  if (typeof value !== 'string') return false;
  const parsed = parseI64(value);
  return parsed === null ? !['ok', 'success', 'completed'].includes(value) : parsed !== 0;
}

/** Mirrors `fn failed_status`. */
function failedStatus(value) {
  const numeric = asI64(value) ?? (typeof value === 'string' ? parseI64(value) : null);
  if (numeric === null) {
    return typeof value === 'string'
      && !['ok', 'success', 'succeeded', 'completed', 'passed'].includes(toAsciiLowercase(value))
      && !expectedStopStatus(value);
  }
  return numeric < 0 || (numeric > 0 && numeric < 100) || numeric >= 400;
}

const EXPECTED_STOPS = ['refused', 'denied', 'cancelled', 'canceled', 'aborted', 'pending', 'awaiting_approval', 'not_granted'];

/** Mirrors `fn expected_stop_status`. */
function expectedStopStatus(value) {
  return typeof value === 'string' && EXPECTED_STOPS.includes(toAsciiLowercase(value));
}

/** Mirrors `fn nonempty_text`. */
function nonemptyText(value) {
  if (value === null || value === undefined) return null;
  if (typeof value === 'string') return trim(value) ? trim(value) : null;
  if (isObject(value)) {
    const message = has(value, 'message') ? nonemptyText(value.message) : null;
    return message ?? compactJson(value);
  }
  return compactJson(value);
}

/** Mirrors `fn command_argument`. */
export function commandArgument(argumentsText) {
  const value = parseJson(argumentsText);
  if (!isObject(value)) return null;
  for (const key of ['command', 'cmd', 'script']) {
    const entry = jsonGet(value, key);
    if (typeof entry === 'string') return entry;
    if (Array.isArray(entry)) {
      const joined = entry.filter((word) => typeof word === 'string').join(' ');
      if (joined) return joined;
    }
  }
  return null;
}

/** Mirrors `fn command_argument_key`. */
export function commandArgumentKey(definition) {
  const holder = isObject(definition) && has(definition, 'function') ? definition.function : definition;
  const parameters = jsonGet(holder, 'parameters');
  if (parameters === undefined) return null;
  const properties = jsonGet(parameters, 'properties');
  if (!isObject(properties)) return null;
  for (const key of Object.keys(properties).sort()) {
    const property = properties[key];
    const describesCommand = ['description', 'title', 'name'].some((field) => {
      const text = asStr(jsonGet(property, field));
      if (text === null) return false;
      const lowered = text.toLowerCase();
      return lowered.includes('command') && !lowered.includes(agenticMessage('tool_result_working_directory_phrase'));
    });
    if (describesCommand && jsonGet(property, 'type') === 'string') return key;
  }
  return null;
}

/** The argument keys `commandArgument` reads on its own (`fn command_argument`). */
const FALLBACK_COMMAND_KEYS = ['command', 'cmd', 'script'];

/**
 * Mirrors `fn project_declared_command_keys`: the transcript as the planner
 * reads it, where every call to a tool whose own schema names its command
 * property `K` (outside `command`/`cmd`/`script`) also carries that value
 * under `command`. A pure projection of the request: nothing is stored
 * between requests, and the client's own messages are not mutated.
 * @param {Array<object>} messages
 * @param {Array<object>} tools the request's tool definitions
 * @returns {Array<object>}
 */
export function projectDeclaredCommandKeys(messages, tools) {
  if (!Array.isArray(tools) || tools.length === 0) return messages;
  const keyOf = new Map();
  const declaredKey = (name) => {
    if (!keyOf.has(name)) {
      const definition = findToolDefinition(tools, name);
      const key = definition ? commandArgumentKey(definition) : null;
      keyOf.set(name, key !== null && !FALLBACK_COMMAND_KEYS.includes(key) ? key : null);
    }
    return keyOf.get(name);
  };
  return messages.map((message) => {
    if (!Array.isArray(message.tool_calls) || message.tool_calls.length === 0) return message;
    let changed = false;
    const calls = message.tool_calls.map((call) => {
      const key = call?.function ? declaredKey(call.function.name) : null;
      if (key === null) return call;
      const value = parseJson(call.function.arguments);
      if (!isObject(value) || !has(value, key) || has(value, 'command')) return call;
      changed = true;
      return { ...call, function: { ...call.function, arguments: JSON.stringify({ command: value[key], ...value }) } };
    });
    return changed ? { ...message, tool_calls: calls } : message;
  });
}

/** Mirrors `fn result_tool`. */
function resultTool(messages, index) {
  const message = messages[index];
  if (message.name) return message.name;
  const id = message.tool_call_id;
  if (!id) return null;
  return findCall(messages, index, id)?.function.name ?? null;
}

function findCall(messages, index, id) {
  for (const prior of messages.slice(0, index)) {
    const call = (prior.tool_calls || []).find((candidate) => candidate.id === id);
    if (call) return call;
  }
  return null;
}

/** Mirrors `fn result_label`. */
function resultLabel(messages, index) {
  const id = messages[index].tool_call_id;
  const call = id ? findCall(messages, index, id) : null;
  const command = call ? commandArgument(call.function.arguments) : null;
  if (command !== null) return command;
  return resultTool(messages, index) ?? 'tool';
}

/** Mirrors `fn extract_urls`. */
function extractUrls(text) {
  const urls = [];
  let rest = text;
  for (;;) {
    const found = [rest.indexOf('https://'), rest.indexOf('http://')].filter((at) => at >= 0);
    if (!found.length) break;
    const candidate = rest.slice(Math.min(...found));
    let end = candidate.length;
    for (let index = 0; index < candidate.length; index += 1) {
      const character = candidate[index];
      if (isWhitespace(character) || '"\'<>)]}'.includes(character)) {
        end = index;
        break;
      }
    }
    urls.push(candidate.slice(0, end));
    rest = candidate.slice(end);
  }
  return urls;
}

/** Mirrors `fn is_listing`. */
function isListing(label) {
  const lower = toAsciiLowercase(label);
  const kind = commandKind(lower);
  return kind !== null ? kind === 'listing' : !isCommand(lower) && (lower.includes('list') || lower.includes('glob'));
}

/** Mirrors `fn is_search`. */
function isSearch(label) {
  const lower = toAsciiLowercase(label);
  const kind = commandKind(lower);
  return kind !== null ? kind === 'search'
    : !isCommand(lower) && ['grep', 'find', 'search'].some((word) => lower.includes(word));
}

/**
 * Mirrors `fn is_command`: a label with operands is a shell command, read by
 * its own word, never by a word inside its operands (`sed … allowlist.txt`,
 * PR #1188 G75); a one-word label is a tool's name.
 */
const isCommand = (lower) => splitWhitespace(lower).length > 1;

/** Mirrors `fn command_kind`: the seeded kind of a command's own word. */
function commandKind(lower) {
  const [word] = splitWhitespace(lower);
  const kinds = cached('result-command-words', () => {
    const root = parseRoot(readText(SHELL_INTENTS_FILE)).children[0];
    const group = childrenNamed(root, 'result_command_words')[0];
    return group === undefined ? [] : childrenNamed(group, 'command').map((node) => [childValue(node, 'word'), childValue(node, 'kind')]);
  });
  return kinds.find(([candidate]) => candidate === word)?.[1] ?? null;
}

/**
 * Mirrors `fn response_language`: the language detected from the request's
 * script, or -- when the script leaves it at the fallback language, which
 * shares its letters with others -- the language the request's own words are
 * seeded in. `Elimina las líneas 2 a 3 de f.txt.` names no Spanish marker,
 * but `elimina`, `las` and `líneas` are seeded only as Spanish (PR #1188 G20).
 * @param {string} prompt
 */
export function responseLanguage(prompt) {
  // The request's own words choose the language, never its quoted payload
  // (PR #1188 G92); a request that is all quotation keeps the whole text.
  const outside = textOutsideQuotedSegments(prompt);
  const detected = detect(/\p{L}/u.test(outside) ? outside : prompt);
  return detected === fallbackLanguage() ? lexicalLanguage(prompt) ?? detected : detected;
}

/** Fewer words seeded only in one language than this are no evidence. */
const MIN_LEXICAL_EVIDENCE = 2;

/** Mirrors `fn seeded_word_languages`: every one-word surface and the languages it is seeded in. */
function seededWordLanguages() {
  return cached('seeded-word-languages', () => {
    const languages = new Map();
    for (const meaning of lexicon()) {
      for (const lexeme of meaning.lexemes) {
        for (const word of lexeme.words) {
          const surface = word.text.toLowerCase();
          if (!/^\p{Alphabetic}+$/u.test(surface)) continue;
          if (!languages.has(surface)) languages.set(surface, new Set());
          languages.get(surface).add(lexeme.language);
        }
      }
    }
    return languages;
  });
}

/**
 * Mirrors `fn lexical_language` (rust/src/agentic_coding/tool_result/response_language.rs):
 * the one language most of the request's unquoted words are seeded in alone,
 * when it is not the fallback, has at least `MIN_LEXICAL_EVIDENCE` such words
 * and more than the fallback has; else null (a tie decides nothing).
 */
function lexicalLanguage(prompt) {
  const seeded = seededWordLanguages();
  const counts = new Map();
  for (const word of textOutsideQuotedSegments(prompt).toLowerCase().split(/[^\p{Alphabetic}]+/u)) {
    const languages = seeded.get(word);
    if (languages === undefined || languages.size !== 1) continue;
    const [language] = languages;
    counts.set(language, (counts.get(language) ?? 0) + 1);
  }
  const fallback = fallbackLanguage();
  const top = Math.max(0, ...counts.values());
  const leaders = [...counts].filter(([, count]) => count === top).map(([language]) => language);
  if (leaders.length !== 1 || top < MIN_LEXICAL_EVIDENCE) return null;
  const [leader] = leaders;
  return leader === fallback || top <= (counts.get(fallback) ?? 0) ? null : leader;
}

/** Mirrors `fn fill` in tool_result.rs. */
function fill(intent, language, values) {
  let text = localizedResponse(intent, language) ?? '';
  for (const [placeholder, value] of values) text = replaceAllLiteral(text, placeholder, value);
  return text;
}
