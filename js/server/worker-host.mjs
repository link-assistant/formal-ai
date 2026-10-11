// A Node host for the browser worker (`js/worker/*.js`).
//
// The worker mirror is plain browser script: it installs globals, fetches
// `seed/*.lino`, and answers through `solve`. This module gives it the
// smallest `window`-shaped realm it needs inside `node:vm`, so the same
// production sources serve three hosts - the browser, `node --test`
// (rust/tests/web/support/browser-runtime.mjs re-exports this file), and the
// JavaScript HTTP server (js/server/), which answers every protocol route
// from the worker exactly as the Rust server answers from the native solver.
//
// Each script runs with its real absolute path as the `filename`, so coverage
// is attributed to the file the browser downloads.

import { readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { createHash } from "node:crypto";
import { workerScriptPrerequisites } from "./worker-script-prerequisites.mjs";

import { serverMessage } from "./messages.mjs";
import { applyProcedureCache } from "./procedure-cache.mjs";

export const REPO_ROOT = path.resolve(import.meta.dirname, "../..");
const WORKER_DIR = path.join(REPO_ROOT, "js/worker");

// Constructor-owned prerequisite: only a genuinely fresh context can be issued.
// No public completion registration or namespace/caller-field authority.
const parserBootstrapContexts = new WeakMap();
const parserBootstrapIdentity = "84de01ad3dfd183c2b1bb80ab273d1bad9498acc44ac0691755b1e6ca3246c40";
function parserBootstrapSource() {
  const absolute=path.join(REPO_ROOT,"js/parser-cache-primordials.js");
  const source=readFileSync(absolute,"utf8");
  if(createHash("sha256").update(source).digest("hex")!==parserBootstrapIdentity) throw new Error("ParserBootstrapSourceIdentityMismatch");
  return {absolute,source};
}
function initializeParserBootstrap(context) {
  const current=parserBootstrapSource();
  // The same captured loader evaluator must actually complete producer evaluation.
  applyWorkerScript(runWorkerScript,new WorkerScript(current.source,{filename:current.absolute}),[context]);
  parserBootstrapContexts.set(context,parserBootstrapIdentity);
  evaluatedWorkerScripts.set(context, new Map([["js/parser-cache-primordials.js", parserBootstrapIdentity]]));
}
function requireParserBootstrap(context) {
  if(parserBootstrapContexts.get(context)!==parserBootstrapIdentity) throw new Error("MissingFreshParserBootstrap");
  parserBootstrapSource();
}

/** Map a worker URL to its web-root path, mirroring the entry's base shim. */
function webRootPath(url) {
  const { pathname } = new URL(String(url), "http://localhost/");
  return pathname.replace(/^\/+/, "").split("?")[0];
}

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
  initializeParserBootstrap(context);
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
    fetch: () => Promise.reject(new Error(serverMessage("worker_network_disabled"))),
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
const evaluatedWorkerScripts = new WeakMap();
const prerequisiteRecords = new Map(workerScriptPrerequisites.map(record => [record.script, record]));
const WorkerScript = vm.Script;
const runWorkerScript = vm.Script.prototype.runInContext;
const parserMemoContexts = new WeakMap();
const counterOwnDescriptor = Object.getOwnPropertyDescriptor;
const counterFreeze = Object.freeze;
const parserMemoSeedIdentity = "4a3b4ef0056d6b191f2b2504189f1e760c419c6ed945e3bb9f022de44d076c4d";
const counterVmScriptDescriptor = counterOwnDescriptor(vm, "Script");
const counterVmPrototypeDescriptor = counterOwnDescriptor(WorkerScript, "prototype");
const counterVmRunDescriptor = counterOwnDescriptor(counterVmPrototypeDescriptor.value, "runInContext");
function sameCounterDescriptor(a,b) {
  return a && b && a.value===b.value && a.get===b.get && a.set===b.set && a.writable===b.writable && a.enumerable===b.enumerable && a.configurable===b.configurable;
}
function counterEvaluatorIntact() {
  return sameCounterDescriptor(counterOwnDescriptor(vm,"Script"),counterVmScriptDescriptor) && sameCounterDescriptor(counterOwnDescriptor(WorkerScript,"prototype"),counterVmPrototypeDescriptor) && sameCounterDescriptor(counterOwnDescriptor(counterVmPrototypeDescriptor.value,"runInContext"),counterVmRunDescriptor);
}
/** Readonly branch evidence, never parser or mutation authority. */
export function parserMemoObservation(context) {
  const entry=parserMemoContexts.get(context);
  if(!entry || !counterEvaluatorIntact()) return null;
  try {
    requireParserBootstrap(context);
    const source=workerSourceText(path.join(REPO_ROOT,"js/seed_loader.js"));
    if(createHash("sha256").update(source).digest("hex")!==parserMemoSeedIdentity) return null;
    const observed=entry.query();
    const inputs=[];
    for(let at=0;at<observed.inputs.length;at++) {
      const row=observed.inputs[at];if(typeof row.text!=="string") return null;
      inputs[at]=counterFreeze({sha256:createHash("sha256").update(row.text).digest("hex"),bytes:Buffer.byteLength(row.text),characters:row.text.length,rootClass:row.rootName.slice(0,128),hits:row.hits,misses:row.misses,bypasses:row.bypasses,evictions:row.evictions});
    }
    return counterFreeze({hits:observed.hits,
    misses:observed.misses,
    bypasses:observed.bypasses,
    evictions:observed.evictions,
    entries:observed.entries,
    retainedCharacters:observed.retainedCharacters,
    retainedNodes:observed.retainedNodes,
    retainedTextBytes:observed.retainedTextBytes,
    retainedPayloadBytes:observed.retainedPayloadBytes,
    maxEntries:observed.maxEntries,
    maxTextBytes:observed.maxTextBytes,
    maxPayloadBytes:observed.maxPayloadBytes,
    diagnosticOmitted:observed.diagnosticOmitted,
    inputs:counterFreeze(inputs),
    seedSourceSha256:parserMemoSeedIdentity,
    bootstrapSourceSha256:parserBootstrapIdentity,
    authority:false});

  } catch { return null; }
}
const applyWorkerScript = Reflect.apply;

class WorkerScriptPrerequisiteError extends Error {}

function workerSourceText(absolute) {
  return readFileSync(absolute, "utf8");
}

function verifyCurrentWorkerPrerequisites(record, evaluated) {
  for (const dependency of record.requires) {
    if (evaluated.get(dependency.script) !== dependency.sourceSha256) {
      throw new WorkerScriptPrerequisiteError("IncompleteWorkerScriptPrerequisite");
    }
    const absolute = path.resolve(REPO_ROOT, dependency.script);
    let currentHash;
    try {
      currentHash = createHash("sha256").update(workerSourceText(absolute)).digest("hex");
    } catch (error) {
      evaluated.delete(dependency.script);
      throw error;
    }
    if (currentHash !== dependency.sourceSha256) {
      evaluated.delete(dependency.script);
      throw new Error("WorkerPrerequisiteSourceIdentityMismatch");
    }
    const inherited = prerequisiteRecords.get(dependency.script);
    if (inherited) verifyCurrentWorkerPrerequisites(inherited, evaluated);
  }
}

function preflightWorkerSources(scripts) {
  const required = new Map(), visited = new Set();
  const visit = (script, expected) => {
    const known = required.get(script);
    if (known !== undefined && known !== expected) throw new Error("WorkerPrerequisiteSourceIdentityMismatch");
    required.set(script, expected);
    if (visited.has(script)) return;
    visited.add(script);
    const record = prerequisiteRecords.get(script);
    if (!record) return;
    if (record.sourceSha256 !== expected) throw new Error("WorkerPrerequisiteSourceIdentityMismatch");
    for (const dependency of record.requires) visit(dependency.script, dependency.sourceSha256);
  };
  for (const script of scripts) {
    const record = prerequisiteRecords.get(script);
    if (record) visit(script, record.sourceSha256);
  }
  for (const [script, expected] of required) {
    const current = createHash("sha256").update(workerSourceText(path.resolve(REPO_ROOT, script))).digest("hex");
    if (current !== expected) throw new Error("WorkerSourceIdentityMismatch");
  }
}

export function loadBrowserScript(context, relativePath) {
  const absolute = path.resolve(REPO_ROOT, relativePath);
  const script = path.relative(REPO_ROOT, absolute).split(path.sep).join("/");
  const required = path.resolve(REPO_ROOT,"js/seed_loader.js");
  const actualIdentity=statSync(absolute),requiredIdentity=statSync(required);
  const isParser=absolute===required || (actualIdentity.dev===requiredIdentity.dev && actualIdentity.ino===requiredIdentity.ino);
  if(isParser) {requireParserBootstrap(context);parserMemoContexts.delete(context);}
  const source = workerSourceText(absolute);
  const sourceHash = createHash("sha256").update(source).digest("hex");
  let evaluated = evaluatedWorkerScripts.get(context);
  if (!evaluated) {
    evaluated = new Map();
    evaluatedWorkerScripts.set(context, evaluated);
  }
  if (script === "js/parser-cache-primordials.js"
      && parserBootstrapContexts.get(context) === parserBootstrapIdentity) {
    requireParserBootstrap(context);
    evaluated.set(script, parserBootstrapIdentity);
    return;
  }
  evaluated.delete(script);
  const record = prerequisiteRecords.get(script);
  if (record) {
    if (sourceHash !== record.sourceSha256) throw new Error("WorkerSourceIdentityMismatch");
    verifyCurrentWorkerPrerequisites(record, evaluated);
  }
  const counterEligible=isParser && counterEvaluatorIntact() && sourceHash===parserMemoSeedIdentity;
  const completion=applyWorkerScript(runWorkerScript, new WorkerScript(source, { filename: absolute }), [context]);
  if(counterEligible && counterEvaluatorIntact() && typeof completion==="function") parserMemoContexts.set(context,{query:completion});
  evaluated.set(script, sourceHash);
  return context;
}

function loadWorkerScripts(context, scripts) {
  let pending = scripts.slice();
  while (pending.length) {
    const deferred = [];
    for (const script of pending) {
      try { loadBrowserScript(context, script); }
      catch (error) {
        if (!(error instanceof WorkerScriptPrerequisiteError)) throw error;
        deferred.push(script);
      }
    }
    if (deferred.length === pending.length) break;
    pending = deferred;
  }
}

/** List the worker mirror files in the order the worker loads them. */
export function workerMirrorFiles() {
  return readdirSync(WORKER_DIR)
    // The entry point loads the list; it is not one of the modules in it.
    .filter((name) => name.endsWith(".js") && name !== "formal_ai_worker.js")
    .sort()
    .map((name) => path.posix.join("js/worker", name));
}

/**
 * Boot the split browser worker mirror (`js/worker/*.js`) exactly the way
 * `formal_ai_worker.js` does: concatenated in filename order into one global
 * scope.
 */
export function loadWorkerMirror(overrides = {}) {
  preflightWorkerSources(workerMirrorFiles());
  const context = createBrowserContext(overrides);
  loadBrowserScript(context, "js/worker-bootstrap-responses.js");
  try { loadBrowserScript(context, "js/seed_loader.js"); } catch (_error) {
    // Optional producers may be absent; dependent scripts remain unevaluated.
  }
  loadWorkerScripts(context, workerMirrorFiles());
  return context;
}

/**
 * Boot the real worker entry point, `js/worker/formal_ai_worker.js`.
 *
 * The entry resolves worker-relative URLs against the web root, so
 * `importScripts` and `fetch` arrive as web-root URLs: `js/` serves the
 * scripts the browser would load, and `seed/*.lino` comes from the canonical
 * `data/seed/` tree, which is exactly what the dev server mirrors into
 * `js/seed/`. The result is a worker booted from production sources and real
 * seed data, so the answers the tests assert are the answers the site gives.
 *
 * `reorderModules`, when given, maps the module list the entry loads to the
 * order to load it in, so a test can prove the modules load in any order.
 */
export function createWorkerContext(overrides = {}, { reorderModules = null } = {}) {
  preflightWorkerSources(workerMirrorFiles());
  const context = createBrowserContext({
    location: { href: "http://localhost/worker/formal_ai_worker.js", search: "" },
    fetch: (url) => {
      const relative = webRootPath(url);
      const onDisk = relative.startsWith("seed/")
        ? path.join(REPO_ROOT, "data", relative)
        : path.join(REPO_ROOT, "js", relative);
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
      const relative = webRootPath(url);
      loadBrowserScript(context, path.posix.join("js", relative));
      if (reorderModules && relative === "worker-modules.js") {
        context.FORMAL_AI_WORKER_MODULES = Object.freeze(reorderModules([...context.FORMAL_AI_WORKER_MODULES]));
      }
    }
  };
  loadBrowserScript(context, "js/worker/formal_ai_worker.js");
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

/**
 * One booted worker realm, answering one `solve` at a time.
 *
 * The worker keeps per-turn state in globals (the forced response language,
 * learned meanings), so concurrent turns would observe each other. Requests
 * queue here instead; the HTTP layer stays concurrent for every route that
 * does not solve.
 */
export class WorkerHost {
  constructor(overrides = {}) {
    this.overrides = overrides;
    this.context = null;
    this.ready = null;
    this.queue = Promise.resolve();
    this.readers = null;
    this.templateFor = null;
  }

  /** Boot the worker and load the seed once. */
  boot() {
    if (!this.ready) {
      this.context = createWorkerContext(this.overrides);
      this.ready = Promise.resolve(evaluate(this.context, "loadSeed()")).then(() => this.context);
    }
    return this.ready;
  }

  /**
   * Run `expression` against the booted realm with `bindings` installed as
   * globals, serialized behind every earlier call.
   */
  run(expression, bindings = {}) {
    const task = this.queue.then(async () => {
      const context = await this.boot();
      Object.assign(context, bindings);
      return plain(await evaluate(context, expression));
    });
    this.queue = task.catch(() => undefined);
    return task;
  }

  /**
   * The worker's seed readers, for server modules that mirror a native
   * protocol step ahead of the solver (rust/src/protocol_memory.rs): the
   * prompt normalizer, the language detector, the lexicon role queries and
   * the raw `response_for` table lookup (`null` when the intent has no text
   * in that language, like `seed::response_for`). They are pure reads of the
   * loaded seed, so they run outside the solve queue.
   */
  async seedReaders() {
    const context = await this.boot();
    if (!this.readers) {
      this.readers = evaluate(context, `({
        normalizePrompt,
        detectLanguage,
        lexiconMentionsRole,
        roleWordForms,
        responseFor(intent, language) {
          const table = MULTILINGUAL_ANSWERS[intent];
          const raw = table && Object.prototype.hasOwnProperty.call(table, language) ? table[language] : null;
          if (raw === null || raw === undefined) return null;
          return typeof raw === "string" ? raw : String(raw.text ?? "");
        },
      })`);
    }
    return this.readers;
  }

  /**
   * The worker's answer to `prompt` after `history` turns. The Rust server
   * greets with the canonical wording, so the worker's per-reply greeting
   * variation (a browser preference) is off here.
   */
  async solve(prompt, history = [], options = {}) {
    const result = await this.run("solve(__serverPrompt, __serverHistory, { greetingVariations: false }, {}, [], __serverOptions)", {
      __serverPrompt: prompt,
      __serverHistory: history,
      __serverOptions: options,
    });
    // Issue #1165 R1165-10: the native `WriteProgram` branch reads the procedure cache file.
    return applyProcedureCache(result, prompt, this, await this.catalogTemplate());
  }

  /**
   * The worker's catalog template for a `(task, language)` pair (rediscovered
   * from documentation first, issue #1165 R1165-1), what `prompt` rendered it
   * to (the inline replacement a customised request makes), and the
   * rediscovered row when there is one, for the procedure-cache read.
   */
  async catalogTemplate() {
    const context = await this.boot();
    if (!this.templateFor) {
      this.templateFor = evaluate(context, `((prompt, task, language) => {
        const template = writeProgramTemplate(task, language);
        if (typeof template !== "string") return { template: null, rendered: null, documented: null };
        const documented = documentedProgram(task, language).recipe;
        return { template, rendered: applyInlineHelloWorldOutputReplacement(prompt, task, template), documented };
      })`);
    }
    return this.templateFor;
  }
}
