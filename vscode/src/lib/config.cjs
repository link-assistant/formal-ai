"use strict";

// Pure configuration mapper for the VS Code extension.
//
// Issue #353 (ROADMAP D2): the extension reads its behaviour from VS Code
// settings (`formal-ai.*`) and turns them into the same `desktopStatus` shape
// the web chat UI already understands from the Electron desktop shell (see
// `desktop/main.cjs`). Keeping this mapping pure — no `vscode` import, no I/O —
// means the host wiring stays thin and the policy is unit-testable.
//
// The web app drives every desktop affordance off this status object via
// `normalizeDesktopStatus(status)` in `src/web/app.js`: it only routes chat to
// the local server when both `apiReady` and `apiBase` are set, and otherwise
// stays on the in-process symbolic engine. So an in-process surface is simply a
// status with an empty `apiBase`.

const DEFAULT_SHELL = "VS Code";
const DEFAULT_HOST = "127.0.0.1";
const DEFAULT_PORT = 18080;
const DEFAULT_IMAGE = "konard/box-dind:2.1.1";
const MEMORY_BUNDLE = "formal_ai_bundle";

// Accept either a VS Code `WorkspaceConfiguration` (which exposes
// `get(key, fallback)`) or a plain object (used by the unit tests). Plain
// objects may use either dotted flat keys (`"server.enabled"`) or nested
// objects (`{ server: { enabled: true } }`).
function makeGetter(raw) {
  if (raw && typeof raw.get === "function") {
    return (key, fallback) => {
      const value = raw.get(key, fallback);
      return value === undefined ? fallback : value;
    };
  }
  const source = raw && typeof raw === "object" ? raw : {};
  return (key, fallback) => {
    if (Object.prototype.hasOwnProperty.call(source, key)) {
      return source[key];
    }
    let node = source;
    for (const part of key.split(".")) {
      if (node && typeof node === "object" && part in node) {
        node = node[part];
      } else {
        return fallback;
      }
    }
    return node === undefined ? fallback : node;
  };
}

// Normalise raw settings into a typed, defaulted record.
function readConfig(raw) {
  const get = makeGetter(raw);
  const port = Number(get("server.port", DEFAULT_PORT));
  return {
    serverEnabled: Boolean(get("server.enabled", false)),
    host: String(get("server.host", DEFAULT_HOST) || DEFAULT_HOST),
    port: Number.isFinite(port) && port > 0 ? Math.floor(port) : DEFAULT_PORT,
    dockerImage: String(get("docker.image", DEFAULT_IMAGE) || DEFAULT_IMAGE),
    allowToolsByDefault: Boolean(get("tools.allowByDefault", false)),
    agentDefaultOn: Boolean(get("agent.defaultOn", false)),
  };
}

// Build the initial `desktopStatus`. `serverCapable` is the host's physical
// ability to spawn a process (true on the Node host, false on the web host);
// the local server only counts as enabled when the user opted in *and* the host
// can actually run it. The status starts with `apiReady: false` and an empty
// `apiBase`; the Node host promotes it via `withApiReady` once `formal-ai serve`
// answers its health check.
function statusFromConfig(raw, options = {}) {
  const cfg = readConfig(raw);
  const shell = String(options.shell || DEFAULT_SHELL);
  const serverCapable = options.serverCapable !== false;
  const serverEnabled = cfg.serverEnabled && serverCapable;
  return {
    shell,
    mode: serverEnabled ? "server" : "in-process",
    apiBase: "",
    staticBase: "",
    graphUrl: "",
    chatUrl: "",
    traceUrl: "",
    memory: MEMORY_BUNDLE,
    agentModeDefault: cfg.agentDefaultOn,
    toolCallPolicy: "explicit-permission",
    apiReady: false,
    apiError: "",
    // Host-side extras (ignored by the web app, used by the extension host).
    dockerImage: cfg.dockerImage,
    allowToolsByDefault: cfg.allowToolsByDefault,
    serverCapable,
    serverEnabled,
    host: cfg.host,
    port: cfg.port,
  };
}

// Promote a status to "server ready": mirror `desktop/main.cjs` (lines 283-290)
// by deriving the chat/graph/trace URLs from the now-known `apiBase`.
function withApiReady(status, apiBase) {
  const base = String(apiBase || "").replace(/\/+$/, "");
  return {
    ...status,
    mode: "server",
    apiBase: base,
    chatUrl: `${base}/v1/chat/completions`,
    graphUrl: `${base}/v1/graph`,
    traceUrl: `${base}/v1/graph?trace=answer_greeting_hi`,
    apiReady: true,
    apiError: "",
  };
}

// Record a server start-up failure without losing the rest of the status, so the
// web app falls back to the in-process engine and surfaces the error.
function withApiError(status, error) {
  return {
    ...status,
    mode: "in-process",
    apiBase: "",
    apiReady: false,
    apiError: error && error.message ? error.message : String(error || "unknown error"),
  };
}

// Environment for the spawned `formal-ai serve` process (mirrors
// `desktop/main.cjs` `scrubbedEnvironment`).
function serverEnv(raw) {
  const cfg = readConfig(raw);
  return {
    FORMAL_AI_HOST: cfg.host,
    FORMAL_AI_PORT: String(cfg.port),
  };
}

module.exports = {
  DEFAULT_SHELL,
  DEFAULT_HOST,
  DEFAULT_PORT,
  DEFAULT_IMAGE,
  MEMORY_BUNDLE,
  readConfig,
  statusFromConfig,
  withApiReady,
  withApiError,
  serverEnv,
};
