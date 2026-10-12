// The debugger view's non-UI logic (issue #667, R383; protocol in
// docs/vscode/debugger.md): the `POST /v1/debug/<action>` client of a
// `serve --debug-session` server, the poll that merges public memory records
// with the session's stage events, the projection of one recorded event onto
// the recipe-diagram, Rust-source and JavaScript-source panes, and the
// on-demand Mermaid renderer (js/mermaid-entry.js, bundled as
// js/mermaid.bundle.js). js/app/debugger-view.jsx only renders what these
// return; every value comes from recorded events, never from a guess.

const MERMAID_BUNDLE = "mermaid.bundle.js";
const POLL_INTERVAL_MS = 1500;
const SVG_PROFILE = { USE_PROFILES: { svg: true, svgFilters: true } };

/** One `POST /v1/debug/<action>`; the session token rides in the JSON body. */
export async function debugSessionCall(apiBase, debugToken, action, extra = {}, fetchImpl = globalThis.fetch) {
  const response = await fetchImpl(`${apiBase.replace(/\/$/, "")}/v1/debug/${action}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ token: debugToken, ...extra }),
  });
  const body = await response.json();
  if (!response.ok) throw new Error(body?.error?.message || `HTTP ${response.status}`);
  return body;
}

/**
 * Poll the local memory, the server's `GET /v1/memory/since` delta and, with a
 * debug token, the session's stage events; `onUpdate({events, rawLinks,
 * state, problem})` after every round. Returns the function that stops it.
 */
export function pollDebugger({
  apiBase = "", debugToken = "", memory = globalThis.window?.FormalAiMemory, onUpdate,
  fetchImpl = globalThis.fetch, interval = POLL_INTERVAL_MS,
}) {
  let active = true;
  let timer;
  let cursor = "";
  let debugCursor = 0;
  let rawLinks = "";
  let state = null;
  const records = new Map();
  const abort = new AbortController();
  async function round() {
    let problem = "";
    try {
      const local = memory ? await memory.listEvents() : [];
      let remote = [];
      if (apiBase) {
        const url = new URL(`${apiBase.replace(/\/$/, "")}/v1/memory/since`);
        if (cursor) url.searchParams.set("event", cursor);
        const response = await fetchImpl(url, { signal: abort.signal });
        if (!response.ok) throw new Error(`Memory stream HTTP ${response.status}`);
        rawLinks = await response.text();
        remote = memory ? memory.parseLinksNotation(rawLinks) : [];
        if (remote.length && remote[remote.length - 1].id) cursor = remote[remote.length - 1].id;
      }
      let staged = [];
      if (apiBase && debugToken) {
        state = await debugSessionCall(apiBase, debugToken, "session", { since: debugCursor }, fetchImpl);
        staged = state.events;
        debugCursor = state.next;
      }
      for (const event of [...local, ...remote, ...staged]) records.set(event.id || JSON.stringify(event), event);
    } catch (error) {
      problem = String(error?.message || error);
    }
    if (!active) return;
    onUpdate({ events: [...records.values()], rawLinks, state, problem });
    timer = setTimeout(round, interval);
  }
  round();
  return () => {
    active = false;
    clearTimeout(timer);
    abort.abort();
  };
}

/** The event a debugger shows first: the paused stage's latest event, else `fallback`. */
export function pausedEventIndex(events, state, fallback) {
  const paused = state?.paused?.[0];
  if (!paused) return fallback;
  for (let index = events.length - 1; index >= 0; index -= 1) {
    const event = events[index];
    if (event.turn === paused.turn && event.stage === paused.stage && event.kind === "stage_paused") return index;
  }
  return fallback;
}

/** One runtime's source pane: `path:line`, the symbol and the excerpt, or null. */
function sourcePane(source, line, excerpt) {
  if (!source) return null;
  const split = source.lastIndexOf(":");
  const file = split < 0 ? source : source.slice(0, split);
  const symbol = split < 0 ? "" : source.slice(split + 1);
  return { location: line ? `${file}:${line}` : file, symbol, excerpt: excerpt || "" };
}

/**
 * Project one recorded event onto the diagram and source panes: a stage event
 * carries `mermaid`, `rust_source`/`rust_line`/`rust_excerpt` and the `js_*`
 * twins; older records name a diagram or a source location directly.
 */
export function stagePanes(event = {}) {
  const diagram = event.mermaid || event.diagram ||
    (event.kind === "recipe_diagram" || event.kind === "mermaid" ? event.content : "") || "";
  const legacy = event.sourceLocation || event.source_location || (event.kind === "source_location" ? event.content : "");
  return {
    diagram,
    rust: sourcePane(event.rust_source || legacy, event.rust_line, event.rust_excerpt),
    js: sourcePane(event.js_source, event.js_line, event.js_excerpt),
  };
}

let mermaidPromise = null;

/** Inject js/mermaid.bundle.js once and resolve its `FormalAiMermaid` API. */
export function loadMermaid(src = MERMAID_BUNDLE, win = globalThis.window) {
  if (win?.FormalAiMermaid) return Promise.resolve(win.FormalAiMermaid);
  if (!win?.document) return Promise.reject(new Error("mermaid_unavailable"));
  if (mermaidPromise) return mermaidPromise;
  mermaidPromise = new Promise((resolve, reject) => {
    const script = win.document.createElement("script");
    script.src = src;
    script.async = true;
    script.onload = () => (win.FormalAiMermaid ? resolve(win.FormalAiMermaid) : reject(new Error("mermaid_unavailable")));
    script.onerror = () => reject(new Error("mermaid_unavailable"));
    win.document.head.appendChild(script);
  }).catch((error) => {
    mermaidPromise = null;
    throw error;
  });
  return mermaidPromise;
}

/**
 * The diagram as sanitized SVG markup, or null — the pane then shows the
 * Mermaid source — when there is no diagram or the renderer cannot run.
 */
export async function renderMermaid(source, load = loadMermaid, purifier = globalThis.window?.DOMPurify) {
  if (!source) return null;
  try {
    const svg = await (await load()).render(source);
    return purifier ? purifier.sanitize(svg, SVG_PROFILE) : svg;
  } catch {
    return null;
  }
}
