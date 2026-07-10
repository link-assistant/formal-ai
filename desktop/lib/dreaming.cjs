"use strict";

const childProcess = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { packagedBinaryPath } = require("./local-server.cjs");

const ENABLE_ENV = "FORMAL_AI_DESKTOP_DREAMING";
const DEFAULT_INITIAL_DELAY_MS = 60_000;
const DEFAULT_INTERVAL_MS = 21_600_000;
const MAX_CAPTURE_BYTES = 64 * 1024;

function truthy(value) {
  return ["1", "true", "yes", "on"].includes(String(value || "").trim().toLowerCase());
}

function falsey(value) {
  return ["0", "false", "no", "off"].includes(String(value || "").trim().toLowerCase());
}

function dreamingEnabled(env = process.env) {
  const raw = env[ENABLE_ENV];
  return raw === undefined || raw === "" ? true : !falsey(raw);
}

function positiveInteger(value, fallback) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : fallback;
}

function appendOptionalInteger(args, flag, value) {
  if (value === undefined || value === null || value === "") {
    return;
  }
  const parsed = Number(value);
  if (Number.isInteger(parsed) && parsed >= 0) {
    args.push(flag, String(parsed));
  }
}

function optionOrEnv(optionValue, envValue) {
  return optionValue === undefined || optionValue === null || optionValue === "" ? envValue : optionValue;
}

function memoryDreamArgs(options = {}) {
  const env = options.env || process.env;
  const args = ["memory", "dream"];
  const memoryPath = optionOrEnv(options.memoryPath, env.FORMAL_AI_MEMORY_PATH);
  if (memoryPath) {
    args.push("--path", String(memoryPath));
  }
  appendOptionalInteger(
    args,
    "--storage-capacity-bytes",
    optionOrEnv(options.storageCapacityBytes, env.FORMAL_AI_DESKTOP_DREAMING_STORAGE_CAPACITY_BYTES),
  );
  appendOptionalInteger(
    args,
    "--free-bytes",
    optionOrEnv(options.freeBytes, env.FORMAL_AI_DESKTOP_DREAMING_FREE_BYTES),
  );
  appendOptionalInteger(
    args,
    "--incoming-bytes",
    optionOrEnv(options.incomingBytes, env.FORMAL_AI_DESKTOP_DREAMING_INCOMING_BYTES),
  );
  appendOptionalInteger(
    args,
    "--target-free-ratio-percent",
    optionOrEnv(options.targetFreeRatioPercent, env.FORMAL_AI_DESKTOP_DREAMING_TARGET_FREE_RATIO_PERCENT),
  );
  if (truthy(optionOrEnv(options.apply, env.FORMAL_AI_DESKTOP_DREAMING_APPLY))) {
    args.push("--apply");
  }
  const backup = optionOrEnv(options.backup, env.FORMAL_AI_DESKTOP_DREAMING_BACKUP);
  if (backup) {
    args.push("--backup", String(backup));
  }
  if (truthy(optionOrEnv(options.confirm, env.FORMAL_AI_DESKTOP_DREAMING_CONFIRM))) {
    args.push("--confirm");
  }
  return args;
}

function memoryDreamCandidates(options = {}) {
  const repoRoot = options.repoRoot || path.resolve(__dirname, "..", "..");
  const env = options.env || process.env;
  const existsSync = options.existsSync || fs.existsSync;
  const dreamArgs = memoryDreamArgs(options);
  const candidates = [];

  if (env.FORMAL_AI_DESKTOP_BINARY) {
    candidates.push({
      command: env.FORMAL_AI_DESKTOP_BINARY,
      args: dreamArgs,
      cwd: repoRoot,
      label: "FORMAL_AI_DESKTOP_BINARY",
    });
  }

  const packaged = packagedBinaryPath(options);
  if (packaged && existsSync(packaged)) {
    candidates.push({
      command: packaged,
      args: dreamArgs,
      cwd: repoRoot,
      label: "bundled formal-ai",
    });
  }

  if (existsSync(path.join(repoRoot, "Cargo.toml"))) {
    candidates.push({
      command: "cargo",
      args: ["run", "--", ...dreamArgs],
      cwd: repoRoot,
      label: "cargo run",
    });
  }

  candidates.push({ command: "formal-ai", args: dreamArgs, cwd: repoRoot, label: "formal-ai on PATH" });
  return candidates;
}

function lowPriorityCandidate(candidate, platform = process.platform) {
  if (platform === "win32") {
    return candidate;
  }
  return {
    ...candidate,
    command: "nice",
    args: ["-n", "19", candidate.command, ...candidate.args],
    label: `low-priority ${candidate.label}`,
  };
}

function appendCapture(current, chunk, maxBytes) {
  const next = `${current}${String(chunk || "")}`;
  if (next.length <= maxBytes) {
    return next;
  }
  return next.slice(next.length - maxBytes);
}

function spawnAndCollect(candidate, options = {}) {
  const spawn = options.spawn || childProcess.spawn;
  const maxBytes = options.maxCaptureBytes || MAX_CAPTURE_BYTES;
  return new Promise((resolve) => {
    let stdout = "";
    let stderr = "";
    let child = null;
    let yielded = false;
    let idlePoll = null;
    const clearIdlePoll = () => {
      if (idlePoll) {
        (options.clearInterval || clearInterval)(idlePoll);
        idlePoll = null;
      }
    };
    try {
      child = spawn(candidate.command, candidate.args, {
        cwd: candidate.cwd,
        env: options.env || process.env,
        stdio: ["ignore", "pipe", "pipe"],
      });
    } catch (error) {
      resolve({
        ok: false,
        label: candidate.label,
        command: candidate.command,
        args: candidate.args,
        code: null,
        stdout,
        stderr,
        reason: error && error.message ? error.message : String(error),
      });
      return;
    }

    if (child && typeof child.unref === "function") {
      child.unref();
    }
    if (child && Number.isInteger(child.pid)) {
      try {
        (options.setPriority || os.setPriority)(
          child.pid,
          options.lowPriorityValue ?? os.constants.priority.PRIORITY_LOW,
        );
      } catch (_error) {
        // Cooperative idle checks remain the cross-platform fallback when the
        // host refuses an OS priority change.
      }
    }
    if (options.isIdle && child && typeof child.kill === "function") {
      idlePoll = (options.setInterval || setInterval)(() => {
        if (!options.isIdle()) {
          yielded = true;
          clearIdlePoll();
          child.kill();
        }
      }, options.idlePollMs || 250);
      if (idlePoll && typeof idlePoll.unref === "function") {
        idlePoll.unref();
      }
    }
    if (child.stdout && typeof child.stdout.on === "function") {
      child.stdout.on("data", (chunk) => {
        stdout = appendCapture(stdout, chunk, maxBytes);
      });
    }
    if (child.stderr && typeof child.stderr.on === "function") {
      child.stderr.on("data", (chunk) => {
        stderr = appendCapture(stderr, chunk, maxBytes);
      });
    }
    child.once("error", (error) => {
      clearIdlePoll();
      resolve({
        ok: false,
        label: candidate.label,
        command: candidate.command,
        args: candidate.args,
        code: null,
        stdout,
        stderr,
        yielded,
        reason: yielded ? "foreground work arrived" : error && error.message ? error.message : String(error),
      });
    });
    child.once("exit", (code, signal) => {
      clearIdlePoll();
      const ok = code === 0 && !yielded;
      resolve({
        ok,
        label: candidate.label,
        command: candidate.command,
        args: candidate.args,
        code,
        signal,
        stdout,
        stderr,
        yielded,
        reason: yielded ? "foreground work arrived" : ok ? "" : stderr.trim() || `exit ${code || signal}`,
      });
    });
  });
}

async function runDreamingOnce(options = {}) {
  if (!dreamingEnabled(options.env || process.env)) {
    return { ok: true, state: "disabled", reason: `${ENABLE_ENV}=off` };
  }
  const platform = options.platform || process.platform;
  const candidates = options.candidates || memoryDreamCandidates(options);
  let lastResult = null;
  for (const candidate of candidates) {
    const runnable =
      options.lowPriority === false ? candidate : lowPriorityCandidate(candidate, platform);
    const result = await spawnAndCollect(runnable, options);
    if (result.yielded) {
      return { ...result, ok: true, state: "foreground-active" };
    }
    if (result.ok) {
      let finalResult = { ...result, state: "ok" };
      const requiredReclaimBytes = planNumber(result.stdout, "required_reclaim_bytes");
      const requiresBiggerStorage = planBoolean(result.stdout, "requires_bigger_storage");
      if (requiredReclaimBytes > 0 && options.requestAutoFreeSpaceConsent) {
        const enabled = await options.requestAutoFreeSpaceConsent(finalResult);
        if (options.persistAutoFreeSpaceChoice) {
          await options.persistAutoFreeSpaceChoice(enabled);
        }
        if (enabled) {
          const applyCandidate = {
            ...candidate,
            args: [...candidate.args, "--apply", "--confirm"],
            label: `${candidate.label} with auto-free-space`,
          };
          const runnableApply =
            options.lowPriority === false
              ? applyCandidate
              : lowPriorityCandidate(applyCandidate, platform);
          finalResult = {
            ...(await spawnAndCollect(runnableApply, options)),
            state: "auto-free-space-applied",
          };
        }
      }
      if (requiresBiggerStorage && options.notifyBiggerStorage) {
        await options.notifyBiggerStorage(finalResult);
      }
      return finalResult;
    }
    lastResult = result;
  }
  return (
    lastResult || {
      ok: false,
      state: "error",
      reason: "no formal-ai memory dream candidate is available",
    }
  );
}

function planNumber(output, key) {
  const match = String(output || "").match(new RegExp(`^\\s*${key}:\\s*(\\d+)\\s*$`, "m"));
  return match ? Number.parseInt(match[1], 10) : 0;
}

function planBoolean(output, key) {
  const match = String(output || "").match(
    new RegExp(`^\\s*${key}:\\s*(true|false)\\s*$`, "mi"),
  );
  return Boolean(match && match[1].toLowerCase() === "true");
}

function createDreamingScheduler(options = {}) {
  const env = options.env || process.env;
  const setTimer = options.setTimeout || setTimeout;
  const clearTimer = options.clearTimeout || clearTimeout;
  const onStatusChange = options.onStatusChange || (() => {});
  const log = options.log || (() => {});
  const isIdle = options.isIdle || (() => true);
  const enabled = dreamingEnabled(env);
  const initialDelayMs = positiveInteger(
    options.initialDelayMs || env.FORMAL_AI_DESKTOP_DREAMING_INITIAL_DELAY_MS,
    DEFAULT_INITIAL_DELAY_MS,
  );
  const intervalMs = positiveInteger(
    options.intervalMs || env.FORMAL_AI_DESKTOP_DREAMING_INTERVAL_MS,
    DEFAULT_INTERVAL_MS,
  );
  let timer = null;
  let stopped = false;
  let running = false;
  let lastResult = null;

  function status() {
    return {
      enabled,
      running,
      scheduled: Boolean(timer),
      initialDelayMs,
      intervalMs,
      lastResult,
    };
  }

  function publish() {
    const snapshot = status();
    onStatusChange(snapshot);
    return snapshot;
  }

  function schedule(delayMs) {
    if (!enabled || stopped || timer) {
      return publish();
    }
    timer = setTimer(() => {
      timer = null;
      runNow()
        .catch((error) => {
          lastResult = {
            ok: false,
            state: "error",
            reason: error && error.message ? error.message : String(error),
          };
          log("dreaming run failed:", lastResult.reason);
        })
        .finally(() => {
          if (!stopped) {
            schedule(intervalMs);
          }
        });
    }, delayMs);
    if (timer && typeof timer.unref === "function") {
      timer.unref();
    }
    return publish();
  }

  async function runNow() {
    if (running) {
      return { ok: true, state: "already-running" };
    }
    if (!isIdle()) {
      lastResult = {
        ok: true,
        state: "foreground-active",
        reason: "waiting for real user idle",
      };
      publish();
      return lastResult;
    }
    running = true;
    publish();
    try {
      lastResult = await runDreamingOnce(options);
      if (!lastResult.ok) {
        log("dreaming run did not complete:", lastResult.reason || lastResult.state);
      }
      return lastResult;
    } finally {
      running = false;
      publish();
    }
  }

  function start() {
    stopped = false;
    if (!enabled) {
      return publish();
    }
    return schedule(initialDelayMs);
  }

  function stop() {
    stopped = true;
    if (timer) {
      clearTimer(timer);
      timer = null;
    }
    return publish();
  }

  return { start, stop, runNow, status };
}

module.exports = {
  ENABLE_ENV,
  DEFAULT_INITIAL_DELAY_MS,
  DEFAULT_INTERVAL_MS,
  dreamingEnabled,
  memoryDreamArgs,
  memoryDreamCandidates,
  lowPriorityCandidate,
  runDreamingOnce,
  planBoolean,
  planNumber,
  createDreamingScheduler,
};
