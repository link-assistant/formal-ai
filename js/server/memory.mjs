// The shared-memory sync routes and the live chat recorder
// (rust/src/server.rs `memory_health_status`, `handle_memory_since_request`,
// `handle_memory_import_request`, `query_param`, `record_exchange_best_effort`,
// rust/src/protocol/recording.rs `messages_exchange_to_record`).
//
// Every request re-opens the store from disk, as the stateless Rust server
// does, so several processes share one log (`FORMAL_AI_MEMORY_PATH`).

import { chatPromptAndHistory, chatToolExecutions } from './chat-request.mjs';
import { sortedKeys } from './json.mjs';
import {
  MAXIMUM_READABLE_SCHEMA,
  MINIMUM_READABLE_SCHEMA,
  SyncStore,
  TARGET_SCHEMA,
  currentContextCapacity,
  preflightMemory,
  sharedMemoryPath,
} from './memory-store.mjs';
import { serverMessage } from './messages.mjs';
import { jsonResponse, linksNotationResponse, messageError } from './response.mjs';

const envOf = (ctx) => ctx?.env || process.env;

/** `query_param`: the raw value of the first `key=value` pair. */
export function queryParam(query, key) {
  for (const pair of String(query || '').split('&')) {
    if (!pair) continue;
    const at = pair.indexOf('=');
    if (at < 0) continue;
    if (pair.slice(0, at) === key) return pair.slice(at + 1);
  }
  return null;
}

/** The shared memory file the server reads and writes. */
export function memoryPath(env = process.env) {
  return sharedMemoryPath(env);
}

/** `memory_health_status` (a `json!` value: keys sorted). */
export function memoryHealthStatus(env = process.env) {
  const status = preflightMemory(sharedMemoryPath(env));
  return sortedKeys({
    schema_version: status.detected_schema_version,
    minimum_readable_schema_version: MINIMUM_READABLE_SCHEMA,
    maximum_readable_schema_version: MAXIMUM_READABLE_SCHEMA,
    target_schema_version: TARGET_SCHEMA,
    compatible: status.compatible,
    migration_required: status.migration_required,
    migration_state: status.migration_state,
  });
}

/** `ContextCapacity::current`. */
export function contextCapacity(env = process.env) {
  return currentContextCapacity(env);
}

/** Log a recording failure to stderr; recording never fails a response. */
function reportRecordFailure(error) {
  process.stderr.write(`${serverMessage('memory_record_failed', { error: error?.message || error })}\n`);
}

/**
 * The per-server memory facade: `events()` reads the shared log,
 * `recordExchange` mirrors `record_exchange_best_effort`.
 */
export function createMemory(env = process.env) {
  return {
    events() {
      try {
        return SyncStore.open(env).events;
      } catch (error) {
        reportRecordFailure(error);
        return [];
      }
    },
    recordExchange(chatMessages, answerText, store = null) {
      try {
        const { prompt } = chatPromptAndHistory(chatMessages || []);
        const answer = String(answerText ?? '');
        if (!prompt.trim() || !answer.trim()) return 0;
        const target = store || SyncStore.open(env);
        return target.recordChatExchangeWithTools(prompt, answer, chatToolExecutions(chatMessages || []));
      } catch (error) {
        reportRecordFailure(error);
        return 0;
      }
    },
  };
}

/** `GET /v1/memory`. */
export function handleMemory(ctx) {
  return linksNotationResponse(200, SyncStore.open(envOf(ctx)).toLinksNotation());
}

/** `GET /v1/memory/since?event=<id>`. */
export function handleMemorySince(ctx, request) {
  const lastSeen = queryParam(request.query, 'event');
  return linksNotationResponse(200, SyncStore.open(envOf(ctx)).deltaLinksNotation(lastSeen));
}

/** `POST /v1/memory/import`. */
export function handleMemoryImport(ctx, request) {
  const store = SyncStore.open(envOf(ctx));
  try {
    const added = store.importLinksNotation(request.body ?? '');
    return jsonResponse(200, sortedKeys({ object: 'memory.import', added, total: store.events.length }));
  } catch (error) {
    return messageError(500, 'memory_persist_failed', { error: error?.message || error });
  }
}
