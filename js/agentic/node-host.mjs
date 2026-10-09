// The Node host for the agentic planner: repository files from disk, the
// shared Links Notation parser, and a booted worker realm (js/agentic/host.mjs).
// Used by the JavaScript server and by node:test suites; a browser host would
// install the same shape from the worker's own globals.

import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { REPO_ROOT, parseLino, readRepoFile } from '../server/lino.mjs';
import { symbolicFromWorker } from '../server/solve.mjs';
import * as webTreeSitter from '../vendor/tree-sitter/web-tree-sitter.mjs';
import { loadRustAstCensus } from './crate/rust_ast_census.mjs';
import { cachedSourceFetch } from './crate/source_cache.mjs';
import { CENSUS_DIR } from './crate/self_ast_census.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { installHost } from './host.mjs';

function stat(path) {
  try {
    return statSync(path);
  } catch {
    return null;
  }
}

/**
 * The entries of the repository directory `relative`, following symlinks
 * for `isDirectory` like Rust's `Path::is_dir`; `[]` when it cannot be read.
 * @param {string} relative
 */
function listRepoDirectory(relative) {
  const directory = path.join(REPO_ROOT, relative);
  let names;
  try {
    names = readdirSync(directory);
  } catch {
    return [];
  }
  return names.map((name) => ({ name, isDirectory: Boolean(stat(path.join(directory, name))?.isDirectory()) }));
}

/** Read committed census documents only when their actual source identity still agrees. */
export function censusDocuments(repository = REPO_ROOT) {
  const visit = (relative) => {
    let entries;
    try { entries = readdirSync(path.join(repository, relative), { withFileTypes: true }); }
    catch { return []; }
    return entries.sort((a, b) => a.name.localeCompare(b.name)).flatMap((entry) => {
      const file = relative + '/' + entry.name;
      if (entry.isDirectory()) return visit(file);
      if (!entry.isFile() || !entry.name.endsWith('.lino')) return [];
      const text = readFileSync(path.join(repository, file), 'utf8');
      const parsed = parseLino(text);
      const fields = parsed.name === 'self_ast_census' ? parsed.children
        : parsed.children.find((node) => node.name === 'self_ast_census')?.children ?? [];
      const field = (name) => fields.find((node) => node.name === name)?.value;
      const target = field('target');
      if (!target?.startsWith('src/') || !target.endsWith('.rs') || target.split('/').includes('..')) return [];
      let source;
      try { source = readFileSync(path.join(repository, 'rust', target), 'utf8'); }
      catch { return []; }
      const contentIdentifier = stableId('source_module', source);
      const byteLength = Buffer.byteLength(source);
      if (field('content_id') !== contentIdentifier || Number(field('byte_len')) !== byteLength) return [];
      return [{ path: file, text, sourceIdentity: { path: target, content_id: contentIdentifier, byte_len: byteLength } }];
    });
  };
  return visit(CENSUS_DIR);
}

/** `cli_env::flag_enabled`: a true spelling of the variable. */
const flagEnabled = (value) => ['1', 'true', 'yes', 'on'].includes(String(value ?? '').trim().toLowerCase());

/** `CurlSourceTransport::get`: the response bytes, null on any failure. */
function curlGet(url) {
  try {
    return new Uint8Array(execFileSync('curl', ['--fail', '--silent', '--show-error', '--location', '--compressed',
      '--max-time', '30', '--user-agent', 'formal-ai (https://github.com/link-assistant/formal-ai; source retrieval)', url],
    { stdio: ['ignore', 'pipe', 'ignore'], maxBuffer: 64 * 1024 * 1024 }));
  } catch {
    return null;
  }
}

const tryOr = (action) => {
  try {
    return action();
  } catch {
    return null;
  }
};

/**
 * The source client `VersionSet::for_generation` builds, read from the
 * environment at each generation: `FORMAL_AI_SOURCE_CACHE_DIR` (default
 * `data`, relative to the working directory like the native client) and
 * online only when `FORMAL_AI_LIVE_FETCH` is set to a true spelling.
 */
function nodeSourceFetch() {
  return cachedSourceFetch({
    cacheDir: process.env.FORMAL_AI_SOURCE_CACHE_DIR || 'data',
    online: flagEnabled(process.env.FORMAL_AI_LIVE_FETCH),
    io: {
      readText: (file) => tryOr(() => readFileSync(file, 'utf8')),
      readBytes: (file) => tryOr(() => new Uint8Array(readFileSync(file))),
      writeBytes: (file, bytes) => writeFileSync(file, bytes),
      createDirAll: (directory) => mkdirSync(directory, { recursive: true }),
      get: curlGet,
    },
  });
}

/**
 * Boot `worker` (a js/server/worker-host.mjs `WorkerHost`) and install the
 * planner host over its realm.
 * @param {{boot: () => Promise<object>}} worker
 * @returns {Promise<object>} the worker realm
 */
export async function installNodeHost(worker) {
  const context = await worker.boot();
  installHost({
    readText: readRepoFile,
    parseLino,
    realm: context,
    solve: async (prompt, history) => symbolicFromWorker(await worker.solve(prompt, history), history),
    isDirectory: (path) => Boolean(stat(path)?.isDirectory()),
    isFile: (path) => Boolean(stat(path)?.isFile()),
    currentDirectory: () => process.cwd(),
    listDirectory: listRepoDirectory,
    censusDocuments,
    sourceFetch: nodeSourceFetch,
  });
  return context;
}

let rustCensus = null;

/**
 * The Rust node-kind census over the vendored web-tree-sitter runtime and
 * tree-sitter-rust grammar (js/vendor/tree-sitter/), loaded once.
 */
export function rustAstCensus() {
  if (rustCensus === null) {
    rustCensus = loadRustAstCensus(webTreeSitter, path.join(REPO_ROOT, 'js/vendor/tree-sitter/tree-sitter-rust.wasm'));
  }
  return rustCensus;
}
