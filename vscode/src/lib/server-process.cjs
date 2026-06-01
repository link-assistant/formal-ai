"use strict";

// Node-only lifecycle for the opt-in local `formal-ai serve` process.
//
// Issue #353 / R3 (ROADMAP D3): when `formal-ai.server.enabled` is on, the Node
// (desktop) host spins up the OpenAI-compatible HTTP server and routes chat
// through `POST /v1/chat/completions`, exactly like the Electron shell
// (`desktop/main.cjs`). This module owns the candidate discovery, the health
// wait, and the spawn; it is required only from `extension.node.cjs` (never the
// web host, which cannot start a process). `apiCandidates` is pure given its
// inputs, so the discovery order is unit-testable without spawning anything.

const childProcess = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");

// Resolution order: an explicit binary override, then `cargo run` inside a repo
// checkout (dev), then `formal-ai` on PATH (installed). Mirrors
// `desktop/main.cjs` `apiCandidates`, minus the Electron-packaged binary.
function apiCandidates(options = {}) {
  const host = String(options.host || "127.0.0.1");
  const port = String(options.port || "18080");
  const repoRoot = options.repoRoot || "";
  const env = options.env || {};
  const args = ["serve", "--host", host, "--port", port];
  const candidates = [];

  const override = env.FORMAL_AI_VSCODE_BINARY || env.FORMAL_AI_DESKTOP_BINARY;
  if (override) {
    candidates.push({ command: override, args, cwd: repoRoot || undefined, label: "binary override" });
  }
  if (repoRoot && fs.existsSync(path.join(repoRoot, "Cargo.toml"))) {
    candidates.push({
      command: "cargo",
      args: ["run", "--quiet", "--", ...args],
      cwd: repoRoot,
      label: "cargo run",
    });
  }
  candidates.push({
    command: "formal-ai",
    args,
    cwd: repoRoot || undefined,
    label: "formal-ai on PATH",
  });
  return candidates;
}

function requestHealth(host, port) {
  return new Promise((resolve, reject) => {
    const request = http.get(`http://${host}:${port}/health`, (response) => {
      response.resume();
      resolve(response.statusCode === 200);
    });
    request.on("error", reject);
    request.setTimeout(2000, () => {
      request.destroy(new Error("health check timed out"));
    });
  });
}

async function waitForApi(host, port, timeoutMs = 180000, now = () => Date.now()) {
  const startedAt = now();
  let lastError = null;
  while (now() - startedAt < timeoutMs) {
    try {
      if (await requestHealth(host, port)) {
        return;
      }
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw lastError || new Error("formal-ai API did not become ready");
}

// Try each candidate in turn; return the first whose health check passes. The
// caller owns the returned child process and must kill it on deactivate.
async function startServer(options = {}) {
  const host = String(options.host || "127.0.0.1");
  const port = String(options.port || "18080");
  const log = typeof options.log === "function" ? options.log : () => {};
  const childEnv = { ...process.env, ...(options.env || {}) };
  let lastError = null;

  for (const candidate of apiCandidates(options)) {
    let child = null;
    try {
      log(`starting formal-ai serve via ${candidate.label}`);
      child = childProcess.spawn(candidate.command, candidate.args, {
        cwd: candidate.cwd,
        env: childEnv,
        stdio: ["ignore", "pipe", "pipe"],
      });
      child.stdout.on("data", (chunk) => log(`[formal-ai] ${String(chunk).trimEnd()}`));
      child.stderr.on("data", (chunk) => log(`[formal-ai] ${String(chunk).trimEnd()}`));

      await Promise.race([
        waitForApi(host, port),
        new Promise((_, reject) => {
          child.once("error", reject);
          child.once("exit", (code, signal) => {
            reject(new Error(`${candidate.label} exited before ready: ${code || signal}`));
          });
        }),
      ]);
      return { process: child, apiBase: `http://${host}:${port}`, label: candidate.label };
    } catch (error) {
      lastError = error;
      if (child && !child.killed) {
        child.kill();
      }
    }
  }
  throw lastError || new Error("could not start formal-ai serve");
}

module.exports = { apiCandidates, requestHealth, waitForApi, startServer };
