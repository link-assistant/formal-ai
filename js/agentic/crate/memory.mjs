// The portable `demo_memory` event log the agentic planner reads: the part of
// rust/src/memory.rs (`MemoryEvent`, `MemoryStore`, `parse_links_notation`,
// `export_links_notation_with_schema`, `format_event_into`, `escape_value`,
// `parse_quoted`, `split_first_token`) and rust/src/memory/upgrade.rs
// (`schema_version_for_loaded_document`, `inspect_memory_bytes`,
// `validate_persisted_memory_text`) reachable from
// `agentic_coding::algorithm_learning` and `agentic_coding::learning_report`.
// File access is the caller's: this module only parses and renders text.

import { rustLines } from '../content.mjs';
import { trimEnd, trimStart } from './rust_str.mjs';

/** Mirrors `ROOT_HEADER` in rust/src/memory.rs. */
export const ROOT_HEADER = 'demo_memory';
/** Mirrors `MINIMUM_READABLE_MEMORY_SCHEMA_VERSION` in rust/src/memory/upgrade.rs. */
export const MINIMUM_READABLE_MEMORY_SCHEMA_VERSION = 1;
/** Mirrors `MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION` in rust/src/memory/upgrade.rs. */
export const MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION = 2;
/** Mirrors `TARGET_MEMORY_SCHEMA_VERSION` in rust/src/memory/upgrade.rs. */
export const TARGET_MEMORY_SCHEMA_VERSION = 2;

/** Mirrors `MemoryEvent::default()` in rust/src/memory.rs (snake_case fields). */
export function memoryEvent(fields = {}) {
  return {
    id: '',
    kind: null,
    role: null,
    intent: null,
    tool: null,
    inputs: null,
    outputs: null,
    content: null,
    sent_at: null,
    demo_label: null,
    conversation_id: null,
    conversation_title: null,
    evidence: [],
    unknown_fields: [],
    access_count: 0,
    write_count: 0,
    ...fields,
  };
}

/** The persisted field spelling of each optional `MemoryEvent` field, in format order. */
const FIELD_KEYS = [
  ['kind', 'kind'],
  ['role', 'role'],
  ['intent', 'intent'],
  ['tool', 'tool'],
  ['inputs', 'inputs'],
  ['outputs', 'outputs'],
  ['content', 'content'],
  ['sentAt', 'sent_at'],
  ['demoLabel', 'demo_label'],
  ['conversationId', 'conversation_id'],
  ['conversationTitle', 'conversation_title'],
];
const FIELD_BY_KEY = new Map(FIELD_KEYS);

const leadingSpaces = (line) => {
  let count = 0;
  while (count < line.length && line[count] === ' ') count += 1;
  return count;
};

/** Rust `str::parse::<u64>()` -> number or null. */
function parseCount(value) {
  return /^\+?[0-9]+$/.test(value) ? Number(value.replace(/^\+/, '')) : null;
}

/** Mirrors `fn escape_value` in rust/src/memory.rs. @param {string} value */
export function escapeValue(value) {
  return String(value).split('\\').join('\\\\').split('"').join('\\"')
    .split('\n').join('\\n')
    .split('\r').join('\\r')
    .split('\t').join('\\t');
}

const UNESCAPES = { n: '\n', r: '\r', t: '\t', '\\': '\\', '"': '"' };

/** Mirrors `fn unescape_value` in rust/src/memory.rs. */
function unescapeValue(value) {
  let out = '';
  const chars = Array.from(value);
  for (let index = 0; index < chars.length; index += 1) {
    const character = chars[index];
    if (character !== '\\') {
      out += character;
      continue;
    }
    index += 1;
    if (index < chars.length) out += UNESCAPES[chars[index]] ?? chars[index];
  }
  return out;
}

/** Mirrors `fn parse_quoted` in rust/src/memory.rs: the first C-escaped quoted scalar, or null. */
export function parseQuoted(rest) {
  const trimmed = trimStart(rest);
  if (!trimmed.startsWith('"')) return null;
  let index = 1;
  while (index < trimmed.length) {
    if (trimmed[index] === '\\') index += 2;
    else if (trimmed[index] === '"') return unescapeValue(trimmed.slice(1, index));
    else index += 1;
  }
  return null;
}

/** Mirrors `fn split_first_token` in rust/src/memory.rs. */
function splitFirstToken(content) {
  const trimmed = trimStart(content);
  const space = trimmed.indexOf(' ');
  return space < 0 ? [trimmed, ''] : [trimmed.slice(0, space), trimmed.slice(space + 1)];
}

/**
 * Mirrors `fn parse_links_notation` in rust/src/memory.rs: a lenient
 * `demo_memory` reader (no header, no events).
 * @param {string} text
 * @returns {Array<object>} `MemoryEvent`s
 */
export function parseLinksNotation(text) {
  const events = [];
  let current = null;
  let sawHeader = false;
  for (const line of rustLines(String(text))) {
    if (trimEnd(line) === '') continue;
    const indent = leadingSpaces(line);
    const content = line.slice(indent);
    if (indent === 0) {
      if (content === ROOT_HEADER) sawHeader = true;
      continue;
    }
    if (!sawHeader) continue;
    if (indent === 2) {
      if (content.startsWith('event ')) {
        if (current) events.push(current);
        current = memoryEvent({ id: parseQuoted(content.slice('event '.length)) ?? '' });
      }
      continue;
    }
    if (indent !== 4 || !current) continue;
    const [key, rest] = splitFirstToken(content);
    const value = parseQuoted(rest);
    if (value === null) continue;
    if (FIELD_BY_KEY.has(key)) current[FIELD_BY_KEY.get(key)] = value;
    else if (key === 'accessCount') current.access_count = parseCount(value) ?? 0;
    else if (key === 'writeCount') current.write_count = Math.max(parseCount(value) ?? 1, 1);
    else if (key === 'evidence') current.evidence = value.split('|').filter((item) => item.length > 0);
    else current.unknown_fields.push([key, value]);
  }
  if (current) events.push(current);
  for (const event of events) if (event.write_count === 0) event.write_count = 1;
  return events;
}

/** Mirrors `fn format_event_into` in rust/src/memory.rs. */
function formatEvent(event) {
  let out = `  event "${escapeValue(event.id)}"\n`;
  for (const [key, field] of FIELD_KEYS) {
    const value = event[field];
    if (value === null || value === undefined || value === '') continue;
    out += `    ${key} "${escapeValue(value)}"\n`;
  }
  if (event.evidence.length > 0) out += `    evidence "${escapeValue(event.evidence.join('|'))}"\n`;
  for (const [key, value] of event.unknown_fields) {
    if (!key || !value) continue;
    out += `    ${key} "${escapeValue(value)}"\n`;
  }
  if (event.access_count > 0) out += `    accessCount "${event.access_count}"\n`;
  out += `    writeCount "${Math.max(event.write_count, 1)}"\n`;
  return out;
}

/**
 * Mirrors `fn export_links_notation_with_schema` in rust/src/memory.rs.
 * @param {Array<object>} events
 * @param {number} schemaVersion
 */
export function exportLinksNotationWithSchema(events, schemaVersion) {
  let out = `${ROOT_HEADER}\n`;
  if (schemaVersion >= 2) out += `  schema_version "${schemaVersion}"\n`;
  for (const event of events) out += formatEvent(event);
  return out;
}

/** Mirrors `fn parse_persisted_quoted` in rust/src/memory/upgrade.rs: one quoted scalar and nothing after it. */
function parsePersistedQuoted(value) {
  const trimmed = trimStart(value);
  if (!trimmed.startsWith('"')) return null;
  let index = 1;
  while (index < trimmed.length) {
    if (trimmed[index] === '\\') {
      index += 2;
    } else if (trimmed[index] === '"') {
      if (trimEnd(trimmed.slice(index + 1)) !== '') return null;
      return unescapeValue(trimmed.slice(1, index));
    } else {
      index += 1;
    }
  }
  return null;
}

/** Mirrors `fn validate_persisted_memory_text` in rust/src/memory/upgrade.rs (true when valid). */
function validPersistedMemory(text) {
  let insideEvent = false;
  let lineNumber = 0;
  for (const raw of rustLines(text)) {
    lineNumber += 1;
    const line = trimEnd(raw);
    if (!line) continue;
    const indent = leadingSpaces(line);
    const content = line.slice(indent);
    if (indent === 0 && lineNumber === 1 && content === ROOT_HEADER) continue;
    if (indent !== 2 && !(indent === 4 && insideEvent)) return false;
    if (indent === 2) insideEvent = false;
    const space = content.indexOf(' ');
    if (space <= 0 || parsePersistedQuoted(content.slice(space + 1)) === null) return false;
    if (indent === 2 && content.slice(0, space) === 'event') insideEvent = true;
  }
  return true;
}

/**
 * Mirrors `fn schema_version_for_loaded_document` in rust/src/memory/upgrade.rs
 * (via `inspect_memory_text`): the schema of a compatible document, or null.
 * @param {string} text
 * @returns {number|null}
 */
export function schemaVersionForLoadedDocument(text) {
  if (text.length === 0) return MINIMUM_READABLE_MEMORY_SCHEMA_VERSION;
  const lines = rustLines(text);
  if (lines.length === 0 || lines[0] !== ROOT_HEADER) return null;
  let detected = null;
  for (const line of lines) {
    const indent = leadingSpaces(line);
    if (indent !== 2) continue;
    const content = line.slice(indent);
    if (!content.startsWith('schema_version ')) continue;
    const value = parsePersistedQuoted(content.slice('schema_version '.length));
    const version = value === null ? null : parseCount(value);
    if (version === null || version === 0 || version > 0xffffffff) return null;
    if (detected !== null) return null;
    detected = version;
  }
  detected = detected ?? MINIMUM_READABLE_MEMORY_SCHEMA_VERSION;
  if (detected < MINIMUM_READABLE_MEMORY_SCHEMA_VERSION || detected > MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION) return null;
  if (!validPersistedMemory(text)) return null;
  return detected;
}

/** Mirrors `struct MemoryStore` in rust/src/memory.rs (the in-memory part). */
export class MemoryStore {
  /** Mirrors `MemoryStore::new`. */
  constructor() {
    this.events_ = [];
    this.schema_version = TARGET_MEMORY_SCHEMA_VERSION;
  }

  /** Mirrors `MemoryStore::events`. */
  events() {
    return this.events_;
  }

  /** Mirrors `MemoryStore::len`. */
  len() {
    return this.events_.length;
  }

  /** Mirrors `MemoryStore::is_empty`. */
  isEmpty() {
    return this.events_.length === 0;
  }

  /** Mirrors `MemoryStore::export_links_notation`. */
  exportLinksNotation() {
    return exportLinksNotationWithSchema(this.events_, this.schema_version);
  }

  /** Mirrors `MemoryStore::replace_from_links_notation`. @param {string} text */
  replaceFromLinksNotation(text) {
    this.schema_version = schemaVersionForLoadedDocument(text) ?? TARGET_MEMORY_SCHEMA_VERSION;
    this.events_ = parseLinksNotation(text);
  }
}
