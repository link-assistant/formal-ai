"use strict";

// Node (desktop) extension host — VS Code desktop and any Node-backed remote.
//
// Issue #353 (ROADMAP D2/D3): this host mirrors the Electron shell
// (`desktop/main.cjs`) inside a VS Code Webview. It can spin up the local
// OpenAI-compatible server (`formal-ai serve`) on opt-in, route the agent's side
// effects through permission-gated host/Docker runners, and reconcile memory with
// the native store — all the desktop affordances, driven by `formal-ai.*`
// settings. The web host (`extension.web.cjs`) shares the same UI but stays
// in-process because a browser cannot spawn a process.
//
// The bridge, status mapper, webview HTML builder, and the tool-router /
// memory-sync clients are all shared with (or reused verbatim from) the desktop
// shell, so the chat UI needs no VS Code-specific code.

const vscode = require("vscode");
const childProcess = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const {
  statusFromConfig,
  withApiReady,
  withApiError,
  serverEnv,
  DEFAULT_IMAGE,
} = require("./lib/config.cjs");
const { createBridge } = require("./lib/bridge.cjs");
const { createChatViewProvider, renderChatWebview } = require("./lib/chat-view.cjs");
const { startServer } = require("./lib/server-process.cjs");

const SHELL = "VS Code";
const VIEW_ID = "formal-ai.chatView";
const REPO_ROOT = path.resolve(__dirname, "..", "..");

// Reuse the desktop shell's tool-router / memory-sync modules verbatim. In a
// checkout they live under `<repo>/desktop/lib`; a packaged `.vsix` copies them
// next to this file (scripts/prepare-resources.mjs → src/lib/vendor).
function requireReused(name) {
  const bundled = path.join(__dirname, "lib", "vendor", name);
  if (fs.existsSync(bundled)) {
    return require(bundled);
  }
  return require(path.join(REPO_ROOT, "desktop", "lib", name));
}
const { createToolRouter } = requireReused("tool-router.cjs");
const { createWebTools } = requireReused("web-tools.cjs");
const { createMemorySync } = requireReused("memory-sync.cjs");
const { resolveSharedMemoryPath } = requireReused("shared-memory.cjs");

function currentConfig() {
  return vscode.workspace.getConfiguration("formal-ai");
}

// Probe Docker once and cache the result; mirrors `desktop/main.cjs`.
let dockerProbe = null;
function dockerIsAvailable() {
  if (dockerProbe === null) {
    try {
      const result = childProcess.spawnSync(
        "docker",
        ["version", "--format", "{{.Server.Version}}"],
        { stdio: ["ignore", "pipe", "pipe"], timeout: 5000 },
      );
      dockerProbe = result.status === 0;
    } catch (_error) {
      dockerProbe = false;
    }
  }
  return dockerProbe;
}

// Run a sandboxed tool call inside the configured `konard/box-dind` image. The
// router passes its default image; the `formal-ai.docker.image` setting wins so
// users can pin their own sandbox. Host shell commands use runOnHost below.
function runInSandbox({ image, tool, command }) {
  const configuredImage = String(currentConfig().get("docker.image", image || DEFAULT_IMAGE));
  return new Promise((resolve, reject) => {
    const logPath = path.join(os.tmpdir(), `formal-ai-${tool}-${process.pid}-${nonceStamp()}.log`);
    const child = childProcess.spawn(
      "docker",
      ["run", "--rm", configuredImage, "sh", "-c", command],
      { stdio: ["ignore", "pipe", "pipe"] },
    );
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

function runOnHost({ tool, command }) {
  return new Promise((resolve) => {
    const logPath = path.join(os.tmpdir(), `formal-ai-${tool}-host-${nonceStamp()}.log`);
    let child = null;
    try {
      child = childProcess.spawn(command, {
        cwd: vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0
          ? vscode.workspace.workspaceFolders[0].uri.fsPath
          : os.homedir(),
        env: process.env,
        shell: true,
        stdio: ["ignore", "pipe", "pipe"],
      });
    } catch (error) {
      resolve({
        exitCode: 1,
        output: "",
        stdout: "",
        stderr: error && error.message ? error.message : String(error),
        logPath,
      });
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
      stderr += error && error.message ? error.message : String(error);
      resolve({ exitCode: 1, output: `${stdout}${stderr}`, stdout, stderr, logPath });
    });
    child.once("exit", (code) => {
      const output = `${stdout}${stderr}`;
      try {
        fs.writeFileSync(logPath, output);
      } catch (_error) {
        /* best-effort log capture */
      }
      resolve({
        exitCode: typeof code === "number" ? code : 1,
        output,
        stdout,
        stderr,
        logPath,
      });
    });
  });
}

// A monotonic-ish stamp for log filenames without depending on wall-clock
// determinism elsewhere.
let stampCounter = 0;
function nonceStamp() {
  stampCounter += 1;
  return `${process.pid}-${stampCounter}`;
}

function activate(context) {
  const output = vscode.window.createOutputChannel("formal-ai");
  const log = (message) => output.appendLine(message);

  // The workspace folder confines `read_local_file`; fall back to the repo root
  // when no folder is open (e.g. an empty window running from a checkout).
  function readRoot() {
    const folders = vscode.workspace.workspaceFolders;
    if (folders && folders.length > 0) {
      return folders[0].uri.fsPath;
    }
    return REPO_ROOT;
  }

  let status = statusFromConfig(currentConfig(), { shell: SHELL, serverCapable: true });
  let serverProc = null;
  let memorySync = null;
  let resolvedView = null;

  const appVersion = (context.extension && context.extension.packageJSON
    && context.extension.packageJSON.version) || status.appVersion || "";

  // Reused desktop tool router: permission-gated local effects + Docker sandbox.
  const browserManifest = path.join(context.extensionPath, "browser-runtime", "executable-path.txt");
  const browserExecutablePath = fs.existsSync(browserManifest)
    ? path.join(
        context.extensionPath,
        "browser-runtime",
        fs.readFileSync(browserManifest, "utf8").trim(),
      )
    : "";
  const webTools = createWebTools({ browserExecutablePath });
  const toolRouter = createToolRouter({
    fetchImpl: globalThis.fetch,
    readFile: (filePath) => fs.promises.readFile(filePath, "utf8"),
    writeFile: (filePath, body) => fs.promises.writeFile(filePath, body, "utf8"),
    readDirectory: (directory) => fs.promises.readdir(directory, { withFileTypes: true }),
    allowedReadRoot: readRoot(),
    resolvePath: (value) => path.resolve(readRoot(), value),
    dockerAvailable: dockerIsAvailable,
    runInSandbox,
    runOnHost,
    webSearch: webTools.search,
    webFetch: webTools.fetch,
  });
  toolRouter.setGrants({ all: status.allowToolsByDefault === true });

  const bridge = createBridge({
    getStatus: () => status,
    toolRouter,
    getMemorySync: () => memorySync,
    // Side-effecting tools and memory sync require a healthy local server;
    // read-only tools are served directly by the extension host.
    serverEnabled: () => Boolean(status.serverEnabled && status.apiReady),
    openExternal: (url) => vscode.env.openExternal(vscode.Uri.parse(url)),
  });

  const host = {
    appVersion,
    getStatus: () => status,
    bridge,
    onView: (view) => {
      resolvedView = view;
      view.onDidDispose(() => {
        if (resolvedView === view) {
          resolvedView = null;
        }
      });
    },
  };

  // Re-render the resolved webview so the embedded initial status (and CSP
  // `connect-src`) reflect the latest server state. `getStatus()` is a one-shot
  // poll on mount in the web app, so a re-render is how a later server-ready /
  // server-error transition reaches the UI.
  async function refreshView() {
    if (!resolvedView) {
      return;
    }
    try {
      await renderChatWebview({ vscode, context, host, webviewView: resolvedView });
    } catch (error) {
      log(`failed to refresh chat view: ${error && error.message ? error.message : error}`);
    }
  }

  function stopServer() {
    if (serverProc && !serverProc.killed) {
      serverProc.kill();
    }
    serverProc = null;
  }

  // Reconcile the running server with the current settings: (re)start it when
  // enabled, stop it when disabled, and promote / fail the status accordingly.
  let applying = null;
  async function applyServerState() {
    const config = currentConfig();
    status = statusFromConfig(config, { shell: SHELL, serverCapable: true });
    toolRouter.setGrants({ all: status.allowToolsByDefault === true });
    stopServer();
    memorySync = null;

    if (status.serverEnabled) {
      log(`starting local server on ${status.host}:${status.port}`);
      try {
        const started = await startServer({
          host: status.host,
          port: status.port,
          repoRoot: REPO_ROOT,
          env: serverEnv(config, { memoryPath: resolveSharedMemoryPath(process.env) }),
          log,
        });
        serverProc = started.process;
        status = withApiReady(status, started.apiBase);
        memorySync = createMemorySync({ apiBase: status.apiBase, fetchImpl: globalThis.fetch });
        log(`local server ready at ${status.apiBase} (${started.label})`);
      } catch (error) {
        status = withApiError(status, error);
        log(`local server failed: ${status.apiError}`);
      }
    }
    await refreshView();
  }

  // Serialise reconciliations so rapid setting changes can't race two server
  // starts against each other.
  function scheduleApply() {
    applying = Promise.resolve(applying)
      .catch(() => {})
      .then(() => applyServerState());
    return applying;
  }

  const provider = createChatViewProvider({ vscode, context, host });
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(VIEW_ID, provider, {
      webviewOptions: { retainContextWhenHidden: true },
    }),
    output,
    { dispose: () => { webTools.close().catch(() => {}); } },
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("formal-ai.openChat", async () => {
      try {
        await vscode.commands.executeCommand(`${VIEW_ID}.focus`);
      } catch (_error) {
        await vscode.commands.executeCommand("workbench.view.extension.formal-ai");
      }
    }),
    vscode.commands.registerCommand("formal-ai.toggleServer", async () => {
      const next = !currentConfig().get("server.enabled", false);
      await currentConfig().update(
        "server.enabled",
        next,
        vscode.ConfigurationTarget.Global,
      );
      // The configuration-change listener performs the actual (re)start.
      vscode.window.setStatusBarMessage(
        next ? "formal-ai: starting local server…" : "formal-ai: local server disabled",
        3000,
      );
    }),
    vscode.commands.registerCommand("formal-ai.syncMemory", async () => {
      const result = await bridge.syncMemory({ lino: "" });
      if (result && result.ok) {
        const pulled = (result.pulled && result.pulled.added) || 0;
        vscode.window.showInformationMessage(`formal-ai: memory synced (${pulled} new event(s)).`);
      } else {
        vscode.window.showWarningMessage(
          `formal-ai: ${result && result.reason ? result.reason : "memory sync unavailable"}.`,
        );
      }
    }),
    vscode.commands.registerCommand("formal-ai.openNetworkView", async () => {
      try {
        await vscode.commands.executeCommand(`${VIEW_ID}.focus`);
      } catch (_error) {
        /* view may not be registered yet */
      }
      if (status.apiReady && status.graphUrl) {
        await vscode.env.openExternal(vscode.Uri.parse(status.traceUrl || status.graphUrl));
      } else {
        vscode.window.showInformationMessage(
          "formal-ai: the links network view renders in the chat panel. Enable formal-ai.server.enabled to expose the /v1/network endpoint.",
        );
      }
    }),
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("formal-ai")) {
        scheduleApply();
      }
    }),
  );

  // Kick the initial reconciliation (starts the server if it is enabled).
  scheduleApply();

  // Expose a tiny surface for tests / programmatic callers.
  return {
    getStatus: () => status,
    isServerRunning: () => Boolean(serverProc && !serverProc.killed),
    dispose: stopServer,
  };
}

function deactivate() {
  // `activate`'s `stopServer` is wired into the returned API and the process is
  // torn down with the extension host; nothing else to clean up here.
}

module.exports = { activate, deactivate };
