#!/usr/bin/env node
// Generate the browser worker's copies of the crate modules the chat routes
// call (R1188-U18, U19, U21): text formalization, dependency summarization,
// page formalization and round-trip translation; and the event log's evidence
// projection every answer carries (R1188-U29).
//
// The browser worker is a classic script (js/worker/formal_ai_worker.js loads
// its modules with `importScripts`), so it cannot import the ES modules of
// js/agentic/crate/. Instead of a second, hand-kept port, every module in the
// import closure of the entry modules below is written to
// js/worker/formal_ai_worker_crate_<name>.js as a factory:
//
//   (self.FORMAL_AI_CRATE_FACTORIES ||= {})["crate/text_formalization.mjs"] =
//     (crateRequire) => { <the module body>; return { <its exports> }; };
//
// Its body is the module's own text, byte for byte, with each `import` turned
// into a `crateRequire` of the module it names and each `export` keyword
// dropped. A factory runs on first use, so the worker loads the files in any
// order. js/worker/formal_ai_worker_crate_modules.js instantiates them and
// installs the worker as the host of js/agentic/host.mjs.
//
// host.mjs reads the seed registry to find the meaning lexicon's files; the
// browser does not fetch data/meta/, so the registry's meaning seeds are
// written to formal_ai_worker_crate_seed_registry.js in its order.
//
// Usage:
//   node scripts/generate-worker-crate-modules.mjs --write   regenerate the files
//   node scripts/generate-worker-crate-modules.mjs --check   fail when a file drifts (CI gate)

import { existsSync, readFileSync, readdirSync, unlinkSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

import { REPO_ROOT } from '../js/server/lino.mjs';

/** The modules the chat routes call; their import closure is generated. */
export const ENTRY_MODULES = Object.freeze([
  'crate/dependency_summarization.mjs',
  'crate/event_log.mjs',
  'crate/page_formalization.mjs',
  'crate/skill_compiler.mjs',
  'crate/skill_procedure.mjs',
  'crate/skill_procedure_artifact.mjs',
  'crate/solver_formalization.mjs',
  'crate/solver_event_projection.mjs',
  'crate/round_trip_translation.mjs',
]);

const AGENTIC_ROOT = 'js/agentic';
const WORKER_ROOT = 'js/worker';
const FILE_PREFIX = 'formal_ai_worker_crate_';
const REGISTRY_FILE = `${FILE_PREFIX}seed_registry.js`;
const LOADER_FILE = `${FILE_PREFIX}modules.js`;
const SEED_REGISTRY = 'data/meta/seed-registry.lino';
const COMMAND = 'node scripts/generate-worker-crate-modules.mjs --write';

const IMPORT = /^(?:import|export)\s*\{([^}]*)\}\s*from\s*'([^']+)';\n/gmu;
const EXPORT_DECLARATION = /^export (async function|function|const|let|class) ([A-Za-z_$][\w$]*)/gmu;
const LOCAL_EXPORT = /^export\s*\{([^}]*)\};\n/gmu;

/**
 * The module name of `specifier` imported from `from`, both relative to
 * js/agentic (`crate/x.mjs`, `host.mjs`).
 * @param {string} from
 * @param {string} specifier
 */
export function resolveModule(from, specifier) {
  return path.posix.normalize(path.posix.join(path.posix.dirname(from), specifier));
}

/** Local export clauses preserve the existing binding and its public alias. */
function localExportBindings(source) {
  const pairs = [...source.matchAll(LOCAL_EXPORT)].flatMap((match) =>
    match[1].split(',').map((binding) => {
      const [local, exposed = local] = binding.trim().split(/\s+as\s+/u);
      return [exposed, local];
    }).filter(([exposed, local]) => exposed && local));
  return new Map(pairs);
}

/** The names a module exports, in source order. @param {string} source */
export function exportedNames(source) {
  const declared = [...source.matchAll(EXPORT_DECLARATION)].map((match) => match[2]);
  const forwarded = [...source.matchAll(IMPORT)].filter((match) => match[0].startsWith('export'))
    .flatMap((match) => match[1].split(',').map((binding) => binding.trim().split(/\s+as\s+/u).at(-1)).filter(Boolean));
  return [...new Set([...declared, ...forwarded, ...localExportBindings(source).keys()])];
}

/**
 * The bindings of one import list as a destructuring pattern
 * (`a, b as c` gives `a, b: c`).
 * @param {string} list
 */
export function bindingPattern(list) {
  return list
    .split(',')
    .map((binding) => binding.trim())
    .filter(Boolean)
    .map((binding) => binding.replace(/^([\w$]+)\s+as\s+([\w$]+)$/u, '$1: $2'))
    .join(', ');
}

/**
 * The modules `name` imports, as `{module, pattern, text}` with the import
 * statement's text.
 * @param {string} name
 * @param {string} source
 */
export function importsOf(name, source) {
  return [...source.matchAll(IMPORT)].map((match) => ({
    module: resolveModule(name, match[2]),
    pattern: bindingPattern(match[1]),
    text: match[0],
  }));
}

/**
 * The import closure of `entries`, dependencies before the modules that
 * import them.
 * @param {readonly string[]} entries
 * @param {(name: string) => string} read
 */
export function importClosure(entries, read) {
  const order = [];
  const visit = (name) => {
    if (order.includes(name)) return;
    order.push(name);
    for (const dependency of importsOf(name, read(name))) visit(dependency.module);
  };
  for (const entry of entries) visit(entry);
  return order.sort();
}

/**
 * The worker file a module is written to.
 * @param {string} name
 */
export function workerFileOf(name) {
  const stem = path.posix.basename(name, '.mjs');
  return `${FILE_PREFIX}${stem}.js`;
}

/**
 * The factory text of one module.
 * @param {string} name
 * @param {string} source
 */
export function factoryOf(name, source) {
  let body = source;
  for (const dependency of importsOf(name, source)) {
    const binding = `const { ${dependency.pattern} } = crateRequire("${dependency.module}");\n`;
    body = body.replace(dependency.text, binding);
  }
  body = body.replace(EXPORT_DECLARATION, '$1 $2').replace(LOCAL_EXPORT, '');
  const aliases = localExportBindings(source);
  const exports = exportedNames(source).map((exported) => {
    const local = aliases.get(exported) ?? exported;
    return local === exported ? `  ${exported},` : `  ${exported}: ${local},`;
  }).join('\n');
  const text = [
    `// Generated by \`${COMMAND}\``,
    `// from ${AGENTIC_ROOT}/${name}.`,
    '// Do not edit by hand: edit that module and regenerate. The browser worker',
    '// cannot import ES modules, so the module is a factory that',
    '// formal_ai_worker_crate_modules.js runs on first use.',
    '',
    `(self.FORMAL_AI_CRATE_FACTORIES ||= {})["${name}"] = (crateRequire) => {`,
    body.trimEnd(),
    '',
    'return Object.freeze({',
    exports,
    '});',
    '};',
    '',
  ].join('\n');
  // Validate the classic-script target before any generated file is written.
  new vm.Script(text, { filename: workerFileOf(name) });
  return text;
}

/**
 * The seeds the registry gives the meaning lexicon, in its order.
 * @param {string} registry
 */
export function meaningSeeds(registry, lexicon = 'meaning') {
  const seeds = [];
  let current = null;
  for (const line of registry.split('\n')) {
    const seed = /^ {2}seed (\S+)\s*$/u.exec(line);
    if (seed) {
      current = seed[1];
      continue;
    }
    if (current !== null && /^ {4}lexicon (\S+)\s*$/u.exec(line)?.[1] === lexicon) seeds.push(current);
  }
  return seeds;
}

/**
 * The registry file text.
 * @param {string[]} seeds
 */
export function registryFileOf(seeds, responseSeeds = []) {
  return [
    `// Generated by \`${COMMAND}\``,
    `// from ${SEED_REGISTRY}. Do not edit by hand.`,
    '//',
    '// The seeds the registry gives the meaning lexicon (`lexicon meaning`), in',
    '// its order: js/agentic/crate/seed_meanings.mjs reads the lexicon from',
    '// them, and the browser does not fetch the registry itself.',
    '',
    'self.FORMAL_AI_CRATE_MEANING_SEEDS = Object.freeze([',
    ...seeds.map((seed) => `  "${seed}",`),
    ']);',
    'self.FORMAL_AI_CRATE_RESPONSE_SEEDS = Object.freeze([',
    ...responseSeeds.map((seed) => `  "${seed}",`),
    ']);',
    '',
  ].join('\n');
}

/** Every generated file, as a map from its worker path to its text. */
export function generatedFiles() {
  const read = (name) => readFileSync(path.join(REPO_ROOT, AGENTIC_ROOT, name), 'utf8');
  const files = new Map();
  for (const name of importClosure(ENTRY_MODULES, read)) {
    files.set(`${WORKER_ROOT}/${workerFileOf(name)}`, factoryOf(name, read(name)));
  }
  const registry = readFileSync(path.join(REPO_ROOT, SEED_REGISTRY), 'utf8');
  files.set(`${WORKER_ROOT}/${REGISTRY_FILE}`, registryFileOf(meaningSeeds(registry), meaningSeeds(registry, 'response')));
  return files;
}

/** The generated worker files on disk now (the hand-written loader excluded). */
function filesOnDisk() {
  return readdirSync(path.join(REPO_ROOT, WORKER_ROOT))
    .filter((file) => file.startsWith(FILE_PREFIX) && file !== LOADER_FILE)
    .map((file) => `${WORKER_ROOT}/${file}`);
}

function main(argv) {
  const write = argv.includes('--write');
  const expected = generatedFiles();
  const drift = [];
  for (const [file, text] of expected) {
    const full = path.join(REPO_ROOT, file);
    if (existsSync(full) && readFileSync(full, 'utf8') === text) continue;
    drift.push(file);
    if (write) writeFileSync(full, text);
  }
  for (const file of filesOnDisk()) {
    if (expected.has(file)) continue;
    drift.push(file);
    if (write) unlinkSync(path.join(REPO_ROOT, file));
  }
  if (write) {
    console.log(`${expected.size} worker crate files, ${drift.length} rewritten.`);
    return 0;
  }
  if (drift.length === 0) {
    console.log(`The ${expected.size} worker crate files are current.`);
    return 0;
  }
  console.error(`Stale worker crate files (run \`${COMMAND}\`):`);
  for (const file of drift) console.error(`  ${file}`);
  return 1;
}

if (import.meta.url === `file://${process.argv[1]}`) process.exitCode = main(process.argv.slice(2));
