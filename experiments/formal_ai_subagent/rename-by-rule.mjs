#!/usr/bin/env node
// Renames files by rule, not by hand (PR #1188, R1188-U5).
//
// The rename map, `data/meta/rename-map.lino`, holds one `rename` link per old
// path and new path, each with the reason the new name says what the file
// holds. This script applies the map and keeps the repository consistent:
//
// - It moves every renamed file, and every companion that follows from it by
//   rule: the TypeScript twin (`js/x.js` -> `ts/x.ts`, `.mjs` -> `.mts`), the
//   `rust/embedded/` mirror, the worker line-budget shard of a browser worker
//   module (`data/meta/worker-line-budget/<stem>.lino`) and the self-AST
//   census of a Rust source (`data/meta/self-ast/src/<path>.lino`) and the
//   files of a Rust module's own directory (`x.rs` brings `x/child.rs`). The
//   map states each rename once; the companions are not repeated in it.
// - It rewrites every reference to an old path across the repository: import
//   and require paths, `include_str!`/`include_bytes!` and `#[path]`
//   arguments, workflow paths, seed lists, budgets, documentation links and
//   test fixtures. A reference is a path token that names the old file: the
//   whole path, a tail of it that no other file shares (`worker/x.js`,
//   `x.js`), or a relative path that resolves to it from the referencing
//   file. A token that names a different file with the same tail is left
//   alone. In Rust sources it also renames the module: `mod old;` and
//   `old::` paths in the same source tree.
// - It re-sorts the generated one-entry-per-line lists the map names under
//   `resort`, so a renamed entry lands where the list's generator puts it.
//
// `--check` changes nothing. It fails when an old path still exists, a new
// path is missing, or any file outside the map's `exclude` prefixes still
// references an old path.
//
// Files are moved on the filesystem by default: other agents commit from the
// same working tree, and git detects the rename from the content when the move
// is committed. `--git-mv` moves with `git mv` instead.
//
// Usage:
//   node experiments/formal_ai_subagent/rename-by-rule.mjs [--tree <name>] [--dry-run] [--git-mv]
//   node experiments/formal_ai_subagent/rename-by-rule.mjs --check [--tree <name>]
//   node experiments/formal_ai_subagent/rename-by-rule.mjs --list [--tree <name>]
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, renameSync, rmdirSync, statSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, posix, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const MAP = 'data/meta/rename-map.lino';
const LARGEST_SCANNED_FILE = 4 * 1024 * 1024;
const PATH_CHARACTER = /[A-Za-z0-9_\-./~@+]/u;
const WORD_CHARACTER = /[A-Za-z0-9_]/u;

/** Resolve historical destination chains so later area moves preserve old rules. */
export function resolveRenameChains(renames, allRenames = renames) {
  const destinations = new Map(allRenames.map((rename) => [rename.from, rename.to]));
  return renames.map((rename) => {
    const seen = new Set([rename.from]);
    let destination = rename.to;
    while (destinations.has(destination)) {
      if (seen.has(destination)) throw new Error(`rename cycle at ${destination}`);
      seen.add(destination);
      destination = destinations.get(destination);
    }
    if (seen.has(destination)) throw new Error(`rename cycle at ${destination}`);
    return { ...rename, to: destination };
  });
}
/**
 * The directories whose Rust files are the top modules of a test binary, so
 * their parent module is not written in a test path (`rust/tests/x.rs` is the
 * binary `x`; `rust/tests/unit/x.rs` is `x::` in the `unit` binary).
 */
const TEST_BINARY_ROOTS = ['rust/tests', 'rust/tests/unit', 'rust/tests/integration', 'rust/tests/source'];

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

/** One indented links-notation line: its depth, head word and quoted value. */
function parseLine(line) {
  const depth = line.length - line.trimStart().length;
  const text = line.trim();
  if (text === '' || text.startsWith('#')) return null;
  const space = text.indexOf(' ');
  const head = space < 0 ? text : text.slice(0, space);
  let value = space < 0 ? '' : text.slice(space + 1).trim();
  if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) value = value.slice(1, -1).replaceAll('\\"', '"');
  return { depth, head, value };
}

/**
 * The rename map: `{exclude, resort, renames}`, each rename
 * `{tree, from, to, reason, review}`.
 * @param {string} source
 */
export function parseRenameMap(source) {
  const map = { exclude: [], resort: [], renames: [] };
  let tree = '';
  let current = null;
  let renameDepth = -1;
  for (const raw of source.split('\n')) {
    const line = parseLine(raw);
    if (!line) continue;
    if (current && line.depth <= renameDepth) current = null;
    if (current) {
      if (['from', 'to', 'reason', 'review'].includes(line.head)) current[line.head] = line.value;
      continue;
    }
    if (line.head === 'exclude') map.exclude.push(line.value);
    else if (line.head === 'resort') map.resort.push(line.value);
    else if (line.head === 'tree') tree = line.value;
    else if (line.head === 'rename') {
      current = { tree, from: '', to: '', reason: '', review: '' };
      renameDepth = line.depth;
      map.renames.push(current);
    }
  }
  for (const rename of map.renames) {
    if (!rename.from || !rename.to) throw new Error(`${MAP}: a rename in tree ${rename.tree} lacks from or to`);
  }
  return map;
}

/** Load the map and named shards without repeating shared exclusions. */
export function loadRenameMap(root = ROOT) {
  const visiting = new Set();
  const load = (path) => {
    if (visiting.has(path)) throw new Error(`rename-map include cycle at ${path}`);
    visiting.add(path);
    const source = readFileSync(join(root, path), 'utf8');
    const map = parseRenameMap(source);
    map.exclude.push(path);
    for (const match of source.matchAll(/^\s*include "([^"]+)"\s*$/gmu)) {
      const shard = load(match[1]);
      map.exclude.push(...shard.exclude);
      map.resort.push(...shard.resort);
      map.renames.push(...shard.renames);
    }
    visiting.delete(path);
    return map;
  };
  return load(MAP);
}

/** The TypeScript twin path of a `js/` source, or null. */
export function typescriptTwin(path) {
  if (!path.startsWith('js/')) return null;
  const rest = path.slice('js/'.length);
  if (rest.endsWith('.mjs')) return `ts/${rest.slice(0, -'.mjs'.length)}.mts`;
  if (rest.endsWith('.jsx')) return `ts/${rest.slice(0, -'.jsx'.length)}.tsx`;
  if (rest.endsWith('.js')) return `ts/${rest.slice(0, -'.js'.length)}.ts`;
  return null;
}

/** The line-budget shard of a browser worker module, or null. */
export function workerBudgetShard(path) {
  const match = /^js\/worker\/([^/]+)\.js$/u.exec(path);
  return match ? `data/meta/worker-line-budget/${match[1]}.lino` : null;
}

/** The self-AST census file of a Rust source, or null. */
export function selfAstCensus(path) {
  const match = /^rust\/(src\/.+)\.rs$/u.exec(path);
  return match ? `data/meta/self-ast/${match[1]}.lino` : null;
}

/**
 * Every move a rename implies: the rename itself and each companion that
 * exists on either side.
 * @param {{from: string, to: string}} rename
 * @param {(path: string) => boolean} exists
 */
export function companionMoves(rename, exists) {
  const moves = [{ from: rename.from, to: rename.to, companion: '' }];
  const derive = (kind, mapping) => {
    for (const move of moves.slice()) {
      const from = mapping(move.from);
      const to = mapping(move.to);
      if (from && to && from !== to && (exists(from) || exists(to))) moves.push({ from, to, companion: kind });
    }
  };
  derive('typescript twin', typescriptTwin);
  derive('worker line budget', workerBudgetShard);
  derive('self-AST census', selfAstCensus);
  derive('embedded mirror', (path) => `rust/embedded/${path}`);
  return moves;
}

// ---------------------------------------------------------------------------
// References
// ---------------------------------------------------------------------------

function segments(path) {
  return path.split('/');
}

/** The directory both paths share, and the rest of each below it. */
export function splitCommonDirectory(from, to) {
  const left = segments(from);
  const right = segments(to);
  let shared = 0;
  while (shared < left.length - 1 && shared < right.length - 1 && left[shared] === right[shared]) shared += 1;
  return { fromRest: left.slice(shared).join('/'), toRest: right.slice(shared).join('/') };
}

/** Whether `tail` is `path` or a whole-segment tail of it. */
function isSegmentTail(tail, path) {
  return tail === path || path.endsWith(`/${tail}`);
}

/**
 * Index tracked paths by basename, so a short reference can be told apart
 * from a different file that shares its tail.
 * @param {Array<string>} paths
 */
export function indexByBasename(paths) {
  const index = new Map();
  for (const path of new Set(paths)) {
    const name = basename(path);
    if (!index.has(name)) index.set(name, []);
    index.get(name).push(path);
  }
  return index;
}

/**
 * Whether the path token `token`, written in `file`, names `target`.
 * @param {string} token
 * @param {string} file
 * @param {string} target
 * @param {Map<string, Array<string>>} byBasename
 */
export function tokenNames(token, file, target, byBasename) {
  if (token.startsWith('./') || token.startsWith('../')) {
    if (posix.normalize(posix.join(posix.dirname(file), token)) === target) return true;
  }
  const bare = token.replace(/^(?:\.{1,2}\/)+/u, '').replace(/^\/+/u, '');
  if (bare === '') return false;
  if (bare.endsWith(`/${target}`) || bare === target) return true;
  if (!isSegmentTail(bare, target)) return false;
  const sharing = (byBasename.get(basename(target)) ?? []).filter((path) => isSegmentTail(bare, path));
  if (sharing.length <= 1) return true;
  return posix.normalize(posix.join(posix.dirname(file), bare)) === target;
}

/** The token that replaces `token` when it names `from` and `from` becomes `to`. */
export function renamedToken(token, from, to) {
  const { fromRest, toRest } = splitCommonDirectory(from, to);
  if (token.endsWith(fromRest) && (token.length === fromRest.length || token[token.length - fromRest.length - 1] === '/')) {
    return token.slice(0, token.length - fromRest.length) + toRest;
  }
  // A tail shorter than the part that changes: keep as many segments.
  const kept = segments(token.replace(/^(?:\.{1,2}\/)+/u, '').replace(/^\/+/u, '')).length;
  const lead = token.slice(0, token.length - token.replace(/^(?:\.{1,2}\/)+/u, '').replace(/^\/+/u, '').length);
  const wanted = Math.max(1, kept + segments(toRest).length - segments(fromRest).length);
  return lead + segments(to).slice(-wanted).join('/');
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&');
}

/**
 * Every reference to a moved path in `text`, as `{start, end, token, move}`.
 * @param {string} text
 * @param {string} file
 * @param {Array<{from: string, to: string}>} moves
 * @param {Map<string, Array<string>>} byBasename
 */
export function findReferences(text, file, moves, byBasename) {
  const byName = new Map();
  for (const move of moves) {
    const name = basename(move.from);
    if (!byName.has(name)) byName.set(name, []);
    byName.get(name).push(move);
  }
  if (byName.size === 0) return [];
  const names = [...byName.keys()].sort((left, right) => right.length - left.length);
  const pattern = new RegExp(names.map(escapeRegExp).join('|'), 'gu');
  const found = [];
  for (const match of text.matchAll(pattern)) {
    const end = match.index + match[0].length;
    const after = text[end] ?? '';
    if (WORD_CHARACTER.test(after) || after === '-' || (after === '.' && WORD_CHARACTER.test(text[end + 1] ?? ''))) continue;
    let start = match.index;
    if (start > 0 && text[start - 1] !== '/' && PATH_CHARACTER.test(text[start - 1])) continue;
    while (start > 0 && PATH_CHARACTER.test(text[start - 1])) start -= 1;
    const token = text.slice(start, end);
    const move = byName.get(match[0]).find((candidate) => tokenNames(token, file, candidate.from, byBasename));
    if (move) found.push({ start, end, token, move });
  }
  return found;
}

/** The Rust module name a `.rs` path declares, or null. */
function rustModuleName(path) {
  if (!path.endsWith('.rs')) return null;
  const name = basename(path, '.rs');
  return ['mod', 'main', 'lib'].includes(name) ? basename(dirname(path)) : name;
}

/** The source tree a Rust file belongs to (`rust/src`, `rust/tests`, ...). */
function rustTree(path) {
  return segments(path).slice(0, 2).join('/');
}

/** Whether `file` is the Rust file that declares the modules of `directory`. */
function declaresModulesOf(file, directory) {
  if (file === `${directory}.rs`) return true;
  return posix.dirname(file) === directory && ['mod.rs', 'main.rs', 'lib.rs'].includes(basename(file));
}

/** The module name of the directory that holds a Rust file (`ci-cd` -> `ci_cd`). */
function parentModuleName(path) {
  return basename(dirname(path)).replaceAll('-', '_');
}

/**
 * Every `old::` module path that names a renamed Rust module, in any file:
 * a test filter or a test name such as `ci_cd::old::case` in a durations list,
 * a workflow or a document. An occurrence names the move when the module name
 * is unique among the Rust files of the repository, or, among namesakes, when
 * it is qualified by the parent module (`ci_cd::old::`) or, unqualified, when
 * exactly one namesake sits at the root of a test binary (its parent module is
 * not written in test paths). Inside a Rust source of the same source tree an
 * unqualified occurrence follows the source's own rule instead (see
 * `renameRustModules`).
 * @param {string} text
 * @param {string} file
 * @param {Array<{from: string, to: string}>} moves
 * @param {Map<string, Array<string>>} byBasename
 * @returns {Array<{start: number, end: number, from: string, to: string, move: object}>}
 */
export function findModuleReferences(text, file, moves, byBasename) {
  const found = [];
  for (const move of moves) {
    const oldName = rustModuleName(move.from);
    const newName = rustModuleName(move.to);
    if (!oldName || !newName || oldName === newName) continue;
    if (posix.dirname(move.from) !== posix.dirname(move.to)) continue;
    if (file.endsWith('.rs') && rustTree(move.from) === rustTree(file)) continue;
    const namesakes = (byBasename.get(`${oldName}.rs`) ?? []).filter((path) => !moves.some((other) => other !== move && other.to === path));
    const roots = namesakes.filter((path) => TEST_BINARY_ROOTS.includes(posix.dirname(path)));
    const pattern = new RegExp(`(^|[^A-Za-z0-9_])${escapeRegExp(oldName)}(?=::)`, 'gmu');
    for (const match of text.matchAll(pattern)) {
      const start = match.index + match[1].length;
      const qualifier = /([A-Za-z_][A-Za-z0-9_]*)::$/u.exec(text.slice(Math.max(0, start - 80), start));
      let names = namesakes.length <= 1;
      if (!names && qualifier) names = qualifier[1] === parentModuleName(move.from);
      else if (!names) names = roots.length === 1 && roots[0] === move.from;
      if (names) found.push({ start, end: start + oldName.length, from: oldName, to: newName, move });
    }
  }
  return found.sort((left, right) => left.start - right.start);
}

/**
 * Rename the Rust modules of `moves` in the Rust source `text` of `file`: the
 * `mod old;` line of the parent module (or a `mod old;` whose `#[path]`
 * attribute names the new file), and `old::` paths within the same source
 * tree when no other module of that tree shares the old name. Elsewhere,
 * `findModuleReferences` finds the `old::` paths.
 * @param {string} text
 * @param {string} file
 * @param {Array<{from: string, to: string}>} moves
 * @param {Map<string, Array<string>>} byBasename
 */
export function renameRustModules(text, file, moves, byBasename) {
  let count = 0;
  let result = text;
  for (const reference of findModuleReferences(text, file, moves, byBasename).reverse()) {
    result = result.slice(0, reference.start) + reference.to + result.slice(reference.end);
    count += 1;
  }
  if (!file.endsWith('.rs')) return { text: result, count };
  for (const move of moves) {
    const oldName = rustModuleName(move.from);
    const newName = rustModuleName(move.to);
    if (!oldName || !newName || oldName === newName || rustTree(move.from) !== rustTree(file)) continue;
    if (posix.dirname(move.from) !== posix.dirname(move.to)) continue;
    if (declaresModulesOf(file, posix.dirname(move.from))) {
      const declaration = new RegExp(`^(\\s*(?:pub(?:\\([a-z]+\\))?\\s+)?mod\\s+)${escapeRegExp(oldName)}(\\s*;)`, 'gmu');
      result = result.replace(declaration, (_whole, before, after) => {
        count += 1;
        return `${before}${newName}${after}`;
      });
    }
    const attributed = new RegExp(`(#\\[path\\s*=\\s*"([^"]*)"\\]\\s*(?:pub(?:\\([a-z]+\\))?\\s+)?mod\\s+)${escapeRegExp(oldName)}(\\s*;)`, 'gmu');
    result = result.replace(attributed, (whole, before, path, after) => {
      if (posix.normalize(posix.join(posix.dirname(file), path)) !== move.to && basename(path) !== basename(move.to)) return whole;
      count += 1;
      return `${before}${newName}${after}`;
    });
    const namesakes = (byBasename.get(`${oldName}.rs`) ?? []).filter((path) => rustTree(path) === rustTree(file));
    if (namesakes.length > 1) continue;
    const usePath = new RegExp(`(^|[^A-Za-z0-9_])${escapeRegExp(oldName)}(?=::)`, 'gmu');
    result = result.replace(usePath, (_whole, before) => {
      count += 1;
      return `${before}${newName}`;
    });
  }
  return { text: result, count };
}

/** The seed registry, which names each `data/seed/<stem>.lino` by its stem. */
const SEED_REGISTRY = 'data/meta/seed-registry.lino';

/**
 * Rename the `seed <stem>` entries of the seed registry whose seed file moved
 * within `data/seed/`; `scripts/generate-seed-registry.*` then regenerates the
 * embedded registries from it.
 * @param {string} text
 * @param {string} file
 * @param {Array<{from: string, to: string}>} moves
 */
export function renameSeedRegistryStems(text, file, moves) {
  if (file !== SEED_REGISTRY) return { text, count: 0 };
  let count = 0;
  let result = text;
  for (const move of moves) {
    const from = /^data\/seed\/([^/]+)\.lino$/u.exec(move.from);
    const to = /^data\/seed\/([^/]+)\.lino$/u.exec(move.to);
    if (!from || !to) continue;
    result = result.replace(new RegExp(`^(\\s*seed )${escapeRegExp(from[1])}$`, 'gmu'), (_whole, before) => {
      count += 1;
      return `${before}${to[1]}`;
    });
  }
  return { text: result, count };
}

/** Sort each run of one-entry-per-line quoted list items, as their generators do. */
export function resortQuotedLists(text) {
  const lines = text.split('\n');
  const item = /^\s*"[^"]*",\s*$/u;
  for (let start = 0; start < lines.length; start += 1) {
    if (!item.test(lines[start])) continue;
    let end = start;
    while (end < lines.length && item.test(lines[end])) end += 1;
    const sorted = lines.slice(start, end).sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
    lines.splice(start, end - start, ...sorted);
    start = end;
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// The repository
// ---------------------------------------------------------------------------

function repositoryFiles() {
  const output = execFileSync('git', ['ls-files', '-z', '-co', '--exclude-standard'], { cwd: ROOT, maxBuffer: 256 * 1024 * 1024 });
  return output.toString('utf8').split('\0').filter((path) => path !== '' && existsSync(join(ROOT, path)));
}

function readText(path) {
  const full = join(ROOT, path);
  const stats = statSync(full);
  if (!stats.isFile() || stats.size > LARGEST_SCANNED_FILE) return null;
  const buffer = readFileSync(full);
  if (buffer.includes(0)) return null;
  return buffer.toString('utf8');
}

function excluded(path, map) {
  return path === MAP || map.exclude.some((prefix) => path.startsWith(prefix));
}

function lineOf(text, offset) {
  let line = 1;
  for (let index = 0; index < offset; index += 1) if (text.charCodeAt(index) === 10) line += 1;
  return line;
}

function selectedRenames(map, tree) {
  return tree ? map.renames.filter((rename) => rename.tree === tree) : map.renames;
}

/** Every file below a directory of the repository, as repository paths. */
function filesBelow(directory) {
  const full = join(ROOT, directory);
  if (!existsSync(full) || !statSync(full).isDirectory()) return [];
  return readdirSync(full, { withFileTypes: true }).flatMap((entry) => {
    const path = `${directory}/${entry.name}`;
    return entry.isDirectory() ? filesBelow(path) : [path];
  });
}

/**
 * The files of a Rust module's own directory, which follow the module: the
 * child modules of `x.rs` live in `x/`, so renaming `x.rs` to `y.rs` moves
 * `x/child.rs` to `y/child.rs`.
 * @param {{from: string, to: string}} move
 */
export function moduleDirectoryMoves(move, below = filesBelow) {
  const name = basename(move.from, '.rs');
  if (!move.from.endsWith('.rs') || ['mod', 'main', 'lib'].includes(name)) return [];
  const fromDirectory = move.from.slice(0, -'.rs'.length);
  const toDirectory = move.to.slice(0, -'.rs'.length);
  const pending = below(fromDirectory).map((path) => ({ from: path, to: `${toDirectory}${path.slice(fromDirectory.length)}` }));
  const done = below(toDirectory).map((path) => ({ from: `${fromDirectory}${path.slice(toDirectory.length)}`, to: path }));
  const seen = new Set();
  return [...pending, ...done]
    .filter((child) => !seen.has(child.from) && seen.add(child.from))
    .map((child) => ({ ...child, companion: 'module directory' }));
}

function allMoves(renames) {
  const exists = (path) => existsSync(join(ROOT, path));
  return renames.flatMap((rename) => {
    const moves = companionMoves(rename, exists);
    const children = moves.flatMap((move) => moduleDirectoryMoves(move));
    return [...moves, ...children].map((move) => ({ ...move, rename }));
  });
}

function moveFile(move, options) {
  const from = join(ROOT, move.from);
  const to = join(ROOT, move.to);
  if (!existsSync(from)) return existsSync(to) ? 'already moved' : 'missing';
  if (existsSync(to)) throw new Error(`both ${move.from} and ${move.to} exist`);
  if (options.dryRun) return 'would move';
  mkdirSync(dirname(to), { recursive: true });
  if (options.gitMove) execFileSync('git', ['mv', move.from, move.to], { cwd: ROOT });
  else renameSync(from, to);
  // A module directory whose last file moved out goes with it.
  for (let directory = dirname(from); directory !== ROOT && readdirSync(directory).length === 0; directory = dirname(directory)) {
    rmdirSync(directory);
  }
  return 'moved';
}

function apply(map, moves, options) {
  for (const move of moves) {
    const outcome = moveFile(move, options);
    console.log(`  ${outcome.padEnd(13)} ${move.from} -> ${move.to}${move.companion ? `  (${move.companion})` : ''}`);
    if (outcome === 'missing') throw new Error(`${move.from} does not exist`);
  }
  const files = repositoryFiles();
  const byBasename = indexByBasename([...files, ...moves.map((move) => move.from)]);
  let rewritten = 0;
  const formatModuleOrder = [];
  for (const file of files) {
    if (excluded(file, map)) continue;
    const text = readText(file);
    if (text === null) continue;
    const references = findReferences(text, file, moves, byBasename);
    let next = text;
    for (const reference of references.reverse()) {
      next = next.slice(0, reference.start) + renamedToken(reference.token, reference.move.from, reference.move.to) + next.slice(reference.end);
    }
    const modules = renameRustModules(next, file, moves, byBasename);
    next = renameSeedRegistryStems(modules.text, file, moves).text;
    if (map.resort.includes(file)) next = resortQuotedLists(next);
    if (next !== text) {
      if (modules.count && file.endsWith('.rs') && !options.dryRun) formatModuleOrder.push(file);
      rewritten += 1;
      console.log(`  rewrote       ${file} (${references.length} path${references.length === 1 ? '' : 's'}${modules.count ? `, ${modules.count} module name${modules.count === 1 ? '' : 's'}` : ''})`);
      if (!options.dryRun) writeFileSync(join(ROOT, file), next);
    }
  }
  // rustfmt keeps each run of `mod` declarations sorted, so a renamed module
  // moves to its sorted place: format every Rust file whose module names
  // changed (they were rustfmt-clean before; CI checks they still are).
  for (const file of formatModuleOrder) {
    try {
      execFileSync('rustfmt', ['--edition', '2024', file], { cwd: ROOT, stdio: 'ignore' });
      console.log(`  formatted     ${file} (mod order)`);
    } catch {
      console.log(`  ::warning::rustfmt could not format ${file}; sort its mod lines by hand`);
    }
  }
  console.log(`\n${moves.length} move(s), ${rewritten} file(s) rewritten${options.dryRun ? ' (dry run)' : ''}.`);
}

function check(map, moves) {
  const failures = [];
  for (const move of moves) {
    if (existsSync(join(ROOT, move.from))) failures.push(`${move.from} still exists (renamed to ${move.to})`);
    if (!existsSync(join(ROOT, move.to))) failures.push(`${move.to} is missing (the new name of ${move.from})`);
  }
  const files = repositoryFiles();
  const byBasename = indexByBasename([...files, ...moves.map((move) => move.from)]);
  for (const file of files) {
    if (excluded(file, map)) continue;
    const text = readText(file);
    if (text === null) continue;
    for (const reference of findReferences(text, file, moves, byBasename)) {
      failures.push(`${file}:${lineOf(text, reference.start)} still references ${reference.move.from} (now ${reference.move.to})`);
    }
    for (const reference of findModuleReferences(text, file, moves, byBasename)) {
      failures.push(`${file}:${lineOf(text, reference.start)} still names the module ${reference.from}:: (now ${reference.to}::)`);
    }
    if (renameSeedRegistryStems(text, file, moves).count > 0) {
      failures.push(`${file} still names a renamed seed by its old stem`);
    }
  }
  if (failures.length > 0) {
    for (const failure of failures) console.error(`::error::${failure}`);
    console.error(`\n${failures.length} stale rename(s); run: node experiments/formal_ai_subagent/rename-by-rule.mjs`);
    process.exit(1);
  }
  console.log(`rename map: ${moves.length} move(s) applied, no stale references.`);
}

function main() {
  const args = process.argv.slice(2);
  const treeAt = args.indexOf('--tree');
  const tree = treeAt >= 0 ? args[treeAt + 1] : '';
  const map = loadRenameMap();
  const renames = selectedRenames(map, tree);
  if (renames.length === 0) {
    console.error(tree ? `${MAP} has no renames in tree ${tree}` : `${MAP} has no renames`);
    process.exit(tree ? 1 : 0);
  }
  const moves = allMoves(resolveRenameChains(renames, map.renames));
  if (args.includes('--list')) {
    for (const move of moves) console.log(`${move.rename.tree}\t${move.from}\t${move.to}${move.companion ? `\t(${move.companion})` : ''}`);
  } else if (args.includes('--check')) {
    check(map, moves);
  } else {
    apply(map, moves, { dryRun: args.includes('--dry-run'), gitMove: args.includes('--git-mv') });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
