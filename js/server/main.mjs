#!/usr/bin/env node
// The JavaScript formal-ai HTTP server: `node js/server/main.mjs --port N`.
//
// Takes the flags and environment of `formal-ai serve` (rust/src/main.rs,
// rust/src/cli_local_transport.rs `ServeArgs`): `--host` / FORMAL_AI_HOST
// (127.0.0.1), `--port` / FORMAL_AI_PORT (8080), `--agent-mode` (or
// FORMAL_AI_AGENT_MODE), and the bearer token from FORMAL_AI_API_BEARER_TOKEN,
// FORMAL_AI_HTTP_BEARER_TOKEN or FORMAL_AI_API_TOKEN. The `--ws` / `--webrtc`
// transports are native-only and refused.

import { pathToFileURL } from 'node:url';

import { createServer } from './http.mjs';
import { createMemory, memoryPath } from './memory.mjs';
import { serverMessage } from './messages.mjs';
import { WorkerHost } from './worker-host.mjs';

const TOKEN_ENV = ['FORMAL_AI_API_BEARER_TOKEN', 'FORMAL_AI_HTTP_BEARER_TOKEN', 'FORMAL_AI_API_TOKEN'];

/** `first_non_empty_env`. */
export function bearerTokenFromEnv(env = process.env) {
  for (const name of TOKEN_ENV) {
    const value = (env[name] || '').trim();
    if (value) return value;
  }
  return null;
}

function flagEnabled(value) {
  return ['1', 'true', 'yes', 'on'].includes(String(value || '').trim().toLowerCase());
}

/** Parse `serve` flags. @param {Array<string>} argv */
export function parseArgs(argv, env = process.env) {
  const options = {
    host: env.FORMAL_AI_HOST || '127.0.0.1',
    port: Number.parseInt(env.FORMAL_AI_PORT || '8080', 10),
    agentMode: flagEnabled(env.FORMAL_AI_AGENT_MODE),
    unsupported: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const [name, inline] = arg.includes('=') ? arg.split(/=(.*)/s, 2) : [arg, null];
    const value = () => (inline !== null ? inline : argv[++index]);
    if (name === '--host') options.host = value();
    else if (name === '--port') options.port = Number.parseInt(value(), 10);
    else if (name === '--agent-mode') options.agentMode = true;
    else if (name === 'serve' && index === 0) continue;
    else options.unsupported = arg;
  }
  return options;
}

/**
 * Start a server; resolves once it listens.
 * @returns {Promise<{server: import('node:http').Server, ctx: object, url: string}>}
 */
export async function startServer({ host = '127.0.0.1', port = 0, agentMode = false, env = process.env, worker } = {}) {
  const ctx = {
    worker: worker || new WorkerHost(),
    memory: createMemory(env),
    agentMode,
    bearerToken: bearerTokenFromEnv(env),
    env,
  };
  await ctx.worker.boot();
  const server = createServer(ctx);
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, host, resolve);
  });
  const address = server.address();
  return { server, ctx, url: `http://${host}:${address.port}` };
}

async function main(argv) {
  const options = parseArgs(argv);
  if (options.unsupported) {
    process.stderr.write(`unsupported argument: ${options.unsupported}\n`);
    return 2;
  }
  process.stderr.write(`${serverMessage('server_shared_memory', { path: memoryPath(process.env) })}\n`);
  const { url } = await startServer(options);
  process.stderr.write(`${serverMessage('server_listening', { url })}\n`);
  return null;
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  main(process.argv.slice(2)).then((code) => {
    if (code !== null) process.exitCode = code;
  });
}
