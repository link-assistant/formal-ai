"use strict";

const { app, BrowserWindow, ipcMain, shell } = require("electron");
const childProcess = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const net = require("node:net");
const path = require("node:path");
const { URL } = require("node:url");

const REPO_ROOT = path.resolve(__dirname, "..");
const API_AUTH_ENV_KEYS = [
  "FORMAL_AI_API_BEARER_TOKEN",
  "FORMAL_AI_HTTP_BEARER_TOKEN",
  "FORMAL_AI_API_TOKEN",
];

let mainWindow = null;
let staticServer = null;
let apiProcess = null;
let desktopStatus = {
  shell: "Electron",
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
  const apiPort = await findFreePort();

  const staticBase = `http://127.0.0.1:${staticPort}`;
  const apiBase = `http://127.0.0.1:${apiPort}`;
  desktopStatus = {
    ...desktopStatus,
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

  await mainWindow.loadURL(`${staticBase}/index.html?desktop=1&api=${encodeURIComponent(apiBase)}`);
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
