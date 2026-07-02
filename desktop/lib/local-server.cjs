"use strict";

const childProcess = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const net = require("node:net");
const path = require("node:path");

const SERVER_OPT_IN_ENV = "FORMAL_AI_DESKTOP_SERVER";
const MODEL_ID = "formal-ai";
const API_AUTH_ENV_KEYS = [
  "FORMAL_AI_API_BEARER_TOKEN",
  "FORMAL_AI_HTTP_BEARER_TOKEN",
  "FORMAL_AI_API_TOKEN",
];

function serverModeRequested(env = process.env) {
  const raw = String(env[SERVER_OPT_IN_ENV] || "")
    .trim()
    .toLowerCase();
  return raw === "1" || raw === "true" || raw === "yes" || raw === "on";
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

function packagedBinaryPath(options = {}) {
  const platform = options.platform || process.platform;
  const resourcesPath = options.resourcesPath || process.resourcesPath || "";
  const binary = platform === "win32" ? "formal-ai.exe" : "formal-ai";
  return resourcesPath ? path.join(resourcesPath, "bin", binary) : "";
}

function scrubbedEnvironment(port, env = process.env) {
  const childEnv = {
    ...env,
    FORMAL_AI_HOST: "127.0.0.1",
    FORMAL_AI_PORT: String(port),
  };
  for (const key of API_AUTH_ENV_KEYS) {
    delete childEnv[key];
  }
  return childEnv;
}

function apiCandidates(port, options = {}) {
  const repoRoot = options.repoRoot || path.resolve(__dirname, "..", "..");
  const env = options.env || process.env;
  const existsSync = options.existsSync || fs.existsSync;
  const args = ["serve", "--host", "127.0.0.1", "--port", String(port)];
  const candidates = [];
  if (env.FORMAL_AI_DESKTOP_BINARY) {
    candidates.push({
      command: env.FORMAL_AI_DESKTOP_BINARY,
      args,
      cwd: repoRoot,
      label: "FORMAL_AI_DESKTOP_BINARY",
    });
  }

  const packaged = packagedBinaryPath(options);
  if (packaged && existsSync(packaged)) {
    candidates.push({ command: packaged, args, cwd: repoRoot, label: "bundled formal-ai" });
  }

  if (existsSync(path.join(repoRoot, "Cargo.toml"))) {
    candidates.push({
      command: "cargo",
      args: ["run", "--", ...args],
      cwd: repoRoot,
      label: "cargo run",
    });
  }

  candidates.push({ command: "formal-ai", args, cwd: repoRoot, label: "formal-ai on PATH" });
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

async function waitForApi(port, options = {}) {
  const timeoutMs = Number(options.timeoutMs || 180000);
  const health = options.requestHealth || requestHealth;
  const now = options.now || (() => Date.now());
  const sleep = options.sleep || ((ms) => new Promise((resolve) => setTimeout(resolve, ms)));
  const startedAt = now();
  let lastError = null;
  while (now() - startedAt < timeoutMs) {
    try {
      if (await health(port)) {
        return;
      }
    } catch (error) {
      lastError = error;
    }
    await sleep(500);
  }
  throw lastError || new Error("formal-ai API did not become ready");
}

async function startApiProcess(port, options = {}) {
  const spawn = options.spawn || childProcess.spawn;
  const stdout = options.stdout || process.stdout;
  const stderr = options.stderr || process.stderr;
  let lastError = null;

  for (const candidate of apiCandidates(port, options)) {
    let child = null;
    try {
      child = spawn(candidate.command, candidate.args, {
        cwd: candidate.cwd,
        env: scrubbedEnvironment(port, options.env || process.env),
        stdio: ["ignore", "pipe", "pipe"],
      });
      if (child.stdout && typeof child.stdout.on === "function") {
        child.stdout.on("data", (chunk) => stdout.write(`[formal-ai] ${chunk}`));
      }
      if (child.stderr && typeof child.stderr.on === "function") {
        child.stderr.on("data", (chunk) => stderr.write(`[formal-ai] ${chunk}`));
      }

      await Promise.race([
        waitForApi(port, options),
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
      if (child && !child.killed && typeof child.kill === "function") {
        child.kill();
      }
    }
  }
  throw lastError || new Error("could not start formal-ai API");
}

function errorMessage(error) {
  return error && error.message ? error.message : String(error || "unknown error");
}

function statusForPort(port, options = {}) {
  const apiBase = port ? `http://127.0.0.1:${port}` : "";
  const ready = Boolean(options.apiReady && apiBase);
  return {
    mode: apiBase ? "server" : "in-process",
    apiBase,
    chatUrl: apiBase ? `${apiBase}/v1/chat/completions` : "",
    graphUrl: apiBase ? `${apiBase}/v1/graph` : "",
    traceUrl: apiBase ? `${apiBase}/v1/graph?trace=answer_greeting_hi` : "",
    apiReady: ready,
    apiError: String(options.apiError || ""),
    agentProvider: {
      type: "local-openai-compatible",
      apiBase,
      openAiBaseUrl: apiBase ? `${apiBase}/v1` : "",
      model: MODEL_ID,
    },
    reused: Boolean(options.reused),
  };
}

function createLocalServerManager(options = {}) {
  const findPort = options.findFreePort || findFreePort;
  const health = options.requestHealth || requestHealth;
  const startProcess =
    options.startApiProcess || ((port) => startApiProcess(port, options));
  const killProcess =
    options.killProcess ||
    ((processRef) => {
      if (processRef && !processRef.killed && typeof processRef.kill === "function") {
        processRef.kill();
      }
    });

  let apiPort = Number.isInteger(options.port) && options.port > 0 ? options.port : 0;
  let apiReady = false;
  let apiError = "";
  let apiProcess = null;
  let ensurePromise = null;

  function currentStatus(extra = {}) {
    return statusForPort(apiPort, {
      apiReady,
      apiError,
      reused: false,
      ...extra,
    });
  }

  async function isHealthy() {
    if (!apiPort || !apiReady) {
      return false;
    }
    try {
      return Boolean(await health(apiPort));
    } catch (error) {
      apiError = errorMessage(error);
      return false;
    }
  }

  async function ensure() {
    if (ensurePromise) {
      return ensurePromise;
    }

    ensurePromise = (async () => {
      try {
        if (await isHealthy()) {
          apiError = "";
          return currentStatus({ reused: true });
        }

        if (!apiPort) {
          apiPort = await findPort();
        } else if (apiProcess) {
          killProcess(apiProcess);
          apiProcess = null;
        }

        apiReady = false;
        apiError = "";
        try {
          apiProcess = await startProcess(apiPort);
          apiReady = true;
          apiError = "";
          return currentStatus({ reused: false });
        } catch (error) {
          apiReady = false;
          apiError = errorMessage(error);
          return currentStatus({ reused: false });
        }
      } finally {
        ensurePromise = null;
      }
    })();

    return ensurePromise;
  }

  function shutdown() {
    if (apiProcess) {
      killProcess(apiProcess);
      apiProcess = null;
    }
    apiReady = false;
  }

  return {
    currentStatus,
    ensure,
    shutdown,
  };
}

module.exports = {
  SERVER_OPT_IN_ENV,
  MODEL_ID,
  API_AUTH_ENV_KEYS,
  serverModeRequested,
  findFreePort,
  packagedBinaryPath,
  scrubbedEnvironment,
  apiCandidates,
  requestHealth,
  waitForApi,
  startApiProcess,
  statusForPort,
  createLocalServerManager,
};
