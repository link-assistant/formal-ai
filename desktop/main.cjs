"use strict";

const { app, BrowserWindow, ipcMain, shell } = require("electron");
const childProcess = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const net = require("node:net");
const os = require("node:os");
const path = require("node:path");
const { URL } = require("node:url");

const { createToolRouter } = require("./lib/tool-router.cjs");
const { createMemorySync } = require("./lib/memory-sync.cjs");
const { createServiceControl } = require("./lib/service-control.cjs");

const REPO_ROOT = path.resolve(__dirname, "..");
const API_AUTH_ENV_KEYS = [
  "FORMAL_AI_API_BEARER_TOKEN",
  "FORMAL_AI_HTTP_BEARER_TOKEN",
  "FORMAL_AI_API_TOKEN",
];

// R3/R4: by default the desktop runs the in-process reasoning agent (the same
// in-browser engine the web demo uses) — no child server, nothing listening.
// Starting the local OpenAI-compatible server is opt-in via this environment
// variable, which is also what unlocks pointing the claude/codex/agent CLIs at
// it. See docs/desktop/server-api.md.
const SERVER_OPT_IN_ENV = "FORMAL_AI_DESKTOP_SERVER";

function serverModeRequested() {
  const raw = String(process.env[SERVER_OPT_IN_ENV] || "")
    .trim()
    .toLowerCase();
  return raw === "1" || raw === "true" || raw === "yes" || raw === "on";
}

let mainWindow = null;
let staticServer = null;
let apiProcess = null;
let desktopStatus = {
  shell: "Electron",
  mode: "in-process",
  apiBase: "",
  staticBase: "",
  graphUrl: "",
  chatUrl: "",
  traceUrl: "",
  memory: "formal_ai_bundle",
  agentModeDefault: false,
  toolCallPolicy: "explicit-permission",
  apiReady: false,
  apiError: "",
};

function packagedBinaryPath() {
  const binary = process.platform === "win32" ? "formal-ai.exe" : "formal-ai";
  if (process.resourcesPath) {
    return path.join(process.resourcesPath, "bin", binary);
  }
  return "";
}

function devWebRoot() {
  return path.join(REPO_ROOT, "src", "web");
}

function packagedWebRoot() {
  return path.join(__dirname, "dist-web");
}

function resolveWebRoot() {
  const packaged = packagedWebRoot();
  if (fs.existsSync(path.join(packaged, "index.html"))) {
    return packaged;
  }
  return devWebRoot();
}

function contentType(filePath) {
  const ext = path.extname(filePath).toLowerCase();
  return (
    {
      ".css": "text/css; charset=utf-8",
      ".html": "text/html; charset=utf-8",
      ".js": "application/javascript; charset=utf-8",
      ".json": "application/json; charset=utf-8",
      ".lino": "text/plain; charset=utf-8",
      ".map": "application/json; charset=utf-8",
      ".png": "image/png",
      ".svg": "image/svg+xml",
      ".wasm": "application/wasm",
    }[ext] || "application/octet-stream"
  );
}

function safeResolve(root, requestPath) {
  let cleanPath = "/";
  try {
    cleanPath = decodeURIComponent(requestPath.split("?")[0] || "/");
  } catch (_error) {
    return null;
  }
  const relative = cleanPath === "/" ? "index.html" : cleanPath.replace(/^\/+/, "");
  const absolute = path.resolve(root, relative);
  const rootWithSeparator = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  if (absolute !== root && !absolute.startsWith(rootWithSeparator)) {
    return null;
  }
  return absolute;
}

function findFreePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = address && typeof address === "object" ? address.port : 0;
      server.close(() => resolve(port));
    });
  });
}

function startStaticServer(root, port) {
  const server = http.createServer((request, response) => {
    const url = new URL(request.url || "/", "http://127.0.0.1");
    let filePath = safeResolve(root, url.pathname);

    if (url.pathname.startsWith("/seed/") && filePath && !fs.existsSync(filePath)) {
      filePath = safeResolve(path.join(REPO_ROOT, "data"), url.pathname);
    }

    if (!filePath) {
      response.writeHead(403, { "content-type": "text/plain; charset=utf-8" });
      response.end("Forbidden");
      return;
    }

    if (fs.existsSync(filePath) && fs.statSync(filePath).isDirectory()) {
      filePath = path.join(filePath, "index.html");
    }

    if (!fs.existsSync(filePath)) {
      response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      response.end("Not found");
      return;
    }

    response.writeHead(200, {
      "content-type": contentType(filePath),
      "cross-origin-opener-policy": "same-origin",
      "cross-origin-embedder-policy": "require-corp",
    });
    fs.createReadStream(filePath).pipe(response);
  });

  return new Promise((resolve, reject) => {
    server.on("error", reject);
    server.listen(port, "127.0.0.1", () => resolve(server));
  });
}

function scrubbedEnvironment(port) {
  const env = {
    ...process.env,
    FORMAL_AI_HOST: "127.0.0.1",
    FORMAL_AI_PORT: String(port),
  };
  for (const key of API_AUTH_ENV_KEYS) {
    delete env[key];
  }
  return env;
}

function apiCandidates(port) {
  const args = ["serve", "--host", "127.0.0.1", "--port", String(port)];
  const candidates = [];
  if (process.env.FORMAL_AI_DESKTOP_BINARY) {
    candidates.push({
      command: process.env.FORMAL_AI_DESKTOP_BINARY,
      args,
      cwd: REPO_ROOT,
      label: "FORMAL_AI_DESKTOP_BINARY",
    });
  }

  const packaged = packagedBinaryPath();
  if (packaged && fs.existsSync(packaged)) {
    candidates.push({ command: packaged, args, cwd: REPO_ROOT, label: "bundled formal-ai" });
  }

  if (fs.existsSync(path.join(REPO_ROOT, "Cargo.toml"))) {
    candidates.push({
      command: "cargo",
      args: ["run", "--", ...args],
      cwd: REPO_ROOT,
      label: "cargo run",
    });
  }

  candidates.push({ command: "formal-ai", args, cwd: REPO_ROOT, label: "formal-ai on PATH" });
  return candidates;
}

function requestHealth(port) {
  return new Promise((resolve, reject) => {
    const request = http.get(`http://127.0.0.1:${port}/health`, (response) => {
      response.resume();
      resolve(response.statusCode === 200);
    });
    request.on("error", reject);
    request.setTimeout(2000, () => {
      request.destroy(new Error("health check timed out"));
    });
  });
}

async function waitForApi(port) {
  const startedAt = Date.now();
  let lastError = null;
  while (Date.now() - startedAt < 180000) {
    try {
      if (await requestHealth(port)) {
        return;
      }
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw lastError || new Error("formal-ai API did not become ready");
}

async function startApiProcess(port) {
  let lastError = null;
  for (const candidate of apiCandidates(port)) {
    let child = null;
    try {
      child = childProcess.spawn(candidate.command, candidate.args, {
        cwd: candidate.cwd,
        env: scrubbedEnvironment(port),
        stdio: ["ignore", "pipe", "pipe"],
      });
      child.stdout.on("data", (chunk) => process.stdout.write(`[formal-ai] ${chunk}`));
      child.stderr.on("data", (chunk) => process.stderr.write(`[formal-ai] ${chunk}`));

      await Promise.race([
        waitForApi(port),
        new Promise((_, reject) => {
          child.once("error", reject);
          child.once("exit", (code, signal) => {
            reject(new Error(`${candidate.label} exited before ready: ${code || signal}`));
          });
        }),
      ]);
      return child;
    } catch (error) {
      lastError = error;
      if (child && !child.killed) {
        child.kill();
      }
    }
  }
  throw lastError || new Error("could not start formal-ai API");
}

async function createMainWindow() {
  const webRoot = resolveWebRoot();
  const staticPort = await findFreePort();
  staticServer = await startStaticServer(webRoot, staticPort);
  const staticBase = `http://127.0.0.1:${staticPort}`;

  if (serverModeRequested()) {
    // Opt-in: spawn the local OpenAI-compatible server on a free loopback port
    // and route chat through it (`POST /v1/chat/completions`).
    const apiPort = await findFreePort();
    const apiBase = `http://127.0.0.1:${apiPort}`;
    desktopStatus = {
      ...desktopStatus,
      mode: "server",
      apiBase,
      staticBase,
      chatUrl: `${apiBase}/v1/chat/completions`,
      graphUrl: `${apiBase}/v1/graph`,
      traceUrl: `${apiBase}/v1/graph?trace=answer_greeting_hi`,
    };

    try {
      apiProcess = await startApiProcess(apiPort);
      desktopStatus = { ...desktopStatus, apiReady: true, apiError: "" };
    } catch (error) {
      desktopStatus = {
        ...desktopStatus,
        apiReady: false,
        apiError: error && error.message ? error.message : String(error),
      };
    }
  } else {
    // Default: in-process agent only. The web app falls back to the in-browser
    // engine when no `apiBase` is advertised (see app.js routing).
    desktopStatus = {
      ...desktopStatus,
      mode: "in-process",
      staticBase,
      apiReady: false,
      apiError: "",
    };
  }

  mainWindow = new BrowserWindow({
    width: 1280,
    height: 840,
    minWidth: 960,
    minHeight: 640,
    title: "formal-ai Desktop",
    webPreferences: {
      preload: path.join(__dirname, "preload.cjs"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.setMenuBarVisibility(false);
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: "deny" };
  });
  mainWindow.webContents.on("will-navigate", (event, url) => {
    if (!url.startsWith(staticBase)) {
      event.preventDefault();
      shell.openExternal(url);
    }
  });

  const apiQuery = desktopStatus.apiBase
    ? `&api=${encodeURIComponent(desktopStatus.apiBase)}`
    : "";
  // The web app now lives under /app/ (issue #479): the site root is the
  // landing page chooser, so the desktop wrapper loads the app directly.
  await mainWindow.loadURL(`${staticBase}/app/index.html?desktop=1${apiQuery}`);
}

async function shutdown() {
  if (staticServer) {
    await new Promise((resolve) => staticServer.close(resolve));
    staticServer = null;
  }
  if (apiProcess && !apiProcess.killed) {
    apiProcess.kill();
    apiProcess = null;
  }
}

// R5d (ROADMAP D2): route the agent's side effects through the local process and
// its Docker sandbox behind an explicit-permission gate. Docker availability is
// probed once and cached; code-exec / shell tools run inside `konard/box-dind`
// (the same image the Telegram microservice uses) and never run unsandboxed.
let dockerProbe = null;
function dockerIsAvailable() {
  if (dockerProbe === null) {
    try {
      const result = childProcess.spawnSync("docker", ["version", "--format", "{{.Server.Version}}"], {
        stdio: ["ignore", "pipe", "pipe"],
        timeout: 5000,
      });
      dockerProbe = result.status === 0;
    } catch (_error) {
      dockerProbe = false;
    }
  }
  return dockerProbe;
}

function runInSandbox({ image, tool, command }) {
  return new Promise((resolve, reject) => {
    const logPath = path.join(os.tmpdir(), `formal-ai-${tool}-${process.pid}-${Date.now()}.log`);
    const child = childProcess.spawn("docker", ["run", "--rm", image, "sh", "-c", command], {
      stdio: ["ignore", "pipe", "pipe"],
    });
    let output = "";
    child.stdout.on("data", (chunk) => {
      output += chunk;
    });
    child.stderr.on("data", (chunk) => {
      output += chunk;
    });
    child.once("error", reject);
    child.once("exit", (code) => {
      try {
        fs.writeFileSync(logPath, output);
      } catch (_error) {
        /* best-effort log capture */
      }
      resolve({ exitCode: typeof code === "number" ? code : 0, output, logPath });
    });
  });
}

// Issue #438 (follow-up): one-click start/stop of the prepared Telegram bot and
// OpenAI-compatible server containers. `runDocker` shells out to the real
// `docker` CLI and collects its result; `createServiceControl` owns the
// lifecycle logic (argument vectors, running-state probes, stale-container
// reaping) so the same contract is exercised by node:test without a daemon.
function runDocker(args) {
  return new Promise((resolve) => {
    let child = null;
    try {
      child = childProcess.spawn("docker", args, {
        stdio: ["ignore", "pipe", "pipe"],
      });
    } catch (error) {
      resolve({ code: 1, stdout: "", stderr: error && error.message ? error.message : String(error) });
      return;
    }
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.once("error", (error) => {
      resolve({ code: 1, stdout, stderr: error && error.message ? error.message : String(error) });
    });
    child.once("exit", (code) => {
      resolve({ code: typeof code === "number" ? code : 1, stdout, stderr });
    });
  });
}

const serviceControl = createServiceControl({
  env: process.env,
  runDocker,
  dockerAvailable: dockerIsAvailable,
});

// The renderer's Services panel drives these handlers: status for the indicators,
// start/stop for the one-click buttons. Each returns a plain status object the UI
// renders directly (running/stopped/missing-config/docker-unavailable).
ipcMain.handle("formalAiDesktop:serviceStatus", () => serviceControl.statusAll());
ipcMain.handle("formalAiDesktop:startService", (_event, request) => {
  const key = request && request.service ? String(request.service) : "";
  const token = request && typeof request.token === "string" ? request.token : "";
  try {
    return serviceControl.start(key, { token });
  } catch (error) {
    return Promise.resolve({
      ok: false,
      key,
      state: "error",
      running: false,
      reason: error && error.message ? error.message : String(error),
    });
  }
});
ipcMain.handle("formalAiDesktop:stopService", (_event, request) => {
  const key = request && request.service ? String(request.service) : "";
  try {
    return serviceControl.stop(key);
  } catch (error) {
    return Promise.resolve({
      ok: false,
      key,
      state: "error",
      running: false,
      reason: error && error.message ? error.message : String(error),
    });
  }
});

const toolRouter = createToolRouter({
  fetchImpl: globalThis.fetch,
  readFile: (filePath) => fs.promises.readFile(filePath, "utf8"),
  allowedReadRoot: REPO_ROOT,
  resolvePath: (value) => path.resolve(REPO_ROOT, value),
  dockerAvailable: dockerIsAvailable,
  runInSandbox,
});

// The renderer's permission toggles (desktop-tool-permission / -agent-permission)
// drive the default-deny grant map. Until the user opts in, every tool call is
// refused and nothing executes.
ipcMain.handle("formalAiDesktop:setToolGrants", (_event, grants) => toolRouter.setGrants(grants));
ipcMain.handle("formalAiDesktop:invokeTool", async (_event, request) => {
  // Server mode is required: tool routing only makes sense once the local app is
  // the execution surface. In-process mode keeps the browser sandbox.
  if (!serverModeRequested()) {
    return {
      ok: false,
      tool: request && request.tool ? String(request.tool) : "",
      status: "refused",
      executed: false,
      reason: "tool routing requires FORMAL_AI_DESKTOP_SERVER",
    };
  }
  return toolRouter.invoke(request);
});

// R5c (ROADMAP D1): reconcile the browser (IndexedDB) memory log with the native
// store over the local server's Links-Notation memory endpoints.
let memorySync = null;
ipcMain.handle("formalAiDesktop:syncMemory", async (_event, payload) => {
  if (!serverModeRequested() || !desktopStatus.apiBase) {
    return { ok: false, status: "unavailable", reason: "memory sync requires the local server" };
  }
  if (!memorySync) {
    memorySync = createMemorySync({ apiBase: desktopStatus.apiBase, fetchImpl: globalThis.fetch });
  }
  try {
    const outbound = payload && typeof payload.lino === "string" ? payload.lino : "";
    const pushed = outbound.trim() ? await memorySync.push(outbound) : { added: 0, total: 0 };
    const pulled = await memorySync.pull();
    return { ok: true, status: "ok", pushed, pulled };
  } catch (error) {
    return { ok: false, status: "error", reason: error && error.message ? error.message : String(error) };
  }
});

ipcMain.handle("formalAiDesktop:getStatus", () => desktopStatus);
ipcMain.handle("formalAiDesktop:openExternal", async (_event, url) => {
  if (typeof url === "string" && /^https?:\/\//i.test(url)) {
    await shell.openExternal(url);
    return true;
  }
  return false;
});

app.whenReady().then(createMainWindow);
app.on("window-all-closed", () => {
  shutdown().finally(() => {
    if (process.platform !== "darwin") {
      app.quit();
    }
  });
});
app.on("before-quit", shutdown);
app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createMainWindow();
  }
});
