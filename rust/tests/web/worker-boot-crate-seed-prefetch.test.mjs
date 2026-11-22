// PR #1188: the browser worker's cold start is a chain of fetches, and each
// link that waits for the previous one delays the ready engine (the local web
// E2E legs saw the composer stay disabled past their window). The crate
// modules read meaning seeds the worker's own list leaves out; `loadCrateSeeds`
// fetched them only after the worker's own seeds had all arrived, one more
// round trip before ready. The worker now starts that fetch at boot, beside its
// own seed load, and `loadCrateSeeds` takes it. This suite boots the worker
// mirror with a logging fetch and checks the order and the count.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

/** Boots the worker with a fetch that logs each web-root path it is asked for. */
async function bootLogged() {
  const requested = [];
  const heldSeed = "seed/agent-info.lino";
  let started;
  let release;
  const began = new Promise(resolve => { started = resolve; });
  const held = new Promise(resolve => { release = resolve; });
  const fetch = async (url) => {
    const relative = new URL(String(url), "http://localhost/").pathname.slice(1);
    requested.push(relative);
    if (relative === heldSeed) {
      started();
      await held;
    }
    const onDisk = relative.startsWith("seed/")
      ? path.join(REPO_ROOT, "data", relative)
      : path.join(REPO_ROOT, "js", relative);
    try {
      const text = readFileSync(onDisk, "utf8");
      return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
    } catch {
      return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
    }
  };
  const worker = createWorkerContext({ fetch });
  const boot = evaluate(worker, "loadSeed()");
  await began;
  await new Promise(resolve => setImmediate(resolve));
  const whileHeld = new Set(requested);
  release();
  await boot;
  return { worker, requested, whileHeld };
}

test("the missing crate meaning seeds are fetched at boot, once, beside the worker's own seeds", { timeout: 20_000 }, async () => {
  const { worker, requested, whileHeld } = await bootLogged();
  const listed = new Set(evaluate(worker, "FORMAL_AI_SEED_FILES").map((file) => file.split("/").pop()));
  const missing = [...evaluate(worker, "FORMAL_AI_CRATE_MEANING_SEEDS")]
    .map((seed) => `seed/${seed}.lino`)
    .filter((file) => !listed.has(file.split("/").pop()));
  assert.ok(missing.length > 0, "the registry names meaning seeds the worker list leaves out");
  assert.equal(requested.some(file => file.startsWith("source-cache/")), false);
  for (const file of missing) {
    assert.equal(requested.filter((each) => each === file).length, 1, `${file} fetched once`);
    assert.ok(whileHeld.has(file), `${file} waited for the worker's own seeds`);
  }
  // `loadCrateSeeds` took the prefetched texts: the crate host reads them.
  const sample = missing[0].split("/").pop();
  assert.ok(evaluate(worker, `crateHostReadText(${JSON.stringify(`data/seed/${sample}`)})`).length > 0);
});
