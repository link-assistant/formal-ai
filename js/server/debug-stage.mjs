// What a step-through debug session shows for one stage (issue #667, R383):
// the turn's recipe as Mermaid source with the stage highlighted, and the
// method-registry `path:symbol` source location — with its line and an
// excerpt — of the handler the turn's route resolves to, in the Rust and in
// the JavaScript runtime. Mirrors rust/src/server/debug_stage.rs.
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
//   - each symbol is located by its definition line in the source tree, so a
//     moved function is found where it now lives. A checkout without sources
//     records no location (empty `*_source`), never a guessed one.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';

import { methodForRoute, methodRegistry } from '../agentic/crate/method_registry.mjs';
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
const DEFINITION_MODIFIERS = ['pub(crate) ', 'pub(super) ', 'pub ', 'export ', 'async ', 'const ', 'unsafe '];
/** Mirrors `const EXCERPT_LINES`: the most lines an excerpt shows. */
export const EXCERPT_LINES = 40;
const CURRENT_CLASS = 'current';
const CURRENT_STYLE = 'stroke-width:4px';

const readText = (root, relative) => {
  try {
    return readFileSync(path.join(root, relative), 'utf8');
  } catch {
    return null;
  }
};

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

/** Mirrors `fn defines`: whether `line` defines `symbol` with `keyword`. */
function defines(line, keyword, symbol) {
  let rest = line.trim();
  for (let stripped = true; stripped;) {
    stripped = false;
    for (const modifier of DEFINITION_MODIFIERS) {
      if (rest.startsWith(modifier)) {
        rest = rest.slice(modifier.length);
        stripped = true;
      }
    }
  }
  const head = `${keyword} ${symbol}`;
  return rest.startsWith(head) && ['(', '<'].includes(rest.charAt(head.length));
}

/** Mirrors `fn tree_files`: the files under `relative` ending in `extension`, sorted. */
function treeFiles(root, relative, extension) {
  let entries;
  try {
    entries = readdirSync(path.join(root, relative), { withFileTypes: true });
  } catch {
    return [];
  }
  return entries.flatMap((entry) => {
    const child = `${relative}/${entry.name}`;
    if (entry.isDirectory()) return treeFiles(root, child, extension);
    return entry.name.endsWith(extension) ? [child] : [];
  }).sort();
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
 * Mirrors `fn locate`: the first definition of `symbol` in the sorted tree.
 * @returns {{path: string, symbol: string, line: number, excerpt: string} | null}
 */
function locate(root, tree, extension, keyword, symbol) {
  for (const file of treeFiles(root, tree, extension)) {
    const lines = (readText(root, file) ?? '').split('\n');
    const index = lines.findIndex((line) => defines(line, keyword, symbol));
    if (index >= 0) return { path: file, symbol, line: index + 1, excerpt: excerpt(lines, index) };
  }
  return null;
}

/** Mirrors `fn rust_location`. */
export function rustLocation(method, root = REPO_ROOT) {
  const symbol = dispatchSymbol(readText(root, DISPATCH_TABLE), method);
  if (symbol) return locate(root, RUST_TREE, '.rs', 'fn', symbol);
  if (!seedHandlers(readText(root, RULES_SEED)).includes(method)) return null;
  const lines = (readText(root, RUST_RULE_RUNNER.path) ?? '').split('\n');
  const index = lines.findIndex((line) => defines(line, 'fn', RUST_RULE_RUNNER.symbol));
  return index < 0 ? null : { ...RUST_RULE_RUNNER, line: index + 1, excerpt: excerpt(lines, index) };
}

/** Mirrors `fn js_location`. */
export function jsLocation(method, root = REPO_ROOT) {
  const wanted = `${BROWSER_HANDLER_PREFIX}${method.replaceAll('_', '')}`.toLowerCase();
  const handler = seedHandlers(readText(root, BROWSER_SEED)).find((name) => name.toLowerCase() === wanted);
  const symbol = handler ?? (seedHandlers(readText(root, RULES_SEED)).includes(method) ? JS_RULE_RUNNER : null);
  return symbol ? locate(root, JS_TREE, '.js', 'function', symbol) : null;
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

const locations = new Map();

/**
 * Mirrors `fn describe_turn`: the method and both source locations, once per
 * turn (each location is looked up once per process and method).
 */
export function describeTurn(stages, root = REPO_ROOT) {
  const method = turnMethod(stages);
  if (!method) return { method, rust: null, js: null };
  const key = `${root}\u0000${method}`;
  if (!locations.has(key)) locations.set(key, { rust: rustLocation(method, root), js: jsLocation(method, root) });
  return { method, ...locations.get(key) };
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
