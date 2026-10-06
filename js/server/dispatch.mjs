// Request dispatch (rust/src/server.rs `handle_api_request_with_auth` and
// `dispatch_api_request_with_auth`): the MCP origin check, the bearer gate,
// then the route the manifest names.

import { handleAnthropicMessages } from './anthropic.mjs';
import { handleChatCompletions } from './openai.mjs';
import { handleConversationContext, handleConversationLearn } from './conversations.mjs';
import { recordApiExchangeIfEnabled } from './dialog-log.mjs';
import { handleGeminiGenerateContent, handleGeminiModel, handleGeminiModels, handleVertexModels } from './gemini.mjs';
import { handleMcp } from './mcp.mjs';
import { handleMemory, handleMemoryImport, handleMemorySince } from './memory.mjs';
import { handleLinks, handleLinksQuery, handleNetwork } from './network.mjs';
import { deprecatedAlias, errorResponse, messageError } from './response.mjs';
import { handleResponses } from './responses.mjs';
import { matchRoute } from './routes.mjs';
import { emptyResponse, handleBundle, handleHealth, handleHello, handleModels } from './service.mjs';
import { handleTelegramWebhook } from './telegram.mjs';

/** Route id -> handler `(ctx, request) => ApiHttpResponse | Promise<...>`. */
export const ROUTE_HANDLERS = Object.freeze({
  options_preflight: () => emptyResponse(204),
  head_probe: () => emptyResponse(200),
  head_probe_vendor: () => emptyResponse(200),
  api_hello: handleHello,
  health: handleHealth,
  models: handleModels,
  network: handleNetwork,
  network_graph_alias: async (ctx, request) => deprecatedAlias(await handleNetwork(ctx, request)),
  bundle: handleBundle,
  links: handleLinks,
  links_query: handleLinksQuery,
  memory: handleMemory,
  memory_since: handleMemorySince,
  memory_import: handleMemoryImport,
  anthropic_messages: handleAnthropicMessages,
  chat_completions: handleChatCompletions,
  responses: handleResponses,
  mcp: handleMcp,
  mcp_stream: () => messageError(405, 'mcp_sse_unsupported'),
  telegram_webhook: handleTelegramWebhook,
  conversation_context: handleConversationContext,
  conversation_learn: handleConversationLearn,
  gemini_models: handleGeminiModels,
  gemini_model: handleGeminiModel,
  gemini_generate_content: (ctx, request) => handleGeminiGenerateContent(ctx, request, false),
  gemini_stream_generate_content: (ctx, request) => handleGeminiGenerateContent(ctx, request, true),
  vertex_models: handleVertexModels,
  vertex_generate_content: (ctx, request) => handleGeminiGenerateContent(ctx, request, false),
  vertex_stream_generate_content: (ctx, request) => handleGeminiGenerateContent(ctx, request, true),
});

/** The first header named `name` (case-insensitive). */
export function headerValue(headers, name) {
  const lower = name.toLowerCase();
  const found = headers.find(([key]) => key.toLowerCase() === lower);
  return found ? found[1] : null;
}

function parseBearerToken(value) {
  const parts = value.split(/\s+/).filter(Boolean);
  if (parts.length !== 2 || parts[0].toLowerCase() !== 'bearer') return null;
  return parts[1];
}

/** `ApiAuthConfig::allows`. */
export function authAllows(token, headers) {
  if (!token) return true;
  const bearer = headers
    .filter(([key]) => key.toLowerCase() === 'authorization')
    .map(([, value]) => parseBearerToken(value))
    .find((value) => value !== null);
  if (bearer === token) return true;
  const apiKey = headers
    .filter(([key]) => ['x-api-key', 'x-goog-api-key', 'anthropic-api-key'].includes(key.toLowerCase()))
    .map(([, value]) => value.trim())
    .find((value) => value.length > 0);
  return apiKey === token;
}

/** `requires_bearer_auth`: the rule the manifest's `auth` column records. */
export function requiresBearerAuth(method, path) {
  return method !== 'OPTIONS' && (path === '/mcp' || path.startsWith('/v1/') || path.startsWith('/api/'));
}

/** `mcp_origin_allowed`: a browser origin must name this host. */
export function mcpOriginAllowed(headers) {
  const origin = headerValue(headers, 'origin');
  if (origin === null) return true;
  const host = headerValue(headers, 'host');
  if (host === null) return false;
  const trimmed = origin.replace(/\/+$/, '');
  const authority = trimmed.startsWith('http://')
    ? trimmed.slice(7)
    : trimmed.startsWith('https://') ? trimmed.slice(8) : null;
  return authority !== null && authority.toLowerCase() === host.toLowerCase();
}

/**
 * Answer one request.
 * @param {object} ctx server context (`worker`, `memory`, `agentMode`, `bearerToken`)
 * @param {{method: string, path: string, headers: Array<[string, string]>, body: string}} request
 */
export async function dispatch(ctx, request) {
  const normalized = request.path.split('?')[0];
  const authorized = !requiresBearerAuth(request.method, normalized) || authAllows(ctx.bearerToken, request.headers);
  const response = await dispatchRoute(ctx, request);
  recordApiExchangeIfEnabled(request, response, authorized, ctx.env || process.env);
  return response;
}

/** `dispatch_api_request_with_auth`. */
async function dispatchRoute(ctx, request) {
  const { method, path, headers, body } = request;
  const queryAt = path.indexOf('?');
  const normalizedPath = queryAt >= 0 ? path.slice(0, queryAt) : path;
  const query = queryAt >= 0 ? path.slice(queryAt + 1) : '';
  if (normalizedPath === '/mcp' && !mcpOriginAllowed(headers)) {
    return messageError(403, 'mcp_origin_not_allowed');
  }
  if (requiresBearerAuth(method, normalizedPath) && !authAllows(ctx.bearerToken, headers)) {
    return messageError(401, 'bearer_token_invalid');
  }
  const matched = matchRoute(method, normalizedPath);
  if (!matched) return messageError(404, 'route_not_found');
  const handler = ROUTE_HANDLERS[matched.route.id];
  try {
    return await handler(ctx, { method, path, normalizedPath, query, headers, body, params: matched.params });
  } catch (error) {
    return errorResponse(500, String(error?.message || error));
  }
}
