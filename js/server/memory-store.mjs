// The portable `demo_memory` event log and its file-backed sync store:
// rust/src/memory.rs (`MemoryEvent`, `parse_links_notation`,
// `export_links_notation_with_schema`, `write_locked_atomic`),
// rust/src/memory/upgrade.rs (`inspect_memory_text`, `preflight_memory_upgrade`),
// rust/src/memory_sync.rs (`SyncStore`, `events_since`, `merge_union_by_id`),
// rust/src/shared_memory.rs (`shared_memory_path`, `ensure_shared_memory_file`)
// and rust/src/context_capacity.rs (`ContextCapacity::current`).
//
// A recorded user turn is linked to the newest anticipation prediction of its
// class (`prediction_hit_event`, js/server/anticipation.mjs). Omitted from the
// Rust store: the native link-cli projection `SyncStore::persist` / `open_at` keep beside the log
// (rust/src/link_store/projection_sync.rs `synchronize_memory_events`: the
// 64 MiB memory-mapped doublets file `<memory>.links`, its `.links.nodes`
// address map, `.links.projected` marker and `.transitions.links` log). That
// file is the link-cli crate's binary doublets store, whose bytes and node
// addresses come from its size-balanced-tree allocator; no server route reads
// it back, and `.lino` stays the recovery source the Rust store rebuilds the
// projection from on its next open, so a log this store writes is complete
// for both servers. The consent-gated auto-free-space pass on import is
// js/server/storage-policy.mjs.

import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

import { predictionHitEvent } from './anticipation.mjs';
import { stableId } from './ids.mjs';
import { serverMessage } from './messages.mjs';
import { LEARNED_CHUNK_KIND, importLearned, takeLearned } from './meta-learned.mjs';
import { packageVersion } from './seed.mjs';
import { applyAutoFreeSpaceForWrite } from './storage-policy.mjs';

export const ROOT_HEADER = 'demo_memory';
export const MINIMUM_READABLE_SCHEMA = 1;
export const MAXIMUM_READABLE_SCHEMA = 2;
export const TARGET_SCHEMA = 2;
export const V1_TO_V2_MIGRATION_ID = 'demo_memory_v1_to_v2';
const MEMORY_DIRECTORY_NAME = '.formal-ai';
const MEMORY_FILE_NAME = 'memory.lino';
const DEFAULT_AVG_UTF8_BYTES_PER_CHAR = 2;

/** `cli_env::parse_bool_env`. */
export function parseBoolEnv(value) {
  if (value === undefined || value === null) return null;
  const lower = String(value).trim().toLowerCase();
  if (['1', 'true', 'yes', 'on'].includes(lower)) return true;
  if (['0', 'false', 'no', 'off'].includes(lower)) return false;
  return null;
}

/** `shared_memory_path`: FORMAL_AI_MEMORY_PATH, else the platform default. */
export function sharedMemoryPath(env = process.env) {
  const configured = env.FORMAL_AI_MEMORY_PATH;
  if (configured && configured.trim()) return configured;
  if (process.platform === 'win32') {
    const base = env.APPDATA || env.HOME || '.';
    return path.join(base, 'formal-ai', MEMORY_FILE_NAME);
  }
  return path.join(env.HOME || '.', MEMORY_DIRECTORY_NAME, MEMORY_FILE_NAME);
}

/** `ensure_shared_memory_file`. */
export function ensureSharedMemoryFile(file) {
  const parent = path.dirname(file);
  if (parent && parent !== '.') {
    const existed = fs.existsSync(parent);
    fs.mkdirSync(parent, { recursive: true });
    if (!existed && process.platform !== 'win32') fs.chmodSync(parent, 0o700);
  }
  fs.closeSync(fs.openSync(file, 'a'));
}

/** A `MemoryEvent` with every optional field absent. */
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

/** Rust `str::lines`. */
export function rustLines(text) {
  const lines = text.split('\n');
  if (lines.length > 0 && lines[lines.length - 1] === '') lines.pop();
  return lines.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
}

function leadingSpaces(line) {
  let count = 0;
  while (count < line.length && line[count] === ' ') count += 1;
  return count;
}

/** `escape_value`. */
export function escapeValue(value) {
  return String(value)
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}

const UNESCAPES = { n: '\n', r: '\r', t: '\t', '\\': '\\', '"': '"' };

function unescapeValue(value) {
  let out = '';
  const chars = [...value];
  for (let index = 0; index < chars.length; index += 1) {
    const ch = chars[index];
    if (ch !== '\\') {
      out += ch;
      continue;
    }
    index += 1;
    if (index < chars.length) out += UNESCAPES[chars[index]] ?? chars[index];
  }
  return out;
}

/** `parse_quoted`: the first C-escaped double-quoted scalar, or null. */
export function parseQuoted(rest) {
  const trimmed = rest.replace(/^\s+/u, '');
  if (!trimmed.startsWith('"')) return null;
  let index = 1;
  while (index < trimmed.length) {
    if (trimmed[index] === '\\') index += 2;
    else if (trimmed[index] === '"') return unescapeValue(trimmed.slice(1, index));
    else index += 1;
  }
  return null;
}

/** Rust `value.parse::<u64>()`. */
function parseCount(value) {
  return /^\+?\d+$/.test(value) ? Number(value.replace(/^\+/, '')) : null;
}

const FIELD_KEYS = {
  kind: 'kind',
  role: 'role',
  intent: 'intent',
  tool: 'tool',
  inputs: 'inputs',
  outputs: 'outputs',
  content: 'content',
  sentAt: 'sent_at',
  demoLabel: 'demo_label',
  conversationId: 'conversation_id',
  conversationTitle: 'conversation_title',
};

/** `parse_links_notation`: a lenient `demo_memory` reader. */
export function parseLinksNotation(text) {
  const events = [];
  let current = null;
  let sawHeader = false;
  for (const line of rustLines(String(text))) {
    if (!line.trimEnd()) continue;
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
    const body = content.replace(/^\s+/u, '');
    const space = body.indexOf(' ');
    const key = space < 0 ? body : body.slice(0, space);
    const value = parseQuoted(space < 0 ? '' : body.slice(space + 1));
    if (value === null) continue;
    if (Object.prototype.hasOwnProperty.call(FIELD_KEYS, key)) current[FIELD_KEYS[key]] = value;
    else if (key === 'accessCount') current.access_count = parseCount(value) ?? 0;
    else if (key === 'writeCount') current.write_count = Math.max(parseCount(value) ?? 1, 1);
    else if (key === 'evidence') current.evidence = value.split('|').filter((item) => item.length > 0);
    else current.unknown_fields.push([key, value]);
  }
  if (current) events.push(current);
  for (const event of events) if (event.write_count === 0) event.write_count = 1;
  return events;
}

/** `format_event_into`. */
function formatEvent(event) {
  let out = `  event "${escapeValue(event.id)}"\n`;
  for (const [key, field] of Object.entries(FIELD_KEYS)) {
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

/** `export_links_notation_with_schema`. */
export function exportLinksNotation(events, schemaVersion) {
  let out = `${ROOT_HEADER}\n`;
  if (schemaVersion >= 2) out += `  schema_version "${schemaVersion}"\n`;
  for (const event of events) out += formatEvent(event);
  return out;
}

/** `parse_persisted_quoted`: one quoted scalar and nothing after it. */
function parsePersistedQuoted(value) {
  const trimmed = value.replace(/^\s+/u, '');
  if (!trimmed.startsWith('"')) return null;
  let index = 1;
  while (index < trimmed.length) {
    if (trimmed[index] === '\\') index += 2;
    else if (trimmed[index] === '"') {
      return trimmed.slice(index + 1).trim() ? null : parseQuoted(trimmed.slice(0, index + 1));
    } else index += 1;
  }
  return null;
}

/** `validate_persisted_memory_text`: `{count}` events, or the `line=N:reason` error. */
function validatePersistedMemoryText(text) {
  let count = 0;
  let insideEvent = false;
  let lineNumber = 0;
  for (const raw of rustLines(text)) {
    lineNumber += 1;
    const line = raw.trimEnd();
    if (!line) continue;
    const indent = leadingSpaces(line);
    const content = line.slice(indent);
    if (indent === 0 && lineNumber === 1 && content === ROOT_HEADER) continue;
    const scope = indent === 2 ? 'root' : 'event';
    if (indent !== 2 && !(indent === 4 && insideEvent)) return { error: `line=${lineNumber}:indent_or_parent_invalid` };
    if (indent === 2) insideEvent = false;
    const space = content.indexOf(' ');
    if (space < 0) return { error: `line=${lineNumber}:${scope}_value_missing` };
    if (space === 0 || parsePersistedQuoted(content.slice(space + 1)) === null) return { error: `line=${lineNumber}:${scope}_value_invalid` };
    if (indent === 2 && content.slice(0, space) === 'event') {
      count += 1;
      insideEvent = true;
    }
  }
  return { count };
}

/** `MemoryUpgradeStatus::base`: the full operator-contract status, in its JSON field order. */
function baseUpgradeStatus(pathExists) {
  return {
    binary_version: packageVersion(),
    path_exists: pathExists,
    detected_schema_version: null,
    minimum_readable_schema_version: MINIMUM_READABLE_SCHEMA,
    maximum_readable_schema_version: MAXIMUM_READABLE_SCHEMA,
    target_schema_version: TARGET_SCHEMA,
    compatible: true,
    migration_required: false,
    migration_id: null,
    rollback_supported: true,
    migration_state: pathExists ? 'ready' : 'missing',
    event_count: 0,
    source_sha256: null,
  };
}

/** `MemoryUpgradeStatus::incompatible` (with the source digest when bytes were read). */
export function incompatibleUpgradeStatus(pathExists, detected, code, reason, bytes = null) {
  return {
    ...baseUpgradeStatus(pathExists),
    detected_schema_version: detected,
    compatible: false,
    rollback_supported: false,
    migration_state: 'incompatible',
    event_count: null,
    source_sha256: bytes === null ? null : sha256Hex(bytes),
    refusal_code: code,
    refusal_reason: reason,
  };
}

const sha256Hex = (bytes) => createHash('sha256').update(bytes).digest('hex');

/** `inspect_memory_bytes` in rust/src/memory/upgrade.rs: the full, side-effect-free status. */
export function inspectMemoryUpgrade(bytes, pathExists) {
  const status = { ...baseUpgradeStatus(pathExists), source_sha256: sha256Hex(bytes) };
  const refuse = (detected, code, reason) => incompatibleUpgradeStatus(pathExists, detected, code, reason, bytes);
  if (bytes.length === 0) {
    // Released binaries create the shared file before the first event: a valid empty schema-1 store.
    status.detected_schema_version = MINIMUM_READABLE_SCHEMA;
    status.migration_required = TARGET_SCHEMA > 1;
    status.migration_id = status.migration_required ? V1_TO_V2_MIGRATION_ID : null;
    status.migration_state = status.migration_required ? 'upgrade_required' : 'ready';
    return status;
  }
  let text;
  try {
    text = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bytes);
  } catch {
    return refuse(null, 'memory_not_utf8', serverMessage('memory_upgrade_not_utf8'));
  }
  const lines = rustLines(text);
  if (lines.length === 0) return refuse(null, 'memory_empty', serverMessage('memory_upgrade_empty'));
  if (lines[0] !== ROOT_HEADER) return refuse(null, 'memory_header_unknown', `memory_header_unknown:expected=${ROOT_HEADER}`);
  let detected = null;
  for (const line of lines) {
    const indent = leadingSpaces(line);
    if (indent !== 2) continue;
    const content = line.slice(indent);
    if (!content.startsWith('schema_version ')) continue;
    const value = parsePersistedQuoted(content.slice('schema_version '.length));
    const version = value === null ? null : parseCount(value);
    if (version === null || version > 0xffffffff) {
      return refuse(null, 'schema_version_invalid', serverMessage('memory_upgrade_schema_version_invalid'));
    }
    if (detected !== null) return refuse(version, 'schema_version_duplicate', serverMessage('memory_upgrade_schema_version_duplicate'));
    detected = version;
  }
  detected = detected ?? MINIMUM_READABLE_SCHEMA;
  status.detected_schema_version = detected;
  if (detected < MINIMUM_READABLE_SCHEMA) {
    return refuse(detected, 'schema_too_old', `schema_too_old:detected=${detected}:minimum=${MINIMUM_READABLE_SCHEMA}`);
  }
  if (detected > MAXIMUM_READABLE_SCHEMA) {
    return refuse(detected, 'schema_too_new', `schema_too_new:detected=${detected}:maximum=${MAXIMUM_READABLE_SCHEMA}`);
  }
  const validated = validatePersistedMemoryText(text);
  if (validated.error !== undefined) return refuse(detected, 'memory_malformed', `memory_malformed:error=${validated.error}`);
  status.event_count = validated.count;
  if (detected < TARGET_SCHEMA) {
    status.migration_required = true;
    status.migration_id = V1_TO_V2_MIGRATION_ID;
    status.migration_state = 'upgrade_required';
  }
  return status;
}

/** `preflight_memory_upgrade` in rust/src/memory/upgrade.rs: the full status, creating nothing. */
export function preflightMemoryUpgrade(file) {
  let bytes;
  try {
    bytes = fs.readFileSync(file);
  } catch (error) {
    if (error.code === 'ENOENT') return baseUpgradeStatus(false);
    return incompatibleUpgradeStatus(true, null, 'memory_unreadable', `memory_unreadable:path=${file}:error=${error.message}`);
  }
  return inspectMemoryUpgrade(bytes, true);
}

/** The schema facts `/health` and `SyncStore::open_at` read, from the full status. */
const schemaFacts = (status) => ({
  detected_schema_version: status.detected_schema_version,
  compatible: status.compatible,
  migration_required: status.migration_required,
  migration_state: status.migration_state,
});

/** `inspect_memory_bytes`, as the schema facts. */
export function inspectMemoryBytes(bytes, pathExists) {
  return schemaFacts(inspectMemoryUpgrade(bytes, pathExists));
}

/** `preflight_memory_upgrade`, as the schema facts. */
export function preflightMemory(file) {
  return schemaFacts(preflightMemoryUpgrade(file));
}

/** `events_since`. */
export function eventsSince(events, lastSeen) {
  if (!lastSeen) return events.slice();
  const index = events.findIndex((event) => event.id === lastSeen);
  return index < 0 ? events.slice() : events.slice(index + 1);
}

const PAYLOAD_FIELDS = Object.values(FIELD_KEYS);

/** `merge_event`: incoming non-empty fields win; counts merge monotonically. */
export function mergeEvent(base, incoming) {
  const pick = (field) => (incoming[field] ? incoming[field] : base[field]);
  const unknown = base.unknown_fields.map(([key, value]) => [key, value]);
  let unknownChanged = false;
  for (const [key, value] of incoming.unknown_fields) {
    const existing = unknown.find(([existingKey]) => existingKey === key);
    if (existing) {
      if (existing[1] !== value) {
        existing[1] = value;
        unknownChanged = true;
      }
    } else {
      unknown.push([key, value]);
      unknownChanged = true;
    }
  }
  const sameEvidence = base.evidence.length === incoming.evidence.length
    && base.evidence.every((item, index) => item === incoming.evidence[index]);
  const payloadChanged = PAYLOAD_FIELDS.some((field) => incoming[field] && base[field] !== incoming[field])
    || (incoming.evidence.length > 0 && !sameEvidence)
    || unknownChanged;
  const observed = Math.max(base.write_count, 1, incoming.write_count, 1);
  const merged = memoryEvent({ id: base.id });
  for (const field of PAYLOAD_FIELDS) merged[field] = pick(field);
  merged.evidence = incoming.evidence.length === 0 ? base.evidence.slice() : incoming.evidence.slice();
  merged.unknown_fields = unknown;
  merged.access_count = Math.max(base.access_count, incoming.access_count);
  merged.write_count = payloadChanged && incoming.write_count <= base.write_count ? observed + 1 : observed;
  return merged;
}

/** `merge_union_by_id`. */
export function mergeUnionById(base, incoming) {
  const merged = base.slice();
  for (const event of incoming) {
    const index = merged.findIndex((existing) => existing.id === event.id);
    if (index >= 0) merged[index] = mergeEvent(merged[index], event);
    else merged.push(event);
  }
  return merged;
}

/** `write_locked_atomic` (temp file + rename; the advisory lock file is created, not held). */
export function writeAtomic(file, contents) {
  const parent = path.dirname(file);
  if (parent) fs.mkdirSync(parent, { recursive: true });
  const name = path.basename(file) || 'memory.lino';
  fs.closeSync(fs.openSync(path.join(parent, `${name}.lock`), 'a'));
  const temp = path.join(parent, `${name}.tmp.${process.pid}`);
  try {
    fs.writeFileSync(temp, contents);
    fs.renameSync(temp, file);
  } catch (error) {
    try {
      fs.unlinkSync(temp);
    } catch {
      // the temp file may never have been created
    }
    throw error;
  }
}

/** `memory_sync::chat_recording_enabled`. */
export function chatRecordingEnabled(env = process.env) {
  return parseBoolEnv(env.FORMAL_AI_RECORD_CHAT) !== false;
}

function memoryDebug(env, key, params) {
  if (parseBoolEnv(env.FORMAL_AI_MEMORY_DEBUG) === true) {
    process.stderr.write(`${serverMessage(key, params)}\n`);
  }
}

/** `SyncStore`: re-read from disk on every open, like the stateless Rust server. */
export class SyncStore {
  constructor(file, events, schemaVersion, compatible, env) {
    this.path = file;
    this.events = events;
    this.schemaVersion = schemaVersion;
    this.compatible = compatible;
    this.env = env;
  }

  /** `SyncStore::open` / `open_at`. */
  static open(env = process.env, file = sharedMemoryPath(env)) {
    let text;
    try {
      text = fs.readFileSync(file);
    } catch (error) {
      if (error.code === 'ENOENT') {
        let compatible = true;
        try {
          ensureSharedMemoryFile(file);
        } catch (initError) {
          memoryDebug(env, 'memory_debug_init_failed', { path: file, error: initError.message });
          compatible = false;
        }
        return new SyncStore(file, [], TARGET_SCHEMA, compatible, env);
      }
      memoryDebug(env, 'memory_debug_read_failed', { path: file, error: error.message });
      return new SyncStore(file, [], TARGET_SCHEMA, false, env);
    }
    let decoded;
    try {
      decoded = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(text);
    } catch (error) {
      memoryDebug(env, 'memory_debug_read_failed', { path: file, error: error.message });
      return new SyncStore(file, [], TARGET_SCHEMA, false, env);
    }
    const status = inspectMemoryBytes(text, true);
    const events = parseLinksNotation(decoded);
    // R1012: chunks learned in earlier sessions return to the meta reasoner.
    importLearned(events);
    return new SyncStore(file, events, status.detected_schema_version ?? TARGET_SCHEMA, status.compatible, env);
  }

  toLinksNotation() {
    return exportLinksNotation(this.events, this.schemaVersion);
  }

  deltaLinksNotation(lastSeen) {
    return exportLinksNotation(eventsSince(this.events, lastSeen), this.schemaVersion);
  }

  /** `import_links_notation`: merge by id, persist, return the number added. */
  importLinksNotation(text) {
    const before = this.events.length;
    this.events = mergeUnionById(this.events, parseLinksNotation(text));
    for (const event of this.events) if (event.write_count === 0) event.write_count = 1;
    const added = this.events.length - before;
    if (this.path) {
      const freed = applyAutoFreeSpaceForWrite(this.events, this.path, Buffer.byteLength(String(text), 'utf8'));
      if (freed) this.events = freed.events;
    }
    this.persist();
    return added;
  }

  /** `record_chat_exchange_with_tools`. */
  recordChatExchangeWithTools(prompt, answer, tools = []) {
    if (!this.path || !chatRecordingEnabled(this.env)) return 0;
    const seed = `${prompt}\u0000${answer}`;
    const userId = stableId('chat_user', seed);
    const recorded = [memoryEvent({ id: userId, kind: 'message', role: 'user', content: prompt, write_count: 1 })];
    const hit = predictionHitEvent(this.events, prompt, userId, this.classify);
    if (hit) recorded.push(hit);
    const evidence = [userId];
    for (const execution of tools) {
      const id = stableId('chat_tool', `${prompt}\u0000${execution.tool}\u0000${execution.inputs}\u0000${execution.outputs}`);
      evidence.push(id);
      recorded.push(memoryEvent({
        id,
        kind: 'tool_call',
        role: 'assistant',
        intent: 'execute_tool',
        tool: execution.tool,
        inputs: execution.inputs,
        outputs: execution.outputs,
        content: `tool:${execution.tool}`,
        evidence: [userId],
        write_count: 1,
      }));
    }
    const learned = takeLearned();
    if (learned !== null) {
      recorded.push(memoryEvent({ id: stableId(LEARNED_CHUNK_KIND, learned), kind: LEARNED_CHUNK_KIND, role: 'assistant', content: learned, write_count: 1 }));
    }
    recorded.push(memoryEvent({
      id: stableId('chat_task', seed),
      kind: 'task',
      role: 'assistant',
      intent: 'solve',
      inputs: prompt,
      outputs: answer,
      evidence,
      write_count: 1,
    }));
    const before = this.events.length;
    this.events = mergeUnionById(this.events, recorded);
    const added = this.events.length - before;
    if (added > 0) this.persist();
    return added;
  }

  persist() {
    if (!this.path) return;
    if (!this.compatible) throw new Error(serverMessage('memory_schema_write_refused'));
    writeAtomic(this.path, this.toLinksNotation());
  }
}

function avgUtf8BytesPerChar(env) {
  const raw = env.FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR;
  const parsed = raw === undefined ? null : parseCount(raw);
  return parsed && parsed > 0 ? parsed : DEFAULT_AVG_UTF8_BYTES_PER_CHAR;
}

/** `storage_policy::existing_ancestor`. */
function existingAncestor(file) {
  let candidate;
  try {
    candidate = fs.statSync(file).isDirectory() ? file : path.dirname(file) || '.';
  } catch {
    candidate = path.dirname(file) || '.';
  }
  while (!fs.existsSync(candidate)) {
    const parent = path.dirname(candidate);
    if (parent === candidate) return '.';
    candidate = parent;
  }
  return candidate;
}

function isMemoryLog(file) {
  return path.extname(file) === '.lino' || path.basename(file).includes('event-log');
}

function directoryBytes(dir) {
  let total = 0;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.isSymbolicLink()) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) total += directoryBytes(full);
    else if (entry.isFile() && isMemoryLog(full)) total += fs.statSync(full).size;
  }
  return total;
}

function memoryUsedBytes(file) {
  let stat;
  try {
    stat = fs.lstatSync(file);
  } catch (error) {
    if (error.code === 'ENOENT') return 0;
    throw error;
  }
  if (stat.isSymbolicLink()) return 0;
  if (stat.isFile()) return stat.size;
  if (!stat.isDirectory()) return 0;
  return directoryBytes(file);
}

/** `ContextCapacity::current` (throws on a filesystem error, like `io::Result`). */
export function currentContextCapacity(env = process.env) {
  const file = sharedMemoryPath(env);
  const stats = fs.statfsSync(existingAncestor(file), { bigint: true });
  const free = Number(stats.bavail * stats.bsize);
  const used = memoryUsedBytes(file);
  const average = Math.max(avgUtf8BytesPerChar(env), 1);
  const windowTokens = Math.floor(free / average);
  const usedTokens = Math.floor(used / average);
  return {
    context_window_tokens: windowTokens,
    context_used_tokens: usedTokens,
    context_used_fraction: windowTokens === 0 ? 0 : usedTokens / windowTokens,
    disk_free_bytes: free,
    memory_used_bytes: used,
    avg_utf8_bytes_per_char: average,
  };
}
