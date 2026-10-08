// The white-box derivation record for every answer (rust/src/derivation.rs,
// issue #1184, E148; requirement R1184-9 three-roots parity).
//
// One event log in, one record out: `recordFor` reads the events in append
// order and nothing else, so the record cannot disagree with the thinking
// trace narrated from the same log. A stage a route never populated stays
// empty and `explainText` prints it as "not recorded" — never fabricated.
//
// The JavaScript root has no `EventLog` type; an event log here is an array
// of `{ kind, payload }` objects in append order (the shape of Rust's
// `EventLog::events()`). Records and their parts keep the Rust field names
// (`answer_id`, `exit_code`, ...), as `execution_evidence.mjs` does for
// `Evidence`; an absent `Option` is `null`.
//
// The applied-rule kinds come from `data/seed/derivation-schema.lino` (its
// `stage ... collects rule` rows), never from a list in this file.
//
// `Derivation::persist`, `Derivation::load` and `explain_answer` are mirrored
// over an injected `io` — `{ readText(path), writeText(path, text),
// createDirAll?(path) }` — because the agentic host offers `readText` for
// repository files only and no write (the same convention as the procedure
// cache in discovery_production.mjs). A node caller passes `node:fs`
// wrappers; the browser worker (js/worker/formal_ai_worker_derivation.js, a
// classic-script twin of this module) injects a store over the app's memory
// event log. `storePath` and `missMessage` keep the path and the miss wording
// one spelling across roots.

import { cached, readText } from '../host.mjs';
import { trim } from '../write_str.mjs';
import { stableId } from './engine_stable_identifier.mjs';
import { pushLinoNode } from './links_format.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';
import { reportText } from './seed_reports.mjs';

/** Mirrors `const NOT_RECORDED`. */
export const NOT_RECORDED = 'not recorded';
/** Mirrors `const SEARCH_REQUEST_KIND`. */
export const SEARCH_REQUEST_KIND = 'web_search:request';
/** Mirrors `const SOURCE_HTTP_KIND`. */
export const SOURCE_HTTP_KIND = 'source:http';
/** Mirrors `const FORMALIZE_FRAGMENT_KIND`. */
export const FORMALIZE_FRAGMENT_KIND = 'formalize:fragment';
/** Mirrors `const DECOMPOSE_PART_KIND`. */
export const DECOMPOSE_PART_KIND = 'decompose:part';
/** Mirrors `const RECOMPOSE_BIND_KIND`. */
export const RECOMPOSE_BIND_KIND = 'recompose:bind';
/** Mirrors `const RENDER_EMIT_KIND`. */
export const RENDER_EMIT_KIND = 'render:emit';
/** Mirrors `const VERIFICATION_KIND`. */
export const VERIFICATION_KIND = 'verify:evidence';
/** Mirrors `const DERIVATIONS_DIR`. */
export const DERIVATIONS_DIR = 'data/cache/derivations';

/** Mirrors `const SCHEMA` (read through the host, not embedded). */
const SCHEMA_PATH = 'data/seed/derivation-schema.lino';
/** Mirrors `const APPLIED_RULE_FIELD`. */
const APPLIED_RULE_FIELD = 'rule';

/** Mirrors `fn applied_rule_kinds`: the schema's `stage` kinds that collect `rule`. */
export function appliedRuleKinds() {
  return cached('crate:derivation_applied_rule_kinds', () => {
    const kinds = new Set();
    for (const schema of parseRoot(readText(SCHEMA_PATH)).children) {
      for (const row of schema.children) {
        if (row.name !== 'stage' || findChildValue(row, 'collects') !== APPLIED_RULE_FIELD) continue;
        const kind = findChildValue(row, 'kind');
        if (kind !== '') kinds.add(kind);
      }
    }
    return kinds;
  });
}

/** Rust `i64` parse: optional sign, ASCII digits, within range; else `null`. @param {string} text */
function parseI64(text) {
  if (!/^[+-]?[0-9]+$/u.test(text)) return null;
  const value = BigInt(text);
  if (value < -(2n ** 63n) || value > 2n ** 63n - 1n) return null;
  return Number(value);
}

/** `Option<i64>` printed the way the record spells it. @param {number|null} code */
function exitText(code) {
  return code === null || code === undefined ? 'none' : String(code);
}

/** Mirrors `str::split_once`. @param {string} text @param {string} separator */
function splitOnce(text, separator) {
  const index = text.indexOf(separator);
  return index < 0 ? null : [text.slice(0, index), text.slice(index + separator.length)];
}

/** Mirrors `str::rsplit_once`. @param {string} text @param {string} separator */
function rsplitOnce(text, separator) {
  const index = text.lastIndexOf(separator);
  return index < 0 ? null : [text.slice(0, index), text.slice(index + separator.length)];
}

/** Mirrors `struct AppliedRule`. @param {string} kind @param {string} detail */
export function appliedRule(kind = '', detail = '') {
  return { kind, detail };
}

/** Mirrors `struct FetchRecord`. */
export function fetchRecord(url = '', sha256 = '', fetched_at = '') {
  return { url, sha256, fetched_at };
}

/** Mirrors `struct VerificationRecord`. @param {number|null} exit_code */
export function verificationRecord(evidence_id = '', command = '', exit_code = null) {
  return { evidence_id, command, exit_code };
}

/** Mirrors `fn VerificationRecord::from_evidence`. */
export function verificationFromEvidence(evidence) {
  return verificationRecord(evidence.evidence_id, evidence.command, evidence.exit_code ?? null);
}

/** Mirrors `fn VerificationRecord::payload`. */
export function verificationPayload(record) {
  return `evidence_id=${record.evidence_id};command=${record.command};exit=${exitText(record.exit_code)}`;
}

/** Mirrors `fn VerificationRecord::parse_payload`: `null` without an `evidence_id=` field. @param {string} payload */
export function parseVerificationPayload(payload) {
  if (!payload.startsWith('evidence_id=')) return null;
  const rest = payload.slice('evidence_id='.length);
  const head = splitOnce(rest, ';command=') ?? splitOnce(rest, ' command=');
  if (head === null) return null;
  const [evidenceId, tail] = head;
  const body = rsplitOnce(tail, ';exit=') ?? rsplitOnce(tail, ' exit=');
  if (body === null) return null;
  if (evidenceId === '') return null;
  const [command, exit] = body;
  return verificationRecord(evidenceId, command, parseI64(exit));
}

/** Mirrors `struct Derivation` (its `Default`, with `answer_id`). @param {string} answerId */
export function emptyDerivation(answerId = '') {
  return {
    answer_id: answerId,
    search_queries: [],
    fetches: [],
    formalized_fragments: [],
    decomposed_parts: [],
    recomposition: null,
    rendering: null,
    verification: [],
    applied_rules: [],
  };
}

/** Mirrors `fn answer_derivation_id`. @param {string} answerText */
export function answerDerivationId(answerText) {
  return stableId('answer', answerText);
}

/**
 * Mirrors `fn Derivation::record_for`.
 * @param {Array<{kind: string, payload: string}>} events in append order
 * @param {string} answerId
 */
export function recordFor(events, answerId) {
  const derivation = emptyDerivation(answerId);
  for (const { kind, payload } of events) {
    if (kind === SEARCH_REQUEST_KIND) derivation.search_queries.push(payload);
    else if (kind === SOURCE_HTTP_KIND) {
      const fetch = parseSourceHttp(payload);
      if (fetch !== null) derivation.fetches.push(fetch);
    } else if (kind === FORMALIZE_FRAGMENT_KIND) derivation.formalized_fragments.push(payload);
    else if (kind === DECOMPOSE_PART_KIND) derivation.decomposed_parts.push(payload);
    else if (kind === RECOMPOSE_BIND_KIND) derivation.recomposition = payload;
    else if (kind === RENDER_EMIT_KIND) derivation.rendering = payload;
    else if (kind === VERIFICATION_KIND) {
      const record = parseVerificationPayload(payload);
      if (record !== null) derivation.verification.push(record);
    } else if (appliedRuleKinds().has(kind)) derivation.applied_rules.push(appliedRule(kind, payload));
  }
  return derivation;
}

/** Mirrors `fn Derivation::to_lino`. */
export function toLino(derivation) {
  let out = pushLinoNode('', 0, 'derivation', null);
  out = pushLinoNode(out, 2, 'answer_id', derivation.answer_id);
  for (const query of derivation.search_queries) out = pushLinoNode(out, 2, 'search_query', query);
  for (const fetch of derivation.fetches) {
    out = pushLinoNode(out, 2, 'fetch', null);
    out = pushLinoNode(out, 4, 'url', fetch.url);
    out = pushLinoNode(out, 4, 'sha256', fetch.sha256);
    out = pushLinoNode(out, 4, 'fetched_at', fetch.fetched_at);
  }
  for (const fragment of derivation.formalized_fragments) out = pushLinoNode(out, 2, 'formalized_fragment', fragment);
  for (const part of derivation.decomposed_parts) out = pushLinoNode(out, 2, 'decomposed_part', part);
  if (derivation.recomposition !== null) out = pushLinoNode(out, 2, 'recomposition', derivation.recomposition);
  if (derivation.rendering !== null) out = pushLinoNode(out, 2, 'rendering', derivation.rendering);
  for (const record of derivation.verification) {
    out = pushLinoNode(out, 2, 'verification', null);
    out = pushLinoNode(out, 4, 'evidence_id', record.evidence_id);
    out = pushLinoNode(out, 4, 'command', record.command);
    out = pushLinoNode(out, 4, 'exit', exitText(record.exit_code));
  }
  for (const rule of derivation.applied_rules) {
    out = pushLinoNode(out, 2, APPLIED_RULE_FIELD, null);
    out = pushLinoNode(out, 4, 'kind', rule.kind);
    out = pushLinoNode(out, 4, 'detail', rule.detail);
  }
  return out;
}

/** Mirrors `fn Derivation::from_lino`: `null` for a non-record or a missing `answer_id`. @param {string} text */
export function fromLino(text) {
  const record = parseRoot(text).children.find((child) => child.name === 'derivation');
  if (record === undefined) return null;
  const answerId = findChildValue(record, 'answer_id');
  if (answerId === '') return null;
  const derivation = emptyDerivation(answerId);
  for (const child of record.children) {
    const field = (name) => findChildValue(child, name);
    if (child.name === 'search_query') derivation.search_queries.push(child.id);
    else if (child.name === 'fetch') derivation.fetches.push(fetchRecord(field('url'), field('sha256'), field('fetched_at')));
    else if (child.name === 'formalized_fragment') derivation.formalized_fragments.push(child.id);
    else if (child.name === 'decomposed_part') derivation.decomposed_parts.push(child.id);
    else if (child.name === 'recomposition') derivation.recomposition = child.id;
    else if (child.name === 'rendering') derivation.rendering = child.id;
    else if (child.name === 'verification') {
      const exit = field('exit');
      derivation.verification.push(verificationRecord(field('evidence_id'), field('command'), exit === 'none' ? null : parseI64(exit)));
    } else if (child.name === APPLIED_RULE_FIELD) derivation.applied_rules.push(appliedRule(field('kind'), field('detail')));
  }
  return derivation;
}

/** One `  stage <name>` block: its rows, or "not recorded". */
function stageBlock(name, rows) {
  const body = rows.length === 0 ? [NOT_RECORDED] : rows;
  return `  stage ${name}\n${body.map((row) => `    ${row}\n`).join('')}`;
}

/** Mirrors `fn Derivation::explain_text`. */
export function explainText(derivation) {
  return [
    `derivation ${derivation.answer_id}\n`,
    stageBlock('search_queries', derivation.search_queries),
    stageBlock('fetches', derivation.fetches.map((fetch) => ['url', fetch.url, 'sha256', fetch.sha256, 'fetched_at', fetch.fetched_at].join(' '))),
    stageBlock('formalized_fragments', derivation.formalized_fragments),
    stageBlock('decomposed_parts', derivation.decomposed_parts),
    `  stage recomposition: ${derivation.recomposition ?? NOT_RECORDED}\n`,
    `  stage rendering: ${derivation.rendering ?? NOT_RECORDED}\n`,
    stageBlock('verification', derivation.verification.map((record) => `${record.command} exit=${exitText(record.exit_code)}`)),
    stageBlock('applied_rules', derivation.applied_rules.map((rule) => `${rule.kind} ${rule.detail}`)),
  ].join('');
}

/** `Path::join` for the `/`-separated paths this module builds. @param {string} root @param {string} relative */
function joinPath(root, relative) {
  if (root === '') return relative;
  return root.endsWith('/') ? `${root}${relative}` : `${root}/${relative}`;
}

/** Mirrors `fn store_path`: `null` unless the id is a `[A-Za-z0-9_-]` token. */
export function storePath(repositoryRoot, answerId) {
  if (!/^[A-Za-z0-9_-]+$/u.test(answerId)) return null;
  return joinPath(joinPath(repositoryRoot, DERIVATIONS_DIR), `${answerId}.lino`);
}

/** The refusal `persist` returns for an id `storePath` will not address. */
export const UNUSABLE_ANSWER_ID = 'derivation_answer_id_unusable';

/** The parent directory of a `/`-separated path (`Path::parent`). @param {string} file */
function parentPath(file) {
  const index = file.lastIndexOf('/');
  return index <= 0 ? '' : file.slice(0, index);
}

/**
 * Mirrors `fn Derivation::persist`: writes `toLino` at `storePath`, creating
 * the directory first when `io.createDirAll` is given. Returns
 * `{ ok: true, path }` or `{ ok: false, error }`.
 */
export function persist(derivation, repositoryRoot, io = {}) {
  const path = storePath(repositoryRoot, derivation.answer_id);
  if (path === null) return { ok: false, error: UNUSABLE_ANSWER_ID };
  if (typeof io.writeText !== 'function') return { ok: false, error: 'derivation_store_has_no_writer' };
  try {
    const parent = parentPath(path);
    if (parent !== '' && typeof io.createDirAll === 'function') io.createDirAll(parent);
    io.writeText(path, toLino(derivation));
    return { ok: true, path };
  } catch (error) {
    return { ok: false, error: `derivation_write_failed:${error?.message ?? error}` };
  }
}

/**
 * Mirrors `fn Derivation::load`: the persisted record, or `null` for a miss —
 * an unusable id, an unreadable file, a non-record, or a record filed under
 * another answer id.
 */
export function load(repositoryRoot, answerId, io = {}) {
  const path = storePath(repositoryRoot, answerId);
  if (path === null || typeof io.readText !== 'function') return null;
  let text;
  try {
    text = io.readText(path);
  } catch {
    return null;
  }
  if (typeof text !== 'string') return null;
  const derivation = fromLino(text);
  return derivation !== null && derivation.answer_id === answerId ? derivation : null;
}

/** Mirrors `fn explain_answer`: `{ ok: true, text }` or `{ ok: false, error }` (the miss message). */
export function explainAnswer(repositoryRoot, answerId, io = {}) {
  const derivation = load(repositoryRoot, answerId, io);
  return derivation === null
    ? { ok: false, error: missMessage(repositoryRoot, answerId) }
    : { ok: true, text: explainText(derivation) };
}

/** Mirrors `fn miss_message`. */
export function missMessage(repositoryRoot, answerId) {
  return reportText('derivation_record_missing', [['answer_id', answerId], ['directory', joinPath(repositoryRoot, DERIVATIONS_DIR)]]);
}

/** Mirrors `fn parse_source_http`: either payload spelling, `null` without a url. @param {string} payload */
export function parseSourceHttp(payload) {
  const trimmed = trim(payload);
  if (trimmed === '') return null;
  let url;
  let entries;
  if (trimmed.startsWith('url=')) {
    const parts = trimmed.slice('url='.length).split(';');
    url = trim(parts[0]);
    entries = parts.slice(1);
  } else {
    const parts = trimmed.split(' ');
    url = parts[0];
    entries = parts.slice(1);
  }
  if (url === '') return null;
  const fields = parseEntries(entries);
  const field = (name) => fields.find(([key]) => key === name)?.[1] ?? '';
  return fetchRecord(url, field('sha256'), field('fetched_at'));
}

/** Mirrors `fn parse_entries`. @param {string[]} entries */
export function parseEntries(entries) {
  const pairs = [];
  for (const raw of entries) {
    const entry = trim(raw);
    if (entry === '') continue;
    const pair = splitOnce(entry, '=');
    if (pair !== null) pairs.push([trim(pair[0]), trim(pair[1])]);
  }
  return pairs;
}
