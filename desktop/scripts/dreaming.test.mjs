import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  DEFAULT_INITIAL_DELAY_MS,
  DEFAULT_INTERVAL_MS,
  createDreamingScheduler,
  dreamingEnabled,
  lowPriorityCandidate,
  memoryDreamCandidates,
  runDreamingOnce,
} = require("../lib/dreaming.cjs");

test("desktop dreaming is enabled by default and accepts explicit opt-out", () => {
  assert.equal(dreamingEnabled({}), true);
  assert.equal(dreamingEnabled({ FORMAL_AI_DESKTOP_DREAMING: "0" }), false);
  assert.equal(dreamingEnabled({ FORMAL_AI_DESKTOP_DREAMING: "off" }), false);
  assert.equal(dreamingEnabled({ FORMAL_AI_DESKTOP_DREAMING: "on" }), true);
});

test("memory dream candidates reuse desktop binary resolution and stay plan-only by default", () => {
  const candidates = memoryDreamCandidates({
    repoRoot: "/repo",
    env: {
      FORMAL_AI_DESKTOP_BINARY: "/bin/formal-ai",
      FORMAL_AI_MEMORY_PATH: "/tmp/memory.lino",
      FORMAL_AI_DESKTOP_DREAMING_FREE_BYTES: "1024",
      FORMAL_AI_DESKTOP_DREAMING_STORAGE_CAPACITY_BYTES: "8192",
    },
    existsSync: (filePath) => filePath === "/repo/Cargo.toml",
    resourcesPath: "",
  });

  assert.deepEqual(candidates[0], {
    command: "/bin/formal-ai",
    args: [
      "memory",
      "dream",
      "--path",
      "/tmp/memory.lino",
      "--storage-capacity-bytes",
      "8192",
      "--free-bytes",
      "1024",
    ],
    cwd: "/repo",
    label: "FORMAL_AI_DESKTOP_BINARY",
  });
  assert.deepEqual(candidates[1], {
    command: "cargo",
    args: [
      "run",
      "--",
      "memory",
      "dream",
      "--path",
      "/tmp/memory.lino",
      "--storage-capacity-bytes",
      "8192",
      "--free-bytes",
      "1024",
    ],
    cwd: "/repo",
    label: "cargo run",
  });
  assert.ok(!candidates[0].args.includes("--apply"));
});

test("low priority wrapping uses nice on Unix-like platforms", () => {
  const candidate = {
    command: "formal-ai",
    args: ["memory", "dream"],
    cwd: "/repo",
    label: "formal-ai on PATH",
  };

  assert.deepEqual(lowPriorityCandidate(candidate, "linux"), {
    command: "nice",
    args: ["-n", "19", "formal-ai", "memory", "dream"],
    cwd: "/repo",
    label: "low-priority formal-ai on PATH",
  });
  assert.equal(lowPriorityCandidate(candidate, "win32"), candidate);
});

test("runDreamingOnce captures successful plan output", async () => {
  const calls = [];
  const result = await runDreamingOnce({
    env: {},
    platform: "win32",
    candidates: [
      {
        command: "formal-ai",
        args: ["memory", "dream"],
        cwd: "/repo",
        label: "formal-ai on PATH",
      },
    ],
    spawn: (command, args) => {
      calls.push({ command, args });
      return fakeChild({ code: 0, stdout: "memory_dreaming_plan\n" });
    },
  });

  assert.equal(result.ok, true);
  assert.equal(result.state, "ok");
  assert.equal(result.stdout, "memory_dreaming_plan\n");
  assert.deepEqual(calls, [{ command: "formal-ai", args: ["memory", "dream"] }]);
});

test("scheduler starts with an unrefed low-priority timer", () => {
  const timers = [];
  const scheduler = createDreamingScheduler({
    env: {},
    setTimeout: (fn, delay) => {
      const timer = {
        fn,
        delay,
        unrefed: false,
        unref() {
          this.unrefed = true;
        },
      };
      timers.push(timer);
      return timer;
    },
    clearTimeout: () => {},
  });

  const status = scheduler.start();

  assert.equal(status.enabled, true);
  assert.equal(status.scheduled, true);
  assert.equal(status.initialDelayMs, DEFAULT_INITIAL_DELAY_MS);
  assert.equal(status.intervalMs, DEFAULT_INTERVAL_MS);
  assert.equal(timers.length, 1);
  assert.equal(timers[0].delay, DEFAULT_INITIAL_DELAY_MS);
  assert.equal(timers[0].unrefed, true);
  scheduler.stop();
});

function fakeChild({ code = 0, signal = null, stdout = "", stderr = "" }) {
  const child = new EventEmitter();
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  child.unref = () => {
    child.unrefed = true;
  };
  queueMicrotask(() => {
    if (stdout) {
      child.stdout.emit("data", stdout);
    }
    if (stderr) {
      child.stderr.emit("data", stderr);
    }
    child.emit("exit", code, signal);
  });
  return child;
}
