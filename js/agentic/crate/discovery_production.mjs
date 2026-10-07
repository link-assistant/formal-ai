// Discovery on the production path: the rediscoverable coding-procedure cache
// (rust/src/discovery_production.rs, issue #1165, E130; requirement R1165-10
// three-roots parity).
//
// The policy is data, not code: `data/seed/program-cache-policy.lino` names
// the fields a cache row must carry (its `entry_requires` rows valued
// `required`), the header a written cache file opens with, and whether the
// embedded `ORACLE_SNAPSHOTS` bootstrap still fronts the cache. This module
// reads that seed through the host and never restates it. `language` and
// `task` are the cache key (`lookup` cannot address a row without them), so
// they are required on top of the seed's fields, in both roots.
//
// Rows keep the Rust field names (`rediscovery_query`, `content_id`, ...), as
// `execution_evidence.mjs` does for `Evidence`; an absent `Option` is `null`.
// `content_id` is a `bigint` so the 64-bit FNV-1a matches Rust bit for bit.
// A Rust `Result` is `{ ok: true, ... }` or `{ ok: false, error }`.
//
// Cache IO: the agentic host offers `readText` for repository files only and
// no write, so a cache is opened over an injectable `io` —
// `{ readText(path), writeText(path, text) }`. Without an `io`, reads go
// through the host's `readText` (a missing file is an empty cache, as in
// Rust) and writes only refresh `cache.file_text`, the exact bytes Rust's
// `ProcedureCache::write` would put on disk. `default_cache_path` is the
// repository-relative `DEFAULT_CACHE_FILE`: the host already resolves paths
// against the repository root, and a browser host has no environment for the
// `FORMAL_AI_PROCEDURE_CACHE` override.
//
// The JavaScript root has no twin of `research_coding_skill_gap`
// (rust/src/coding_research_learning.rs), so `cachedOrResearch` takes the
// research round as an injected function. Rust's solver reads the cache
// through `cached_write_program` in its `WriteProgram` branch; its twin
// `cachedWriteProgram` is here, but no JavaScript root carries that catalog
// `write_program` recipe arm yet (js/agentic/general_execution.mjs ports only
// the program-contract arm, and the browser worker has no cache file), so no
// JavaScript solver path calls either function yet.

import { cached, readText as hostReadText, realm } from '../host.mjs';
import { pushLinoField } from './links_format.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

/** Mirrors `const POLICY` (read through the host, not embedded). */
const POLICY_PATH = 'data/seed/program-cache-policy.lino';
/** Mirrors `const GRAMMARS` (read through the host, not embedded). */
const GRAMMARS_PATH = 'data/seed/program-cst-grammars.lino';
/** Mirrors `pub const DEFAULT_CACHE_FILE`. */
export const DEFAULT_CACHE_FILE = 'data/cache/coding-procedure-cache.lino';
/** The refusal `ProcedureCache::store` returns for a row missing a policy field. */
export const ENTRY_REFUSAL = 'procedure_cache_entry_requires_rediscovery_query_and_source';
/** The value an `entry_requires` row carries when the field must be non-blank. */
const REQUIRED_MARK = 'required';
/** The cache key fields every row carries in addition to the seed's. */
const KEY_FIELDS = Object.freeze(['language', 'task']);

const FNV_OFFSET = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;
const U64_MASK = 0xffffffffffffffffn;

/** The policy seed's root record. */
function policyRoot() {
  return cached('crate:discovery_production_policy', () => parseRoot(hostReadText(POLICY_PATH)).children[0] ?? null);
}

/**
 * Mirrors `fn fnv1a64`: FNV-1a (64-bit) of `bytes`, as a `bigint`.
 * A string is hashed as its UTF-8 bytes (`str::as_bytes`).
 * @param {Uint8Array|string} bytes
 */
export function fnv1a64(bytes) {
  const data = typeof bytes === 'string' ? new TextEncoder().encode(bytes) : bytes;
  let hash = FNV_OFFSET;
  for (const byte of data) {
    hash ^= BigInt(byte);
    hash = (hash * FNV_PRIME) & U64_MASK;
  }
  return hash;
}

/**
 * Mirrors `struct RediscoverableRecipe` (a constructor for the record).
 * @param {{language?: string, task?: string, rediscovery_query?: string,
 *   rediscovery_source?: string, entry?: string, verified_output?: string,
 *   content_id?: bigint}} fields
 */
export function rediscoverableRecipe(fields = {}) {
  return {
    language: fields.language ?? '',
    task: fields.task ?? '',
    rediscovery_query: fields.rediscovery_query ?? '',
    rediscovery_source: fields.rediscovery_source ?? '',
    entry: fields.entry ?? '',
    verified_output: fields.verified_output ?? '',
    content_id: fields.content_id ?? 0n,
  };
}

/** Mirrors `RediscoverableRecipe::content_address`. @param {string} entry */
export function contentAddress(entry) {
  return fnv1a64(entry);
}

/**
 * The fields a row must carry non-blank: the cache key plus every
 * `entry_requires` row of the policy seed valued `required`.
 */
export function requiredEntryFields() {
  return cached('crate:discovery_production_required_fields', () => {
    const requires = (policyRoot()?.children || []).find((node) => node.name === 'entry_requires');
    const declared = (requires?.children || [])
      .filter((node) => node.value === REQUIRED_MARK)
      .map((node) => node.name);
    return Object.freeze([...KEY_FIELDS, ...declared.filter((name) => !KEY_FIELDS.includes(name))]);
  });
}

/** Mirrors `RediscoverableRecipe::is_valid_cache_entry`, over the seed's fields. */
export function isValidCacheEntry(recipe) {
  return requiredEntryFields().every((field) => typeof recipe[field] === 'string' && recipe[field].trim() !== '');
}

/** Mirrors `fn bootstrap_cache_active`. */
export function bootstrapCacheActive() {
  const bootstrap = (policyRoot()?.children || []).find((node) => node.name === 'bootstrap');
  return Boolean(bootstrap) && findChildValue(bootstrap, 'active') === 'true';
}

/** Rust `str::lines`: split on `\n`, drop a trailing `\r`, no final empty line. @param {string} text */
function lines(text) {
  if (text === '') return [];
  const parts = text.split('\n');
  if (text.endsWith('\n')) parts.pop();
  return parts.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
}

/** Mirrors `fn cache_file_header`: the seed's `cache_file_header`, one `# ` line per line. */
export function cacheFileHeader() {
  const header = findChildValue(policyRoot(), 'cache_file_header');
  return lines(header).map((line) => (line === '' ? '#\n' : `# ${line}\n`)).join('');
}

/** Mirrors `fn grammar_exists`. @param {string} language */
export function grammarExists(language) {
  const needle = asciiLower(language.trim());
  if (needle === '') return false;
  const root = cached('crate:discovery_production_grammars', () => parseRoot(hostReadText(GRAMMARS_PATH)).children[0] ?? null);
  if (!root) return false;
  return root.children
    .filter((node) => node.name === 'cst_grammar')
    .some((grammar) => asciiLower(grammar.id) === needle
      || asciiLower(findChildValue(grammar, 'program_language')) === needle);
}

/**
 * Mirrors `CodingOracle::knows_language` (rust/src/knowledge.rs): the worker
 * realm's `codingOracleKnowsLanguage` snapshot lookup behind this seed's
 * `bootstrap` gate.
 * @param {string} language
 */
export function oracleKnowsLanguage(language) {
  return bootstrapCacheActive() && Boolean(realm().codingOracleKnowsLanguage(language));
}

/** Mirrors `fn knows_language`. @param {string} language */
export function knowsLanguage(language) {
  return grammarExists(language)
    && ((bootstrapCacheActive() && oracleKnowsLanguage(language)) || languageHasCacheRow(language));
}

/** Mirrors `fn language_has_cache_row`. @param {string} language */
function languageHasCacheRow(language) {
  return load().recipes.some((recipe) => eqIgnoreAsciiCase(recipe.language, language));
}

/** Rust `to_ascii_lowercase`. @param {string} text */
function asciiLower(text) {
  return text.replace(/[A-Z]/gu, (character) => character.toLowerCase());
}

/** Rust `eq_ignore_ascii_case`. */
function eqIgnoreAsciiCase(left, right) {
  return asciiLower(left) === asciiLower(right);
}

/** Rust `u64::from_str_radix(hex, 16).ok()`; `null` when it does not parse. @param {string} hex */
function parseHexU64(hex) {
  if (!/^\+?[0-9a-fA-F]+$/u.test(hex)) return null;
  const value = BigInt(`0x${hex.replace(/^\+/u, '')}`);
  return value > U64_MASK ? null : value;
}

/**
 * Mirrors `ProcedureCache::load_at`: a missing file loads as an empty cache.
 * @param {string} path
 * @param {{readText?: (path: string) => string, writeText?: (path: string, text: string) => void}} io
 */
export function loadAt(path, io = {}) {
  const cache = { path, recipes: [], file_text: null, io };
  let text = null;
  try {
    text = (io.readText ?? hostReadText)(path);
  } catch {
    text = null;
  }
  if (typeof text === 'string') parse(cache, text);
  return cache;
}

/** Mirrors `ProcedureCache::load`. @param {object} io */
export function load(io = {}) {
  return loadAt(defaultCachePath(), io);
}

/** Mirrors `ProcedureCache::recipes`. */
export function recipes(cache) {
  return cache.recipes;
}

/** Mirrors `ProcedureCache::lookup`: the row or `null`, case-insensitive. */
export function lookup(cache, language, task) {
  return cache.recipes.find((recipe) => eqIgnoreAsciiCase(recipe.language, language.trim())
    && eqIgnoreAsciiCase(recipe.task, task.trim())) ?? null;
}

/**
 * Mirrors `ProcedureCache::store`: `{ ok: true }` or `{ ok: false, error }`.
 * The stored row is a copy whose `content_id` is recomputed from `entry`.
 */
export function store(cache, recipe) {
  if (!isValidCacheEntry(recipe)) return { ok: false, error: ENTRY_REFUSAL };
  const row = { ...recipe, content_id: contentAddress(recipe.entry) };
  cache.recipes = cache.recipes.filter((present) => !(eqIgnoreAsciiCase(present.language, row.language)
    && eqIgnoreAsciiCase(present.task, row.task)));
  cache.recipes.push(row);
  return write(cache);
}

/** Mirrors `ProcedureCache::delete_all`. */
export function deleteAll(cache) {
  cache.recipes = [];
  write(cache);
}

/** Mirrors `fn parse` (private in Rust): a row whose stored `content_id` drifted is dropped. */
function parse(cache, text) {
  const document = parseRoot(text).children[0];
  if (!document) return;
  for (const record of document.children) {
    if (!record.name.startsWith('procedure_')) continue;
    const stored = findChildValue(record, 'content_id').trim();
    const recipe = rediscoverableRecipe({
      language: findChildValue(record, 'language'),
      task: findChildValue(record, 'task'),
      rediscovery_query: findChildValue(record, 'rediscovery_query'),
      rediscovery_source: findChildValue(record, 'rediscovery_source'),
      entry: findChildValue(record, 'entry'),
      verified_output: findChildValue(record, 'verified_output'),
      content_id: (stored.startsWith('0x') ? parseHexU64(stored.slice(2)) : null) ?? 0n,
    });
    if (!isValidCacheEntry(recipe)) continue;
    if (recipe.content_id !== contentAddress(recipe.entry)) continue;
    cache.recipes.push(recipe);
  }
}

/** Mirrors the slug `fn write` names a row with. */
function rowSlug(recipe) {
  let slug = '';
  for (const character of `procedure_${recipe.language}_${recipe.task}`) {
    slug += /^[A-Za-z0-9_]$/u.test(character) ? character.toLowerCase() : '_';
  }
  return slug;
}

/** Mirrors the text `fn write` puts on disk. */
export function cacheFileText(cache) {
  let out = cacheFileHeader();
  out = pushLinoField(out, 0, 'coding_procedure_cache', null);
  out = pushLinoField(out, 2, 'version', '"1"');
  for (const recipe of cache.recipes) {
    out += `  ${rowSlug(recipe)}\n`;
    for (const field of ['language', 'task', 'rediscovery_query', 'rediscovery_source', 'entry', 'verified_output']) {
      out = pushLinoField(out, 4, field, quote(recipe[field]));
    }
    out = pushLinoField(out, 4, 'content_id', `"0x${recipe.content_id.toString(16).padStart(16, '0')}"`);
  }
  return out;
}

/** Mirrors `fn write`: refreshes `file_text`, then hands it to `io.writeText` when one is injected. */
function write(cache) {
  cache.file_text = cacheFileText(cache);
  if (typeof cache.io?.writeText !== 'function') return { ok: true };
  try {
    cache.io.writeText(cache.path, cache.file_text);
    return { ok: true };
  } catch (error) {
    return { ok: false, error: `procedure_cache_write_failed:${error?.message ?? error}` };
  }
}

/**
 * Mirrors `fn cached_write_program` (R1165-1/R1165-2): the cache row a
 * concrete `write_program` answer reuses, or `null`. A prompt that customised
 * the template (`rendered !== template`) asked for a different program than
 * the row verified, so only an unmodified request is answered from the cache.
 */
export function cachedWriteProgram(cache, language, task, template, rendered) {
  if (rendered !== template) return null;
  return lookup(cache, language, task);
}

/** Mirrors `enum CachedOrDiscovered` variant tags. */
export const CachedOrDiscovered = Object.freeze({ Cached: 'cached', Discovered: 'discovered' });

/** Mirrors `CachedOrDiscovered::recipe`. */
export function answerRecipe(answer) {
  return answer.recipe;
}

/** Mirrors `CachedOrDiscovered::was_cached`. */
export function wasCached(answer) {
  return answer.kind === CachedOrDiscovered.Cached;
}

/**
 * Mirrors `fn cached_or_research`: a cache row answers; a miss runs the
 * injected research round, stores the verified procedure, and returns it.
 *
 * `gap` exposes `linksNotation()` and `nextQuery()` (the `CodingResearchGap`
 * readers Rust uses). `research(gap)` stands in for
 * `research_coding_skill_gap(gap, ledger, client, candidate_source,
 * expected_output, approval)` and returns `{ ok: true, execution: { output,
 * cycle } }` or `{ ok: false, error: { reason, cycle } }`.
 *
 * Returns `{ ok: true, answer: { kind, recipe } }` or `{ ok: false, error }`.
 */
export function cachedOrResearch(cache, gap, research, expectedOutput, rediscoverySource) {
  const language = gapField(gap, 'language');
  const task = gapField(gap, 'task');
  const hit = lookup(cache, language, task);
  if (hit) return { ok: true, answer: { kind: CachedOrDiscovered.Cached, recipe: { ...hit } } };
  const rediscoveryQuery = gap.nextQuery();
  const outcome = research(gap);
  if (!outcome.ok) return { ok: false, error: outcome.error };
  const { execution } = outcome;
  const recipe = rediscoverableRecipe({
    language,
    task,
    rediscovery_query: rediscoveryQuery,
    rediscovery_source: rediscoverySource,
    entry: execution.output,
    verified_output: expectedOutput,
    content_id: contentAddress(execution.output),
  });
  const stored = store(cache, { ...recipe });
  if (!stored.ok) return { ok: false, error: { reason: stored.error, cycle: execution.cycle } };
  return { ok: true, answer: { kind: CachedOrDiscovered.Discovered, recipe } };
}

/** Mirrors `fn gap_field`: a `field "value"` line of the gap's notation. */
export function gapField(gap, field) {
  const prefix = `${field} `;
  for (const line of gap.linksNotation().split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith(prefix)) return trimmed.slice(prefix.length).replace(/^"+|"+$/gu, '');
  }
  return '';
}

/** Mirrors `fn default_cache_path` (repository-relative; see the module note). */
export function defaultCachePath() {
  return DEFAULT_CACHE_FILE;
}

/** Mirrors `fn quote`: the parser's backslash dialect. @param {string} value */
export function quote(value) {
  return `"${value.replaceAll('\\', '\\\\').replaceAll('"', '\\"').replaceAll('\n', '\\n')}"`;
}
