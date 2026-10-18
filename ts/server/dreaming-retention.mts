// Eviction eligibility (origin plus verified reconstruction) and the usage
// counts the planner ranks by: rust/src/dreaming/retention.rs
// (`reconstruction_record`, `reclaimable_bytes`, `classify_event`) and the
// part of rust/src/associative_persistence.rs the planner's `usage_counts`
// reads (`AssociativeMemory::from_memory_events`, `retention_score`).

import { stableId } from './ids.mjs';
import { bundledSeedFiles } from './seed.mjs';
import { readRepoFile } from './lino.mjs';
import { memoryEvent } from './memory-store.mjs';
import {
  containsAny, debugOption, debugStr, estimateEventBytes, lexicon, lowerOpt,
} from './dreaming-support.mjs';

export const DURABILITY = Object.freeze({
  IrreplaceableRaw: 'IrreplaceableRaw',
  RetainedLearning: 'RetainedLearning',
  DeletedConversation: 'DeletedConversation',
  RecomputableCache: 'RecomputableCache',
  RecomputableIntermediate: 'RecomputableIntermediate',
});

/** Mirrors rust/src/dreaming.rs `DreamingDurability::is_reclaimable`. */
export function isReclaimable(durability) {
  return durability === DURABILITY.DeletedConversation
    || durability === DURABILITY.RecomputableCache
    || durability === DURABILITY.RecomputableIntermediate;
}

/** Mirrors rust/src/dreaming.rs `DreamingDurability::pressure_priority`. */
export function pressurePriority(durability) {
  if (durability === DURABILITY.DeletedConversation) return 0;
  if (durability === DURABILITY.RecomputableCache) return 1;
  if (durability === DURABILITY.RecomputableIntermediate) return 2;
  return 9;
}

/** Rust `format!("{event:?}")` of a `MemoryEvent` (the derived `Debug`). */
export function memoryEventDebug(event) {
  const option = (field) => `${field}: ${debugOption(event[field])}`;
  const fields = [
    `id: ${debugStr(event.id)}`,
    ...['kind', 'role', 'intent', 'tool', 'inputs', 'outputs', 'content', 'sent_at',
      'demo_label', 'conversation_id', 'conversation_title'].map(option),
    `evidence: [${event.evidence.map(debugStr).join(', ')}]`,
    `unknown_fields: [${event.unknown_fields.map(([key, value]) => `(${debugStr(key)}, ${debugStr(value)})`).join(', ')}]`,
    `access_count: ${event.access_count}`,
    `write_count: ${event.write_count}`,
  ];
  return `MemoryEvent { ${fields.join(', ')} }`;
}

/** Mirrors rust/src/dreaming/retention.rs `reconstruction_record`. */
export function reconstructionRecord(event) {
  if (event.role !== 'cache') return null;
  const record = memoryEvent({
    kind: 'cache_reconstruction',
    role: 'system',
    inputs: event.id,
    tool: event.tool ?? null,
    conversation_id: event.conversation_id ?? null,
    evidence: event.evidence.slice(),
    unknown_fields: event.unknown_fields.map(([key, value]) => [key, value]),
  });
  record.id = stableId('cache_reconstruction', memoryEventDebug(record));
  return record;
}

/** Mirrors rust/src/dreaming/retention.rs `reclaimable_bytes`. */
export function reclaimableBytes(event, durability) {
  let retained = 0;
  if (durability === DURABILITY.RecomputableCache) {
    const record = reconstructionRecord(event);
    retained = record ? estimateEventBytes(record) : 0;
  }
  return Math.max(estimateEventBytes(event) - retained, 0);
}

let seedFilesCache = null;

/** `crate::seed::seed_files()`: the bundled seed files as `(path, contents)`. */
function seedFiles() {
  if (!seedFilesCache) seedFilesCache = bundledSeedFiles().map((file) => [file, readRepoFile(file)]);
  return seedFilesCache;
}

const ORIGINAL_ROLES = ['user', 'assistant', 'tool'];

function rediscoverable(evidence) {
  const prefix = 'rediscover:https://';
  if (!evidence.startsWith(prefix)) return false;
  const location = evidence.slice(prefix.length);
  return location.length > 0 && !/\p{White_Space}/u.test(location) && !location.startsWith('/') && !location.includes('@');
}

/** Mirrors rust/src/dreaming/retention.rs `classify_event`. */
export function classifyEvent(event, deletedConversations) {
  if (event.conversation_id !== null && event.conversation_id !== undefined
    && deletedConversations.has(event.conversation_id)) {
    return DURABILITY.DeletedConversation;
  }
  const cues = lexicon();
  if (event.role !== null && event.role !== undefined
    && ORIGINAL_ROLES.some((original) => lowerOpt(event.role) === original)) {
    return DURABILITY.IrreplaceableRaw;
  }
  const kind = lowerOpt(event.kind);
  const tool = lowerOpt(event.tool);
  const content = lowerOpt(event.content);
  const evidence = event.evidence.join('\n').toLowerCase();
  if (containsAny(kind, cues.learning_kind_cues)
    || containsAny(content, cues.learning_content_cues)
    || evidence.includes('learning')) {
    return DURABILITY.RetainedLearning;
  }
  const reconstructable = event.role === 'cache'
    && (event.evidence.some(rediscoverable)
      || (event.evidence.some((entry) => entry === 'reconstruct:embedded-seed')
        && seedFiles().some(([file, contents]) => event.tool === file && event.content === contents)));
  if (reconstructable && (containsAny(kind, cues.cache_kind_cues) || containsAny(tool, cues.cache_tool_cues))) {
    return DURABILITY.RecomputableCache;
  }
  return DURABILITY.IrreplaceableRaw;
}

/** Mirrors rust/src/associative_persistence.rs `event_expression_text`. */
function eventExpressionText(event) {
  const parts = [];
  for (const field of ['kind', 'role', 'intent', 'tool', 'inputs', 'outputs', 'content', 'conversation_title', 'demo_label']) {
    if (event[field] !== null && event[field] !== undefined) parts.push(String(event[field]));
  }
  parts.push(...event.evidence);
  return parts.join('\n');
}

/** `reference == id || reference.strip_suffix(id).ends_with(':')`. */
function referenceNames(reference, id) {
  return reference === id || (reference.endsWith(id) && reference.slice(0, reference.length - id.length).endsWith(':'));
}

/**
 * Mirrors rust/src/dreaming.rs `usage_counts`: per event, the uniform
 * `AssociativeMemory::retention_score` over
 * `AssociativeMemory::from_memory_events` — reads + writes (of the last event
 * carrying the id) + distinct incoming + distinct outgoing associations.
 */
export function usageCounts(events) {
  const expressions = new Map();
  for (const event of events) {
    if (!event.id) continue;
    expressions.set(event.id, { reads: event.access_count, writes: Math.max(event.write_count, 1) });
  }
  const outgoing = new Map();
  const incoming = new Map();
  const identified = events.filter((event) => event.id);
  for (const source of identified) {
    const searchable = eventExpressionText(source);
    for (const target of identified) {
      if (source.id === target.id) continue;
      const explicit = source.evidence.some((reference) => referenceNames(reference, target.id));
      if (!explicit && !searchable.includes(target.id)) continue;
      if (!outgoing.has(source.id)) outgoing.set(source.id, new Set());
      if (outgoing.get(source.id).has(target.id)) continue;
      outgoing.get(source.id).add(target.id);
      incoming.set(target.id, (incoming.get(target.id) ?? 0) + 1);
    }
  }
  const degree = (id) => (outgoing.get(id)?.size ?? 0) + (incoming.get(id) ?? 0);
  return events.map((event) => {
    const expression = expressions.get(event.id);
    if (!expression) return 0;
    return expression.reads + expression.writes + degree(event.id);
  });
}
