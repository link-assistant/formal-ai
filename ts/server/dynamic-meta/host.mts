import { serverMessage } from '../messages.mjs';
import { timingSafeEqual } from 'node:crypto';
import { readSync } from 'node:fs';

import { createRecipeDiscovery } from './discovery.mjs';
import { createMetaRuntime, MetaRequestError, requireApproval } from './runtime.mjs';
import { errorResponse, jsonResponse } from '../response.mjs';
import { parseBoolEnv } from '../memory-store.mjs';
import { META_REQUEST_LIMIT } from './wire.mjs';

function tokenMatches(expected, observed) {
  if (!expected || !observed) return false;
  const left = Buffer.from(expected);
  const right = Buffer.from(observed);
  return left.length === right.length && timingSafeEqual(left, right);
}

export function configuredMetaHost(env, { adapters, fetchImpl } = {}) {
  const path = env.FORMAL_AI_DYNAMIC_META_STORE?.trim();
  if (!path) return null;
  const adminToken = env.FORMAL_AI_DYNAMIC_META_ADMIN_TOKEN?.trim();
  const discover = createRecipeDiscovery({
    env, fetchImpl, origins: (env.FORMAL_AI_DYNAMIC_META_SOURCE_ORIGINS ?? '').split(',').map((part) => part.trim()).filter(Boolean),
  });
  const runtime = createMetaRuntime({
    path, adapters,
    triggers: parseBoolEnv(env.FORMAL_AI_DYNAMIC_META_TRIGGERS) !== false,
    selfModification: parseBoolEnv(env.FORMAL_AI_DYNAMIC_META_SELF_MODIFICATION) === true,
  });
  return {
    ...runtime,
    async act(request, actor) {
      if (request?.action === 'discover') {
        requireApproval(request, actor, { admin: true });
        return { sources: await discover(request.sources, actor.signal) };
      }
      return runtime.act(request, actor);
    },
    isAdmin: (token) => tokenMatches(adminToken, token),
  };
}

export async function handleDynamicMeta(ctx, request, { admin, signal } = {}) {
  if (!ctx.dynamicMeta || request.method !== 'POST') return errorResponse(404, serverMessage('dynamic-meta-unavailable'));
  try {
    if (Buffer.byteLength(request.body) > META_REQUEST_LIMIT) throw new MetaRequestError(413, serverMessage('dynamic-meta-request-size'));
    const action = JSON.parse(request.body);
    return jsonResponse(200, await ctx.dynamicMeta.act(action, { admin, signal }));
  } catch (error) {
    if (error.code?.startsWith('ERR_SQLITE')) {
      ctx.logger?.log('error', 'dynamic_meta.store_error', { error });
      return errorResponse(500, serverMessage('dynamic-meta-store-error'));
    }
    return errorResponse(error.status ?? 400, error.message);
  }
}

export async function runMetaCli(argv, env = process.env) {
  if (argv.length !== 0) {
    process.stderr.write(serverMessage('dynamic-meta-cli-usage'));
    return 2;
  }
  let host;
  try {
    host = configuredMetaHost(env);
    if (!host) throw new Error(serverMessage('dynamic-meta-cli-store'));
    if (process.stdin.isTTY) throw new Error(serverMessage('dynamic-meta-cli-stdin'));
    const buffer = Buffer.alloc(META_REQUEST_LIMIT + 1);
    let length = 0;
    while (length < buffer.length) {
      const count = readSync(0, buffer, length, buffer.length - length);
      if (count === 0) break;
      length += count;
    }
    if (length > META_REQUEST_LIMIT) throw new Error(serverMessage('dynamic-meta-request-size'));
    const request = JSON.parse(buffer.subarray(0, length).toString('utf8'));
    const result = await host.act(request, { admin: true });
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    return 0;
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    return 1;
  } finally {
    host?.close();
  }
}
