"use strict";

const childProcess = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const OUT_OF_BOX_ENGINE = "out-of-box";
const PASSTHROUGH_ENGINES = Object.freeze([
  { id: "agent", label: "Agent", command: "agent" },
  { id: "codex", label: "Codex", command: "codex" },
  { id: "claude", label: "Claude", command: "claude" },
]);

function commandExists(command, options = {}) {
  const spawnSync = options.spawnSync || childProcess.spawnSync;
  const platform = options.platform || process.platform;
  const lookup = platform === "win32" ? "where" : "which";
  try {
    const result = spawnSync(lookup, [command], {
      env: options.env || process.env,
      stdio: "ignore",
      windowsHide: true,
    });
    return Boolean(result) && result.status === 0;
  } catch (_error) {
    return false;
  }
}

function detectAvailableEngines(options = {}) {
  const exists = options.commandExists || ((command) => commandExists(command, options));
  const engines = [{
    id: OUT_OF_BOX_ENGINE,
    label: "Out of the box",
    type: "native",
    available: true,
  }];
  for (const candidate of PASSTHROUGH_ENGINES) {
    if (exists(candidate.command)) {
      engines.push({ ...candidate, type: "passthrough", available: true });
    }
  }
  return engines;
}

function selectDefaultEngine(engines, savedEngine = "") {
  const available = new Set(
    (Array.isArray(engines) ? engines : []).filter((engine) => engine && engine.available !== false)
      .map((engine) => String(engine.id || "")),
  );
  const saved = String(savedEngine || "").trim();
  if (saved && available.has(saved)) {
    return saved;
  }
  if (available.has("agent")) {
    return "agent";
  }
  return OUT_OF_BOX_ENGINE;
}

function readSavedEngine(preferencePath, options = {}) {
  const readFileSync = options.readFileSync || fs.readFileSync;
  try {
    const parsed = JSON.parse(readFileSync(preferencePath, "utf8"));
    return String(parsed && parsed.activeEngine || "");
  } catch (_error) {
    return "";
  }
}

function createEngineManager(options = {}) {
  const preferencePath = String(options.preferencePath || "");
  const engines = detectAvailableEngines(options);
  let activeEngine = selectDefaultEngine(
    engines,
    preferencePath ? readSavedEngine(preferencePath, options) : "",
  );

  function status() {
    return {
      activeEngine,
      engines: engines.map((engine) => ({ ...engine })),
    };
  }

  function setActiveEngine(value) {
    const requested = String(value || "").trim();
    if (!engines.some((engine) => engine.id === requested && engine.available !== false)) {
      throw new Error(`Desktop engine is unavailable: ${requested || "(empty)"}`);
    }
    activeEngine = requested;
    if (preferencePath) {
      const mkdirSync = options.mkdirSync || fs.mkdirSync;
      const writeFileSync = options.writeFileSync || fs.writeFileSync;
      mkdirSync(path.dirname(preferencePath), { recursive: true });
      writeFileSync(preferencePath, `${JSON.stringify({ activeEngine }, null, 2)}\n`, "utf8");
    }
    return status();
  }

  return { status, setActiveEngine };
}

module.exports = {
  OUT_OF_BOX_ENGINE,
  PASSTHROUGH_ENGINES,
  commandExists,
  detectAvailableEngines,
  selectDefaultEngine,
  createEngineManager,
};
