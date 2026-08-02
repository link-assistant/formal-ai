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
const crypto = require("node:crypto");

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
  "fs.read",
  "fs.write",
  "fs.list",
  "fs.move",
  "shell.run",
  "http.fetch",
  "http.post",
  "dom.query",
  "dom.extract",
  "archive.pack",
  "archive.unpack",
  "process.status",
]);

const SANDBOXED_TOOLS = Object.freeze(["eval_js", "code_exec"]);
const COMPUTER_USE_TOOLS = Object.freeze([
  "fs.read", "fs.write", "fs.list", "fs.move", "shell.run", "http.fetch",
  "http.post", "dom.query", "dom.extract", "archive.pack", "archive.unpack",
  "process.status",
]);
const COMPUTER_USE_EFFECTS = Object.freeze([
  "fs.write", "fs.move", "shell.run", "http.post", "archive.pack", "archive.unpack",
]);
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
  const moveFile = options.moveFile || null;
  const createDirectory = options.createDirectory || null;
  const runInSandbox = options.runInSandbox || null;
  const runOnHost = options.runOnHost || null;
  const dockerAvailable =
    typeof options.dockerAvailable === "function"
      ? options.dockerAvailable
      : () => Boolean(runInSandbox);
  const allowedReadRoot = options.allowedReadRoot || null;
  const computerUseRoot = options.computerUseRoot || allowedReadRoot;
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

  function computerSafePath(input, value) {
    if (!computerUseRoot) return { error: "computer-use isolation root is not configured" };
    const planId = String(input && input.plan_id || "");
    if (
      planId.length > 128
      || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(planId)
      || planId === "."
      || planId === ".."
    ) {
      return { error: "computer-use plan_id is not safe for workspace isolation" };
    }
    const relative = String(value || "");
    if (!relative || path.isAbsolute(relative)) {
      return { error: "computer-use paths must be non-empty and relative" };
    }
    const planRoot = path.resolve(computerUseRoot, planId);
    const requested = path.resolve(planRoot, relative);
    const confined = path.relative(planRoot, requested);
    if (confined.startsWith("..") || path.isAbsolute(confined)) {
      return { error: "path is outside the isolated computer-use workspace" };
    }
    return { requested };
  }

  function computerArchiveTarget(input, destination, entryPath) {
    const destinationPath = computerSafePath(input, destination);
    const entry = String(entryPath || "");
    if (!destinationPath.requested || !entry || path.isAbsolute(entry)) {
      return {
        error: destinationPath.error || "archive entry path must be non-empty and relative",
      };
    }
    const relative = path.join(String(destination), entry);
    const targetPath = computerSafePath(input, relative);
    if (!targetPath.requested) return targetPath;
    const confined = path.relative(destinationPath.requested, targetPath.requested);
    if (
      confined === ""
      || confined === ".."
      || confined.startsWith(`..${path.sep}`)
      || path.isAbsolute(confined)
    ) {
      return { error: "archive entry is outside the requested extraction directory" };
    }
    return { requested: targetPath.requested, relative };
  }

  function setGrants(next) {
    grants = next && typeof next === "object" ? { ...next } : { all: false };
    return grants;
  }

  function sha256(value) {
    return crypto.createHash("sha256").update(String(value ?? "")).digest("hex");
  }

  function computerRecord(tool, input, result, postconditionVerified) {
    const planId = String(input && input.plan_id || "desktop");
    const stepId = String(input && input.step_id || `${tool}-01`);
    const effectPassed = Boolean(result && result.ok);
    const postPassed = effectPassed && Boolean(postconditionVerified);
    const postcondition = String(input && input.postcondition || "primitive output verified");
    return {
      ...result,
      tool,
      verified: effectPassed && postPassed,
      verificationEvents: [
        { id: `${planId}:${stepId}:precondition`, phase: "precondition", passed: true, detail: String(input && input.precondition) },
        { id: `${planId}:${stepId}:effect`, phase: "effect", passed: effectPassed, detail: effectPassed ? `executed=${tool}` : String(result && result.reason || "effect failed") },
        { id: `${planId}:${stepId}:postcondition`, phase: "postcondition", passed: postPassed, detail: postPassed ? postcondition : `postcondition_failed: ${postcondition}` },
      ],
    };
  }

  function computerFailure(tool, input, status, reason) {
    const planId = String(input && input.plan_id || "desktop");
    const stepId = String(input && input.step_id || `${tool}-01`);
    return {
      ...failure(tool, status, reason),
      verified: false,
      verificationEvents: [
        { id: `${planId}:${stepId}:precondition`, phase: "precondition", passed: false, detail: reason },
        { id: `${planId}:${stepId}:effect`, phase: "effect", passed: false, detail: reason },
        { id: `${planId}:${stepId}:postcondition`, phase: "postcondition", passed: false, detail: reason },
      ],
    };
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

  async function readLocalFile(tool, input, checkPath = safePath) {
    if (typeof readFile !== "function") {
      return failure(tool, "unavailable", "no filesystem reader is configured");
    }
    const checked = checkPath(input && (input.path || input.filePath || input.file_path));
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

  async function writeLocalFile(tool, input, edits = null, checkPath = safePath) {
    if (typeof writeFile !== "function" || typeof readFile !== "function") {
      return failure(tool, "unavailable", "filesystem writer is not configured");
    }
    const checked = checkPath(input && (input.path || input.filePath || input.file_path));
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
    output.sort((left, right) => left.path.localeCompare(right.path));
    return output;
  }

  async function observedFile(input, relative) {
    if (typeof readFile !== "function") return null;
    const checked = computerSafePath(input, relative);
    if (!checked.requested) return null;
    try {
      const body = String(await readFile(checked.requested));
      return { path: checked.requested, body, sha256: sha256(body) };
    } catch {
      return null;
    }
  }

  async function readComputerFile(tool, input, relative) {
    return readLocalFile(
      tool,
      { ...input, path: relative },
      (value) => computerSafePath(input, value),
    );
  }

  async function writeComputerFile(tool, input, relative, content) {
    const checked = computerSafePath(input, relative);
    if (!checked.requested) return failure(tool, "forbidden", checked.error);
    if (typeof createDirectory === "function") {
      try {
        await createDirectory(path.dirname(checked.requested));
      } catch (error) {
        return failure(
          tool,
          "error",
          error && error.message ? error.message : String(error),
        );
      }
    }
    return writeLocalFile(
      tool,
      { ...input, path: relative, content },
      null,
      (value) => computerSafePath(input, value),
    );
  }

  async function computerDirectoryEntries(input, relative) {
    if (typeof readDirectory !== "function") throw new Error("directory reader is not configured");
    const checked = computerSafePath(input, relative);
    if (!checked.requested) throw new Error(checked.error);
    const entries = await readDirectory(checked.requested);
    return entries
      .map((entry) => typeof entry === "string" ? entry : entry.name)
      .sort((left, right) => left.localeCompare(right));
  }

  async function verifyComputerPostcondition(tool, input, result) {
    if (!result || !result.ok) return false;
    if (tool === "fs.read") {
      const observed = await observedFile(input, input.path);
      return Boolean(observed)
        && observed.body === String(result.body ?? "")
        && observed.sha256 === result.sha256;
    }
    if (tool === "fs.write") {
      const observed = await observedFile(input, input.path);
      return Boolean(observed)
        && observed.body === String(input.content ?? "")
        && observed.sha256 === result.sha256;
    }
    if (tool === "fs.list") {
      try {
        const observed = await computerDirectoryEntries(input, input.path);
        return JSON.stringify(observed) === JSON.stringify(result.entries);
      } catch {
        return false;
      }
    }
    if (tool === "fs.move") {
      const source = await observedFile(input, input.from);
      const destination = await observedFile(input, input.to);
      return source === null
        && Boolean(destination)
        && destination.sha256 === result.sha256;
    }
    if (tool === "shell.run") {
      const observed = await observedFile(input, input.output);
      return Boolean(observed)
        && observed.body === String(result.body ?? "")
        && observed.sha256 === result.sha256;
    }
    if (tool === "http.fetch" || tool === "http.post") {
      const observed = await observedFile(input, input.save_as);
      return Boolean(observed)
        && observed.body === String(result.body ?? "")
        && observed.sha256 === result.sha256
        && result.provenance
        && result.provenance.sha256 === result.sha256;
    }
    if (tool === "dom.query" || tool === "dom.extract") {
      const observed = await observedFile(input, input.save_as);
      return Boolean(observed)
        && observed.body === String(result.body ?? "")
        && observed.sha256 === result.sha256;
    }
    if (tool === "archive.pack") {
      const observed = await observedFile(input, input.archive);
      return Boolean(observed) && observed.sha256 === result.sha256;
    }
    if (tool === "archive.unpack") {
      if (!Array.isArray(result.entries) || result.entries.length === 0) return false;
      const archiveFile = await observedFile(input, input.archive);
      if (!archiveFile || archiveFile.sha256 !== result.sha256) return false;
      let archive;
      try {
        archive = JSON.parse(archiveFile.body);
      } catch {
        return false;
      }
      if (
        archive.format !== "formal-ai-archive-v1"
        || !Array.isArray(archive.entries)
        || JSON.stringify(archive.entries.map((entry) => entry.path)) !== JSON.stringify(result.entries)
      ) {
        return false;
      }
      for (const entry of archive.entries) {
        const target = computerArchiveTarget(input, input.destination, entry.path);
        if (!target.requested) return false;
        const restored = await observedFile(
          input,
          target.relative,
        );
        if (
          !restored
          || restored.body !== Buffer.from(String(entry.contentBase64 || ""), "base64").toString()
        ) {
          return false;
        }
      }
      return true;
    }
    if (tool === "process.status") {
      const observed = await observedFile(input, input.save_as);
      if (!observed || observed.sha256 !== result.sha256) return false;
      try {
        const status = JSON.parse(observed.body);
        return status.state === "running" && status.scope === "isolated_workspace";
      } catch {
        return false;
      }
    }
    return false;
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

  async function structuredShell(tool, input) {
    const operation = String(input && input.operation || "");
    if (!["count_lines", "filter_csv", "unique_csv"].includes(operation)) {
      return failure(tool, "invalid_input", `unsupported allowlisted operation: ${operation}`);
    }
    const source = await readComputerFile(tool, input, input.input);
    if (!source.ok) return source;
    const lines = String(source.body || "").split(/\r?\n/);
    if (lines.at(-1) === "") lines.pop();
    let body = "";
    if (operation === "count_lines") {
      body = `${lines.length}\n`;
    } else {
      const header = String(lines.shift() || "").split(",");
      const column = header.indexOf(String(input.column || ""));
      if (column < 0) return failure(tool, "invalid_input", `CSV column not found: ${input.column || ""}`);
      if (operation === "filter_csv") {
        body = `${header.join(",")}\n${lines.filter((line) => line.split(",")[column] === String(input.equals || "")).join("\n")}\n`;
      } else {
        const values = [...new Set(lines.map((line) => line.split(",")[column]).filter(Boolean))].sort();
        body = `${values.join("\n")}\n`;
      }
    }
    return writeComputerFile(tool, input, input.output, body);
  }

  async function computerHttp(tool, input) {
    const url = String(input && input.url || "");
    if (!/^https?:\/\//i.test(url)) {
      return failure(tool, "invalid_input", `${tool} requires an http(s) url`);
    }
    if (typeof fetchImpl !== "function") return failure(tool, "unavailable", "no fetch implementation is configured");
    try {
      const method = tool === "http.post" ? "POST" : "GET";
      const response = await fetchImpl(url, {
        method,
        ...(method === "POST" ? { body: String(input.body || "") } : {}),
      });
      const body = typeof response.text === "function" ? await response.text() : "";
      if (input.save_as) {
        const saved = await writeComputerFile(tool, input, input.save_as, body);
        if (!saved.ok) return saved;
      }
      const digest = sha256(body);
      return {
        ok: true, tool, status: "ok", executed: true, servedBy: "local-process",
        method, url, httpStatus: response.status, body, sha256: digest,
        provenance: { method, url, status: response.status, sha256: digest, cachePath: String(input.save_as || "") },
      };
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  function queryHtml(html, selector) {
    const escaped = String(selector || "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const pattern = selector.startsWith("#")
      ? new RegExp(`<([a-z][\\w-]*)[^>]*\\bid=["']${escaped.slice(1)}["'][^>]*>([\\s\\S]*?)<\\/\\1>`, "i")
      : new RegExp(`<${escaped}[^>]*>([\\s\\S]*?)<\\/${escaped}>`, "i");
    const match = pattern.exec(html);
    if (!match) return null;
    return String(match[2] ?? match[1] ?? "").replace(/<[^>]*>/g, "").trim();
  }

  async function computerDom(tool, input) {
    const source = await readComputerFile(tool, input, input.source);
    if (!source.ok) return source;
    let body;
    try {
      if (tool === "dom.extract") {
        const segments = String(input.pointer || "").split("/").slice(1).map((value) => value.replace(/~1/g, "/").replace(/~0/g, "~"));
        let value = JSON.parse(source.body);
        for (const segment of segments) value = value[segment];
        if (value === undefined) return failure(tool, "invalid_input", `JSON pointer not found: ${input.pointer || ""}`);
        body = typeof value === "string" ? value : JSON.stringify(value);
      } else {
        body = queryHtml(String(source.body || ""), String(input.selector || ""));
        if (body === null) return failure(tool, "invalid_input", `selector not found: ${input.selector || ""}`);
      }
      const saved = await writeComputerFile(tool, input, input.save_as, body);
      return saved.ok ? { ...saved, body, sha256: sha256(body) } : saved;
    } catch (error) {
      return failure(tool, "error", error && error.message ? error.message : String(error));
    }
  }

  async function computerArchive(tool, input) {
    if (tool === "archive.pack") {
      const entries = [];
      for (const filePath of Array.isArray(input.paths) ? [...input.paths].sort() : []) {
        const read = await readComputerFile(tool, input, filePath);
        if (!read.ok) return read;
        entries.push({ path: String(filePath), contentBase64: Buffer.from(String(read.body)).toString("base64") });
      }
      if (!entries.length) return failure(tool, "invalid_input", "archive.pack requires paths");
      const body = JSON.stringify({ format: "formal-ai-archive-v1", entries });
      const saved = await writeComputerFile(tool, input, input.archive, body);
      return saved.ok ? { ...saved, entries: entries.map((entry) => entry.path), sha256: sha256(body) } : saved;
    }
    const read = await readComputerFile(tool, input, input.archive);
    if (!read.ok) return read;
    try {
      const archive = JSON.parse(read.body);
      if (archive.format !== "formal-ai-archive-v1" || !Array.isArray(archive.entries)) {
        return failure(tool, "invalid_input", "unsupported archive format");
      }
      for (const entry of archive.entries) {
        const target = computerArchiveTarget(input, input.destination, entry.path);
        if (!target.requested) return failure(tool, "forbidden", target.error);
        const content = Buffer.from(String(entry.contentBase64 || ""), "base64").toString();
        const saved = await writeComputerFile(tool, input, target.relative, content);
        if (!saved.ok) return saved;
      }
      return { ok: true, tool, status: "ok", executed: true, servedBy: "local-process", entries: archive.entries.map((entry) => entry.path), sha256: sha256(read.body) };
    } catch (error) {
      return failure(tool, "invalid_input", error && error.message ? error.message : String(error));
    }
  }

  async function invokeComputerPrimitive(tool, input) {
    const verificationFields = ["plan_id", "step_id", "precondition", "postcondition"];
    if (verificationFields.some((field) => typeof input[field] !== "string" || !input[field].trim())) {
      return computerFailure(
        tool,
        input,
        "invalid_input",
        "computer_use_verification_context_required",
      );
    }
    if (COMPUTER_USE_EFFECTS.includes(tool) && input.confirmed !== true) {
      return computerFailure(tool, input, "confirmation_required", `explicit confirmation required for ${tool}`);
    }
    let result;
    if (tool === "fs.read") result = await readComputerFile(tool, input, input.path);
    else if (tool === "fs.write") {
      result = await writeComputerFile(tool, input, input.path, input.content);
    } else if (tool === "fs.list") {
      try {
        result = {
          ok: true,
          tool,
          status: "ok",
          executed: true,
          servedBy: "local-process",
          entries: await computerDirectoryEntries(input, input.path),
        };
      } catch (error) {
        result = failure(tool, "error", error && error.message ? error.message : String(error));
      }
    }
    else if (tool === "fs.move") {
      if (typeof moveFile !== "function") result = failure(tool, "unavailable", "filesystem move is not configured");
      else {
        const from = computerSafePath(input, input.from);
        const to = computerSafePath(input, input.to);
        if (!from.requested || !to.requested) result = failure(tool, "forbidden", from.error || to.error);
        else {
          try {
            const source = await observedFile(input, input.from);
            if (!source) return computerFailure(tool, input, "error", "move source is not readable");
            await moveFile(from.requested, to.requested);
            result = { ok: true, tool, status: "ok", executed: true, servedBy: "local-process", from: from.requested, to: to.requested, sha256: source.sha256 };
          } catch (error) {
            result = failure(tool, "error", error && error.message ? error.message : String(error));
          }
        }
      }
    } else if (tool === "shell.run") result = await structuredShell(tool, input);
    else if (tool === "http.fetch" || tool === "http.post") result = await computerHttp(tool, input);
    else if (tool === "dom.query" || tool === "dom.extract") result = await computerDom(tool, input);
    else if (tool === "archive.pack" || tool === "archive.unpack") result = await computerArchive(tool, input);
    else if (tool === "process.status") {
      const body = JSON.stringify({ state: "running", scope: "isolated_workspace", planId: String(input.plan_id || "desktop") });
      const saved = await writeComputerFile(tool, input, input.save_as, body);
      result = saved.ok ? { ...saved, state: "running", scope: "isolated_workspace", sha256: sha256(body) } : saved;
    }
    if (result && result.ok && result.body !== undefined && !result.sha256) result.sha256 = sha256(result.body);
    const completed = result || failure(tool, "unknown_tool", `unsupported tool: ${tool}`);
    const postconditionVerified = await verifyComputerPostcondition(tool, input, completed);
    return computerRecord(tool, input, completed, postconditionVerified);
  }

  async function invoke(request) {
    const tool = String((request && request.tool) || "");
    const dispatchTool = canonicalTool(tool);
    const input = (request && request.input) || {};
    if (!SUPPORTED_TOOLS.includes(tool) && !SUPPORTED_TOOLS.includes(dispatchTool)) {
      return failure(tool, "unknown_tool", `unsupported tool: ${tool || "(none)"}`);
    }
    // Explicit-permission gate (default-deny) runs before any side effect.
    const needsExplicitGrant = COMPUTER_USE_TOOLS.includes(tool)
      || (!READ_ONLY_TOOLS.includes(tool) && !READ_ONLY_TOOLS.includes(dispatchTool));
    if (needsExplicitGrant && !isPermitted(grants, tool)) {
      return COMPUTER_USE_TOOLS.includes(tool)
        ? computerFailure(tool, input, "refused", "tool call denied by explicit-permission policy")
        : refusal(tool, "tool call denied by explicit-permission policy");
    }
    if (COMPUTER_USE_TOOLS.includes(tool)) return invokeComputerPrimitive(tool, input);
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
    COMPUTER_USE_TOOLS,
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
  COMPUTER_USE_TOOLS,
  READ_ONLY_TOOLS,
  TOOL_ALIASES,
  canonicalTool,
  isPermitted,
  createToolRouter,
};
