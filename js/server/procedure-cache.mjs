// The procedure-cache read of the native `WriteProgram` branch
// (rust/src/solver.rs, issue #1165 R1165-1/R1165-2, three-roots parity
// R1165-10), on the JavaScript server's solve path.
//
// Rust loads `ProcedureCache::load()` (the `FORMAL_AI_PROCEDURE_CACHE`
// override, else `data/cache/coding-procedure-cache.lino`) on every catalog
// `write_program` solve and asks `cached_write_program` for a verified row.
// The browser worker cannot read a file, so it logs the miss the committed,
// row-less cache produces (js/worker/formal_ai_worker_solver_events.js). This
// module is the server half: after the worker solved, it reads the same cache
// file through `js/agentic/crate/discovery_production.mjs`, calls
// `cachedWriteProgram` with the catalog template and the rendered program the
// worker answered with, and on a hit does what Rust does: the template in the
// answer becomes the cached entry, and the `procedure_cache` event becomes
// `outcome=hit language=<slug> task=<slug> content_id=0x<16 hex>`.

import { readFileSync } from 'node:fs';
import path from 'node:path';

import { cachedWriteProgram, defaultCachePath, loadAt } from '../agentic/crate/discovery_production.mjs';
import { hasHost } from '../agentic/host.mjs';
import { installNodeHost } from '../agentic/node-host.mjs';
import { REPO_ROOT } from './lino.mjs';

const WRITE_PROGRAM_INTENT = 'write_program';
const CACHE_EVENT = 'procedure_cache';
const CACHE_ENV = 'FORMAL_AI_PROCEDURE_CACHE';
const PARAMETER_PREFIX = 'program_parameter:';

/** Mirrors `fn default_cache_path`: the environment override, else the repository file. */
export function procedureCachePath() {
  const explicit = String(process.env[CACHE_ENV] ?? '').trim();
  return explicit ? path.resolve(explicit) : path.join(REPO_ROOT, defaultCachePath());
}

/** Mirrors `ProcedureCache::load`: a missing file loads as an empty cache. */
export function loadProcedureCache() {
  return loadAt(procedureCachePath(), { readText: (file) => readFileSync(file, 'utf8') });
}

/** The `program_parameter:<name>:<value>` evidence the catalog answer carries. */
function programParameter(result, name) {
  const prefix = `${PARAMETER_PREFIX}${name}:`;
  const link = (Array.isArray(result.evidence) ? result.evidence : []).map(String).find((item) => item.startsWith(prefix));
  return link ? link.slice(prefix.length) : '';
}

/** Mirrors `render_fields`. */
function renderFields(fields) {
  return fields.map(([name, value]) => `${name}=${value}`).join(' ');
}

/**
 * Answer a catalog `write_program` result from the procedure cache when it has
 * a verified row, as the native solver does; any other result is returned as is.
 * @param {object} result the worker's `solve` value
 * @param {string} prompt the prompt it answered
 * @param {{boot: () => Promise<object>}} worker the booted worker host
 * @param {(prompt: string, task: string, language: string) => {template: ?string, rendered: ?string}} templateFor
 *   the worker's catalog template for the pair and what the prompt rendered it to
 */
export async function applyProcedureCache(result, prompt, worker, templateFor) {
  if (result?.intent !== WRITE_PROGRAM_INTENT || !Array.isArray(result.solverEvents)) return result;
  const index = result.solverEvents.findIndex((event) => event.kind === CACHE_EVENT);
  if (index < 0) return result;
  const language = programParameter(result, 'language');
  const task = programParameter(result, 'task');
  const { template, rendered } = templateFor(prompt, task, language);
  if (typeof template !== 'string') return result;
  // The cache rows are validated against the policy seed, read through the agentic host.
  if (!hasHost()) await installNodeHost(worker);
  const recipe = cachedWriteProgram(loadProcedureCache(), language, task, template, rendered);
  if (!recipe) return result;
  const contentId = `0x${recipe.content_id.toString(16).padStart(16, '0')}`;
  const solverEvents = result.solverEvents.slice();
  solverEvents[index] = {
    kind: CACHE_EVENT,
    payload: renderFields([['outcome', 'hit'], ['language', recipe.language], ['task', recipe.task], ['content_id', contentId]]),
  };
  return { ...result, content: String(result.content ?? '').replaceAll(template, () => recipe.entry), solverEvents };
}
