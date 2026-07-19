import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createBridge } = require("../src/lib/bridge.cjs");

// A tiny fake tool router that records whether a tool actually executed.
function fakeRouter() {
  const calls = [];
  let grants = { all: false };
  return {
    calls,
    setGrants: (next) => {
      grants = next;
      return grants;
    },
    isReadOnly: (tool) => ["web_search", "web_fetch"].includes(tool),
    invoke: async (request) => {
      calls.push(request);
      return { ok: true, tool: request.tool, status: "ok", executed: true, servedBy: "fake" };
    },
  };
}

test("getStatus returns the host-provided status", async () => {
  const bridge = createBridge({ getStatus: () => ({ shell: "VS Code", mode: "in-process" }) });
  const status = await bridge.getStatus();
  assert.equal(status.shell, "VS Code");
  assert.equal(status.mode, "in-process");
});

test("setToolGrants delegates to the tool router", async () => {
  const router = fakeRouter();
  const bridge = createBridge({ toolRouter: router });
  const result = await bridge.setToolGrants({ all: true });
  assert.deepEqual(result, { all: true });
});

test("invokeTool is refused when the server is not enabled (nothing executes)", async () => {
  const router = fakeRouter();
  const bridge = createBridge({ toolRouter: router, serverEnabled: false });
  const result = await bridge.invokeTool({ tool: "code_exec", input: { command: "echo hi" } });
  assert.equal(result.ok, false);
  assert.equal(result.status, "refused");
  assert.equal(result.executed, false);
  assert.match(result.reason, /local server/);
  assert.equal(router.calls.length, 0, "router must not run when the server is off");
});

test("invokeTool routes to the tool router when the server is enabled", async () => {
  const router = fakeRouter();
  const bridge = createBridge({ toolRouter: router, serverEnabled: true });
  const result = await bridge.invokeTool({ tool: "http_fetch", input: { url: "https://x" } });
  assert.equal(result.ok, true);
  assert.equal(result.executed, true);
  assert.equal(router.calls.length, 1);
});

test("read-only web tools route without enabling the local server", async () => {
  const router = fakeRouter();
  const bridge = createBridge({ toolRouter: router, serverEnabled: false });
  const result = await bridge.invokeTool({ tool: "web_search", input: { query: "formal ai" } });
  assert.equal(result.ok, true);
  assert.equal(result.executed, true);
  assert.equal(router.calls.length, 1);
});

test("invokeTool reports unavailable when no router is configured", async () => {
  const bridge = createBridge({ serverEnabled: true });
  const result = await bridge.invokeTool({ tool: "http_fetch" });
  assert.equal(result.ok, false);
  assert.equal(result.status, "unavailable");
});

test("serverEnabled may be a live predicate", async () => {
  const router = fakeRouter();
  let enabled = false;
  const bridge = createBridge({ toolRouter: router, serverEnabled: () => enabled });
  assert.equal((await bridge.invokeTool({ tool: "http_fetch" })).status, "refused");
  enabled = true;
  assert.equal((await bridge.invokeTool({ tool: "http_fetch" })).ok, true);
});

test("syncMemory is unavailable without the server", async () => {
  const bridge = createBridge({
    getStatus: () => ({ apiBase: "" }),
    serverEnabled: false,
    memorySync: { push: async () => ({}), pull: async () => ({}) },
  });
  const result = await bridge.syncMemory({ lino: "formal_ai_bundle" });
  assert.equal(result.ok, false);
  assert.equal(result.status, "unavailable");
});

test("syncMemory pushes then pulls when the server is ready", async () => {
  const events = [];
  const bridge = createBridge({
    getStatus: () => ({ apiBase: "http://127.0.0.1:18080" }),
    serverEnabled: true,
    memorySync: {
      push: async (lino) => {
        events.push(["push", lino]);
        return { added: 1, total: 2 };
      },
      pull: async () => {
        events.push(["pull"]);
        return { delta: "", added: 0, lastSeen: null };
      },
    },
  });
  const result = await bridge.syncMemory({ lino: "formal_ai_bundle\n  event 1" });
  assert.equal(result.ok, true);
  assert.equal(result.status, "ok");
  assert.deepEqual(events[0], ["push", "formal_ai_bundle\n  event 1"]);
  assert.deepEqual(events[1], ["pull"]);
});

test("openExternal accepts http(s) urls only", async () => {
  const opened = [];
  const bridge = createBridge({ openExternal: async (url) => opened.push(url) });
  assert.equal(await bridge.openExternal("https://example.com"), true);
  assert.equal(await bridge.openExternal("javascript:alert(1)"), false);
  assert.equal(await bridge.openExternal("file:///etc/passwd"), false);
  assert.deepEqual(opened, ["https://example.com"]);
});

test("dispatch routes by method name and rejects unknown methods", async () => {
  const bridge = createBridge({ getStatus: () => ({ shell: "VS Code" }) });
  const status = await bridge.dispatch("getStatus");
  assert.equal(status.shell, "VS Code");
  await assert.rejects(() => bridge.dispatch("danger"), /unknown bridge method/);
});
