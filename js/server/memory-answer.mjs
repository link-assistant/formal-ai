// Answers read straight from the memory store, ahead of the solver:
// rust/src/protocol_memory.rs (`answer_from_memory_if_requested`,
// `answer_memory_inspection`, `memory_events_with_request_history`), the
// recall half of rust/src/solver_handlers/conversation_memory/mod.rs
// (`answer_memory_recall`, `try_memory_recall`, `memory_recall_matches`,
// `render_memory_recall_report`), the record projection of
// rust/src/link_store.rs (`memory_events_to_link_records`) and the
// `finalize_simple` / `build_evidence_links` tail of rust/src/solver_handlers
// and rust/src/event_log.rs those answers end with.
//
// Seed reads (prompt normalization, language detection, lexicon roles,
// response templates) go through the worker's own readers
// (`WorkerHost.seedReaders`), so both servers read one seed. The fixed
// report wording Rust writes inline lives in data/meta/server-messages.lino.

import { stableId } from './ids.mjs';
import { serverMessage } from './messages.mjs';
import { thinkingStepsFromEvents } from './solver-trace.mjs';

const REQUEST_HISTORY_CONVERSATION_ID = 'request_history';
const LEGACY_CONVERSATION = 'legacy';
const KNOWLEDGE_SCHEMA_VERSION = '0.2.0';
const RECALL_LANGUAGES = new Set(['ru', 'zh', 'hi']);
const ROLE_MEMORY_RETRIEVAL_CORRECTION = 'memory_retrieval_correction';
const ROLE_MEMORY_LINK_COUNT_QUERY = 'memory_link_count_query';
const ROLE_MEMORY_INVENTORY_QUERY = 'memory_inventory_query';
const ROLE_MEMORY_ROOT_LINKS_QUERY = 'memory_root_links_query';
const ROLE_CONVERSATION_RECALL_QUERY = 'conversation_recall_query';
const ROLE_CONVERSATION_RECALL_OTHER_QUERY = 'conversation_recall_other_query';
const encoder = new TextEncoder();

/** Mirrors rust/src/event_log.rs `EventLog` (append, events, first_of). */
export class EventLog {
  constructor() {
    this.events = [];
  }

  /** `EventLog::append`: the event id is `stable_id(kind, "{kind}:{index}:{payload}")`. */
  append(kind, payload) {
    const text = String(payload);
    const id = stableId(kind, `${kind}:${this.events.length}:${text}`);
    this.events.push({ id, kind, payload: text });
    return id;
  }

  firstOf(kind) {
    return this.events.find((event) => event.kind === kind) || null;
  }
}

/** Kinds whose evidence link is `kind:payload` in `build_evidence_links`. */
const PAYLOAD_EVIDENCE_KINDS = new Set([
  'language', 'language_from', 'language_to', 'meaning', 'cache_hit', 'filter:user',
  'intent', 'legacy_intent', 'diagnostic_mode',
]);

/**
 * Mirrors rust/src/event_log.rs `build_evidence_links` for the events the
 * memory answers append: payload-carrying kinds keep their payload, the
 * `response` event is its own link, and every other kind is addressed by its
 * event id (the function's catch-all arm).
 */
export function buildEvidenceLinks(prompt, log, responseLink) {
  const links = [`prompt:${stableId('prompt', prompt)}`];
  for (const event of log.events) {
    if (event.kind === 'response') links.push(event.payload);
    else if (PAYLOAD_EVIDENCE_KINDS.has(event.kind)) links.push(`${event.kind}:${event.payload}`);
    else links.push(`${event.kind}:${event.id}`);
  }
  if (!links.includes(responseLink)) links.push(responseLink);
  return links;
}

/**
 * Mirrors rust/src/solver_handlers/mod.rs `finalize_simple`: close the log
 * with the intent, candidate, validation, response and trace events, then
 * project it into a `SymbolicAnswer`. (`enforce_questions` is not ported: the
 * memory reports it would see carry no question of their own.)
 */
export function finalizeSimple(prompt, log, intent, responseLink, body, confidence) {
  log.append('intent', intent);
  if (!log.firstOf('candidate')) log.append('candidate', intent);
  if (!log.firstOf('validation')) log.append('validation', 'accepted_without_extra_constraints');
  log.append('response', responseLink);
  if (!log.firstOf('trace:simplification')) log.append('trace:simplification', 'smallest_sufficient');
  log.append('trace', intent);
  return {
    intent,
    answer: body,
    confidence,
    evidence_links: buildEvidenceLinks(prompt, log, responseLink),
    thinking_steps: thinkingStepsFromEvents(log.events, body),
    links_notation: '',
  };
}

function byteLength(text) {
  return encoder.encode(text).length;
}

function present(value) {
  return value !== null && value !== undefined && String(value) !== '';
}

/** Mirrors rust/src/link_store.rs `canonical_memory_event`. */
function canonicalMemoryEvent(event) {
  const fields = new Map();
  const push = (key, value) => {
    if (present(value)) fields.set(key, String(value));
  };
  push('id', event.id);
  push('kind', event.kind);
  push('role', event.role);
  push('intent', event.intent);
  push('tool', event.tool);
  push('inputs', event.inputs);
  push('outputs', event.outputs);
  push('content', event.content);
  push('sentAt', event.sent_at);
  push('demoLabel', event.demo_label);
  push('conversationId', event.conversation_id);
  push('conversationTitle', event.conversation_title);
  (event.evidence || []).forEach((evidence, index) => {
    fields.set(`evidence_${String(index).padStart(4, '0')}`, String(evidence));
  });
  fields.set('accessCount', String(event.access_count || 0));
  fields.set('writeCount', String(Math.max(1, event.write_count || 0)));
  return [...fields.keys()]
    .sort()
    .map((key) => `${key}=${byteLength(fields.get(key))}:${fields.get(key)};`)
    .join('');
}

/** Mirrors rust/src/link_store.rs `memory_event_to_link_record`. */
export function memoryEventToLinkRecord(event, sequence) {
  const canonical = canonicalMemoryEvent(event);
  const sourceId = event.id ? String(event.id) : stableId('memory_event', `${sequence}:${canonical}`);
  const recordId = stableId('memory_event', `${sequence}:${sourceId}:${canonical}`);
  const subtype = [event.kind, event.role, event.intent].find((value) => value !== null && value !== undefined)
    ?? 'memory_event';
  const links = [];
  const doublet = (from, to) => links.push({ index: stableId('doublet', `${from}->${to}`), from, to });
  const field = (key, value) => {
    if (!present(value)) return;
    doublet(recordId, `field:${key}`);
    doublet(`field:${key}`, `value:${value}`);
  };
  doublet(recordId, 'Type');
  doublet('Type', 'MemoryEvent');
  doublet('MemoryEvent', 'SubType');
  doublet('SubType', subtype);
  doublet(subtype, 'Value');
  doublet(recordId, sourceId);
  doublet(recordId, `schema_version:${KNOWLEDGE_SCHEMA_VERSION}`);
  field('id', sourceId);
  field('kind', event.kind);
  field('role', event.role);
  field('intent', event.intent);
  field('tool', event.tool);
  field('inputs', event.inputs);
  field('outputs', event.outputs);
  field('content', event.content);
  field('sentAt', event.sent_at);
  field('demoLabel', event.demo_label);
  field('conversationId', event.conversation_id);
  field('conversationTitle', event.conversation_title);
  for (const evidence of event.evidence || []) field('evidence', evidence);
  if ((event.access_count || 0) > 0) field('accessCount', String(event.access_count));
  field('writeCount', String(Math.max(1, event.write_count || 0)));
  return { stable_id: recordId, source_id: sourceId, links };
}

/** Mirrors rust/src/link_store.rs `memory_events_to_link_records`. */
export function memoryEventsToLinkRecords(events) {
  return events.map((event, index) => memoryEventToLinkRecord(event, index));
}

/** Mirrors rust/src/protocol_memory.rs `memory_events_with_request_history`. */
export function memoryEventsWithRequestHistory(memoryEvents, history) {
  return [
    ...memoryEvents,
    ...history.map((turn, index) => ({
      id: `request-history-${index}`,
      kind: 'message',
      role: String(turn.role).toLowerCase(),
      content: String(turn.content ?? ''),
      conversation_id: REQUEST_HISTORY_CONVERSATION_ID,
      conversation_title: serverMessage('memory_request_history_title'),
      evidence: [],
      access_count: 0,
      write_count: 0,
    })),
  ];
}

/** Mirrors `clean_recall_term`. */
function cleanRecallTerm(raw) {
  const term = raw
    .trim()
    .replace(/^[\s`"':\-_.,?!()]+|[\s`"':\-_.,?!()]+$/gu, '')
    .split(/\s+/u)
    .filter(Boolean)
    .join(' ');
  return term || null;
}

/** Mirrors `term_from_form`. */
function termFromForm(form, normalized) {
  let raw;
  if (form.slot === 'prefix') {
    if (!normalized.startsWith(form.before)) return null;
    raw = normalized.slice(form.before.length);
  } else if (form.slot === 'suffix') {
    if (!normalized.endsWith(form.after)) return null;
    raw = normalized.slice(0, normalized.length - form.after.length);
  } else if (form.slot === 'circumfix') {
    if (!normalized.startsWith(form.before)) return null;
    const rest = normalized.slice(form.before.length);
    if (!rest.endsWith(form.after)) return null;
    raw = rest.slice(0, rest.length - form.after.length);
  } else {
    return null;
  }
  return cleanRecallTerm(raw);
}

/** Mirrors `recall_term_for_role`. */
function recallTermForRole(seed, role, normalized) {
  for (const form of seed.roleWordForms(role)) {
    const term = termFromForm(form, normalized);
    if (term) return term;
  }
  return null;
}

/** Mirrors `recognize_recall_query`. */
export function recognizeRecallQuery(seed, normalized) {
  const term = recallTermForRole(seed, ROLE_CONVERSATION_RECALL_QUERY, normalized);
  if (term) return { term, scope: 'conversation' };
  const other = recallTermForRole(seed, ROLE_CONVERSATION_RECALL_OTHER_QUERY, normalized);
  return other ? { term: other, scope: 'other_conversations' } : null;
}

/** Mirrors `memory_event_field_values`. */
function memoryEventFieldValues(event) {
  const fields = [];
  const push = (name, value) => {
    if (value !== null && value !== undefined && String(value).trim()) fields.push([name, String(value)]);
  };
  push('id', event.id);
  push('kind', event.kind);
  push('role', event.role);
  push('intent', event.intent);
  push('tool', event.tool);
  push('inputs', event.inputs);
  push('outputs', event.outputs);
  push('content', event.content);
  push('sentAt', event.sent_at);
  push('demoLabel', event.demo_label);
  push('conversationId', event.conversation_id);
  push('conversationTitle', event.conversation_title);
  for (const evidence of event.evidence || []) push('evidence', evidence);
  return fields;
}

/** Mirrors `event_in_recall_scope`. */
function eventInRecallScope(event, query, currentConversationId) {
  const conversationId = event.conversation_id ?? LEGACY_CONVERSATION;
  return query.scope !== 'other_conversations'
    || currentConversationId === null
    || currentConversationId !== conversationId;
}

/** Mirrors `memory_match`. */
function memoryMatch(index, event, detail) {
  const role = [event.role, event.kind, event.intent].find((value) => value !== null && value !== undefined)
    ?? 'event';
  return {
    eventIndex: index + 1,
    role: String(role).replace(/[A-Z]/g, (letter) => letter.toLowerCase()),
    conversationId: event.conversation_id ?? LEGACY_CONVERSATION,
    conversationTitle: event.conversation_title ?? '',
    sentAt: event.sent_at ?? '',
    detail,
  };
}

/** Mirrors `memory_recall_matches`. */
function memoryRecallMatches(seed, events, query, currentConversationId, triggerText) {
  const needle = seed.normalizePrompt(query.term);
  if (!needle) return [];
  const trigger = seed.normalizePrompt(triggerText);
  const matches = [];
  events.forEach((event, index) => {
    if (!eventInRecallScope(event, query, currentConversationId)) return;
    for (const [name, raw] of memoryEventFieldValues(event)) {
      const value = raw.trim();
      if (!value) continue;
      const haystack = seed.normalizePrompt(value);
      if (!haystack.includes(needle)) continue;
      if (name === 'content' && trigger && haystack === trigger) continue;
      matches.push(memoryMatch(index, event, { field: name, value }));
    }
  });
  memoryEventsToLinkRecords(events).forEach((record, index) => {
    const event = events[index];
    if (!eventInRecallScope(event, query, currentConversationId)) return;
    for (const link of record.links) {
      if (!seed.normalizePrompt(`${link.from} ${link.to}`).includes(needle)) continue;
      matches.push(memoryMatch(index, event, { from: link.from, to: link.to }));
    }
  });
  return matches;
}

/** Mirrors `MemoryRecallMatch::log_fragment`. */
function logFragment(matched) {
  const { detail } = matched;
  return 'field' in detail ? `field=${detail.field} value=${detail.value}` : `link=${detail.from}->${detail.to}`;
}

/** Mirrors `MemoryRecallMatch::render_line`. */
function renderLine(matched) {
  const stamp = matched.sentAt ? ` [${matched.sentAt}]` : '';
  const { detail } = matched;
  if (!('field' in detail)) return serverMessage('memory_recall_link_line', { stamp, from: detail.from, to: detail.to });
  const label = detail.field === 'content' ? matched.role : detail.field;
  return `${label}${stamp}: ${detail.value}`;
}

/** Conversation ids in first-seen order (`memory_conversation_count`). */
function conversationIds(matches) {
  const ids = [];
  for (const matched of matches) if (!ids.includes(matched.conversationId)) ids.push(matched.conversationId);
  return ids;
}

/** Mirrors `render_memory_recall_report`. */
function renderMemoryRecallReport(query, matches, language) {
  const slug = RECALL_LANGUAGES.has(language) ? language : 'en';
  if (!matches.length) return serverMessage(`memory_recall_none_${slug}`, { term: query.term });
  const ids = conversationIds(matches);
  const lines = [serverMessage(`memory_recall_found_${slug}`, {
    term: query.term, count: matches.length, conversations: ids.length,
  })];
  for (const conversationId of ids) {
    const titled = matches.find((matched) => matched.conversationId === conversationId && matched.conversationTitle);
    const title = titled ? titled.conversationTitle : '';
    const label = !title || title === conversationId ? conversationId : `${title} (${conversationId})`;
    lines.push(serverMessage('memory_recall_conversation', { label }));
    for (const matched of matches.filter((item) => item.conversationId === conversationId)) {
      lines.push(`  - ${renderLine(matched)}`);
    }
  }
  return lines.join('\n').trimEnd();
}

/**
 * Mirrors rust/src/solver_handlers/conversation_memory/mod.rs
 * `answer_memory_recall` / `try_memory_recall`.
 */
export function answerMemoryRecall(seed, prompt, events, currentConversationId) {
  const query = recognizeRecallQuery(seed, seed.normalizePrompt(prompt));
  if (!query) return null;
  const log = new EventLog();
  log.append('impulse', prompt);
  const matches = memoryRecallMatches(seed, events, query, currentConversationId, prompt);
  log.append('filter:memory_query', query.term);
  log.append('filter:memory_scope', query.scope);
  log.append('filter:memory_matches', String(matches.length));
  log.append('filter:memory_conversations', String(conversationIds(matches).length));
  for (const matched of matches) {
    log.append('memory_match', `event=${matched.eventIndex} conversation=${matched.conversationId} `
      + `title=${matched.conversationTitle} role=${matched.role} ${logFragment(matched)}`);
  }
  const body = renderMemoryRecallReport(query, matches, seed.detectLanguage(prompt));
  return finalizeSimple(prompt, log, 'conversation_recall', 'response:conversation_recall', body, 0.9);
}

/** Mirrors rust/src/seed.rs `render_response` (no language fallback). */
function renderResponse(seed, intent, language, values) {
  const template = seed.responseFor(intent, language);
  if (template === null) return null;
  return Object.entries(values).reduce((text, [name, value]) => text.split(`{${name}}`).join(String(value)), template);
}

/** Mirrors rust/src/seed.rs `localized_response` (language, `unknown`, then English). */
function localizedResponse(seed, intent, language) {
  return seed.responseFor(intent, language) ?? seed.responseFor(intent, 'unknown') ?? seed.responseFor(intent, 'en');
}

/** Mirrors rust/src/protocol_memory.rs `render_counts`. */
function renderCounts(seed, counts, language) {
  if (!counts.size) return localizedResponse(seed, 'memory_inventory_empty', language);
  const items = [];
  for (const name of [...counts.keys()].sort()) {
    const item = renderResponse(seed, 'memory_inventory_item', language, { name, count: counts.get(name) });
    if (item === null) return null;
    items.push(item);
  }
  return items.join(', ');
}

function inspectionBody(seed, normalized, history, memoryEvents, language) {
  const isRootQuery = (text) => seed.lexiconMentionsRole(ROLE_MEMORY_ROOT_LINKS_QUERY, text);
  const records = memoryEventsToLinkRecords(memoryEvents);
  if (seed.lexiconMentionsRole(ROLE_MEMORY_LINK_COUNT_QUERY, normalized)) {
    const links = records.reduce((total, record) => total + record.links.length, 0);
    return ['memory_link_count', renderResponse(seed, 'memory_link_count', language, { records: records.length, links })];
  }
  if (seed.lexiconMentionsRole(ROLE_MEMORY_INVENTORY_QUERY, normalized)) {
    const kinds = new Map();
    const conversations = new Map();
    for (const event of memoryEvents) {
      const kind = event.kind ?? 'memory_event';
      kinds.set(kind, (kinds.get(kind) || 0) + 1);
      if (event.conversation_id !== null && event.conversation_id !== undefined) {
        conversations.set(event.conversation_id, (conversations.get(event.conversation_id) || 0) + 1);
      }
    }
    const kindText = renderCounts(seed, kinds, language);
    const conversationText = renderCounts(seed, conversations, language);
    if (kindText === null || conversationText === null) return ['memory_inventory', null];
    return ['memory_inventory', renderResponse(seed, 'memory_inventory', language, {
      records: memoryEvents.length, kinds: kindText, conversations: conversationText,
    })];
  }
  const previousRootQuery = history.some((turn) =>
    String(turn.role).toLowerCase() === 'user' && isRootQuery(seed.normalizePrompt(turn.content)));
  const correction = seed.lexiconMentionsRole(ROLE_MEMORY_RETRIEVAL_CORRECTION, normalized) && previousRootQuery;
  if (!isRootQuery(normalized) && !correction) return null;
  const listing = records.length
    ? records.map((record) => `- ((${record.stable_id}: ${record.stable_id} ${record.source_id}))`).join('\n')
    : localizedResponse(seed, 'memory_root_links_empty', language);
  if (listing === null) return ['memory_root_links', null];
  return ['memory_root_links', renderResponse(seed, 'memory_root_links', language, { listing })];
}

/** Mirrors rust/src/protocol_memory.rs `answer_memory_inspection`. */
export function answerMemoryInspection(seed, prompt, history, memoryEvents) {
  const found = inspectionBody(seed, seed.normalizePrompt(prompt), history, memoryEvents, seed.detectLanguage(prompt));
  if (!found || found[1] === null) return null;
  const [intent, body] = found;
  const log = new EventLog();
  log.append('impulse', prompt);
  log.append('memory:inspect', intent);
  return finalizeSimple(prompt, log, intent, `response:${intent}`, body, 1);
}

/**
 * Mirrors rust/src/protocol_memory.rs `answer_from_memory_if_requested`.
 *
 * The exact query languages (`is_exact_memory_query` /
 * `execute_memory_query_with_options`, rust/src/memory_query_language.rs) are
 * not ported to the JavaScript server yet; such a prompt falls through to the
 * recall and then to the solver.
 * @param {object} seed the worker's seed readers (`WorkerHost.seedReaders`)
 * @param {string} prompt
 * @param {Array<{role: string, content: string}>} history
 * @param {Array<object>} memoryEvents the store's `MemoryEvent`s
 */
export function answerFromMemoryIfRequested(seed, prompt, history, memoryEvents) {
  const inspection = answerMemoryInspection(seed, prompt, history, memoryEvents);
  if (inspection) return inspection;
  if (!memoryEvents.length) return null;
  const events = memoryEventsWithRequestHistory(memoryEvents, history);
  return answerMemoryRecall(seed, prompt, events, REQUEST_HISTORY_CONVERSATION_ID);
}
