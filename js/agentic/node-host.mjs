// The Node host for the agentic planner: repository files from disk, the
// shared Links Notation parser, and a booted worker realm (js/agentic/host.mjs).
// Used by the JavaScript server and by node:test suites; a browser host would
// install the same shape from the worker's own globals.

import { execFileSync } from 'node:child_process';
import { lstatSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { REPO_ROOT, parseLino, readRepoFile } from '../server/lino.mjs';
import { runtimeCachePaths } from '../server/runtime-cache.mjs';
import { symbolicFromWorker } from '../server/solve.mjs';
import * as webTreeSitter from '../vendor/tree-sitter/web-tree-sitter.mjs';
import { loadRustAstCensus } from './crate/rust_ast_census.mjs';
import { cachedSourceFetch } from './crate/source_cache.mjs';
import { CENSUS_DIR } from './crate/self_ast_census.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { installHost } from './host.mjs';
import { validateRepairProgram } from '../server/repair-program-syntax.mjs';
import { createNodeSourceSessionHost } from '../server/node-source-session-host.mjs';
import { sourceQualifiedSeedOwners } from '../../scripts/generate-seed-registry.mjs';

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

/** Separate generator provenance: failed or stale source never supplies seed owners. */
export function generatedSeedOwnerDeclarations(repository = REPO_ROOT) {
  try { return sourceQualifiedSeedOwners(repository).declarations; }
  catch { return []; }
}

/** Read committed census documents only when their actual source identity still agrees. */
function censusRegularPath(repository, relative, kind) {
  try {
    const parts = relative.split('/');
    if (parts.some(part => !part || part === '.' || part === '..')) return false;
    let current = path.resolve(repository);
    if (!lstatSync(current).isDirectory()) return false;
    for (let index = 0; index < parts.length; index += 1) {
      current = path.join(current, parts[index]);
      const status = lstatSync(current);
      if (status.isSymbolicLink()) return false;
      if (index < parts.length - 1 || kind === 'directory') {
        if (!status.isDirectory()) return false;
      } else if (!status.isFile()) return false;
    }
    return true;
  } catch { return false; }
}

function observeCensusDocuments(repository) {
  let complete = true;
  const incomplete = () => { complete = false; return []; };
  const visit = relative => {
    if (!censusRegularPath(repository, relative, 'directory')) return incomplete();
    let entries;
    try { entries = readdirSync(path.join(repository, relative), {withFileTypes: true}); }
    catch { return incomplete(); }
    return entries.sort((a, b) => a.name.localeCompare(b.name)).flatMap(entry => {
      const file = relative + '/' + entry.name;
      if (entry.isSymbolicLink()) return incomplete();
      if (entry.isDirectory()) return visit(file);
      if (!entry.name.endsWith('.lino')) return [];
      if (!censusRegularPath(repository, file, 'file')) return incomplete();
      let text, parsed;
      try {
        text = readFileSync(path.join(repository, file), 'utf8');
        parsed = parseLino(text);
      } catch { return incomplete(); }
      if (parsed.name !== 'self_ast_census') return incomplete();
      const fields = parsed.children;
      const required = ['target', 'content_id', 'byte_len'];
      if (required.some(name => {
        const values = fields.filter(node => node.name === name);
        return values.length !== 1 || values[0].children.length !== 0
          || typeof values[0].value !== 'string';
      })) return incomplete();
      const field = name => fields.find(node => node.name === name).value;
      const target = field('target');
      if (!target.startsWith('src/') || !target.endsWith('.rs')
        || !censusRegularPath(repository, 'rust/' + target, 'file')) return incomplete();
      let source;
      try { source = readFileSync(path.join(repository, 'rust', target), 'utf8'); }
      catch { return incomplete(); }
      const contentIdentifier = stableId('source_module', source);
      const byteLength = Buffer.byteLength(source);
      if (field('content_id') !== contentIdentifier || Number(field('byte_len')) !== byteLength) {
        return incomplete();
      }
      return [{path: file, text, sourceIdentity: {
        path: target, content_id: contentIdentifier, byte_len: byteLength,
      }}];
    });
  };
  const documents = visit(CENSUS_DIR);
  return {documents, complete};
}

/** Validated documents retain their original payload; incompleteness grants no unique owner. */
export function censusDocuments(repository = REPO_ROOT) {
  return observeCensusDocuments(repository).documents;
}

/** A failed or stale observed census source prevents unqualified uniqueness claims. */
export function censusSourceClosureComplete(repository = REPO_ROOT) {
  try { return observeCensusDocuments(repository).complete; }
  catch { return false; }
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

const readCacheOrMissing = (action) => {
  try {
    return action();
  } catch (error) {
    if (error.code === 'ENOENT') return null;
    throw error;
  }
};

/**
 * The source client `VersionSet::for_generation` builds, read from the
 * environment at each generation: `FORMAL_AI_SOURCE_CACHE_DIR` (default
 * the user's runtime cache, outside the caller repository) and
 * online only when `FORMAL_AI_LIVE_FETCH` is set to a true spelling.
 */
function nodeSourceFetch() {
  return cachedSourceFetch({
    cacheDir: process.env.FORMAL_AI_SOURCE_CACHE_DIR || runtimeCachePaths().runtimeRoot,
    online: flagEnabled(process.env.FORMAL_AI_LIVE_FETCH),
    io: {
      readText: (file) => readCacheOrMissing(() => readFileSync(file, 'utf8')),
      readBytes: (file) => readCacheOrMissing(() => new Uint8Array(readFileSync(file))),
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
  const baseHost = {
    readText: readRepoFile,
    parseLino,
    realm: context,
    solve: async (prompt, history) => symbolicFromWorker(await worker.solve(prompt, history), history),
    isDirectory: (path) => Boolean(stat(path)?.isDirectory()),
    isFile: (path) => Boolean(stat(path)?.isFile()),
    currentDirectory: () => process.cwd(),
    listDirectory: listRepoDirectory,
    censusDocuments,
    censusSourceClosureComplete,
    generatedSeedOwnerDeclarations,
    sourceFetch: nodeSourceFetch,
    validateProgram: validateRepairProgram,
  };
  const session = createNodeSourceSessionHost(baseHost.readText);
  installHost({ ...baseHost, sourceSession: session, sourceOperation: session.sourceOperation });
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
