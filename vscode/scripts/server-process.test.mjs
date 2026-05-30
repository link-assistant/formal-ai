import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const { apiCandidates, requestHealth, waitForApi } = require("../src/lib/server-process.cjs");

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "..", ".."); // a real checkout: has Cargo.toml

test("apiCandidates falls back to formal-ai on PATH with no repo and no override", () => {
  const candidates = apiCandidates({});
  assert.equal(candidates.length, 1);
  assert.equal(candidates[0].command, "formal-ai");
  assert.equal(candidates[0].label, "formal-ai on PATH");
  assert.deepEqual(candidates[0].args, ["serve", "--host", "127.0.0.1", "--port", "18080"]);
});

test("apiCandidates adds a cargo-run candidate when a Cargo.toml is present", () => {
  const candidates = apiCandidates({ repoRoot });
  assert.equal(candidates.length, 2);
  assert.equal(candidates[0].command, "cargo");
  assert.equal(candidates[0].label, "cargo run");
  assert.deepEqual(candidates[0].args, [
    "run", "--quiet", "--", "serve", "--host", "127.0.0.1", "--port", "18080",
  ]);
  assert.equal(candidates[0].cwd, repoRoot);
  assert.equal(candidates[1].command, "formal-ai");
});

test("apiCandidates puts an explicit binary override first", () => {
  const candidates = apiCandidates({
    repoRoot,
    env: { FORMAL_AI_VSCODE_BINARY: "/opt/formal-ai" },
  });
  assert.equal(candidates[0].command, "/opt/formal-ai");
  assert.equal(candidates[0].label, "binary override");
  // override, then cargo (Cargo.toml present), then PATH
  assert.deepEqual(candidates.map((c) => c.label), [
    "binary override", "cargo run", "formal-ai on PATH",
  ]);
});

test("apiCandidates also honours FORMAL_AI_DESKTOP_BINARY", () => {
  const candidates = apiCandidates({ env: { FORMAL_AI_DESKTOP_BINARY: "/opt/desktop-bin" } });
  assert.equal(candidates[0].command, "/opt/desktop-bin");
  assert.equal(candidates[0].label, "binary override");
});

test("apiCandidates threads a custom host and port into every candidate", () => {
  const candidates = apiCandidates({ repoRoot, host: "0.0.0.0", port: 9099 });
  for (const candidate of candidates) {
    assert.ok(candidate.args.includes("0.0.0.0"));
    assert.ok(candidate.args.includes("9099"));
  }
});

test("requestHealth resolves true on a 200 /health and false otherwise", async () => {
  const server = http.createServer((req, res) => {
    if (req.url === "/health") {
      res.writeHead(200);
      res.end("ok");
    } else {
      res.writeHead(404);
      res.end();
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  try {
    assert.equal(await requestHealth("127.0.0.1", port), true);
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("requestHealth resolves false when the server answers non-200", async () => {
  const server = http.createServer((req, res) => {
    res.writeHead(503);
    res.end();
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  try {
    assert.equal(await requestHealth("127.0.0.1", port), false);
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("waitForApi throws once the timeout window has elapsed", async () => {
  // timeoutMs = 0 means the polling loop exits before any health request, so
  // this neither hits the network nor waits — it just proves the timeout path.
  await assert.rejects(() => waitForApi("127.0.0.1", 1, 0), /did not become ready/);
});
