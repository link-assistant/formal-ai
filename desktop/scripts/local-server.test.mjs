import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  createLocalServerManager,
  scrubbedEnvironment,
  serverModeRequested,
} = require("../lib/local-server.cjs");

function makeManager(options = {}) {
  const starts = [];
  const healthChecks = [];
  const healthResults = [...(options.healthResults || [])];
  const manager = createLocalServerManager({
    findFreePort: async () => options.port || 19090,
    requestHealth: async (port) => {
      healthChecks.push(port);
      return healthResults.length > 0 ? healthResults.shift() : true;
    },
    startApiProcess: async (port) => {
      starts.push(port);
      if (options.startError) {
        throw options.startError;
      }
      return { pid: starts.length, killed: false, kill() { this.killed = true; } };
    },
  });
  return { manager, starts, healthChecks };
}

test("ensure starts the local OpenAI-compatible server and exposes provider apiBase", async () => {
  const { manager, starts, healthChecks } = makeManager({ port: 19191 });
  const status = await manager.ensure();

  assert.equal(status.apiReady, true);
  assert.equal(status.mode, "server");
  assert.equal(status.apiBase, "http://127.0.0.1:19191");
  assert.equal(status.chatUrl, "http://127.0.0.1:19191/v1/chat/completions");
  assert.deepEqual(status.agentProvider, {
    type: "local-openai-compatible",
    apiBase: "http://127.0.0.1:19191",
    openAiBaseUrl: "http://127.0.0.1:19191/v1",
    model: "formal-symbolic-production",
  });
  assert.deepEqual(starts, [19191]);
  assert.deepEqual(healthChecks, []);
});

test("ensure reuses an already healthy server instead of starting twice", async () => {
  const { manager, starts, healthChecks } = makeManager({ healthResults: [true] });

  const first = await manager.ensure();
  const second = await manager.ensure();

  assert.equal(first.reused, false);
  assert.equal(second.reused, true);
  assert.equal(second.apiReady, true);
  assert.deepEqual(starts, [19090]);
  assert.deepEqual(healthChecks, [19090]);
});

test("ensure restarts when the remembered server is no longer healthy", async () => {
  const { manager, starts, healthChecks } = makeManager({ healthResults: [false] });

  await manager.ensure();
  const restarted = await manager.ensure();

  assert.equal(restarted.reused, false);
  assert.equal(restarted.apiReady, true);
  assert.deepEqual(starts, [19090, 19090]);
  assert.deepEqual(healthChecks, [19090]);
});

test("concurrent ensure calls share one start operation", async () => {
  let releaseStart;
  const starts = [];
  const manager = createLocalServerManager({
    findFreePort: async () => 19292,
    requestHealth: async () => true,
    startApiProcess: async (port) => {
      starts.push(port);
      await new Promise((resolve) => {
        releaseStart = resolve;
      });
      return { pid: 1, killed: false, kill() { this.killed = true; } };
    },
  });

  const first = manager.ensure();
  const second = manager.ensure();
  await new Promise((resolve) => setImmediate(resolve));
  releaseStart();
  const statuses = await Promise.all([first, second]);

  assert.equal(statuses[0].apiReady, true);
  assert.equal(statuses[1].apiReady, true);
  assert.deepEqual(starts, [19292]);
});

test("ensure records startup failures without losing the reserved apiBase", async () => {
  const { manager, starts } = makeManager({
    port: 19393,
    startError: new Error("boom"),
  });
  const status = await manager.ensure();

  assert.equal(status.apiReady, false);
  assert.equal(status.mode, "server");
  assert.equal(status.apiBase, "http://127.0.0.1:19393");
  assert.equal(status.apiError, "boom");
  assert.deepEqual(starts, [19393]);
});

test("server opt-in accepts truthy values only", () => {
  for (const value of ["1", "true", "yes", "on", " TRUE "]) {
    assert.equal(serverModeRequested({ FORMAL_AI_DESKTOP_SERVER: value }), true);
  }
  for (const value of ["", "0", "false", "no", "off", "agent"]) {
    assert.equal(serverModeRequested({ FORMAL_AI_DESKTOP_SERVER: value }), false);
  }
});

test("child server environment is loopback-only and scrubs API bearer tokens", () => {
  const env = scrubbedEnvironment(19494, {
    PATH: "/bin",
    FORMAL_AI_API_BEARER_TOKEN: "api-secret",
    FORMAL_AI_HTTP_BEARER_TOKEN: "http-secret",
    FORMAL_AI_API_TOKEN: "legacy-secret",
  });

  assert.equal(env.PATH, "/bin");
  assert.equal(env.FORMAL_AI_HOST, "127.0.0.1");
  assert.equal(env.FORMAL_AI_PORT, "19494");
  assert.equal(Object.hasOwn(env, "FORMAL_AI_API_BEARER_TOKEN"), false);
  assert.equal(Object.hasOwn(env, "FORMAL_AI_HTTP_BEARER_TOKEN"), false);
  assert.equal(Object.hasOwn(env, "FORMAL_AI_API_TOKEN"), false);
});
