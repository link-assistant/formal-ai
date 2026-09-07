#!/usr/bin/env node

/**
 * Interop shim around `use-m`.
 *
 * `use-m` resolves a package with `createRequire(...).resolve` and imports the
 * resolved file, so a dual package such as `command-stream@>=0.19.0` loads
 * through its **CommonJS** entry point. It unwraps the callable `default` only
 * when the namespace carries nothing but known metadata keys.
 *
 * Node.js 22.12+ adds a synthetic `'module.exports'` named export to CommonJS
 * namespaces whose names `cjs-module-lexer` cannot infer
 * (https://nodejs.org/api/esm.html#commonjs-namespaces). That extra key is not
 * in the metadata set of `use-m`, so the unwrap is skipped:
 *
 *   node v20.20.2  -> keys [default]                    typeof loaded.$ : function
 *   node v24.19.0  -> keys [default, 'module.exports']  typeof loaded.$ : undefined
 *
 * On Node 24 — the version every workflow in this template requests —
 * `const { $ } = await use('command-stream')` therefore yields `undefined` and
 * the first tagged template dies with `TypeError: $ is not a function`.
 * Upstream tracker: https://github.com/link-foundation/use-m/issues/72
 *
 * This module normalises every shape observed so far (Node 20, 22, 24 and Bun)
 * and, when nothing callable is found, fails with an error naming the keys it
 * did see, in place of the opaque `$ is not a function`.
 *
 * Set CI_SCRIPTS_DEBUG=1 to trace the resolved module shape (default off).
 *
 * Usage:
 *   import { loadCommandStream } from './use-module.mjs';
 *   const { $ } = await loadCommandStream();
 */

import { debug } from './debug-print.mjs';

/** Unpinned CDN entry point for use-m, kept in one place. */
export const USE_M_URL = 'https://unpkg.com/use-m/use.js';

/** Cached `use` function, so a process fetches use.js at most once. */
let cachedUse = null;

/**
 * Candidate containers for the real module object, in resolution order.
 * `module.exports` is the Node >= 22.12 synthetic CommonJS export.
 * @param {unknown} loaded
 * @returns {unknown[]}
 */
function candidates(loaded) {
  return [
    loaded,
    loaded?.default,
    loaded?.['module.exports'],
    loaded?.default?.default,
  ];
}

/**
 * Human-readable description of a loaded module, used in error messages.
 * @param {unknown} loaded
 * @returns {string}
 */
export function describeModule(loaded) {
  if (loaded === null || loaded === undefined) {
    return String(loaded);
  }
  if (typeof loaded !== 'object' && typeof loaded !== 'function') {
    return typeof loaded;
  }
  return `${typeof loaded} with keys [${Object.keys(loaded).join(', ')}]`;
}

/**
 * Pick the object that actually carries `exportName` out of whatever `use-m`
 * returned for `moduleName`.
 *
 * @param {unknown} loaded value returned by `await use(moduleName)`
 * @param {string} exportName named export the caller needs
 * @param {string} moduleName package name, used for the error message only
 * @returns {Record<string, unknown>} object exposing `exportName`
 */
export function resolveNamedExport(loaded, exportName, moduleName) {
  for (const candidate of candidates(loaded)) {
    if (candidate && typeof candidate[exportName] === 'function') {
      debug(`resolved ${moduleName}.${exportName}`, {
        received: describeModule(loaded),
        via: candidate === loaded ? 'namespace' : 'unwrapped',
      });
      return candidate;
    }
  }
  throw new Error(
    `use('${moduleName}') did not expose a callable "${exportName}". ` +
      `Received ${describeModule(loaded)}. This usually means the CommonJS ` +
      `interop of use-m did not unwrap the module (see scripts/use-module.mjs).`
  );
}

/**
 * Per-attempt deadline. Without it a stalled CDN connection is bounded only by
 * undici's 300 s `headersTimeout`, so one fetch can burn five minutes of a
 * job's `timeout-minutes`.
 */
export const DEFAULT_TIMEOUT_MS = 15000;

/** Total attempts, including the first one. */
export const DEFAULT_ATTEMPTS = 3;

/** Delay before the second attempt; doubled for each attempt after it. */
export const DEFAULT_RETRY_DELAY_MS = 2000;

/**
 * One fetch-and-evaluate attempt, with a deadline on the request.
 *
 * The deadline is an `AbortController` armed by a timer that is always
 * cleared. `AbortSignal.timeout()` would leave the deadline unref'd, so a
 * stalled fetch would go unaborted when nothing else keeps the event loop
 * alive.
 *
 * A non-2xx response is reported as an HTTP failure; eval-ing an error page as
 * JavaScript would only produce an opaque `SyntaxError`.
 *
 * @param {{fetchImpl: typeof fetch, url: string, timeoutMs: number}} options
 * @returns {Promise<(name: string) => Promise<unknown>>}
 */
async function fetchUseOnce({ fetchImpl, url, timeoutMs }) {
  const controller = timeoutMs > 0 ? new AbortController() : null;
  const timer = controller
    ? setTimeout(
        () =>
          controller.abort(
            new Error(
              `Timed out after ${timeoutMs}ms while fetching use-m from ${url}`
            )
          ),
        timeoutMs
      )
    : null;
  try {
    const response = await (controller
      ? fetchImpl(url, { signal: controller.signal })
      : fetchImpl(url));
    if (!response.ok) {
      throw new Error(
        `Failed to fetch use-m from ${url}: ${response.status} ${response.statusText || ''}`.trim()
      );
    }
    const source = await response.text();
    // use-m ships as an eval-able bundle; this is its documented entry point.
    const evaluated = await eval(source);
    const use = evaluated?.use ?? evaluated?.default?.use;
    if (typeof use !== 'function') {
      throw new Error(
        `use-m loaded from ${url} did not export a callable "use". ` +
          `Received ${describeModule(evaluated)}.`
      );
    }
    return use;
  } catch (error) {
    // `fetch` reports an abort as its own error; the reason says what happened.
    throw controller?.signal.aborted ? controller.signal.reason : error;
  } finally {
    if (timer) {
      clearTimeout(timer);
    }
  }
}

/**
 * Fill in the load defaults, keeping every knob injectable from a test.
 * @param {Record<string, unknown>} options
 * @returns {{fetchImpl: typeof fetch, url: string, attempts: number,
 *   timeoutMs: number, retryDelayMs: number,
 *   sleep: (ms: number) => Promise<void>}}
 */
function loadSettings(options) {
  return {
    fetchImpl: options.fetchImpl ?? fetch,
    url: options.url ?? USE_M_URL,
    attempts: options.attempts ?? DEFAULT_ATTEMPTS,
    timeoutMs: options.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    retryDelayMs: options.retryDelayMs ?? DEFAULT_RETRY_DELAY_MS,
    sleep:
      options.sleep ??
      ((ms) => new Promise((resolve) => setTimeout(resolve, ms))),
  };
}

/**
 * Fetch and evaluate use-m, caching the result for the whole process.
 *
 * The load is a network dependency of every release script, so it is bounded
 * on both axes: each attempt carries a deadline and a transient failure is
 * retried with exponential backoff. The worst case stays well inside a job's
 * `timeout-minutes` (3 x 15 s + 2 s + 4 s = 51 s by default).
 *
 * The final error names the CDN, the URL and the attempt count. A bare
 * `TypeError: fetch failed` names neither, which makes a third-party outage
 * look like a defect in the release logic.
 *
 * @param {{fetchImpl?: typeof fetch, url?: string, attempts?: number,
 *   timeoutMs?: number, retryDelayMs?: number,
 *   sleep?: (ms: number) => Promise<void>}} [options] injection seam for tests
 * @returns {Promise<(name: string) => Promise<unknown>>}
 */
export async function loadUse(options = {}) {
  const settings = loadSettings(options);
  const { url, attempts } = settings;
  if (cachedUse && !options.fetchImpl) {
    return cachedUse;
  }
  let lastError;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      const use = await fetchUseOnce(settings);
      debug('loaded use-m', { url, attempt });
      if (!options.fetchImpl) {
        cachedUse = use;
      }
      return use;
    } catch (error) {
      lastError = error;
      debug('use-m load attempt failed', {
        url,
        attempt,
        attempts,
        error: error?.message,
      });
      if (attempt < attempts) {
        await settings.sleep(settings.retryDelayMs * 2 ** (attempt - 1));
      }
    }
  }
  throw new Error(
    `Failed to load use-m from ${url} after ${attempts} attempt(s): ` +
      `${lastError?.message ?? String(lastError)}. This is a network ` +
      'dependency of the release scripts, not a defect in the published ' +
      'package; re-run the job when the CDN answers again.',
    { cause: lastError }
  );
}

/**
 * Load a package through use-m and return it with CommonJS namespaces
 * normalised around a named export the caller needs.
 *
 * @param {string} moduleName package specifier passed to use-m
 * @param {string} exportName named export that must be callable
 * @param {(name: string) => Promise<unknown>} [use] pre-loaded use-m function
 * @returns {Promise<Record<string, unknown>>}
 */
export async function useModule(moduleName, exportName, use) {
  const resolvedUse = use ?? (await loadUse());
  return resolveNamedExport(
    await resolvedUse(moduleName),
    exportName,
    moduleName
  );
}

/**
 * Load `command-stream` with `$` guaranteed to be callable.
 *
 * @param {(name: string) => Promise<unknown>} [use] pre-loaded use-m function
 * @returns {Promise<Record<string, unknown>>} command-stream exports
 */
export function loadCommandStream(use) {
  return useModule('command-stream', '$', use);
}

/**
 * Load `lino-arguments` with `makeConfig` guaranteed to be callable.
 *
 * @param {(name: string) => Promise<unknown>} [use] pre-loaded use-m function
 * @returns {Promise<Record<string, unknown>>} lino-arguments exports
 */
export function loadLinoArguments(use) {
  return useModule('lino-arguments', 'makeConfig', use);
}
