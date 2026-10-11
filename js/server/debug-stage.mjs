// What a step-through debug session shows for one stage (issue #667, R383):
// the turn's recipe as Mermaid source with the stage highlighted; the
// `path:symbol` source location — with its line and an excerpt — of the code
// that emits the stage, in the Rust and in the JavaScript runtime
// (debug-stage-sources.mjs); and, as a separate field, the method-registry
// method the turn's route resolves to with its handler in both runtimes.
// Mirrors rust/src/server/debug_stage.rs.
//
// Everything is derived from live data, nothing is recorded by hand:
//   - the method is `MethodRegistry::method_for_route` over the route the
//     turn's `formalize` stage names (else its `dispatch_handler` stage);
//   - the Rust symbol is the method's row in the `HANDLER_FUNCTIONS` table of
//     rust/src/solver_dispatch.rs, or the rule interpreter's `run_handler` for
//     a method data/seed/handler-rules.lino implements;
//   - the JavaScript symbol is the browser handler of
//     data/seed/browser-handler-precedence.lino named `try` + the method (case
//     and underscores aside), or the worker's `runHandlerRuleSet` for a rule set;
//   - a stage's emitter is the function data/meta/debug-stage-sources.lino
//     lists for the stage's `source_event` (debug-stage-sources.mjs `stageEmitter`);
//   - each symbol is located by its definition line in the source tree, so a
//     moved function is found where it now lives. A checkout without sources
//     records no location (empty `*_source`), never a guessed one.

import { methodForRoute, methodRegistry } from '../agentic/crate/method_registry.mjs';
import { defines, readStageSources, readText, stageEmitter, treeFiles } from './debug-stage-sources.mjs';
import { REPO_ROOT } from './lino.mjs';

/** Mirrors `const DISPATCH_TABLE`. */
const DISPATCH_TABLE = 'rust/src/solver_dispatch.rs';
const DISPATCH_TABLE_START = 'const HANDLER_FUNCTIONS';
const DISPATCH_TABLE_END = '];';
const RULES_SEED = 'data/seed/handler-rules.lino';
const BROWSER_SEED = 'data/seed/browser-handler-precedence.lino';
const SEED_HANDLER_PREFIX = '  handler ';
const RUST_TREE = 'rust/src';
const JS_TREE = 'js/worker';
const RUST_RULE_RUNNER = { path: 'rust/src/rule_interpreter.rs', symbol: 'run_handler' };
const JS_RULE_RUNNER = 'runHandlerRuleSet';
const BROWSER_HANDLER_PREFIX = 'try';
const ROUTE_STEPS = ['formalize', 'dispatch_handler'];
const RUST = 'rust';
const JS = 'js';
const RUST_KEYWORD = 'fn';
const JS_KEYWORD = 'function';
/** Mirrors `const EXCERPT_LINES`: the most lines an excerpt shows. */
export const EXCERPT_LINES = 40;
const CURRENT_CLASS = 'current';
const CURRENT_STYLE = 'stroke-width:4px';

/** Mirrors `fn seed_handlers`: the `  handler <name>` rows of a seed document. */
function seedHandlers(text) {
  return (text ?? '').split('\n')
    .filter((line) => line.startsWith(SEED_HANDLER_PREFIX))
    .map((line) => line.slice(SEED_HANDLER_PREFIX.length).trim())
    .filter(Boolean);
}

/** Mirrors `fn dispatch_symbol`: the function the dispatch table binds `method` to. */
function dispatchSymbol(text, method) {
  const start = (text ?? '').indexOf(DISPATCH_TABLE_START);
  if (start < 0) return null;
  const end = text.indexOf(DISPATCH_TABLE_END, start);
  for (const line of text.slice(start, end < 0 ? text.length : end).split('\n')) {
    const row = line.trim();
    if (!row.startsWith('("')) continue;
    const close = row.indexOf('",', 2);
    if (close < 0 || row.slice(2, close) !== method) continue;
    const rest = row.slice(close + 2).trim();
    const symbol = rest.slice(0, rest.indexOf(')') < 0 ? rest.length : rest.indexOf(')')).trim();
    return symbol || null;
  }
  return null;
}

/** Mirrors `fn excerpt`: the definition through its closing brace, capped. */
function excerpt(lines, index) {
  const indent = lines[index].slice(0, lines[index].length - lines[index].trimStart().length);
  const out = [];
  for (let at = index; at < lines.length && out.length < EXCERPT_LINES; at += 1) {
    out.push(lines[at]);
    if (at > index && lines[at] === `${indent}}`) break;
  }
  return out.join('\n');
}

/**
 * Mirrors `fn locate_in`: the definition of `symbol` in the file `file`.
 * @returns {{path: string, symbol: string, line: number, excerpt: string} | null}
 */
function locateIn(root, file, keyword, symbol) {
  const lines = (readText(root, file) ?? '').split('\n');
  const index = lines.findIndex((line) => defines(line, keyword, symbol));
  return index < 0 ? null : { path: file, symbol, line: index + 1, excerpt: excerpt(lines, index) };
}

/** Mirrors `fn locate`: the first definition of `symbol` in the sorted tree. */
function locate(root, tree, extension, keyword, symbol) {
  for (const file of treeFiles(root, tree, extension)) {
    const found = locateIn(root, file, keyword, symbol);
    if (found) return found;
  }
  return null;
}

/** Mirrors `fn rust_location`. */
export function rustLocation(method, root = REPO_ROOT) {
  const symbol = dispatchSymbol(readText(root, DISPATCH_TABLE), method);
  if (symbol) return locate(root, RUST_TREE, '.rs', RUST_KEYWORD, symbol);
  if (!seedHandlers(readText(root, RULES_SEED)).includes(method)) return null;
  return locateIn(root, RUST_RULE_RUNNER.path, RUST_KEYWORD, RUST_RULE_RUNNER.symbol);
}

/** Mirrors `fn js_location`. */
export function jsLocation(method, root = REPO_ROOT) {
  const wanted = `${BROWSER_HANDLER_PREFIX}${method.replaceAll('_', '')}`.toLowerCase();
  const handler = seedHandlers(readText(root, BROWSER_SEED)).find((name) => name.toLowerCase() === wanted);
  const symbol = handler ?? (seedHandlers(readText(root, RULES_SEED)).includes(method) ? JS_RULE_RUNNER : null);
  return symbol ? locate(root, JS_TREE, '.js', JS_KEYWORD, symbol) : null;
}

/** Mirrors `fn turn_method`: the registry method the turn's route resolves to. */
export function turnMethod(stages) {
  for (const kind of ROUTE_STEPS) {
    const stage = stages.find((candidate) => String(candidate?.step ?? '') === kind);
    const route = String(stage?.detail ?? '').trim();
    if (route) return methodForRoute(methodRegistry(), route)?.name ?? '';
  }
  return '';
}

const cache = new Map();

/** One lookup per process and key. */
function cached(key, compute) {
  if (!cache.has(key)) cache.set(key, compute());
  return cache.get(key);
}

/**
 * Mirrors `fn stage_location`: where the code that emits a stage of event
 * `kind` is defined in `runtime`, given the routed handler's symbol there.
 */
export function stageLocation(runtime, kind, handler, root = REPO_ROOT) {
  const sources = cached(`${root}\u0000sources`, () => readStageSources(root));
  const emitter = stageEmitter(sources, runtime, kind, handler, root);
  const keyword = runtime === RUST ? RUST_KEYWORD : JS_KEYWORD;
  return emitter ? locateIn(root, emitter.path, keyword, emitter.symbol) : null;
}

/**
 * Mirrors `fn describe_turn`: the routed method with its handler in both
 * runtimes, and for every stage the code that emits it in both runtimes.
 * Each location is looked up once per process.
 */
export function describeTurn(stages, root = REPO_ROOT) {
  const method = turnMethod(stages);
  const handlers = method
    ? cached(`${root}\u0000method\u0000${method}`, () => ({ rust: rustLocation(method, root), js: jsLocation(method, root) }))
    : { rust: null, js: null };
  const at = (runtime, kind, handler) => cached(`${root}\u0000stage\u0000${runtime}\u0000${kind}\u0000${handler}`,
    () => (kind ? stageLocation(runtime, kind, handler, root) : null));
  return {
    method,
    ...handlers,
    stages: stages.map((stage) => {
      const kind = String(stage?.source_event ?? '');
      return { rust: at(RUST, kind, handlers.rust?.symbol ?? ''), js: at(JS, kind, handlers.js?.symbol ?? '') };
    }),
  };
}

/** Mirrors `fn label`: a Mermaid node label, quotes entity-escaped. */
const label = (text) => String(text).replaceAll('"', '#quot;');

/**
 * Mirrors `fn stage_diagram`: the turn's stages as a Mermaid flowchart, top
 * level stages chained in order, a sub-stage hanging off its parent, and the
 * stage at `current` highlighted.
 */
export function stageDiagram(stages, current) {
  const lines = ['flowchart TD'];
  stages.forEach((stage, index) => lines.push(`    s${index}["${index} ${label(stage?.step ?? '')}"]`));
  let previous = -1;
  stages.forEach((stage, index) => {
    const parent = stage?.parent_id ? stages.findIndex((candidate) => candidate?.id === stage.parent_id) : -1;
    if (parent >= 0) {
      lines.push(`    s${parent} -.-> s${index}`);
      return;
    }
    if (previous >= 0) lines.push(`    s${previous} --> s${index}`);
    previous = index;
  });
  lines.push(`    classDef ${CURRENT_CLASS} ${CURRENT_STYLE}`);
  if (current >= 0 && current < stages.length) lines.push(`    class s${current} ${CURRENT_CLASS}`);
  return lines.join('\n');
}

/** Mirrors `fn location_fields`: the event fields of one runtime's location. */
export function locationFields(prefix, location) {
  return {
    [`${prefix}_source`]: location ? `${location.path}:${location.symbol}` : '',
    [`${prefix}_line`]: location ? location.line : 0,
    [`${prefix}_excerpt`]: location ? location.excerpt : '',
  };
}
