import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import childProcess from "node:child_process";
import { syncBuiltinESMExports } from "node:module";
import { fileURLToPath } from "node:url";

const scriptPath = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "prepare-resources.mjs",
);

// Issue #808: `fs.cpSync` rewrites symlink targets to absolute paths unless
// `verbatimSymlinks` is set. Absolute links inside the packaged .app make
// `codesign --verify --deep` fail with "invalid destination for symbolic link
// in bundle", which broke every macOS build.
test("browser runtime copies keep symbolic links verbatim", () => {
  const source = fs.readFileSync(scriptPath, "utf8");
  assert.match(source, /fs\.cpSync\([^)]*verbatimSymlinks:\s*true/s);
});

test("verbatimSymlinks preserves relative framework aliases", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "formal-ai-resources-"));
  const from = path.join(root, "from");
  const to = path.join(root, "to");
  fs.mkdirSync(path.join(from, "Versions", "A"), { recursive: true });
  fs.symlinkSync("A", path.join(from, "Versions", "Current"));

  fs.cpSync(from, to, { recursive: true, verbatimSymlinks: true });

  assert.equal(fs.readlinkSync(path.join(to, "Versions", "Current")), "A");
  fs.rmSync(root, { recursive: true, force: true });
});

import { createHash } from "node:crypto";
import { bundleNativeBinary } from "./native-binary-resources.mjs";

// Exercise the real bundler export with data files and an injected built-in
// process result. These controls never execute a native binary.
test("copied startup shares the selected-artifact cap and immutable job deadline", () => {
  const originalSpawn = childProcess.spawnSync;
  const originalNow = Date.now;
  const originalDeadline = process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
  let time = 900_000;
  let command = null;
  let behavior = () => ({ status: 0, signal: null, stdout: "formal-ai 9.9.9\n", stderr: "" });
  try {
    Date.now = () => time;
    childProcess.spawnSync = (file, args, options) => {
      command = { file, args, options };
      return behavior(file);
    };
    syncBuiltinESMExports();
    nativeFixture((options, put) => {
      const source = put("producer/formal-ai.exe", "native data fixture");
      const invoke = () => bundleNativeBinary({ ...options, platform: "win32", configuredBinary: source, required: true, expectedVersion: "9.9.9" });
      process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = "1000";
      const receipt = invoke();
      assert.equal(command.file, receipt.destination);
      assert.deepEqual(command.args, ["--version"]);
      assert.equal(command.options.timeout, 30_000);
      assert.equal(command.options.maxBuffer, 1_048_576);
      assert.equal(receipt.version, "formal-ai 9.9.9");
      assert.equal(receipt.sha256, createHash("sha256").update(fs.readFileSync(source)).digest("hex"));
      time = 990_000; invoke(); assert.equal(command.options.timeout, 10_000);
      time = 900_000; delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
      invoke(); assert.equal(command.options.timeout, 10_000);
      for (const invalid of ["", " ", "NaN", "Infinity", "1e3", "0x1000", "-1", "0", "900", "1000.5", "9007199254740991", "4000000"]) {
        process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = invalid; command = null;
        assert.throws(invoke, /deadline/);
        assert.equal(command, null);
      }
      process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = "1000";
      for (const result of [
        { error: Object.assign(new Error("timeout fixture"), { code: "ETIMEDOUT" }) },
        { status: null, signal: "SIGTERM", stdout: "formal-ai 9.9.9" },
        { status: 7, signal: null, stdout: "formal-ai 9.9.9" },
        { status: 0, signal: null, stdout: "formal-ai 0.0.0" },
      ]) {
        behavior = () => result;
        assert.throws(invoke, /startup\/version verification failed/);
      }
      behavior = () => {
        time += 12_000;
        return { error: Object.assign(new Error("timeout fixture"), { code: "ETIMEDOUT" }), stdout: "x".repeat(8000), stderr: "y".repeat(8000) };
      };
      assert.throws(invoke, error => {
        const prefix = "Desktop native binary startup/version verification failed: ";
        assert.ok(error.message.startsWith(prefix));
        const diagnostic = JSON.parse(error.message.slice(prefix.length));
        assert.equal(diagnostic.timeoutMs, 30_000);
        assert.equal(diagnostic.elapsedMs, 12_000);
        assert.equal(diagnostic.error.code, "ETIMEDOUT");
        assert.equal(diagnostic.status, null);
        assert.equal(diagnostic.signal, null);
        assert.equal(diagnostic.stdout.length, 4096);
        assert.equal(diagnostic.stdoutCharacters, 8000);
        assert.equal(diagnostic.stdoutTruncated, true);
        assert.equal(diagnostic.stderr.length, 4096);
        assert.equal(diagnostic.stderrCharacters, 8000);
        assert.equal(diagnostic.stderrTruncated, true);
        assert.ok(error.message.length < 10_000);
        return true;
      });
      behavior = () => ({ error: new Error("short failure"), stdout: "short", stderr: "tiny" });
      assert.throws(invoke, error => {
        const diagnostic = JSON.parse(error.message.slice(error.message.indexOf("{")));
        assert.equal(diagnostic.stdoutCharacters, 5);
        assert.equal(diagnostic.stdoutTruncated, false);
        assert.equal(diagnostic.stderrCharacters, 4);
        assert.equal(diagnostic.stderrTruncated, false);
        return true;
      });
      time = 900_000;
      behavior = () => {
        time = 1_000_000;
        return { status: 0, signal: null, stdout: "formal-ai 9.9.9" };
      };
      assert.throws(invoke, /deadline expired/);
      time = 990_000;
      behavior = () => {
        process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = "2000";
        time = 1_000_000;
        return { status: 0, signal: null, stdout: "formal-ai 9.9.9" };
      };
      assert.throws(invoke, /deadline expired/);
      time = 900_000; process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = "1000";
      behavior = destination => {
        fs.writeFileSync(destination, "mutated after process fixture");
        return { status: 0, signal: null, stdout: "formal-ai 9.9.9" };
      };
      assert.throws(invoke, /changed during startup/);
    });
  } finally {
    childProcess.spawnSync = originalSpawn;
    syncBuiltinESMExports();
    Date.now = originalNow;
    if (originalDeadline === undefined) delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
    else process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = originalDeadline;
  }
});

function nativeFixture(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "formal-ai-native-resources-"));
  const options = { repoRoot: root, outputBin: path.join(root, "desktop", "bin") };
  const put = (relative, bytes) => {
    const file = path.join(root, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes);fs.chmodSync(file, 0o755);return file;
  };
  try { run(options, put); } finally { fs.rmSync(root, { recursive: true, force: true }); }
}

test("Cargo manifest output is bundled byte-for-byte before any root fallback", () => nativeFixture((options, put) => {
  const bytes = Buffer.from("native fixture\0\xff");
  put("rust/target/release/formal-ai", bytes);
  put("target/release/formal-ai", "stale root fixture");
  const receipt = bundleNativeBinary({ ...options, platform: "linux" });
  assert.deepEqual(fs.readFileSync(receipt.destination), bytes);
  assert.equal(receipt.sha256, createHash("sha256").update(bytes).digest("hex"));
  assert.equal(receipt.source, path.join(options.repoRoot, "rust/target/release/formal-ai"));
}));

test("an explicit missing engine never borrows a stale conventional binary", () => nativeFixture((options, put) => {
  put("target/release/formal-ai", "stale");
  assert.throws(() => bundleNativeBinary({ ...options, configuredBinary: path.join(options.repoRoot, "missing") }), /missing/);
}));

test("required release mode refuses absent or debug-only engines", () => nativeFixture((options, put) => {
  assert.equal(bundleNativeBinary(options), null);
  put("rust/target/debug/formal-ai", "debug");
  assert.throws(() => bundleNativeBinary({ ...options, platform: "linux", required: true }), /missing/);
}));

test("Windows names and explicit binary selection preserve exact input bytes", () => nativeFixture((options, put) => {
  const source = put("producer/formal-ai.exe", "Windows fixture");
  const receipt = bundleNativeBinary({ ...options, platform: "win32", configuredBinary: source });
  assert.equal(path.basename(receipt.destination), "formal-ai.exe");
  assert.equal(fs.readFileSync(receipt.destination, "utf8"), "Windows fixture");
}));

test("required mode executes the copied engine and checks its exact package version", { skip: process.platform === "win32" }, () => nativeFixture((options, put) => {
  const source = put("producer/formal-ai", "#!/bin/sh\nprintf 'formal-ai 9.9.9\\n'\n");
  const receipt = bundleNativeBinary({ ...options, configuredBinary: source, required: true, expectedVersion: "9.9.9" });
  assert.equal(receipt.version, "formal-ai 9.9.9");
  assert.throws(() => bundleNativeBinary({ ...options, configuredBinary: source, required: true, expectedVersion: "9.9.8" }), /verification failed/);
}));

test("required mode rejects a failed or nonexecutable copied engine", { skip: process.platform === "win32" }, () => nativeFixture((options, put) => {
  const source = put("producer/formal-ai", "#!/bin/sh\nprintf 'formal-ai 9.9.9\\n'\nexit 7\n");
  assert.throws(() => bundleNativeBinary({ ...options, configuredBinary: source, required: true, expectedVersion: "9.9.9" }), /verification failed/);
  fs.writeFileSync(source, "not an executable engine");
  assert.throws(() => bundleNativeBinary({ ...options, configuredBinary: source, required: true, expectedVersion: "9.9.9" }), /verification failed/);
}));
