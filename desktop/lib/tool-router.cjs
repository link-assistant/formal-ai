"use strict";

// Permission-gated tool dispatch for the desktop main process.
//
// Issue #347 / R5d (ROADMAP D2): when the local server is on, the agent's side
// effects (web fetches, tool calls, code execution) run through the *local* app
// and its Docker sandbox instead of a remote service. Every call passes an
// explicit-permission gate first (default-deny); denied calls return a
// structured refusal and nothing executes. Code-exec / shell tools run inside a
// `konard/box-dind` container — the same image the Telegram microservice uses —
// with a graceful fallback when Docker is unavailable.
//
// The factory takes its effectful dependencies as injectables so the policy and
// dispatch logic are unit-testable without a live network, filesystem, or Docker
// daemon. Per R7 the wire payloads between renderer and main stay Links-Notation
// friendly (plain structured objects), and no new external REST surface is added.

const SANDBOX_IMAGE = "konard/box-dind:2.1.1";

// The tool vocabulary mirrors the browser environment (see app.js); each maps to
// a local executor here. `code_exec` / `shell` are sandboxed; the rest are
// direct local effects behind the permission gate.
const SUPPORTED_TOOLS = Object.freeze([
  "http_fetch",
  "url_navigate",
  "eval_js",
  "read_local_file",
  "code_exec",
  "shell",
]);

const SANDBOXED_TOOLS = Object.freeze(["eval_js", "code_exec", "shell"]);

function refusal(tool, reason) {
  return {
    ok: false,
    tool,
    status: "refused",
    executed: false,
    reason,
  };
}

function failure(tool, status, reason) {
  return {
    ok: false,
    tool,
    status,
    executed: false,
    reason,
  };
}

// Default-deny: a tool runs only when the grants map explicitly enables it. An
// `all` grant opts every tool in at once (used by the "allow tools" toggle).
function isPermitted(grants, tool) {
  if (!grants || typeof grants !== "object") {
    return false;
  }
  if (grants.all === true) {
    return true;
  }
  return grants[tool] === true;
}

function createToolRouter(options = {}) {
  const fetchImpl = options.fetchImpl || globalThis.fetch;
  const readFile = options.readFile || null;
  const runInSandbox = options.runInSandbox || null;
  const dockerAvailable =
    typeof options.dockerAvailable === "function"
      ? options.dockerAvailable
      : () => Boolean(runInSandbox);
  const allowedReadRoot = options.allowedReadRoot || null;
  const resolvePath = options.resolvePath || ((value) => String(value || ""));

  // Mutable grant state, updated from the renderer's permission toggles.
  let grants = { all: false };

  function setGrants(next) {
    grants = next && typeof next === "object" ? { ...next } : { all: false };
    return grants;
  }

  async function httpFetch(tool, input) {
    const url = String((input && input.url) || "");
    if (!/^https?:\/\//i.test(url)) {
      return failure(tool, "invalid_input", "http_fetch requires an http(s) url");
    }
    if (typeof fetchImpl !== "function") {
      return failure(tool, "unavailable", "no fetch implementation is configured");
    }
    try {
      const response = await fetchImpl(url, { method: "GET" });
      const body = typeof response.text === "function" ? await response.text() : "";
      return {
        ok: true,
        tool,
        status: "ok",
        executed: true,
        servedBy: "local-process",
        httpStatus: response.status,
        body,
      };
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  async function readLocalFile(tool, input) {
    if (typeof readFile !== "function") {
      return failure(tool, "unavailable", "no filesystem reader is configured");
    }
    const requested = resolvePath(String((input && input.path) || ""));
    if (!requested) {
      return failure(tool, "invalid_input", "read_local_file requires a path");
    }
    // Confine reads to an allowed root when one is configured.
    if (allowedReadRoot && !requested.startsWith(allowedReadRoot)) {
      return failure(tool, "forbidden", "path is outside the allowed root");
    }
    try {
      const body = await readFile(requested);
      return {
        ok: true,
        tool,
        status: "ok",
        executed: true,
        servedBy: "local-process",
        path: requested,
        body: String(body),
      };
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  async function sandboxed(tool, input) {
    if (!dockerAvailable()) {
      // Graceful fallback: never run code-exec / shell unsandboxed.
      return failure(
        tool,
        "sandbox_unavailable",
        `Docker sandbox (${SANDBOX_IMAGE}) is unavailable; refusing to run unsandboxed`,
      );
    }
    if (typeof runInSandbox !== "function") {
      return failure(tool, "unavailable", "no sandbox runner is configured");
    }
    const command = String((input && (input.command || input.code)) || "");
    if (!command.trim()) {
      return failure(tool, "invalid_input", `${tool} requires a command`);
    }
    try {
      const result = await runInSandbox({ image: SANDBOX_IMAGE, tool, command });
      return {
        ok: true,
        tool,
        status: "ok",
        executed: true,
        servedBy: "box-dind",
        image: SANDBOX_IMAGE,
        exitCode: result && typeof result.exitCode === "number" ? result.exitCode : 0,
        logPath: result && result.logPath ? String(result.logPath) : "",
        body: result && result.output ? String(result.output) : "",
      };
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  async function invoke(request) {
    const tool = String((request && request.tool) || "");
    const input = (request && request.input) || {};
    if (!SUPPORTED_TOOLS.includes(tool)) {
      return failure(tool, "unknown_tool", `unsupported tool: ${tool || "(none)"}`);
    }
    // Explicit-permission gate (default-deny) runs before any side effect.
    if (!isPermitted(grants, tool)) {
      return refusal(tool, "tool call denied by explicit-permission policy");
    }
    if (SANDBOXED_TOOLS.includes(tool)) {
      return sandboxed(tool, input);
    }
    if (tool === "read_local_file") {
      return readLocalFile(tool, input);
    }
    // http_fetch and url_navigate are both local GET fetches.
    return httpFetch(tool, input);
  }

  return {
    SANDBOX_IMAGE,
    SUPPORTED_TOOLS,
    SANDBOXED_TOOLS,
    setGrants,
    getGrants: () => ({ ...grants }),
    isPermitted: (tool) => isPermitted(grants, tool),
    invoke,
  };
}

module.exports = {
  SANDBOX_IMAGE,
  SUPPORTED_TOOLS,
  SANDBOXED_TOOLS,
  isPermitted,
  createToolRouter,
};
