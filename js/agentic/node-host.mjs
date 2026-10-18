import {activeDriveCapture} from '../provenance/capture-context.mjs';
import {recordFor,toLino,answerDerivationId} from './crate/derivation.mjs';
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
import {scanSignatureSymbols} from '../../scripts/lib/signature-census.mjs';
import {sourceModelFrontend} from '../../scripts/source-model.mjs';
import {workflowPin} from '../../scripts/translate-js-rust.mjs';
import {readSyntaxRules, sourceToLinks, ownedSourceLinks, writeSourceLinks, verifySourceLinks} from '../meta/readable-source.js';
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
    return entries.sort((left, right) => Buffer.compare(Buffer.from(left.name), Buffer.from(right.name))).flatMap(entry => {
      const file = relative + '/' + entry.name;
      if (entry.isSymbolicLink()) return incomplete();
      if (entry.isDirectory()) return visit(file);
      if (!entry.name.endsWith('.rs')) return [];
      if (!censusRegularPath(repository, file, 'file')) return incomplete();
      let source, symbols;
      try {
        const status = lstatSync(path.join(repository, file));
        if (status.size > 4 * 1024 * 1024) return incomplete();
        source = new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(readFileSync(path.join(repository, file)));
        symbols = scanSignatureSymbols(source).map(symbol => ({kind: symbol.kind, name: symbol.name,
          start_line: symbol.startLine, end_line: symbol.endLine}));
      } catch { return incomplete(); }
      const target = file.slice('rust/'.length);
      return [{path: file, text: null, module: {path: target, fidelity: 'navigation', symbols},
        sourceIdentity: {path: target, content_id: stableId('source_module', source), byte_len: Buffer.byteLength(source)}}];
    });
  };
  const documents = visit('rust/src');
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
export async function installNodeHost(worker, options = {}) {
  // Caller-owned capability only. Source documents and conversation text never grant installation.
  const freezeGrant = value => {
    if (value && typeof value === "object") {
      for (const child of Object.values(value)) freezeGrant(child);
      Object.freeze(value);
    }
    return value;
  };
  if(options===null||typeof options!=="object")throw Error("host_options_required");
  const grantDescriptor=Object.getOwnPropertyDescriptor(options,"prerequisiteGrant");
  if(grantDescriptor&&!Object.hasOwn(grantDescriptor,"value"))throw Error("host_grant_must_be_data");
  const suppliedGrant=grantDescriptor?.value??null;
  const prerequisiteGrant = suppliedGrant === null ? null
    : freezeGrant(JSON.parse(JSON.stringify(suppliedGrant)));
  const sourceDescriptor = Object.getOwnPropertyDescriptor(options, "sourceModelDirectory");
  if (sourceDescriptor && !Object.hasOwn(sourceDescriptor, "value")) throw Error("host_source_model_directory_must_be_data");
  const sourceModelDirectory = sourceDescriptor?.value ?? process.env.FORMAL_AI_META_LANGUAGE ?? null;
  const context = await worker.boot();

  let sourceParser = null;
  const sourceDocument = async modulePath => {
    if (typeof sourceModelDirectory !== 'string' || !sourceModelDirectory) throw Error('PinnedSourceModelSdkUnavailable');
    const inventory = observeCensusDocuments(REPO_ROOT);
    const observed = inventory.documents.find(document => document.sourceIdentity.path === modulePath);
    if (!inventory.complete || !observed) throw Error('IncompleteOwnedSourceInventory');
    sourceParser ??= sourceModelFrontend(path.resolve(sourceModelDirectory), workflowPin(readRepoFile('.github/workflows/layered-ci.yml')));
    const parse = await sourceParser;
    const source = readRepoFile('rust/' + modulePath);
    if (stableId('source_module', source) !== observed.sourceIdentity.content_id) throw Error('SourceVersionChanged');
    const rules = readSyntaxRules(readRepoFile('data/meta/source-model/syntax-rules.lino'));
    const model = ownedSourceLinks(modulePath, sourceToLinks(source, 'Rust', {parse, rules}));
    verifySourceLinks(model, {parse, rules});
    return writeSourceLinks(model);
  };
  const baseHost = {
    readText: readRepoFile,
    prerequisiteGrant: () => prerequisiteGrant,
    parseLino,
    realm: context,
    planningObserver:()=>{
      const capture=activeDriveCapture();
      if(capture&&'engineGenesis' in capture&&typeof capture.engineGenesis==='function'){
        capture.engineGenesis(()=>worker.provenanceGenesis?.()??null);
      }
      return capture&&'planningObservation' in capture&&typeof capture.planningObservation==='function'
        ?capture.planningObservation:null;
    },
    solve:async(prompt,history)=>{
      const result=await worker.solve(prompt,history),symbolic=symbolicFromWorker(result,history);
      const capture=activeDriveCapture();
      if(capture&&'engineObservation' in capture&&typeof capture.engineObservation==='function'){
        const events=Array.isArray(symbolic.solver_events)?symbolic.solver_events:[];
        capture.engineObservation({schema:'formal-ai-worker-observation-v1',prompt,history,result,symbolic,
          observedInputs:worker.provenanceInputs?.()??null,
          eventLog:events,derivation:toLino(recordFor(events,answerDerivationId(symbolic.answer)))});
      }
      return symbolic;
    },
    isDirectory: (path) => Boolean(stat(path)?.isDirectory()),
    isFile: (path) => Boolean(stat(path)?.isFile()),
    currentDirectory: () => process.cwd(),
    listDirectory: listRepoDirectory,
    censusDocuments,
    censusSourceClosureComplete,
    sourceDocument,
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
