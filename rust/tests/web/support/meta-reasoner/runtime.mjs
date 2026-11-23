// The recursive meta reasoner (js/worker/formal_ai_worker_meta_reasoner.js,
// data/seed/meta-reasoning.lino). Every prompt here is absent from the code
// and the seed: the answers are derived — grounded, enumerated, verified —
// never looked up. Dictionary knowledge comes only from real captures under
// rust/tests/fixtures/meta-reasoner (sha256 in capture-manifest.lino).
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "../browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../../../..");
const FIXTURES = path.join(REPO_ROOT, "rust/tests/fixtures/meta-reasoner");

/** The committed captures as `url → body`, each checked against the manifest. */
function captures() {
  const manifest = readFileSync(path.join(FIXTURES, "capture-manifest.lino"), "utf8");
  const out = new Map();
  for (const block of manifest.split("\n  capture ").slice(1)) {
    const word = block.split("\n")[0].trim();
    const url = /url "([^"]+)"/u.exec(block)[1];
    const sha256 = /sha256 ([0-9a-f]{64})/u.exec(block)[1];
    const body = readFileSync(path.join(FIXTURES, "captures", `${word}.json`), "utf8");
    assert.equal(createHash("sha256").update(body).digest("hex"), sha256, `${word} capture is unedited`);
    out.set(url, body);
  }
  return out;
}

const replay = captures();

/** True when a real capture of `word` is committed or pre-cached. */
function captured(word) {
  return existsSync(path.join(FIXTURES, "captures", `${word}.json`)) || existsSync(path.join(REPO_ROOT, "data/cache/wiktionary/en", `${word}.json`));
}
const worker = createWorkerContext({
  fetch: async (url) => {
    const target = String(url);
    if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
      // Web-root assets: scripts from js/, seed files from data/seed/.
      const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return { ok: true, status: 200, text: async () => text };
      } catch {
        return { ok: false, status: 404, text: async () => "" };
      }
    }
    const body = replay.get(target);
    if (body !== undefined) return { ok: true, status: 200, text: async () => body };
    // The repository's pre-cached Wiktionary captures (data/cache/wiktionary).
    const cached = /^https:\/\/api\.dictionaryapi\.dev\/api\/v2\/entries\/en\/([a-z]+)$/u.exec(target);
    const file = cached && path.join(REPO_ROOT, "data/cache/wiktionary/en", `${cached[1]}.json`);
    if (file && existsSync(file)) {
      const text = readFileSync(file, "utf8");
      return { ok: true, status: 200, text: async () => text };
    }
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

async function reason(prompt) {
  await ready;
  return plain(await evaluate(
    worker,
    `metaReasonTurn(${JSON.stringify(prompt)}, "en", {}).then((r) => ({
      goal: r.goal, status: r.status, needs: r.needs, lookups: r.lookups,
      program: r.program, verification: r.verification, probe: r.probe || null,
      unknowns: r.unknowns, events: r.trace.events.map((e) => e.kind + ": " + e.detail),
      lino: r.derivationLino }))`,
  ));
}

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

export {captured, reason, solve, worker, ready, evaluate, FIXTURES};
