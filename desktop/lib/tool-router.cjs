"use strict";

// Permission-gated tool dispatch for the desktop main process.
//
// Issue #347 / R5d (ROADMAP D2): when the local server is on, the agent's side
// effects (web fetches, tool calls, code execution) run through the *local* app
// instead of a remote service. Read-only capabilities are available out of the
// box; writes, shell, and code execution pass an explicit-permission gate first
// (default-deny). Denied calls return a structured refusal and nothing executes.
// Shell commands run on the host
// process by default (the user's desktop machine); code-exec / eval-js tools
// run inside a `konard/box-dind` container — the same image the Telegram
// microservice uses — with a graceful fallback when Docker is unavailable. A
// shell request may still opt into Docker isolation with `input.isolation =
// "docker"` so both execution targets remain available.
//
// The factory takes its effectful dependencies as injectables so the policy and
// dispatch logic are unit-testable without a live network, filesystem, or Docker
// daemon. Per R7 the wire payloads between renderer and main stay Links-Notation
// friendly (plain structured objects), and no new external REST surface is added.

const path = require("node:path");

const SANDBOX_IMAGE = "konard/box-dind:2.1.1";

// The tool vocabulary mirrors the browser environment (see app.js); each maps to
// a local executor here. `code_exec` / `eval_js` are sandboxed, `shell` is host
// shell by default, and the rest are direct local capabilities.
const SUPPORTED_TOOLS = Object.freeze([
  "web_search",
  "web_fetch",
  "read_file",
  "write_file",
  "edit_file",
  "multi_edit",
  "grep",
  "glob",
  "list_directory",
  "read_many_files",
  "http_fetch",
  "url_navigate",
  "eval_js",
  "read_local_file",
  "code_exec",
  "shell",
  "bash",
  "read",
  "write",
  "edit",
  "multiedit",
  "ls",
  "find_files",
  "search_web",
  "fetch_url",
  "exec_command",
]);

const SANDBOXED_TOOLS = Object.freeze(["eval_js", "code_exec"]);
const READ_ONLY_TOOLS = Object.freeze([
  "web_search",
  "web_fetch",
  "http_fetch",
  "url_navigate",
  "read_file",
  "read_local_file",
  "grep",
  "glob",
  "list_directory",
  "read_many_files",
]);
// Agent frontends use different names for the same capability. Normalize their
// common spellings here so routing depends on capability, not a provider's
// preferred tool label.
const TOOL_ALIASES = Object.freeze({
  bash: "shell",
  exec_command: "shell",
  read: "read_file",
  read_local_file: "read_file",
  write: "write_file",
  edit: "edit_file",
  multiedit: "multi_edit",
  ls: "list_directory",
  find_files: "glob",
  search_web: "web_search",
  fetch_url: "web_fetch",
});

function canonicalTool(tool) {
  const normalized = String(tool || "").trim().toLowerCase();
  return TOOL_ALIASES[normalized] || normalized;
}

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
  return grants[tool] === true || grants[canonicalTool(tool)] === true;
}

function createToolRouter(options = {}) {
  const fetchImpl = options.fetchImpl || globalThis.fetch;
  const readFile = options.readFile || null;
  const writeFile = options.writeFile || null;
  const readDirectory = options.readDirectory || null;
  const runInSandbox = options.runInSandbox || null;
  const runOnHost = options.runOnHost || null;
  const dockerAvailable =
    typeof options.dockerAvailable === "function"
      ? options.dockerAvailable
      : () => Boolean(runInSandbox);
  const allowedReadRoot = options.allowedReadRoot || null;
  const resolvePath = options.resolvePath || ((value) => String(value || ""));
  const webSearch = options.webSearch || null;
  const webFetch = options.webFetch || null;

  // Mutable grant state, updated from the renderer's permission toggles.
  let grants = { all: false };

  function safePath(value) {
    const requested = resolvePath(String(value || ""));
    if (!requested) return { error: "a path is required" };
    if (allowedReadRoot) {
      const relative = path.relative(allowedReadRoot, requested);
      if (relative.startsWith("..") || path.isAbsolute(relative)) {
        return { error: "path is outside the allowed root" };
      }
    }
    return { requested };
  }

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
    const checked = safePath(input && (input.path || input.filePath || input.file_path));
    if (!checked.requested) {
      return failure(
        tool,
        checked.error === "path is outside the allowed root" ? "forbidden" : "invalid_input",
        checked.error,
      );
    }
    const requested = checked.requested;
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

  async function writeLocalFile(tool, input, edits = null) {
    if (typeof writeFile !== "function" || typeof readFile !== "function") {
      return failure(tool, "unavailable", "filesystem writer is not configured");
    }
    const checked = safePath(input && (input.path || input.filePath || input.file_path));
    if (!checked.requested) return failure(tool, "forbidden", checked.error);
    try {
      let body = edits ? String(await readFile(checked.requested)) : String(input.content ?? "");
      for (const edit of edits || []) {
        const before = String(edit.oldString ?? edit.old_string ?? "");
        if (!before || !body.includes(before)) {
          return failure(tool, "invalid_input", "edit target was not found");
        }
        body = body.replace(before, String(edit.newString ?? edit.new_string ?? ""));
      }
      await writeFile(checked.requested, body);
      return {
        ok: true,
        tool,
        status: "ok",
        executed: true,
        servedBy: "local-process",
        path: checked.requested,
        body,
      };
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  async function directoryEntries(root, recursive) {
    if (typeof readDirectory !== "function") throw new Error("directory reader is not configured");
    const output = [];
    async function visit(directory) {
      const entries = await readDirectory(directory);
      for (const entry of entries) {
        const name = typeof entry === "string" ? entry : entry.name;
        const fullPath = path.join(directory, name);
        const isDirectory = typeof entry !== "string" && entry.isDirectory();
        output.push({ path: fullPath, isDirectory });
        if (recursive && isDirectory && output.length < 10000) await visit(fullPath);
      }
    }
    await visit(root);
    return output;
  }

  function wildcardRegex(pattern) {
    const escaped = String(pattern || "*")
      .replace(/[.+^${}()|[\]\\]/g, "\\$&")
      .replace(/\*\*/g, "__DOUBLE_STAR__")
      .replace(/\*/g, "[^/]*")
      .replace(/__DOUBLE_STAR__/g, ".*")
      .replace(/\?/g, ".");
    return new RegExp(`^${escaped}$`, "i");
  }

  async function inspectFiles(tool, input) {
    const checked = safePath(input && (input.path || input.directory || "."));
    if (!checked.requested) return failure(tool, "forbidden", checked.error);
    try {
      if (tool === "list_directory") {
        const entries = await directoryEntries(checked.requested, false);
        return { ok: true, tool, status: "ok", executed: true, servedBy: "local-process", entries, body: entries.map((entry) => entry.path).join("\n") };
      }
      const entries = await directoryEntries(checked.requested, true);
      const files = entries.filter((entry) => !entry.isDirectory);
      if (tool === "glob") {
        const matcher = wildcardRegex(input.pattern || input.glob || "**");
        const matches = files.map((entry) => path.relative(checked.requested, entry.path)).filter((name) => matcher.test(name));
        return { ok: true, tool, status: "ok", executed: true, servedBy: "local-process", matches, body: matches.join("\n") };
      }
      const pattern = String(input.pattern || input.query || "");
      if (!pattern) return failure(tool, "invalid_input", "grep requires a pattern");
      const matches = [];
      for (const file of files) {
        const body = String(await readFile(file.path));
        body.split(/\r?\n/).forEach((line, index) => {
          if (line.includes(pattern)) matches.push(`${path.relative(checked.requested, file.path)}:${index + 1}:${line}`);
        });
      }
      return { ok: true, tool, status: "ok", executed: true, servedBy: "local-process", matches, body: matches.join("\n") };
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  async function hostShell(tool, input) {
    if (typeof runOnHost !== "function") {
      return failure(tool, "unavailable", "no host shell runner is configured");
    }
    const command = String((input && input.command) || "");
    if (!command.trim()) {
      return failure(tool, "invalid_input", "shell requires a command");
    }
    try {
      const result = await runOnHost({ tool, command });
      const stdout = result && result.stdout ? String(result.stdout) : "";
      const stderr = result && result.stderr ? String(result.stderr) : "";
      const body = result && result.output ? String(result.output) : `${stdout}${stderr}`;
      return {
        ok: true,
        tool,
        status: "ok",
        executed: true,
        servedBy: "host-shell",
        isolation: "host",
        exitCode: result && typeof result.exitCode === "number" ? result.exitCode : 0,
        logPath: result && result.logPath ? String(result.logPath) : "",
        stdout,
        stderr,
        body,
      };
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  async function sandboxed(tool, input) {
    if (!dockerAvailable()) {
      // Graceful fallback: never run sandbox-requested effects without Docker.
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
        isolation: "docker",
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
    const dispatchTool = canonicalTool(tool);
    const input = (request && request.input) || {};
    if (!SUPPORTED_TOOLS.includes(tool) && !SUPPORTED_TOOLS.includes(dispatchTool)) {
      return failure(tool, "unknown_tool", `unsupported tool: ${tool || "(none)"}`);
    }
    // Explicit-permission gate (default-deny) runs before any side effect.
    if (!READ_ONLY_TOOLS.includes(tool) && !READ_ONLY_TOOLS.includes(dispatchTool) && !isPermitted(grants, tool)) {
      return refusal(tool, "tool call denied by explicit-permission policy");
    }
    if (dispatchTool === "web_search" || dispatchTool === "web_fetch") {
      const executor = dispatchTool === "web_search" ? webSearch : webFetch;
      if (typeof executor !== "function") {
        return failure(tool, "unavailable", `${tool} executor is not configured`);
      }
      try {
        const result = await executor(input);
        return {
          ok: true,
          tool,
          status: "ok",
          executed: true,
          servedBy: "web-capture",
          ...result,
        };
      } catch (error) {
        return failure(tool, "error", error && error.message ? error.message : String(error));
      }
    }
    if (dispatchTool === "read_file") return readLocalFile(tool, input);
    if (dispatchTool === "write_file") return writeLocalFile(tool, input);
    if (dispatchTool === "edit_file") return writeLocalFile(tool, input, [input]);
    if (dispatchTool === "multi_edit") {
      return writeLocalFile(tool, input, Array.isArray(input.edits) ? input.edits : []);
    }
    if (["grep", "glob", "list_directory"].includes(dispatchTool)) {
      return inspectFiles(dispatchTool, input);
    }
    if (dispatchTool === "read_many_files") {
      const paths = Array.isArray(input.paths) ? input.paths : [];
      const results = await Promise.all(paths.map((filePath) => readLocalFile(tool, { path: filePath })));
      return { ok: results.every((result) => result.ok), tool, status: "ok", executed: true, servedBy: "local-process", results, body: results.map((result) => result.body || result.reason).join("\n") };
    }
    if (SANDBOXED_TOOLS.includes(dispatchTool)) {
      return sandboxed(tool, input);
    }
    if (dispatchTool === "shell") {
      return input && input.isolation === "docker"
        ? sandboxed(tool, input)
        : hostShell(tool, input);
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
    isReadOnly: (tool) => READ_ONLY_TOOLS.includes(tool) || READ_ONLY_TOOLS.includes(canonicalTool(tool)),
    invoke,
  };
}

module.exports = {
  SANDBOX_IMAGE,
  SUPPORTED_TOOLS,
  SANDBOXED_TOOLS,
  READ_ONLY_TOOLS,
  TOOL_ALIASES,
  canonicalTool,
  isPermitted,
  createToolRouter,
};
