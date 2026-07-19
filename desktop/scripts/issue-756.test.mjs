import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { resolveSharedMemoryPath } = require("../lib/shared-memory.cjs");
const { scrubbedEnvironment } = require("../lib/local-server.cjs");
const { serviceDefinitions } = require("../lib/service-control.cjs");
const { serverEnv } = require("../../vscode/src/lib/config.cjs");

test("desktop shared path honors overrides and platform defaults", () => {
  assert.equal(
    resolveSharedMemoryPath({ FORMAL_AI_MEMORY_PATH: "/custom/memory.lino" }, {
      platform: "linux",
      homeDir: "/home/alice",
    }),
    "/custom/memory.lino",
  );
  assert.equal(
    resolveSharedMemoryPath({}, { platform: "darwin", homeDir: "/Users/alice" }),
    "/Users/alice/.formal-ai/memory.lino",
  );
  assert.equal(
    resolveSharedMemoryPath({ APPDATA: "C:\\Users\\alice\\AppData\\Roaming" }, {
      platform: "win32",
      pathImpl: require("node:path").win32,
    }),
    "C:\\Users\\alice\\AppData\\Roaming\\formal-ai\\memory.lino",
  );
});

test("desktop and VS Code child servers receive the shared host memory file", () => {
  const desktopEnv = scrubbedEnvironment(19494, {
    PATH: "/bin",
    HOME: "/home/alice",
  });
  assert.equal(desktopEnv.FORMAL_AI_MEMORY_PATH, "/home/alice/.formal-ai/memory.lino");

  const vscodeEnv = serverEnv(
    { "server.host": "127.0.0.1", "server.port": 18080 },
    { memoryPath: "/home/alice/.formal-ai/memory.lino" },
  );
  assert.equal(vscodeEnv.FORMAL_AI_MEMORY_PATH, "/home/alice/.formal-ai/memory.lino");
});

test("all desktop Docker services share one memory mount and container path", () => {
  const definitions = serviceDefinitions({
    HOME: "/home/alice",
    TELEGRAM_BOT_TOKEN: "token",
  });
  const argumentSets = [
    definitions.telegram.buildRunArgs({ token: "token" }),
    definitions.server.buildRunArgs(),
    definitions.agent.buildRunArgs(),
  ];
  for (const args of argumentSets) {
    assert.ok(args.includes("/home/alice/.formal-ai:/root/.formal-ai"));
    assert.ok(args.includes("FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino"));
  }
  assert.ok(argumentSets[0].includes("formal-ai-telegram-docker:/var/lib/docker"));
  assert.ok(argumentSets[1].includes("formal-ai-server-docker:/var/lib/docker"));
  assert.ok(argumentSets[2].includes("formal-ai-agent-docker:/var/lib/docker"));
});