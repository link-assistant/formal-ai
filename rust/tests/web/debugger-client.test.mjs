// The debugger view's non-UI logic (issue #667, R383, js/debugger-client.js):
// the debug-session client, the poll that merges memory records with stage
// events, the projection of a recorded stage event onto the recipe-diagram,
// Rust-source and JavaScript-source panes (the code that emits that stage),
// and the on-demand Mermaid renderer
// with its source fallback. js/app/debugger-view.jsx renders exactly these.

import assert from "node:assert/strict";
import { describe, test } from "node:test";

import {
  debugSessionCall, loadMermaid, pausedEventIndex, pollDebugger, renderMermaid, stagePanes,
} from "../../../js/debugger-client.js";
import { hasHost, installHost } from "../../../js/agentic/host.mjs";
import { DebugSession } from "../../../js/server/debug-session.mjs";
import { parseLino, readRepoFile } from "../../../js/server/lino.mjs";

const TOKEN = "debugger-client-token";
const ROUTED = [
  { id: "a", step: "impulse", detail: "What is 2 + 2?", source_event: "impulse" },
  { id: "b", step: "formalize", detail: "arithmetic", source_event: "intent_formalization:route" },
  { id: "c", step: "deformalize", detail: "2 + 2 = 4", source_event: "response" },
];

const jsonResponse = (status, body) => ({ ok: status < 400, status, json: async () => body, text: async () => JSON.stringify(body) });

describe("the debug-session client", () => {
  test("posts the action with the token in the body and surfaces the error envelope", async () => {
    const calls = [];
    const fetchImpl = async (url, init) => {
      calls.push([String(url), JSON.parse(init.body)]);
      return calls.length === 1
        ? jsonResponse(200, { object: "debug.session" })
        : jsonResponse(409, { error: { message: "debug_stage_not_paused" } });
    };
    assert.deepEqual(await debugSessionCall("http://127.0.0.1:1/", TOKEN, "advance", { turn: "turn_1", stage: 0 }, fetchImpl),
      { object: "debug.session" });
    assert.deepEqual(calls[0], ["http://127.0.0.1:1/v1/debug/advance", { token: TOKEN, turn: "turn_1", stage: 0 }]);
    await assert.rejects(debugSessionCall("http://127.0.0.1:1", TOKEN, "advance", {}, fetchImpl), /^Error: debug_stage_not_paused$/);
  });

  test("the poll merges memory and stage events and advances both cursors", async () => {
    const urls = [];
    const memory = {
      listEvents: async () => [{ id: "local_1", kind: "user" }],
      parseLinksNotation: (text) => (text ? [{ id: "remote_1", kind: "assistant" }] : []),
    };
    const fetchImpl = async (url, init) => {
      urls.push(init?.body ? JSON.parse(init.body).since : String(url));
      if (String(url).includes("/v1/memory/since")) return { ok: true, status: 200, text: async () => "(remote_1)" };
      return jsonResponse(200, { events: [{ id: "debug_event_1", kind: "stage_paused" }], next: 1, paused: [] });
    };
    const updates = [];
    let stop;
    await new Promise((resolve) => {
      stop = pollDebugger({
        apiBase: "http://127.0.0.1:1", debugToken: TOKEN, memory, fetchImpl, interval: 1,
        onUpdate: (update) => {
          updates.push(update);
          if (updates.length === 2) resolve();
        },
      });
    });
    stop();
    assert.deepEqual(updates[0].events.map((event) => event.id), ["local_1", "remote_1", "debug_event_1"]);
    assert.equal(updates[0].rawLinks, "(remote_1)");
    assert.equal(updates[0].problem, "");
    assert.deepEqual(urls, ["http://127.0.0.1:1/v1/memory/since", 0, "http://127.0.0.1:1/v1/memory/since?event=remote_1", 1]);
  });
});

describe("the stage panes", () => {
  test("a recorded stage event fills the diagram and the panes of the code that emits it", async () => {
    if (!hasHost()) installHost({ readText: readRepoFile, parseLino });
    const session = new DebugSession(TOKEN);
    session.pause();
    const held = session.gate(ROUTED);
    const pane = (path, line, symbol, excerpt) => ({ location: `${path}:${line}`, symbol, excerpt });
    const event = session.events[0];
    const panes = stagePanes(event);
    assert.equal(panes.diagram, event.mermaid);
    assert.match(panes.diagram, /\n {4}class s0 current$/);
    assert.deepEqual(panes.rust, pane("rust/src/solver.rs", event.rust_line,
      "solve_with_history_probability_store_and_intent_cache", event.rust_excerpt));
    assert.deepEqual(panes.js, pane("js/worker/formal_ai_worker_solver_events.js", event.js_line,
      "solverEventLog", event.js_excerpt));
    assert.ok(event.rust_line > 0 && event.js_line > 0);
    // The panes follow the stage: the last one is the finalizer's.
    session.advance("turn_1", 0);
    session.advance("turn_1", 1);
    const last = stagePanes(session.events.at(-1));
    assert.deepEqual([last.rust.symbol, last.js.symbol], ["finalize_simple", "solverEventLog"]);
    assert.match(last.rust.location, /^rust\/src\/solver_handlers\/mod\.rs:\d+$/);
    session.release();
    await held;
  });

  test("an event without a location shows none, and older records still project", () => {
    assert.deepEqual(stagePanes({ kind: "stage_paused", mermaid: "flowchart TD", rust_source: "", js_source: "" }),
      { diagram: "flowchart TD", rust: null, js: null });
    assert.deepEqual(stagePanes({ kind: "recipe_diagram", content: "graph LR" }), { diagram: "graph LR", rust: null, js: null });
    assert.deepEqual(stagePanes({ source_location: "src/x.rs:run" }).rust, { location: "src/x.rs", symbol: "run", excerpt: "" });
    assert.deepEqual(stagePanes(), { diagram: "", rust: null, js: null });
  });

  test("the paused stage's event is shown first", () => {
    const events = [
      { kind: "stage_paused", turn: "turn_1", stage: 0 },
      { kind: "stage_advanced", turn: "turn_1", stage: 0 },
      { kind: "stage_paused", turn: "turn_1", stage: 1 },
      { kind: "user" },
    ];
    assert.equal(pausedEventIndex(events, { paused: [{ turn: "turn_1", stage: 1 }] }, 3), 2);
    assert.equal(pausedEventIndex(events, { paused: [] }, 3), 3);
    assert.equal(pausedEventIndex(events, null, 1), 1);
  });
});

describe("the Mermaid renderer", () => {
  test("renders sanitized SVG, and falls back to the source when it cannot", async () => {
    const sanitized = [];
    const purifier = { sanitize: (svg, profile) => { sanitized.push(profile); return `clean:${svg}`; } };
    const load = async () => ({ render: async (source) => `<svg>${source}</svg>` });
    assert.equal(await renderMermaid("flowchart TD", load, purifier), "clean:<svg>flowchart TD</svg>");
    assert.deepEqual(sanitized, [{ USE_PROFILES: { svg: true, svgFilters: true } }]);
    assert.equal(await renderMermaid("", load, purifier), null, "no diagram, nothing rendered");
    assert.equal(await renderMermaid("flowchart TD", async () => { throw new Error("offline"); }), null);
    assert.equal(await renderMermaid("bad", async () => ({ render: async () => { throw new Error("parse"); } })), null);
  });

  test("the bundle is injected once, from the page's own origin", async () => {
    await assert.rejects(loadMermaid("mermaid.bundle.js", {}), /mermaid_unavailable/);
    const api = { render: async () => "<svg/>" };
    assert.equal(await loadMermaid("mermaid.bundle.js", { FormalAiMermaid: api }), api);
    const appended = [];
    const win = {
      document: { createElement: () => ({}), head: { appendChild: (script) => appended.push(script) } },
    };
    const first = loadMermaid("mermaid.bundle.js?v=1", win);
    const second = loadMermaid("mermaid.bundle.js?v=1", win);
    assert.equal(first, second, "one injection serves every caller");
    assert.equal(appended.length, 1);
    assert.equal(appended[0].src, "mermaid.bundle.js?v=1");
    win.FormalAiMermaid = api;
    appended[0].onload();
    assert.equal(await first, api);
  });
});
