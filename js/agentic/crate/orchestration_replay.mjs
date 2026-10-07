// `crate::orchestration::replay` (rust/src/orchestration/replay.rs): replay a
// recorded external-agent session byte for byte (issue #703, R703-7). A
// session replays only when its bytes are the canonical pretty rendering, its
// events form an unbroken SHA-256 chain, and the session fields the events
// stand for (status, workspace effects, continuation) agree with those events.
// `replayContinuation` adds the ancestry check: a correction session replays
// only against the exact parent bytes its continuation digest names.
//
// The Rust replay re-serializes the typed struct; JavaScript has no struct, so
// the canonical check is spelled out here as the same field order, the same
// optional-field rule (`skip_serializing_if = "Option::is_none"`) and the same
// `serde_json::to_vec_pretty` layout plus a terminal newline.

import { sha256Hex } from './source_fetch.mjs';

export const SESSION_SCHEMA = 'formal-ai-agent-session-v1';

const SESSION_FIELDS = ['schema', 'cli', 'target', 'task', 'model', 'base_url', 'workspace', 'program', 'args', 'status', 'exit_code', 'wall_time_ms', 'stdout', 'stderr', 'changes', 'verification', 'events'];
const SESSION_OPTIONAL = ['native_session', 'continuation'];
const EVENT_FIELDS = ['sequence', 'kind', 'detail', 'previous_sha256', 'sha256'];
const CHANGE_FIELDS = ['path', 'kind', 'before_sha256', 'after_sha256', 'bytes_changed'];
const VERIFICATION_FIELDS = ['program', 'args', 'exit_code', 'stdout', 'stderr', 'timed_out', 'passed'];
const NATIVE_FIELDS = ['id', 'resume_command'];
const CONTINUATION_FIELDS = ['parent_session_sha256', 'native_session_id', 'disproved_claim', 'evidence'];

const STATUSES = ['succeeded', 'failed', 'timed_out'];
const TARGETS = ['formal_ai', 'vendor'];
const CHANGE_KINDS = ['added', 'modified', 'removed'];

// The terminal event `run_agent` / `verify_workspace` record for each status
// (rust/src/orchestration/runner.rs).
const PROCESS_OUTCOMES = { process_succeeded: 'succeeded', process_failed: 'failed', process_timed_out: 'timed_out' };
const COMPOSITION_OUTCOMES = { composition_verification_passed: 'succeeded', composition_verification_failed: 'failed' };
const COMPOSITION_STARTED = 'composition_verification_started';
const WORKSPACE_EFFECT = 'workspace_effect';
const NATIVE_SESSION_RESUMED = 'native_session_resumed';

/** Mirrors `enum ReplayError`; `message` is its `Display` rendering. */
export class ReplayError extends Error {
  /** @param {string} code @param {string|number|null} [detail] */
  constructor(code, detail = null) {
    super(detail === null ? code : `${code}:${detail}`);
    this.code = code;
    this.detail = detail;
  }
}

const fail = (code, detail) => { throw new ReplayError(code, detail); };

// Fields serde reads as `None` when absent; the canonical rendering writes them.
const NULLABLE = new Set(['exit_code', 'before_sha256', 'after_sha256']);

/**
 * Presence of one serialized struct's fields (a missing field is the serde
 * error `ReplayError::Json`). Returns whether the keys are also in canonical
 * order with no unknown or null optional field; a session that is not is
 * rejected as non-canonical once its events have been checked, the order the
 * Rust replay reports them in.
 */
function checkFields(value, required, optional = []) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) fail('json', 'expected_object');
  const missing = required.find((field) => !(field in value) && !NULLABLE.has(field));
  if (missing) fail('json', `missing_field:${missing}`);
  const keys = Object.keys(value);
  if (required.some((field, index) => keys[index] !== field)) return false;
  let cursor = 0;
  for (const key of keys.slice(required.length)) {
    const position = optional.indexOf(key, cursor);
    if (position < 0 || value[key] === null) return false;
    cursor = position + 1;
  }
  return true;
}

const isString = (value) => typeof value === 'string';
const isCount = (value) => Number.isSafeInteger(value) && value >= 0;
const isCode = (value) => value === undefined || value === null || Number.isSafeInteger(value);
const isStrings = (value) => Array.isArray(value) && value.every(isString);

/** The serde type checks; returns whether every struct is in canonical form. */
function checkTypes(session) {
  let canonical = checkFields(session, SESSION_FIELDS, SESSION_OPTIONAL);
  const strings = ['schema', 'cli', 'task', 'model', 'base_url', 'workspace', 'program', 'stdout', 'stderr'];
  if (!strings.every((field) => isString(session[field]))) fail('json', 'expected_string');
  if (!TARGETS.includes(session.target) || !STATUSES.includes(session.status)) fail('json', 'unknown_variant');
  if (!isStrings(session.args) || !isCode(session.exit_code) || !isCount(session.wall_time_ms)) fail('json', 'invalid_type');
  for (const change of asArray(session.changes)) {
    canonical = checkFields(change, CHANGE_FIELDS) && canonical;
    if (!isString(change.path) || !CHANGE_KINDS.includes(change.kind) || !isCount(change.bytes_changed)) fail('json', 'invalid_type');
  }
  for (const result of asArray(session.verification)) {
    canonical = checkFields(result, VERIFICATION_FIELDS) && canonical;
    if (typeof result.passed !== 'boolean' || typeof result.timed_out !== 'boolean') fail('json', 'invalid_type');
  }
  for (const event of asArray(session.events)) {
    canonical = checkFields(event, EVENT_FIELDS) && canonical;
    if (!isCount(event.sequence) || !EVENT_FIELDS.slice(1).every((field) => isString(event[field]))) fail('json', 'invalid_type');
  }
  if (session.native_session != null) canonical = checkFields(session.native_session, NATIVE_FIELDS) && canonical;
  if (session.continuation != null) canonical = checkFields(session.continuation, CONTINUATION_FIELDS) && canonical;
  return canonical;
}

const asArray = (value) => (Array.isArray(value) ? value : fail('json', 'expected_array'));

/** Mirrors `fn verify_events`: sequence, previous-event link, event digest. */
function verifyEvents(events) {
  let previous = '0'.repeat(64);
  for (const [sequence, event] of events.entries()) {
    if (event.sequence !== sequence) fail('event_sequence', sequence);
    if (event.previous_sha256 !== previous) fail('event_chain', sequence);
    const payload = `${event.sequence}\0${event.kind}\0${event.detail}\0${event.previous_sha256}`;
    if (event.sha256 !== sha256Hex(payload)) fail('event_digest', sequence);
    previous = event.sha256;
  }
}

/**
 * Mirrors `fn verify_bindings`: the session fields the event chain stands for
 * must say what the chain says. A session that ran an agent records exactly
 * one terminal process event; a composed-verifier session records one
 * composition outcome and no workspace effect.
 */
function verifyBindings(session) {
  const kinds = session.events.map((event) => event.kind);
  const composed = kinds.includes(COMPOSITION_STARTED);
  const outcomes = composed ? COMPOSITION_OUTCOMES : PROCESS_OUTCOMES;
  const terminal = kinds.filter((kind) => kind in outcomes);
  if (terminal.length !== 1 || outcomes[terminal[0]] !== session.status) fail('event_binding', 'status');
  const effects = session.events.filter((event) => event.kind === WORKSPACE_EFFECT).map((event) => event.detail);
  const changed = session.changes.map((change) => change.path);
  if (effects.length !== changed.length || effects.some((path, index) => path !== changed[index])) fail('event_binding', 'effects');
  if (kinds.includes(NATIVE_SESSION_RESUMED) !== (session.continuation != null)) fail('event_binding', 'continuation');
}

/** Mirrors `serde_json::to_vec_pretty` plus the terminal newline. */
export function canonicalSessionText(session) {
  return `${JSON.stringify(session, null, 2)}\n`;
}

/** Mirrors `fn session_sha256`: digest of the canonical replay bytes. */
export function sessionSha256(session) {
  return sha256Hex(canonicalSessionText(session));
}

/**
 * Mirrors `fn replay_session`. @param {string} text the recorded bytes as UTF-8
 * @returns {object} the replayed session
 */
export function replaySession(text) {
  let session;
  try {
    session = JSON.parse(text);
  } catch (error) {
    fail('json', error.message);
  }
  const ordered = checkTypes(session);
  if (session.schema !== SESSION_SCHEMA) fail('unsupported_session_schema');
  verifyEvents(session.events);
  verifyBindings(session);
  if (!ordered || canonicalSessionText(session) !== text) fail('non_canonical_session');
  return session;
}

/**
 * Mirrors `fn replay_continuation`: the child replays on its own and names the
 * exact parent bytes as its ancestor.
 */
export function replayContinuation(parentText, childText) {
  replaySession(parentText);
  const child = replaySession(childText);
  if (!child.continuation || child.continuation.parent_session_sha256 !== sha256Hex(parentText)) fail('broken_ancestry');
  return child;
}
