// Which code emits each stage of a debugged turn (issue #667, R383), in the
// Rust and in the JavaScript runtime. Mirrors rust/src/server/debug_stage_sources.rs.
//
// A stage is a curated solver event (`source_event`: `impulse`, `language`,
// `calculation:engine`, `response`, ...). data/meta/debug-stage-sources.lino,
// written by scripts/generate-debug-stage-sources.mjs from the source tree and
// checked current in CI, lists for every curated event kind the functions that
// append it in each runtime, and each runtime's source trees and solver entry.
// When one function appends a kind, it is the stage's emitter. When several
// do, the emitter is the one reachable in the fewest calls (at most
// `REACH_DEPTH`) from the turn's routed handler, else from the solver entry;
// a kind none of them reaches records no location, never a guessed one. Calls
// are read by name from the function bodies of the runtime's trees.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';

/** Mirrors `const STAGE_SOURCES`. */
export const STAGE_SOURCES = 'data/meta/debug-stage-sources.lino';
/** Mirrors `const REACH_DEPTH`: the most calls between a root and an emitter. */
export const REACH_DEPTH = 2;
const DEFINITION_MODIFIERS = ['pub(crate) ', 'pub(super) ', 'pub ', 'export ', 'async ', 'const ', 'unsafe '];
const IDENTIFIER = /^[A-Za-z_$][A-Za-z0-9_$]*/;
const CALL = /([A-Za-z_$][A-Za-z0-9_$]*)\s*\(/g;
const BODY_LINES = 4000;
/** Mirrors `const ROW_INDENT`: the indentation of a row inside the table's root record. */
const ROW_INDENT = '  ';

export const readText = (root, relative) => {
  try {
    return readFileSync(path.join(root, relative), 'utf8');
  } catch {
    return null;
  }
};

/** Mirrors `fn defined_name`: the name `line` defines with `keyword`, or null. */
export function definedName(line, keyword) {
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
  if (!rest.startsWith(`${keyword} `)) return null;
  const name = IDENTIFIER.exec(rest.slice(keyword.length + 1))?.[0];
  if (!name) return null;
  return ['(', '<'].includes(rest.charAt(keyword.length + 1 + name.length)) ? name : null;
}

/** Mirrors `fn defines`: whether `line` defines `symbol` with `keyword`. */
export const defines = (line, keyword, symbol) => definedName(line, keyword) === symbol;

/** Mirrors `fn tree_files`: the files under `relative` ending in `extension`, sorted. */
export function treeFiles(root, relative, extension) {
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

/** Mirrors `fn body_end`: the index of the line closing the definition at `index`. */
export function bodyEnd(lines, index) {
  const first = lines[index];
  const indent = first.slice(0, first.length - first.trimStart().length);
  const opens = first.split('{').length - 1;
  if (opens > 0 && opens === first.split('}').length - 1) return index;
  for (let at = index + 1; at < lines.length && at - index < BODY_LINES; at += 1) {
    if (lines[at] === `${indent}}`) return at;
  }
  return index;
}

/**
 * Mirrors `fn read_stage_sources`: the runtimes (keyword, trees, entries) and
 * the per-kind emitters of the stage-sources table, or null without one.
 */
export function readStageSources(root) {
  const text = readText(root, STAGE_SOURCES);
  if (text === null) return null;
  const runtimes = new Map();
  const events = new Map();
  let runtime = null;
  let event = null;
  for (const line of text.split('\n')) {
    if (!line.startsWith(ROW_INDENT) || line.trimStart().startsWith('#')) continue;
    const depth = line.length - line.trimStart().length;
    const [head, ...values] = line.trim().split(/\s+/).map((token) => token.replace(/^"(.*)"$/, '$1'));
    if (depth === 2 && head === 'runtime') {
      runtime = { name: values[0], keyword: '', trees: [], entries: [] };
      runtimes.set(values[0], runtime);
      event = null;
    } else if (depth === 2 && head === 'event') {
      event = { kind: values[0], step: '', sites: new Map() };
      events.set(values[0], event);
      runtime = null;
    } else if (depth === 4 && runtime) {
      if (head === 'keyword') runtime.keyword = values[0];
      else if (head === 'tree') runtime.trees.push({ path: values[0], extension: values[1] });
      else if (head === 'entry') runtime.entries.push({ path: values[0], symbol: values[1] });
    } else if (depth === 4 && event) {
      if (head === 'step') event.step = values[0];
      else if (!event.sites.has(head)) event.sites.set(head, [{ path: values[0], symbol: values[1] }]);
      else event.sites.get(head).push({ path: values[0], symbol: values[1] });
    }
  }
  return { runtimes, events };
}

/**
 * Mirrors `struct FunctionIndex`: every function the runtime's trees define,
 * by name, with the names its body calls.
 */
export class FunctionIndex {
  constructor(root, runtime) {
    this.calls = new Map();
    for (const tree of runtime.trees) {
      for (const file of treeFiles(root, tree.path, tree.extension)) {
        const lines = (readText(root, file) ?? '').split('\n');
        lines.forEach((line, index) => {
          const name = definedName(line, runtime.keyword);
          if (!name) return;
          const body = lines.slice(index, bodyEnd(lines, index) + 1).join('\n');
          const called = this.calls.get(name) ?? new Set();
          for (const match of body.matchAll(CALL)) called.add(match[1]);
          this.calls.set(name, called);
        });
      }
    }
  }

  /**
   * Mirrors `fn nearest`: the first of `candidates` (in their order) at the
   * smallest call depth from `roots`, at most `REACH_DEPTH`, or null.
   */
  nearest(roots, candidates) {
    let level = new Set(roots);
    const seen = new Set(level);
    for (let depth = 0; depth <= REACH_DEPTH && level.size; depth += 1) {
      const hit = candidates.find((candidate) => level.has(candidate.symbol));
      if (hit) return hit;
      const next = new Set();
      for (const name of level) {
        for (const called of this.calls.get(name) ?? []) {
          if (!seen.has(called)) {
            seen.add(called);
            next.add(called);
          }
        }
      }
      level = next;
    }
    return null;
  }
}

const indexes = new Map();

/** Mirrors `fn function_index`: one index per process, root and runtime. */
function functionIndex(root, runtime) {
  const key = `${root}\u0000${runtime.name}`;
  if (!indexes.has(key)) indexes.set(key, new FunctionIndex(root, runtime));
  return indexes.get(key);
}

/**
 * Mirrors `fn stage_emitter`: the `{path, symbol}` of the function that emits
 * a stage of event `kind` in `runtimeName`, given the turn's routed handler
 * symbol in that runtime (empty when the turn has none), or null.
 */
export function stageEmitter(sources, runtimeName, kind, handler, root) {
  const candidates = sources?.events.get(kind)?.sites.get(runtimeName) ?? [];
  if (candidates.length < 2) return candidates[0] ?? null;
  const runtime = sources.runtimes.get(runtimeName);
  if (!runtime) return null;
  const index = functionIndex(root, runtime);
  return (handler ? index.nearest([handler], candidates) : null)
    ?? index.nearest(runtime.entries.map((entry) => entry.symbol), candidates);
}
