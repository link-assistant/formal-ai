// A minimal browser runtime for loading the site's production JavaScript under
// `node --test` (issue #895).
//
// The scripts under `src/web/` that the pages load as plain `<script>` tags —
// `preferences.js`, `i18n.js`, `syntax-highlight.js`, `memory.js`,
// `seed_loader.js`, `site-chrome.js`, the per-page configs — and the worker
// mirror under `src/web/worker/*.js` are the *production* browser sources, not
// build output. Running them through `node:vm` with a `filename` of the real
// file makes V8 attribute coverage to that path, so
// `node --test --experimental-test-coverage` measures the browser denominator
// against the same files the browser downloads.
//
// The stubs below are deliberately small: enough to let a module install its
// `window.FormalAi*` namespace and run its pure logic, and no more. Anything a
// test needs beyond that it passes in explicitly, so a stub can never quietly
// stand in for behaviour the assertions claim to check.

import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import vm from "node:vm";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const WORKER_DIR = path.join(REPO_ROOT, "src/web/worker");

/** An in-memory `localStorage` good enough for the preference round-trips. */
export function createStorage(initial = {}) {
  const data = new Map(Object.entries(initial));
  return {
    get length() {
      return data.size;
    },
    key: (index) => [...data.keys()][index] ?? null,
    getItem: (key) => (data.has(key) ? data.get(key) : null),
    setItem: (key, value) => data.set(key, String(value)),
    removeItem: (key) => data.delete(key),
    clear: () => data.clear(),
  };
}

function createElement() {
  const element = {
    style: {},
    dataset: {},
    children: [],
    textContent: "",
    innerHTML: "",
    classList: { add() {}, remove() {}, toggle() {}, contains: () => false },
    setAttribute() {},
    getAttribute: () => null,
    removeAttribute() {},
    appendChild(child) {
      element.children.push(child);
      return child;
    },
    append() {},
    addEventListener() {},
    removeEventListener() {},
    querySelector: () => null,
    querySelectorAll: () => [],
    remove() {},
    focus() {},
    click() {},
  };
  return element;
}

function createDocument() {
  return {
    documentElement: createElement(),
    head: createElement(),
    body: createElement(),
    title: "",
    readyState: "complete",
    createElement,
    createTextNode: (text) => ({ textContent: text }),
    createDocumentFragment: createElement,
    getElementById: () => null,
    querySelector: () => null,
    querySelectorAll: () => [],
    addEventListener() {},
    removeEventListener() {},
  };
}

/**
 * A `window`-shaped sandbox. `overrides` replaces or adds globals; `fetch`
 * rejects by default so a module that reaches for the network in a unit test
 * fails loudly instead of hanging.
 */
export function createBrowserContext(overrides = {}) {
  const sandbox = {};
  const context = vm.createContext(sandbox);
  Object.assign(sandbox, {
    globalThis: sandbox,
    window: sandbox,
    self: sandbox,
    console,
    document: createDocument(),
    localStorage: createStorage(),
    sessionStorage: createStorage(),
    navigator: { language: "en", languages: ["en"], userAgent: "node-test" },
    location: {
      href: "http://localhost/app/",
      origin: "http://localhost",
      pathname: "/app/",
      search: "",
      hash: "",
    },
    history: { replaceState() {}, pushState() {} },
    matchMedia: () => ({
      matches: false,
      addEventListener() {},
      removeEventListener() {},
      addListener() {},
      removeListener() {},
    }),
    fetch: () => Promise.reject(new Error("network disabled in unit tests")),
    setTimeout,
    clearTimeout,
    setInterval,
    clearInterval,
    queueMicrotask,
    TextEncoder,
    TextDecoder,
    URL,
    URLSearchParams,
    crypto,
    atob: (value) => Buffer.from(value, "base64").toString("binary"),
    btoa: (value) => Buffer.from(value, "binary").toString("base64"),
    addEventListener() {},
    removeEventListener() {},
    dispatchEvent: () => true,
    CustomEvent: class CustomEvent {
      constructor(type, init = {}) {
        this.type = type;
        this.detail = init.detail;
      }
    },
    postMessage() {},
    importScripts() {},
    indexedDB: undefined,
    ...overrides,
  });
  return context;
}

/**
 * Run a repo-relative browser script inside `context`.
 *
 * The absolute path is passed as the script `filename`, which is what makes the
 * file show up in the coverage report under its real repository path.
 */
export function loadBrowserScript(context, relativePath) {
  const absolute = path.join(REPO_ROOT, relativePath);
  const source = readFileSync(absolute, "utf8");
  new vm.Script(source, { filename: absolute }).runInContext(context);
  return context;
}

/** List the worker mirror files in the order the worker loads them. */
export function workerMirrorFiles() {
  return readdirSync(WORKER_DIR)
    .filter((name) => name.endsWith(".js"))
    .sort()
    .map((name) => path.posix.join("src/web/worker", name));
}

/**
 * Boot the split browser worker mirror (`src/web/worker/*.js`) exactly the way
 * `formal_ai_worker.js` does: concatenated in filename order into one global
 * scope.
 */
export function loadWorkerMirror(overrides = {}) {
  const context = createBrowserContext(overrides);
  for (const file of workerMirrorFiles()) {
    loadBrowserScript(context, file);
  }
  return context;
}

/**
 * Boot the real worker entry point, `src/web/formal_ai_worker.js`.
 *
 * `importScripts` resolves against `src/web/` and loads the same files the
 * browser would; `fetch` serves `seed/*.lino` from the canonical `data/seed/`
 * tree, which is exactly what the dev server mirrors into `src/web/seed/`. The
 * result is a worker booted from production sources and real seed data, so the
 * answers the tests assert are the answers the site gives.
 */
export function createWorkerContext(overrides = {}) {
  const context = createBrowserContext({
    location: { href: "http://localhost/formal_ai_worker.js", search: "" },
    fetch: (url) => {
      const relative = String(url).split("?")[0].replace(/^\.?\//, "");
      const onDisk = relative.startsWith("seed/")
        ? path.join(REPO_ROOT, "data", relative)
        : path.join(REPO_ROOT, "src/web", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
      } catch {
        return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
      }
    },
    ...overrides,
  });
  context.importScripts = (...urls) => {
    for (const url of urls) {
      const relative = String(url).split("?")[0].replace(/^\.?\//, "");
      loadBrowserScript(context, path.posix.join("src/web", relative));
    }
  };
  loadBrowserScript(context, "src/web/formal_ai_worker.js");
  return context;
}

/**
 * Copy a value out of the sandbox realm into a host-realm plain value.
 *
 * Objects created inside a `vm` context have that context's `Object.prototype`,
 * which `assert.deepEqual` from `node:assert/strict` treats as a difference. The
 * JSON round-trip drops the foreign prototypes so assertions compare the data.
 */
export function plain(value) {
  return value === undefined ? undefined : JSON.parse(JSON.stringify(value));
}

/** Evaluate an expression against a loaded context, e.g. a worker function. */
export function evaluate(context, expression) {
  return vm.runInContext(expression, context);
}
