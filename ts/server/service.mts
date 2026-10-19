// The service routes that answer without the solver: the reachability probes,
// `/api/hello`, `/health`, `/v1/models` and `/v1/bundle`
// (rust/src/server.rs `dispatch_api_request_with_auth`).

import { f64, sortedKeys } from './json.mjs';
import { contextCapacity, memoryHealthStatus } from './memory.mjs';
import { serverMessage } from './messages.mjs';
import { errorResponse, jsonResponse, linksNotationResponse, rawResponse, JSON_TYPE } from './response.mjs';
import { canonicalModelId, mergedBundle, packageVersion } from './seed.mjs';

/** `server::ADVERTISED_MAX_OUTPUT_TOKENS`. */
export const ADVERTISED_MAX_OUTPUT_TOKENS = 8192;

/** `OPTIONS *` and `HEAD` probes: an empty body. */
export function emptyResponse(statusCode) {
  return rawResponse(statusCode, JSON_TYPE, '');
}

/** `GET /api/hello`: Anthropic's connection warm-up. */
export function handleHello() {
  return jsonResponse(200, { message: serverMessage('hello') });
}

/** `GET /health`. */
export function handleHealth() {
  return jsonResponse(200, sortedKeys({
    memory: memoryHealthStatus(),
    model: canonicalModelId(),
    status: 'ok',
    version: packageVersion(),
  }));
}

function capacityJson(capacity) {
  return {
    avg_utf8_bytes_per_char: capacity.avg_utf8_bytes_per_char,
    context_used_fraction: f64(capacity.context_used_fraction),
    context_used_tokens: capacity.context_used_tokens,
    context_window_tokens: capacity.context_window_tokens,
    disk_free_bytes: capacity.disk_free_bytes,
    memory_used_bytes: capacity.memory_used_bytes,
  };
}

/** `GET /v1/models` (`handle_openai_models_request`; a `json!` value, keys sorted). */
export function handleModels() {
  let capacity;
  try {
    capacity = contextCapacity();
  } catch (error) {
    return errorResponse(500, String(error.message || error));
  }
  const id = canonicalModelId();
  const context = capacityJson(capacity);
  const shared = {
    avg_utf8_bytes_per_char: capacity.avg_utf8_bytes_per_char,
    context,
    context_used_fraction: f64(capacity.context_used_fraction),
    context_used_tokens: capacity.context_used_tokens,
    context_window: capacity.context_window_tokens,
    context_window_tokens: capacity.context_window_tokens,
    disk_free_bytes: capacity.disk_free_bytes,
  };
  return jsonResponse(200, sortedKeys({
    data: [{
      ...shared,
      id,
      memory_used_bytes: capacity.memory_used_bytes,
      object: 'model',
      owned_by: 'link-assistant',
      slug: id,
    }],
    models: [{
      ...shared,
      id,
      max_output_tokens: ADVERTISED_MAX_OUTPUT_TOKENS,
      memory_used_bytes: capacity.memory_used_bytes,
      name: id,
      slug: id,
    }],
    object: 'list',
    rate_limit: { requests_per_minute: 60, tokens_per_minute: 60000 },
  }));
}

/** `GET /v1/bundle`. */
export function handleBundle() {
  return linksNotationResponse(200, mergedBundle());
}
