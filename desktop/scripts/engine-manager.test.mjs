import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";

const require = createRequire(import.meta.url);
const {
  OUT_OF_BOX_ENGINE,
  createEngineManager,
  detectAvailableEngines,
  selectDefaultEngine,
} = require("../lib/engine-manager.cjs");

test("detects only installed passthrough engines and always offers out-of-box", () => {
  const engines = detectAvailableEngines({
    commandExists: (command) => ["agent", "codex", "claude"].includes(command),
  });

  assert.deepEqual(engines.map(({ id }) => id), [
    OUT_OF_BOX_ENGINE,
    "agent",
    "codex",
    "claude",
  ]);
});

test("agent is the default when installed and out-of-box is the fallback", () => {
  const withAgent = detectAvailableEngines({ commandExists: (command) => command === "agent" });
  const withoutAgent = detectAvailableEngines({ commandExists: () => false });

  assert.equal(selectDefaultEngine(withAgent), "agent");
  assert.equal(selectDefaultEngine(withoutAgent), OUT_OF_BOX_ENGINE);
});

test("an available saved choice wins and an unavailable one is ignored", () => {
  const engines = detectAvailableEngines({ commandExists: (command) => command === "codex" });

  assert.equal(selectDefaultEngine(engines, "codex"), "codex");
  assert.equal(selectDefaultEngine(engines, "claude"), OUT_OF_BOX_ENGINE);
});

test("persists an explicit override and restores it on the next launch", () => {
  let saved = "";
  const dependencies = {
    preferencePath: "/profile/desktop-engine.json",
    commandExists: (command) => ["agent", "codex"].includes(command),
    mkdirSync: () => {},
    readFileSync: () => saved,
    writeFileSync: (_path, value) => { saved = value; },
  };
  const firstLaunch = createEngineManager(dependencies);
  assert.equal(firstLaunch.status().activeEngine, "agent");

  firstLaunch.setActiveEngine("out-of-box");
  assert.equal(JSON.parse(saved).activeEngine, "out-of-box");
  assert.equal(createEngineManager(dependencies).status().activeEngine, "out-of-box");
  assert.throws(() => firstLaunch.setActiveEngine("claude"), /unavailable/);
});
