import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
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
