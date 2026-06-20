"use strict";

const { app, BrowserWindow, ipcMain, shell } = require("electron");
const childProcess = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const { URL } = require("node:url");

const { createToolRouter, SUPPORTED_TOOLS } = require("./lib/tool-router.cjs");
const { createAgentProvider } = require("./lib/agent-provider.cjs");
const { createMemorySync } = require("./lib/memory-sync.cjs");
const { createServiceControl } = require("./lib/service-control.cjs");
const {
  createLocalServerManager,
  findFreePort,
  serverModeRequested,
} = require("./lib/local-server.cjs");

const REPO_ROOT = path.resolve(__dirname, "..");

let mainWindow = null;
let staticServer = null;
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
  agentExecutionProvider: { type: "in-process" },
  apiReady: false,
  apiError: "",
};

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

const localServerManager = createLocalServerManager({
  repoRoot: REPO_ROOT,
  env: process.env,
  resourcesPath: process.resourcesPath,
  platform: process.platform,
  stdout: process.stdout,
  stderr: process.stderr,
});

function applyLocalServerStatus(status) {
  desktopStatus = {
    ...desktopStatus,
    ...status,
    agentExecutionProvider: agentProvider.status(),
  };
  return desktopStatus;
}

async function ensureAgentServer() {
  const status = await localServerManager.ensure();
  return applyLocalServerStatus(status);
}

async function createMainWindow() {
  const webRoot = resolveWebRoot();
  const staticPort = await findFreePort();
  staticServer = await startStaticServer(webRoot, staticPort);
  const staticBase = `http://127.0.0.1:${staticPort}`;

  // Default: in-process agent only. The web app falls back to the in-browser
  // engine when no `apiBase` is advertised (see app.js routing). The legacy
  // startup opt-in still starts the server immediately; agent/full-auto mode
  // can also start it later through `formalAiDesktop:ensureAgentServer`.
  desktopStatus = {
    ...desktopStatus,
    mode: "in-process",
    staticBase,
    apiBase: "",
    chatUrl: "",
    graphUrl: "",
    traceUrl: "",
    apiReady: false,
    apiError: "",
    agentExecutionProvider: agentProvider.status(),
  };
  if (serverModeRequested()) {
    await ensureAgentServer();
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
  localServerManager.shutdown();
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
    if (key === "agent") {
      return serviceControl.installAgentEnvironment();
    }
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
ipcMain.handle("formalAiDesktop:installAgentEnvironment", () => {
  try {
    return serviceControl.installAgentEnvironment();
  } catch (error) {
    return Promise.resolve({
      ok: false,
      key: "agent",
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

// Issue #515 / R11: entering Agent or Full Auto mode needs a ready local
// OpenAI-compatible backend for the later Agent CLI provider. This handler
// starts `formal-ai serve` if needed, health-checks it, and reuses a healthy
// existing process instead of spawning twice.
ipcMain.handle("formalAiDesktop:ensureAgentServer", async () => {
  try {
    return await ensureAgentServer();
  } catch (error) {
    return applyLocalServerStatus(
      localServerManager.currentStatus({
        apiReady: false,
        apiError: error && error.message ? error.message : String(error),
      }),
    );
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

// Issue #516 / E4: swappable execution seam. The in-process provider is the
// default hermetic path; FORMAL_AI_AGENT_PROVIDER=commander selects the
// agent-commander adapter, which drives @link-assistant/agent through
// `start-agent` inside the Formal-AI container contract.
const agentProvider = createAgentProvider({
  type: process.env.FORMAL_AI_AGENT_PROVIDER,
  toolRouter,
  workingDirectory: REPO_ROOT,
  containerName: "formal-ai-agent",
});

// The renderer's permission toggles (desktop-tool-permission / -agent-permission)
// drive the default-deny grant map. Until the user opts in, every tool call is
// refused and nothing executes.
ipcMain.handle("formalAiDesktop:setToolGrants", (_event, grants) => toolRouter.setGrants(grants));
ipcMain.handle("formalAiDesktop:invokeTool", async (_event, request) => {
  const tool = request && request.tool ? String(request.tool) : "";
  if (!SUPPORTED_TOOLS.includes(tool) || !toolRouter.isPermitted(tool)) {
    return toolRouter.invoke(request);
  }
  const readyStatus = desktopStatus.apiReady ? desktopStatus : await ensureAgentServer();
  // Server mode is required: tool routing only makes sense once the local app
  // is the execution surface. Agent mode auto-starts that server; if startup
  // fails, keep the default-deny shape and do not execute anything.
  if (!readyStatus.apiReady) {
    return {
      ok: false,
      tool,
      status: "refused",
      executed: false,
      reason: readyStatus.apiError || "tool routing requires the local server",
    };
  }
  return toolRouter.invoke(request);
});

ipcMain.handle("formalAiDesktop:runAgentProvider", async (_event, request) => {
  const payload = request && typeof request === "object" ? request : {};
  if (payload.grants && typeof payload.grants === "object") {
    toolRouter.setGrants(payload.grants);
  }
  const readyStatus = agentProvider.type === "commander"
    ? desktopStatus.apiReady
      ? desktopStatus
      : await ensureAgentServer()
    : desktopStatus;
  return agentProvider.run({
    ...payload,
    apiBase: readyStatus.apiBase,
    agentProvider: readyStatus.agentProvider,
    workingDirectory: payload.workingDirectory || REPO_ROOT,
  });
});

// R5c (ROADMAP D1): reconcile the browser (IndexedDB) memory log with the native
// store over the local server's Links-Notation memory endpoints.
let memorySync = null;
ipcMain.handle("formalAiDesktop:syncMemory", async (_event, payload) => {
  if (!desktopStatus.apiReady || !desktopStatus.apiBase) {
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
