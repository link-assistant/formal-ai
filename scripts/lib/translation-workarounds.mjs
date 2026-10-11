// Temporary workarounds of the js -> rust translation (R1188-U30).
//
// Where the pinned meta-language cannot yet translate a construct, or emits
// Rust that does not hold together across modules, a recorded workaround in
// this repository completes the translation. Each one is a row of
// data/meta/translation-workarounds.lino with the upstream issue that will
// retire it; the projection header names the workarounds a module needed, and
// the ledger counts the items they translated apart from upstream's own.
//
// The workarounds here:
//   import-pruning   upstream translates `import { a, b } from './m.mjs'` to
//                    `use crate::m::{a, b};` even when `m` carries `b`, so
//                    the `use` names an item the Rust does not have. The
//                    unbound names are dropped (no translated item uses them:
//                    upstream carries every item that names an unbound
//                    import); an import left with no name is carried.
//   crate-assembly   upstream roots a module's own helpers at the crate
//                    (`crate::ml_*`) while its imports name sibling modules
//                    (`use crate::m::…`), so no single crate holds both. The
//                    compile check (`--compile`, CI only) assembles each root
//                    as one crate of modules and re-roots each module's own
//                    paths at the module; the committed projections are not
//                    changed.

import { createHash } from 'node:crypto';

const sha256 = (text) => createHash('sha256').update(text, 'utf8').digest('hex');

export const WORKAROUNDS_FILE = 'data/meta/translation-workarounds.lino';
/** The marker of a block a workaround translated or completed. */
export const WORKAROUND = '// formal-ai:workaround ';
/** The header line naming the workarounds a projection holds. */
export const APPLIED = '// formal-ai:workarounds ';

// ---------------------------------------------------------------- the record

/**
 * The workaround records: `workaround <name>` with `lifts`, `upstream`,
 * `scope` and `how` children. The workarounds that rewrite the source before
 * translation are grouped under `lowering`, which states their scope once.
 * @param {string} text the contents of data/meta/translation-workarounds.lino
 * @returns {{ name: string, lifts: string, upstream: string, scope: string, how: string }[]}
 */
export function readWorkarounds(text) {
  const records = [];
  let current = null;
  let group = null;
  for (const line of text.split('\n')) {
    if (/^\S/u.test(line)) group = line.trim();
    const head = /^( *)workaround (\S+)$/u.exec(line);
    if (head) {
      const nested = head[1].length > 0;
      if (nested && group !== 'lowering') {
        current = null;
        continue;
      }
      current = { name: head[2], lifts: '', upstream: '', scope: nested ? 'lowering' : '', how: '', depth: head[1].length + 2 };
      records.push(current);
      continue;
    }
    const field = /^( +)(lifts|upstream|scope|how) "((?:[^"\\]|\\.)*)"$/u.exec(line);
    if (field && current && field[1].length === current.depth) current[field[2]] = field[3].replace(/\\"/gu, '"');
    else if (!/^\s*(#.*)?$/u.test(line)) current = null;
  }
  return records.map(({ depth, ...record }) => record);
}

// ---------------------------------------------------------------- the import graph

/**
 * The relative named imports of a module: `import { a, b as c } from './m.mjs'`.
 * @param {string} source
 * @returns {{ specifier: string, names: { imported: string, local: string }[] }[]}
 */
export function namedImports(source) {
  return [...source.matchAll(/^import\s*\{([^}]*)\}\s*from\s*'(\.{1,2}\/[^']+)'\s*;?/gmu)].map((match) => ({
    specifier: match[2],
    names: match[1].split(',').map((entry) => entry.trim()).filter(Boolean).map((entry) => {
      const [imported, local] = entry.split(/\s+as\s+/u);
      return { imported, local: local ?? imported };
    }),
  }));
}

/**
 * The module a relative specifier names, when it is one of `modules` in the
 * same directory (meta-language roots each module's crate at its directory,
 * and refuses an import above it).
 * @param {string} path the importing module
 * @param {string} specifier
 * @param {Set<string>} modules
 * @returns {string|null}
 */
export function resolveImport(path, specifier, modules) {
  if (!specifier.startsWith('./') || specifier.slice(2).includes('/')) return null;
  const target = `${path.slice(0, path.lastIndexOf('/'))}/${specifier.slice(2)}`;
  return modules.has(target) ? target : null;
}

/**
 * Each module's modules in scope that it imports.
 * @param {Map<string, string>} sources path -> source
 * @returns {Map<string, string[]>}
 */
export function importGraph(sources) {
  const modules = new Set(sources.keys());
  return new Map([...sources].map(([path, source]) => [path, [...new Set(namedImports(source)
    .map((entry) => resolveImport(path, entry.specifier, modules)).filter(Boolean))]]));
}

/**
 * The modules to translate together with `named`: what they import (whose
 * signatures bind their names) and what imports them (whose translation the
 * signatures change), each transitively.
 * @param {string[]} named
 * @param {Map<string, string[]>} graph
 * @returns {string[]}
 */
export function translationClosure(named, graph) {
  const importers = new Map();
  for (const [path, deps] of graph) for (const dep of deps) importers.set(dep, [...(importers.get(dep) ?? []), path]);
  const out = new Set();
  const seen = new Set();
  const visit = (start, edges, tag) => {
    const stack = [start];
    while (stack.length) {
      const path = stack.pop();
      if (seen.has(`${tag}:${path}`)) continue;
      seen.add(`${tag}:${path}`);
      out.add(path);
      stack.push(...(edges.get(path) ?? []));
    }
  };
  // What imports them first, then everything those import.
  for (const path of named) visit(path, importers, 'importers');
  for (const path of [...out]) visit(path, graph, 'imports');
  return [...out].sort();
}

// ---------------------------------------------------------------- import-pruning

/**
 * Prunes a translated import block: the names its module's signatures do not
 * bind are dropped from the `use`. Returns the block unchanged when every
 * name is bound, a workaround block when some are, and a carried block (with
 * its refusal) when none is.
 * @param {{ kind: string, text: string, source: string }} block a translated import_statement block
 * @param {Set<string>|null} bound the names the imported module's signatures give, null when it is not in scope
 * @returns {{ kind: string, text?: string, marker?: string, refusal?: string, rule?: string }}
 */
export function pruneImport(block, bound) {
  const [entry] = namedImports(block.source);
  const lines = block.text.split('\n');
  const useAt = lines.findIndex((line) => line.startsWith('use crate::'));
  if (!entry || useAt < 0) return block;
  const keep = entry.names.map((name) => bound !== null && bound.has(name.imported));
  if (keep.every(Boolean)) return block;
  const marker = lines[0].replace(/^\/\/ meta-language:translated (\S+) (\S+).*$/u, '$1 $2');
  if (!keep.some(Boolean)) {
    return { kind: 'carried', marker: `${WORKAROUND}import-pruning carried ${marker}`, refusal: 'import of names its module does not translate', source: block.source, term: block.term };
  }
  const match = /^use (crate::[\w:]+?)::(?:\{(.*)\}|(\w+(?: as \w+)?));$/u.exec(lines[useAt]);
  const entries = match ? (match[2] ?? match[3]).split(', ') : [];
  if (entries.length !== entry.names.length) {
    return { kind: 'carried', marker: `${WORKAROUND}import-pruning carried ${marker}`, refusal: 'import the workaround cannot read', source: block.source, term: block.term };
  }
  const kept = entries.filter((_, index) => keep[index]);
  const code = `use ${match[1]}::${kept.length === 1 ? kept[0] : `{${kept.join(', ')}}`};`;
  const source = lines.slice(1, useAt);
  return {
    kind: 'workaround',
    rule: 'import-pruning',
    term: block.term,
    source: block.source,
    text: [`${WORKAROUND}import-pruning ${marker} items=1 sha256=${sha256(code)}`, ...source, code].join('\n'),
  };
}

// ---------------------------------------------------------------- crate-assembly

/**
 * Re-roots one module's own crate paths at the module: `crate::x` becomes
 * `crate::<module>::x` unless `x` is another module of the crate (an import),
 * outside string and character literals and comments.
 * @param {string} text the module's projection
 * @param {string} module its name in the crate
 * @param {Set<string>} modules every module of the crate
 * @returns {string}
 */
export function rerootModule(text, module, modules) {
  const own = new Set([...text.matchAll(/\b(?:fn|struct|enum|mod|const|static|type|trait)\s+([A-Za-z_]\w*)/gu)].map((match) => match[1]));
  let out = '';
  let index = 0;
  while (index < text.length) {
    const rest = text.slice(index);
    const skip = /^(?:\/\/[^\n]*|"(?:[^"\\]|\\.)*"|'(?:[^'\\\n]|\\[^\n]{1,10})')/su.exec(rest);
    if (skip) {
      out += skip[0];
      index += skip[0].length;
      continue;
    }
    const path = /^\bcrate::([A-Za-z_]\w*)/u.exec(rest);
    if (path && !/\w/u.test(text[index - 1] ?? '')) {
      const segment = path[1];
      out += modules.has(segment) && !own.has(segment) ? path[0] : `crate::${module}::${segment}`;
      index += path[0].length;
      continue;
    }
    out += text[index];
    index += 1;
  }
  return out;
}

/** The crate-level attributes every assembled crate carries (meta-language's own). */
export const CRATE_ALLOW = '#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons, dead_code)]';

/**
 * One crate of modules from the projections of one root: the crate root's
 * text and each module file's text, re-rooted.
 * @param {{ module: string, text: string }[]} projections
 * @param {string[]} extra lines after the module declarations (a `fn main`)
 * @returns {{ root: string, files: { name: string, text: string }[] }}
 */
export function assembleCrate(projections, extra = []) {
  const modules = new Set(projections.map((entry) => entry.module));
  return {
    root: [CRATE_ALLOW, ...projections.map((entry) => `pub mod ${entry.module};`), ...extra, ''].join('\n'),
    files: projections.map((entry) => ({ name: `${entry.module}.rs`, text: rerootModule(entry.text, entry.module, modules) })),
  };
}
