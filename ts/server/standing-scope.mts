// The scope a learned standing requirement belongs to (issue #1559):
// rust/src/standing_scope.rs.
//
// Every recorded exchange carries the scope it was made in as evidence entries
// `scope:user=<id>`, `scope:repository=<host/owner/repo>` and
// `scope:session=<id>`:
//
// - `user` is the client-declared end user (OpenAI `user`, Anthropic
//   `metadata.user_id`) — a requirement that user states follows that user into
//   every later conversation;
// - `session` is recorded for an agent session (a request that carries tools):
//   the client's first message configures one task, so what is learned there
//   stays in that session unless sharing is requested with
//   `FORMAL_AI_SHARE_STANDING_REQUIREMENTS=1`;
// - `repository` names the repository an agent session works on, so a trace
//   shows which task a requirement came from.
//
// An exchange with neither a user nor a session scope belongs to the owner of
// the memory store, and what it teaches applies everywhere in that store, as
// before. An amendment applies to a request only when every scope pair it
// carries equals the request's own.

import { AsyncLocalStorage } from 'node:async_hooks';

import { lexicon, splitWhitespace } from './dreaming-support.mjs';
import { stableId } from './ids.mjs';
import { isSessionInstruction } from './standing-requirement-bounds.mjs';

export const SCOPE_PREFIX = 'scope:';
/** The evidence entry of an amendment generalized from several scopes and verified by replay. */
export const GENERALIZED_SCOPE = 'scope:generalized';
/** The scope keys in their canonical order. */
export const SCOPE_KEYS = ['user', 'repository', 'session'];
const SHARE_VARIABLE = 'FORMAL_AI_SHARE_STANDING_REQUIREMENTS';
const URL_TRAILING = new Set([')', ']', '}', '>', '"', "'", '`', ',', ';', ':', '!', '?', '.']);

const storage = new AsyncLocalStorage();

/** Run `fn` with `scope` as the scope of the request being answered. */
export function runInRequestScope(scope, fn) {
  return storage.run(scope, fn);
}

/** The scope of the request being answered, or null outside one. */
export function currentRequestScope() {
  return storage.getStore() ?? null;
}

function messageText(message) {
  const { content } = message;
  if (typeof content === 'string') return content;
  if (!Array.isArray(content)) return '';
  return content.filter((part) => part && typeof part.text === 'string').map((part) => part.text).join('\n');
}

/** Mirrors rust/src/standing_scope.rs `request_user`. */
export function requestUser(body) {
  let object;
  try {
    object = typeof body === 'string' ? JSON.parse(body) : body;
  } catch {
    return null;
  }
  if (!object || typeof object !== 'object' || Array.isArray(object)) return null;
  const candidates = [object.user, object.metadata?.user_id];
  for (const candidate of candidates) {
    if (typeof candidate === 'string' && candidate.trim()) return candidate.trim();
  }
  return null;
}

function strippedUrl(raw) {
  const chars = [...raw];
  let end = chars.length;
  while (end > 0 && URL_TRAILING.has(chars[end - 1])) end -= 1;
  return chars.slice(0, end).join('');
}

/**
 * Mirrors rust/src/standing_scope.rs `repository_of`: `host/owner/repo` of the
 * first URL that points at an issue or pull request in `text`.
 */
export function repositoryOf(text) {
  const kinds = lexicon().task_reference_words;
  for (const raw of splitWhitespace(String(text ?? ''))) {
    const at = raw.indexOf('://');
    if (at <= 0) continue;
    const segments = strippedUrl(raw.slice(at + 3)).split('/').filter(Boolean);
    if (segments.length < 4 || !segments[0].includes('.')) continue;
    if (!kinds.includes(segments[3].toLowerCase())) continue;
    const repository = segments[2].toLowerCase().replace(/\.git$/u, '');
    return `${segments[0].toLowerCase()}/${segments[1].toLowerCase()}/${repository}`;
  }
  return null;
}

/** Mirrors rust/src/standing_scope.rs `sharing_requested`. */
export function sharingRequested(env = process.env) {
  const value = String(env[SHARE_VARIABLE] ?? '').trim().toLowerCase();
  return value === '1' || value === 'true' || value === 'yes';
}

/**
 * Mirrors rust/src/standing_scope.rs `request_scope`: the scope of one request
 * from its raw body, chat messages and advertised tools.
 */
export function requestScope({ body = null, messages = [], tools = [], env = process.env } = {}) {
  const user = requestUser(body);
  const configuring = messages.filter((message) => ['system', 'developer', 'user'].includes(String(message.role).toLowerCase()));
  const repository = repositoryOf(configuring.map(messageText).join('\n'));
  const first = messages.find((message) => String(message.role).toLowerCase() === 'user');
  const session = first ? stableId('session', messageText(first)) : null;
  const agentSession = Array.isArray(tools) && tools.length > 0;
  return { user, repository, session, agent_session: agentSession, shared: sharingRequested(env) };
}

/**
 * Mirrors rust/src/standing_scope.rs `recorded_scope_evidence`: the scope
 * entries a recorded exchange carries. A user scope wins; an agent session is
 * scoped to its session (and repository) unless sharing is requested.
 */
export function recordedScopeEvidence(scope) {
  if (!scope) return [];
  if (scope.user) return [`${SCOPE_PREFIX}user=${scope.user}`];
  if (!scope.agent_session || scope.shared || !scope.session) return [];
  const entries = [];
  if (scope.repository) entries.push(`${SCOPE_PREFIX}repository=${scope.repository}`);
  entries.push(`${SCOPE_PREFIX}session=${scope.session}`);
  return entries;
}

/** Mirrors rust/src/standing_scope.rs `scope_pairs`: `[key, value]` pairs in evidence, canonical order. */
export function scopePairs(evidence) {
  const pairs = [];
  for (const entry of evidence ?? []) {
    const text = String(entry);
    if (!text.startsWith(SCOPE_PREFIX)) continue;
    const body = text.slice(SCOPE_PREFIX.length);
    const equals = body.indexOf('=');
    if (equals <= 0) continue;
    const key = body.slice(0, equals);
    if (SCOPE_KEYS.includes(key) && !pairs.some(([existing]) => existing === key)) pairs.push([key, body.slice(equals + 1)]);
  }
  return pairs.sort(([left], [right]) => SCOPE_KEYS.indexOf(left) - SCOPE_KEYS.indexOf(right));
}

/** Mirrors rust/src/standing_scope.rs `scope_key`: a canonical text key of scope pairs ('' when global). */
export function scopeKey(pairs) {
  return pairs.map(([key, value]) => `${key}=${value}`).join('\n');
}

/** Mirrors rust/src/standing_scope.rs `in_scope`: every pair equals the request's own. */
export function inScope(pairs, scope) {
  return pairs.every(([key, value]) => scope !== null && scope !== undefined && scope[key] === value);
}

/**
 * Mirrors rust/src/standing_scope.rs `events_in_scope`: memory as one request
 * sees it (issue #1559 §6) — events recorded in another scope, and amendments
 * learned there, are dropped. With no request scope (an internal caller),
 * every scoped event is dropped.
 */
export function eventsInScope(events, scope) {
  if (!events.some((event) => scopePairs(event.evidence).length > 0)) return events;
  return events.filter((event) => inScope(scopePairs(event.evidence), scope));
}

/**
 * Mirrors rust/src/standing_scope.rs `is_instruction_prompt`: the recorded
 * prompt configures the session rather than stating a user requirement — it
 * opens like a system prompt, or it is the first user message of an agent
 * session that carries no system message (a client delivering its system
 * prompt as the first user turn, issue #1559 §1).
 */
export function isInstructionPrompt(prompt, messages, scope) {
  if (isSessionInstruction(prompt)) return true;
  if (!scope || !scope.agent_session) return false;
  const roles = messages.map((message) => String(message.role).toLowerCase());
  if (roles.includes('system') || roles.includes('developer')) return false;
  const first = messages.find((message) => String(message.role).toLowerCase() === 'user');
  return first !== undefined && messageText(first).trim() === String(prompt).trim();
}
