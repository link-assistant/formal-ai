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

test("scheduler yields while foreground work is active", async () => {
  let spawned = 0;
  const scheduler = createDreamingScheduler({
    env: {},
    isIdle: () => false,
    candidates: [
      { command: "formal-ai", args: ["memory", "dream"], cwd: "/repo", label: "test" },
    ],
    spawn: () => {
      spawned += 1;
      return fakeChild({ code: 0 });
    },
  });

  const result = await scheduler.runNow();

  assert.equal(result.state, "foreground-active");
  assert.equal(spawned, 0);
});

test("a running dreaming child yields when foreground work arrives", async () => {
  let idle = true;
  let killed = false;
  const child = new EventEmitter();
  child.pid = 42;
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  child.unref = () => {};
  child.kill = () => {
    killed = true;
    queueMicrotask(() => child.emit("exit", null, "SIGTERM"));
  };
  const resultPromise = runDreamingOnce({
    env: {},
    platform: "win32",
    candidates: [
      { command: "formal-ai", args: ["memory", "dream"], cwd: "/repo", label: "test" },
    ],
    isIdle: () => idle,
    spawn: () => child,
    setPriority: () => {},
    setInterval: callback => {
      idle = false;
      queueMicrotask(callback);
      return { unref() {} };
    },
    clearInterval: () => {},
  });

  const result = await resultPromise;
  assert.equal(killed, true);
  assert.equal(result.ok, true);
  assert.equal(result.state, "foreground-active");
});

test("storage pressure asks consent, applies it, and surfaces migration", async () => {
  const calls = [];
  let persisted = false;
  let migrationNotified = false;
  const result = await runDreamingOnce({
    env: {},
    platform: "win32",
    candidates: [
      { command: "formal-ai", args: ["memory", "dream"], cwd: "/repo", label: "test" },
    ],
    setPriority: () => {},
    spawn: (_command, args) => {
      calls.push(args);
      return fakeChild({
        code: 0,
        stdout: "required_reclaim_bytes: 128\nrequires_bigger_storage: true\n",
      });
    },
    requestAutoFreeSpaceConsent: async () => true,
    persistAutoFreeSpaceChoice: async value => {
      persisted = value;
    },
    notifyBiggerStorage: async () => {
      migrationNotified = true;
    },
  });

  assert.equal(result.state, "auto-free-space-applied");
  assert.equal(persisted, true);
  assert.equal(migrationNotified, true);
  assert.deepEqual(calls[1].slice(-2), ["--apply", "--confirm"]);
  // A consented auto-apply must always carry a backup (issue #540 §4).
  const backupIndex = calls[1].indexOf("--backup");
  assert.ok(backupIndex >= 0, "auto-apply must pass --backup");
  assert.ok(calls[1][backupIndex + 1].endsWith(".pre-dream-backup.lino"));
});

test("auto-apply keeps an explicitly configured backup path", async () => {
  const calls = [];
  await runDreamingOnce({
    env: {},
    platform: "win32",
    candidates: [
      {
        command: "formal-ai",
        args: ["memory", "dream", "--path", "/data/mem.lino", "--backup", "/data/own.lino"],
        cwd: "/repo",
        label: "test",
      },
    ],
    setPriority: () => {},
    spawn: (_command, args) => {
      calls.push(args);
      return fakeChild({ code: 0, stdout: "required_reclaim_bytes: 128\n" });
    },
    requestAutoFreeSpaceConsent: async () => true,
    persistAutoFreeSpaceChoice: async () => {},
  });
  assert.equal(calls[1].filter(argument => argument === "--backup").length, 1);
  assert.equal(calls[1][calls[1].indexOf("--backup") + 1], "/data/own.lino");
});

test("declined automatic cleanup is persisted without applying removals", async () => {
  const choices = [];
  const calls = [];
  const result = await runDreamingOnce({
    env: {},
    platform: "win32",
    candidates: [
      { command: "formal-ai", args: ["memory", "dream"], cwd: "/repo", label: "test" },
    ],
    spawn: (_command, args) => {
      calls.push(args);
      return fakeChild({ code: 0, stdout: "required_reclaim_bytes: 128\n" });
    },
    requestAutoFreeSpaceConsent: async () => false,
    persistAutoFreeSpaceChoice: async value => choices.push(value),
  });

  assert.equal(result.state, "ok");
  assert.deepEqual(choices, [false]);
  assert.equal(calls.length, 1);
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
