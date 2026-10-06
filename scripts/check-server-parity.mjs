#!/usr/bin/env node
// Server parity runner (R1013, JavaScript-first doctrine).
//
// Starts the JavaScript server (js/server/main.mjs) and the Rust server
// (`formal-ai serve`) side by side, each with a fresh memory file and the
// corpus bearer token, sends every request in
// rust/tests/fixtures/server-parity/requests.lino to both in order, and
// compares what comes back.
//
// Two kinds of difference are told apart:
//
// * protocol shape - status code, content type, the CORS / deprecation
//   headers, the JSON skeleton (field names, their order, value types), the
//   SSE frame sequence. Never allowed: any one fails the run.
// * content - string and number values inside an identical skeleton (answer
//   text, thinking sentences, token counts) and Links Notation bodies. These
//   are the solver differences the JavaScript twin is still closing; the
//   requests that show one are listed in data/meta/server-parity-ratchet.lino,
//   and the list must match exactly, so the count only falls.
//
// Volatile values are normalized first: `created` / `created_at` timestamps,
// the crate `version`, `{prefix}_{16 hex}` stable ids, disk and memory byte
// counts, and the parser's own wording after an error message's first colon.
//
// Usage:
//   node scripts/check-server-parity.mjs [--rust-binary PATH] [--list]
//          [--only id,id] [--write-ratchet] [--rust-url URL --js-url URL]
// The binary defaults to $FORMAL_AI_RUST_BINARY, then
// rust/target/release/formal-ai, then target/release/formal-ai.

import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { childValue, childrenNamed, parseLino } from '../js/server/lino.mjs';
import { serverRoutes } from '../js/server/routes.mjs';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CORPUS_FILE = 'rust/tests/fixtures/server-parity/requests.lino';
const RATCHET_FILE = 'data/meta/server-parity-ratchet.lino';
const COMPARED_HEADERS = [
  'content-type',
  'access-control-allow-origin',
  'access-control-allow-methods',
  'access-control-allow-headers',
  'deprecation',
  'link',
  'connection',
];
const VOLATILE_NUMBERS = new Set([
  'created',
  'created_at',
  'createTime',
  'inputTokenLimit',
  'disk_free_bytes',
  'memory_used_bytes',
  'context_window',
  'context_window_tokens',
  'context_used_tokens',
  'context_used_fraction',
]);
// Keys a solver answer carries only sometimes (serde skips them when empty).
const ANSWER_OPTIONAL_KEYS = new Set(['learning_trace', 'summary', 'parent_id']);
const STABLE_ID = /^([A-Za-z]+(?:_[A-Za-z]+)*)_[0-9a-f]{16}$/;

/** Parse the corpus. */
export function loadCorpus(text) {
  const root = parseLino(text);
  return {
    token: childValue(root, 'bearer_token'),
    requests: childrenNamed(root, 'request').map((node) => ({
      id: node.value,
      route: childValue(node, 'route'),
      method: childValue(node, 'method'),
      path: childValue(node, 'path'),
      auth: childValue(node, 'auth') || 'bearer',
      headers: childrenNamed(node, 'header').map((header) => header.value),
      body: childrenNamed(node, 'body').length ? childValue(node, 'body') : null,
    })),
  };
}

/** Parse the ratchet. */
export function loadRatchet(text) {
  const root = parseLino(text);
  return {
    ceiling: Number(childValue(root, 'answer_divergence_ceiling') || 0),
    listed: childrenNamed(root, 'divergence').map((node) => node.value),
  };
}

/** Render a ratchet file for `ids`. */
export function renderRatchet(ids) {
  const lines = [
    '# Known content divergences between the JavaScript and Rust servers (R1013).',
    '#',
    '# scripts/check-server-parity.mjs sends the corpus in',
    '# rust/tests/fixtures/server-parity/requests.lino to both servers. A request',
    '# listed here answers with the same protocol shape on both but different',
    '# content - answer text, thinking sentences, token counts or Links Notation -',
    '# because the JavaScript solver has not caught up there yet. Shape',
    '# differences are never listed, they fail outright. The list must match the',
    '# run exactly, so the ceiling only falls - regenerate it with',
    '# `node scripts/check-server-parity.mjs --write-ratchet` once a twin closes.',
    'server_parity_ratchet',
    `  answer_divergence_ceiling ${ids.length}`,
    ...ids.map((id) => `  divergence ${id}`),
  ];
  return `${lines.join('\n')}\n`;
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

/**
 * Send one request over a raw socket and read the response until the server
 * closes the connection (both servers answer `connection: close`). A raw
 * reader sees exactly the bytes on the wire, including a body a server writes
 * after a HEAD response, which an HTTP client library would refuse.
 */
function send(base, request) {
  const url = new URL(base);
  const lines = [`${request.method} ${request.path} HTTP/1.1`, `host: ${url.host}`];
  for (const line of request.headers) lines.push(line.replaceAll('{host}', url.host));
  for (const [name, value] of Object.entries(request.authHeader || {})) lines.push(`${name}: ${value}`);
  const body = request.body === null ? null : Buffer.from(request.body, 'utf8');
  if (body) {
    if (!request.headers.some((line) => /^content-type:/i.test(line))) lines.push('content-type: application/json');
    lines.push(`content-length: ${body.length}`);
  }
  lines.push('connection: close');
  const head = Buffer.from(`${lines.join('\r\n')}\r\n\r\n`, 'utf8');
  return new Promise((resolve, reject) => {
    const socket = net.connect({ host: url.hostname, port: Number(url.port) });
    const chunks = [];
    socket.setTimeout(180000, () => socket.destroy(new Error(`timeout: ${request.id}`)));
    socket.on('connect', () => socket.end(body ? Buffer.concat([head, body]) : head));
    socket.on('data', (chunk) => chunks.push(chunk));
    socket.on('error', reject);
    socket.on('close', () => {
      const raw = Buffer.concat(chunks);
      const split = raw.indexOf('\r\n\r\n');
      if (split < 0) {
        reject(new Error(`no response head for ${request.id}`));
        return;
      }
      const [statusLine, ...headerLines] = raw.subarray(0, split).toString('utf8').split('\r\n');
      const headers = {};
      for (const line of headerLines) {
        const at = line.indexOf(':');
        headers[line.slice(0, at).trim().toLowerCase()] = line.slice(at + 1).trim();
      }
      resolve({
        status: Number(statusLine.split(' ')[1]),
        statusLine,
        headers,
        body: raw.subarray(split + 4).toString('utf8'),
      });
    });
  });
}

async function waitForHealth(base, child, label) {
  const deadline = Date.now() + 180000;
  while (Date.now() < deadline) {
    if (child && child.exitCode !== null) throw new Error(`${label} server exited with ${child.exitCode}`);
    try {
      const response = await send(base, { id: 'health', method: 'GET', path: '/health', headers: [], body: null });
      if (response.status === 200) return;
    } catch {
      // not listening yet
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`${label} server did not become healthy`);
}

async function startServer(label, command, args, token) {
  const home = mkdtempSync(path.join(os.tmpdir(), `formal-ai-parity-${label}-`));
  const port = await freePort();
  const env = { ...process.env };
  delete env.FORMAL_AI_AGENT_MODE;
  for (const name of ['FORMAL_AI_HTTP_BEARER_TOKEN', 'FORMAL_AI_API_TOKEN']) delete env[name];
  Object.assign(env, {
    HOME: home,
    FORMAL_AI_API_BEARER_TOKEN: token,
    FORMAL_AI_MEMORY_PATH: path.join(home, 'memory.lino'),
    FORMAL_AI_DIALOG_LOG_DIR: path.join(home, 'dialogs'),
    FORMAL_AI_RECORD_CHAT: '0',
    FORMAL_AI_DREAMING: '0',
    FORMAL_AI_SILENT: 'true',
  });
  const child = spawn(command, [...args, '--port', String(port)], { cwd: REPO, env, stdio: ['ignore', 'ignore', 'pipe'] });
  let stderr = '';
  child.stderr.on('data', (chunk) => {
    stderr = (stderr + chunk).slice(-4000);
  });
  const base = `http://127.0.0.1:${port}`;
  try {
    await waitForHealth(base, child, label);
  } catch (error) {
    child.kill('SIGKILL');
    throw new Error(`${error.message}\n${stderr}`);
  }
  return {
    base,
    stop() {
      child.kill('SIGKILL');
      rmSync(home, { recursive: true, force: true });
    },
  };
}

function authHeader(request, token) {
  switch (request.auth) {
    case 'none':
      return null;
    case 'wrong':
      return { authorization: 'Bearer server-parity-wrong-token' };
    case 'api_key':
      return { 'x-api-key': token };
    case 'goog_api_key':
      return { 'x-goog-api-key': token };
    default:
      return { authorization: `Bearer ${token}` };
  }
}

/** Replace volatile values so equal behaviour compares equal. */
export function normalize(value, key = '') {
  if (Array.isArray(value)) return value.map((item) => normalize(item, key));
  if (value && typeof value === 'object') {
    const out = {};
    for (const [name, item] of Object.entries(value)) out[name] = normalize(item, name);
    // An error message carries the parser's own wording after its first colon.
    if (typeof out.message === 'string' && (Object.hasOwn(out, 'type') || Object.hasOwn(out, 'code'))) {
      const at = out.message.indexOf(':');
      if (at >= 0) out.message = `${out.message.slice(0, at)}: <detail>`;
    }
    return out;
  }
  if (typeof value === 'number' && VOLATILE_NUMBERS.has(key)) return 0;
  if (typeof value === 'string') {
    if (key === 'version') return '<version>';
    if (VOLATILE_NUMBERS.has(key)) return '<volatile>';
    const id = STABLE_ID.exec(value);
    if (id) return `${id[1]}_<id>`;
  }
  return value;
}

/** The JSON skeleton: field names in order and value types, never values. */
export function skeleton(value) {
  if (value === null) return 'null';
  if (Array.isArray(value)) {
    const merged = value.map(skeleton).reduce(mergeSkeleton, null);
    return merged === null ? ['array'] : ['array', merged];
  }
  if (typeof value === 'object') {
    return {
      object: Object.entries(value)
        .filter(([key]) => !ANSWER_OPTIONAL_KEYS.has(key))
        .map(([key, item]) => [key, skeleton(item)]),
    };
  }
  return typeof value;
}

function mergeSkeleton(left, right) {
  if (left === null) return right;
  if (left?.object && right?.object) {
    const keys = [...left.object];
    for (const entry of right.object) {
      if (!keys.some(([key]) => key === entry[0])) keys.push(entry);
    }
    return { object: keys };
  }
  return JSON.stringify(left) === JSON.stringify(right) ? left : { union: [left, right] };
}

/** Compare skeletons; an empty array matches any array. */
function sameSkeleton(left, right) {
  if (Array.isArray(left) && Array.isArray(right)) {
    if (left.length === 1 || right.length === 1) return true;
    return sameSkeleton(left[1], right[1]);
  }
  if (left?.object && right?.object) {
    if (left.object.length !== right.object.length) return false;
    return left.object.every(([key, item], index) =>
      right.object[index][0] === key && sameSkeleton(item, right.object[index][1]),
    );
  }
  return JSON.stringify(left) === JSON.stringify(right);
}

function parseJson(text) {
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch {
    return { ok: false };
  }
}

/** Split an SSE body into `{event, data}` frames, data parsed when JSON. */
export function sseFrames(body) {
  return body
    .split('\n\n')
    .filter((frame) => frame.trim())
    .map((frame) => {
      let event = '';
      const data = [];
      for (const line of frame.split('\n')) {
        if (line.startsWith('event:')) event = line.slice(6).trim();
        else if (line.startsWith('data:')) data.push(line.slice(5).replace(/^ /, ''));
      }
      const text = data.join('\n');
      const parsed = parseJson(text);
      return { event, data: parsed.ok ? normalize(parsed.value) : text };
    });
}

/** Consecutive frames with one shape count once (chunk counts follow answer length). */
function frameShapes(frames) {
  const shapes = [];
  for (const frame of frames) {
    const shape = JSON.stringify([frame.event, typeof frame.data === 'string' ? frame.data : skeleton(frame.data)]);
    if (shapes[shapes.length - 1] !== shape) shapes.push(shape);
  }
  return shapes;
}

/**
 * Compare one exchange. Returns `{kind: 'same' | 'content' | 'protocol', detail}`.
 */
export function compareResponses(rust, js, method = 'GET') {
  if (rust.statusLine !== js.statusLine) return { kind: 'protocol', detail: `status ${rust.statusLine} != ${js.statusLine}` };
  for (const name of COMPARED_HEADERS) {
    if ((rust.headers[name] ?? null) !== (js.headers[name] ?? null)) {
      return { kind: 'protocol', detail: `header ${name}: ${rust.headers[name]} != ${js.headers[name]}` };
    }
  }
  // A HEAD answer has no body for any client; its declared length must agree.
  if (method === 'HEAD') {
    return rust.headers['content-length'] === js.headers['content-length']
      ? { kind: 'same' }
      : { kind: 'protocol', detail: `HEAD content-length ${rust.headers['content-length']} != ${js.headers['content-length']}` };
  }
  const type = String(rust.headers['content-type'] || '');
  if (type.startsWith('text/event-stream')) {
    const left = sseFrames(rust.body);
    const right = sseFrames(js.body);
    const leftShapes = frameShapes(left);
    const rightShapes = frameShapes(right);
    if (leftShapes.length !== rightShapes.length) {
      return { kind: 'protocol', detail: `sse frames ${leftShapes.join(' ')} != ${rightShapes.join(' ')}` };
    }
    for (let index = 0; index < leftShapes.length; index += 1) {
      const a = JSON.parse(leftShapes[index]);
      const b = JSON.parse(rightShapes[index]);
      if (a[0] !== b[0] || !sameSkeleton(a[1], b[1])) {
        return { kind: 'protocol', detail: `sse frame ${index}: ${leftShapes[index]} != ${rightShapes[index]}` };
      }
    }
    return JSON.stringify(left) === JSON.stringify(right)
      ? { kind: 'same' }
      : { kind: 'content', detail: 'sse content differs' };
  }
  if (type.startsWith('application/json') && (rust.body || js.body)) {
    const left = parseJson(rust.body);
    const right = parseJson(js.body);
    if (!left.ok || !right.ok) return { kind: 'protocol', detail: 'body is not JSON on one side' };
    const a = normalize(left.value);
    const b = normalize(right.value);
    if (!sameSkeleton(skeleton(a), skeleton(b))) {
      return { kind: 'protocol', detail: `json shape ${JSON.stringify(skeleton(a))} != ${JSON.stringify(skeleton(b))}` };
    }
    return JSON.stringify(a) === JSON.stringify(b) ? { kind: 'same' } : { kind: 'content', detail: firstDifference(a, b) };
  }
  return rust.body === js.body ? { kind: 'same' } : { kind: 'content', detail: 'body text differs' };
}

function firstDifference(left, right, at = '$') {
  if (JSON.stringify(left) === JSON.stringify(right)) return null;
  if (left && right && typeof left === 'object' && typeof right === 'object') {
    for (const key of Object.keys(left)) {
      const found = firstDifference(left[key], right[key], `${at}.${key}`);
      if (found) return found;
    }
  }
  return `${at}: ${JSON.stringify(left)?.slice(0, 120)} != ${JSON.stringify(right)?.slice(0, 120)}`;
}

function parseArgs(argv) {
  const options = { list: false, write: false, only: null, rustBinary: null, rustUrl: null, jsUrl: null };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--list') options.list = true;
    else if (arg === '--write-ratchet') options.write = true;
    else if (arg === '--only') options.only = new Set(argv[++index].split(','));
    else if (arg === '--rust-binary') options.rustBinary = argv[++index];
    else if (arg === '--rust-url') options.rustUrl = argv[++index];
    else if (arg === '--js-url') options.jsUrl = argv[++index];
    else throw new Error(`unknown argument ${arg}`);
  }
  return options;
}

function resolveBinary(explicit) {
  const candidates = [explicit, process.env.FORMAL_AI_RUST_BINARY, 'rust/target/release/formal-ai', 'target/release/formal-ai']
    .filter(Boolean)
    .map((candidate) => path.resolve(REPO, candidate));
  return candidates.find((candidate) => existsSync(candidate)) || null;
}

/** Every manifest route must be exercised; every corpus route must exist. */
export function coverageProblems(corpus, routes = serverRoutes()) {
  const ids = new Set(routes.map((route) => route.id));
  const used = new Set(corpus.requests.map((request) => request.route).filter(Boolean));
  const problems = [];
  for (const id of ids) if (!used.has(id)) problems.push(`manifest route ${id} has no parity request`);
  for (const id of used) if (!ids.has(id)) problems.push(`parity requests name unknown route ${id}`);
  return problems;
}

async function main(argv) {
  const options = parseArgs(argv);
  const corpus = loadCorpus(readFileSync(path.join(REPO, CORPUS_FILE), 'utf8'));
  const problems = coverageProblems(corpus);
  for (const problem of problems) console.error(`::error::${problem}`);
  if (problems.length) return 1;

  const servers = [];
  try {
    let rustBase = options.rustUrl;
    let jsBase = options.jsUrl;
    if (!rustBase) {
      const binary = resolveBinary(options.rustBinary);
      if (!binary) {
        console.error('::error::no Rust server binary; build it or pass --rust-binary / FORMAL_AI_RUST_BINARY');
        return 1;
      }
      const rust = await startServer('rust', binary, ['serve'], corpus.token);
      servers.push(rust);
      rustBase = rust.base;
    }
    if (!jsBase) {
      const js = await startServer('js', process.execPath, ['js/server/main.mjs'], corpus.token);
      servers.push(js);
      jsBase = js.base;
    }

    const protocol = [];
    const content = [];
    let same = 0;
    for (const request of corpus.requests) {
      if (options.only && !options.only.has(request.id)) continue;
      const prepared = { ...request, authHeader: authHeader(request, corpus.token) };
      const rust = await send(rustBase, prepared);
      const js = await send(jsBase, prepared);
      const verdict = compareResponses(rust, js, request.method);
      if (verdict.kind === 'protocol') protocol.push([request.id, verdict.detail]);
      else if (verdict.kind === 'content') content.push([request.id, verdict.detail]);
      else same += 1;
    }

    console.log(`server parity: ${same} identical, ${content.length} content divergence(s), ${protocol.length} protocol divergence(s)`);
    for (const [id, detail] of protocol) console.error(`::error::protocol divergence in ${id}: ${detail}`);
    if (options.list) for (const [id, detail] of content) console.log(`  content ${id}: ${detail}`);
    if (options.only) return protocol.length ? 1 : 0;

    const ids = content.map(([id]) => id);
    if (options.write) {
      writeFileSync(path.join(REPO, RATCHET_FILE), renderRatchet(ids));
      console.log(`wrote ${RATCHET_FILE} with ${ids.length} divergence(s)`);
      return protocol.length ? 1 : 0;
    }
    const ratchet = loadRatchet(readFileSync(path.join(REPO, RATCHET_FILE), 'utf8'));
    let status = protocol.length ? 1 : 0;
    const unlisted = ids.filter((id) => !ratchet.listed.includes(id));
    for (const id of unlisted) {
      console.error(`::error::new content divergence in ${id}: make the JavaScript server answer like the Rust one (R1013).`);
      status = 1;
    }
    if (ratchet.listed.length !== ratchet.ceiling) {
      console.error(`::error::${RATCHET_FILE} lists ${ratchet.listed.length} divergence(s) under a ceiling of ${ratchet.ceiling}.`);
      status = 1;
    }
    if (ids.length > ratchet.ceiling) {
      console.error(`::error::${ids.length} content divergence(s) above the ceiling ${ratchet.ceiling}.`);
      status = 1;
    } else if (ids.length < ratchet.ceiling) {
      const closed = ratchet.listed.filter((id) => !ids.includes(id));
      console.error(`::error::parity improved (${closed.join(', ')}): run --write-ratchet to lower the ceiling to ${ids.length}.`);
      status = 1;
    }
    return status;
  } finally {
    for (const server of servers) server.stop();
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(
    (code) => {
      process.exitCode = code;
    },
    (error) => {
      console.error(`::error::${error.stack || error}`);
      process.exitCode = 1;
    },
  );
}
