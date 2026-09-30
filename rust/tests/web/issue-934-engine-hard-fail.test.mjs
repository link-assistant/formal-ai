// Issue #934 (E82): a failed WASM instantiation must surface as a visible
// "engine unavailable" state instead of silently answering from the
// JavaScript mirror, and every answer must record its engine in the trace.
// The diagnostic JS fallback stays reachable only through the explicit
// ?jsfallback=1 worker URL flag.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  createBrowserContext,
  evaluate,
  loadBrowserScript,
  plain,
  REPO_ROOT,
} from "./support/browser-runtime.mjs";

/** Build a worker realm whose WebAssembly engine fails to instantiate. */
function unavailableWorkerContext({ diagnostic = false } = {}) {
  const posted = [];
  const context = createBrowserContext({
    location: {
      href: "http://localhost/worker/formal_ai_worker.js",
      search: diagnostic ? "?jsfallback=1" : "",
    },
    fetch: (url) => {
      const pathname = new URL(String(url), "http://localhost/").pathname;
      if (pathname.endsWith("formal_ai_worker.wasm")) {
        // Simulates a 404/corrupt .wasm: the loader must not fall back
        // silently (the manual test in the issue corrupts the artifact).
        return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
      }
      const relative = pathname.replace(/^\/+/, "").split("?")[0];
      const onDisk = relative.startsWith("seed/")
        ? path.join(REPO_ROOT, "data", relative)
        : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
      } catch {
        return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
      }
    },
    WebAssembly: {
      instantiate: () => Promise.reject(new Error("wasm unavailable in test")),
    },
    postMessage: (message) => posted.push(plain(message)),
    importScripts: (...urls) => {
      for (const url of urls) {
        const pathname = new URL(String(url), "http://localhost/").pathname;
        loadBrowserScript(context, path.posix.join("js", pathname.replace(/^\/+/, "")));
      }
    },
  });
  loadBrowserScript(context, "js/worker/formal_ai_worker.js");
  return { context, posted };
}

/** Poll the captured postMessage log until a reply for a request lands. */
async function waitForReply(posted, requestId, { timeoutMs = 3000 } = {}) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const found = posted.find(
      (message) => message.kind === "message" && message.requestId === requestId,
    );
    if (found) return found;
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  return undefined;
}

test("a failed engine load posts engine_unavailable and refuses to answer", async () => {
  const { context, posted } = unavailableWorkerContext();
  await evaluate(context, "init()");
  const ready = posted.find((message) => message.kind === "ready");
  assert.equal(ready.mode, "engine unavailable");
  assert.match(ready.engineError, /404|wasm unavailable/);
  assert.ok(
    posted.some((message) => message.kind === "engine_unavailable"),
    "the worker must broadcast engine_unavailable loudly",
  );

  const answers = posted.filter((message) => message.kind === "message");
  await evaluate(
    context,
    "self.onmessage({ data: { kind: 'message', requestId: 'r1', prompt: 'hello' } })",
  );
  const refusal = await waitForReply(posted, "r1");
  assert.ok(refusal, "the refusal must be posted as a message reply");
  assert.equal(refusal.intent, "engine_unavailable");
  assert.equal(refusal.engine, "unavailable");
  assert.equal(refusal.confidence, 0);
  assert.match(refusal.content, /could not be loaded/);
  assert.ok(
    refusal.evidence.some((row) => row.startsWith("engine:unavailable")),
    "the refusal carries engine provenance in its evidence",
  );
  assert.equal(answers.length, 0);
});

test("the diagnostic override answers from JS with honest provenance", async () => {
  const { context, posted } = unavailableWorkerContext({ diagnostic: true });
  await evaluate(context, "init()");
  const ready = posted.find((message) => message.kind === "ready");
  assert.equal(ready.mode, "js fallback (diagnostic override)");

  await evaluate(
    context,
    "self.onmessage({ data: { kind: 'message', requestId: 'r2', prompt: 'hello' } })",
  );
  const answer = await waitForReply(posted, "r2");
  assert.ok(answer, "the override path still answers");
  assert.equal(answer.engine, "js (diagnostic override)");
  assert.ok(
    (answer.evidence || []).some((row) => row === "engine:js (diagnostic override)"),
    "the diagnostic answer is marked as JS-produced in the trace",
  );
  assert.notEqual(answer.intent, "engine_unavailable");
});

test("the source no longer contains the silent fallback assignment", () => {
  const source = readFileSync(
    path.join(REPO_ROOT, "js/worker/formal_ai_worker_20.js"),
    "utf8",
  );
  assert.ok(!source.includes('mode = "js fallback"'), "the silent fallback is gone");
  assert.ok(source.includes("engine_unavailable"), "the loud state is present");
  assert.ok(
    source.includes("engine:${engine}") || source.includes("`engine:${engine}`"),
    "answers record engine provenance",
  );
});
