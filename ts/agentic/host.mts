// The host the JavaScript agentic planner runs against (R1015).
//
// The planner (js/agentic/*.mjs) is a port of rust/src/agentic_coding/ and
// stays host-agnostic so the browser worker can load it later: it never
// touches `node:fs` or the network itself. Everything it needs from outside
// comes through one installed host:
//
// * `readText(relative)` - a repository file (seed and meta data) as text,
//   the twin of the Rust `include_str!` of `embedded/data/...`;
// * `parseLino(text)` - the shared Links Notation parser
//   (`js/seed_loader.js`, `FormalAiSeed.parse`), returning
//   `{name, id, value, children}` nodes;
// * `realm` - the booted browser worker's global scope, whose ports of
//   `crate::seed::lexicon()`, `crate::engine::normalize_prompt`,
//   `crate::language::detect` and the handler rules the planner calls instead
//   of porting them twice;
// * `solve(prompt, history)` - the host solver's answer, a Promise of the
//   Rust `SymbolicAnswer` shape (`{intent, answer, confidence,
//   evidence_links, thinking_steps, links_notation}`), for the few routes
//   that consult `crate::solve_with_history`;
// * `isDirectory(path)` / `isFile(path)` / `currentDirectory()` - the few
//   filesystem probes the Rust planner makes (`Path::is_dir`, the server's
//   own working directory). A browser host answers `false` / `null`;
// * `listDirectory(relative)` - the entries of a repository directory as
//   `[{name, isDirectory}]` (the twin of build.rs's `fs::read_dir` over
//   `rust/src` that embeds `OWNED_SOURCE_FILES`). A browser host answers `[]`.
//
// The JavaScript server installs a host once its worker has booted
// (js/server/agentic.mjs); node:test suites install the same one through
// js/agentic/node-host.mjs.

let current = null;
const caches = new Map();

/**
 * Install the host every planner module reads through.
 * @param {{readText: Function, parseLino: Function, realm: object,
 *   isDirectory?: Function, isFile?: Function, currentDirectory?: Function,
 *   listDirectory?: Function}} next
 */
export function installHost(next) {
  current = next;
  caches.clear();
}

/** The installed host; throws when none is installed. */
export function host() {
  if (!current) throw new Error('agentic_host_not_installed');
  return current;
}

/** Whether a host is installed. */
export function hasHost() {
  return current !== null;
}

/** A repository file as text. @param {string} relative */
export function readText(relative) {
  return host().readText(relative);
}

/** Parse Links Notation text. @param {string} text */
export function parseLino(text) {
  return host().parseLino(text);
}

/** The worker realm's global scope (lexicon, normalizer, language). */
export function realm() {
  return host().realm;
}

/**
 * Memoize `build()` under `key` for the lifetime of the installed host (the
 * twin of a Rust `OnceLock`).
 * @template T
 * @param {string} key
 * @param {() => T} build
 * @returns {T}
 */
export function cached(key, build) {
  if (!caches.has(key)) caches.set(key, build());
  return caches.get(key);
}

/**
 * `crate::solve_with_history`: the host solver's `SymbolicAnswer`.
 * @param {string} prompt
 * @param {Array<{role: string, content: string}>} history
 * @returns {Promise<object>}
 */
export function solve(prompt, history = []) {
  return host().solve(prompt, history);
}

/** `Path::is_dir`. @param {string} path */
export function isDirectory(path) {
  return Boolean(current?.isDirectory?.(path));
}

/** `Path::is_file`. @param {string} path */
export function isFile(path) {
  return Boolean(current?.isFile?.(path));
}

/** `std::env::current_dir`, or null. */
export function currentDirectory() {
  return current?.currentDirectory?.() ?? null;
}

/**
 * The entries of the repository directory `relative` (`fs::read_dir`), or
 * `[]` when the host cannot list it.
 * @param {string} relative
 * @returns {Array<{name: string, isDirectory: boolean}>}
 */
export function listDirectory(relative) {
  return current?.listDirectory?.(relative) ?? [];
}

/** Children of `node` named `name`. */
export function childrenNamed(node, name) {
  return (node?.children || []).filter((child) => child.name === name);
}

/** The value of the first child of `node` named `name`, or ''. */
export function childValue(node, name) {
  return childrenNamed(node, name)[0]?.value ?? '';
}
