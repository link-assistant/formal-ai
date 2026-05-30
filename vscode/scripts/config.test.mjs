import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  readConfig,
  statusFromConfig,
  withApiReady,
  withApiError,
  serverEnv,
  DEFAULT_HOST,
  DEFAULT_PORT,
  DEFAULT_IMAGE,
} = require("../src/lib/config.cjs");

test("readConfig fills defaults from an empty settings object", () => {
  const cfg = readConfig({});
  assert.equal(cfg.serverEnabled, false);
  assert.equal(cfg.host, DEFAULT_HOST);
  assert.equal(cfg.port, DEFAULT_PORT);
  assert.equal(cfg.dockerImage, DEFAULT_IMAGE);
  assert.equal(cfg.allowToolsByDefault, false);
  assert.equal(cfg.agentDefaultOn, false);
});

test("readConfig reads a VS Code WorkspaceConfiguration via get()", () => {
  const overrides = {
    "server.enabled": true,
    "server.host": "0.0.0.0",
    "server.port": 9999,
    "docker.image": "example/box:1",
    "tools.allowByDefault": true,
    "agent.defaultOn": true,
  };
  const workspaceConfig = {
    get: (key, fallback) => (key in overrides ? overrides[key] : fallback),
  };
  const cfg = readConfig(workspaceConfig);
  assert.equal(cfg.serverEnabled, true);
  assert.equal(cfg.host, "0.0.0.0");
  assert.equal(cfg.port, 9999);
  assert.equal(cfg.dockerImage, "example/box:1");
  assert.equal(cfg.allowToolsByDefault, true);
  assert.equal(cfg.agentDefaultOn, true);
});

test("readConfig also accepts nested plain objects", () => {
  const cfg = readConfig({ server: { enabled: true, port: 4242 } });
  assert.equal(cfg.serverEnabled, true);
  assert.equal(cfg.port, 4242);
});

test("readConfig clamps an invalid port back to the default", () => {
  assert.equal(readConfig({ "server.port": "not-a-number" }).port, DEFAULT_PORT);
  assert.equal(readConfig({ "server.port": -5 }).port, DEFAULT_PORT);
});

test("statusFromConfig defaults to the in-process surface", () => {
  const status = statusFromConfig({}, { shell: "VS Code" });
  assert.equal(status.shell, "VS Code");
  assert.equal(status.mode, "in-process");
  assert.equal(status.apiBase, "");
  assert.equal(status.apiReady, false);
  assert.equal(status.toolCallPolicy, "explicit-permission");
  assert.equal(status.memory, "formal_ai_bundle");
});

test("statusFromConfig keeps the web host in-process even when the server is enabled", () => {
  // The web (browser) host cannot spawn a process, so an enabled server must
  // not flip the surface into server mode.
  const status = statusFromConfig({ "server.enabled": true }, {
    shell: "VS Code Web",
    serverCapable: false,
  });
  assert.equal(status.serverCapable, false);
  assert.equal(status.serverEnabled, false);
  assert.equal(status.mode, "in-process");
  assert.equal(status.apiBase, "");
});

test("statusFromConfig marks server mode on a capable host when enabled", () => {
  const status = statusFromConfig({ "server.enabled": true }, {
    shell: "VS Code",
    serverCapable: true,
  });
  assert.equal(status.serverEnabled, true);
  assert.equal(status.mode, "server");
  // apiBase only appears once the server is actually ready.
  assert.equal(status.apiBase, "");
  assert.equal(status.apiReady, false);
});

test("statusFromConfig reflects the agent default-on setting", () => {
  assert.equal(statusFromConfig({ "agent.defaultOn": true }, {}).agentModeDefault, true);
  assert.equal(statusFromConfig({}, {}).agentModeDefault, false);
});

test("withApiReady derives the chat/graph/trace URLs from apiBase", () => {
  const base = statusFromConfig({ "server.enabled": true }, { serverCapable: true });
  const ready = withApiReady(base, "http://127.0.0.1:18080/");
  assert.equal(ready.apiReady, true);
  assert.equal(ready.apiBase, "http://127.0.0.1:18080");
  assert.equal(ready.chatUrl, "http://127.0.0.1:18080/v1/chat/completions");
  assert.equal(ready.graphUrl, "http://127.0.0.1:18080/v1/graph");
  assert.match(ready.traceUrl, /\/v1\/graph\?trace=/);
});

test("withApiError falls back to in-process and records the message", () => {
  const base = statusFromConfig({ "server.enabled": true }, { serverCapable: true });
  const failed = withApiError(base, new Error("port in use"));
  assert.equal(failed.apiReady, false);
  assert.equal(failed.mode, "in-process");
  assert.equal(failed.apiBase, "");
  assert.equal(failed.apiError, "port in use");
});

test("serverEnv maps host/port to the formal-ai serve environment", () => {
  const env = serverEnv({ "server.host": "127.0.0.1", "server.port": 18080 });
  assert.equal(env.FORMAL_AI_HOST, "127.0.0.1");
  assert.equal(env.FORMAL_AI_PORT, "18080");
});
