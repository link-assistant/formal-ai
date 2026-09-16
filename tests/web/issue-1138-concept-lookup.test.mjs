// Issue #1138, plan 01 L13: the browser must resolve a word the seed does not
// contain through the *same* bounded registry walk the Rust path uses, and must
// label anything it composes from that meaning unverified.
//
// These tests boot the real worker (`src/web/formal_ai_worker.js`, which loads
// `src/web/worker/formal_ai_worker_source_walk.js` and
// `…_concept_lookup.js`) with the real seed registry and replay the same
// committed captures under `tests/fixtures/issue-1138-b1/` that
// `tests/unit/issue_1138_concept_lookup.rs` replays. Nothing here is a
// hand-written gloss: every byte came from a real service through the
// production fetch path and its sha256 is recorded in `capture-manifest.lino`.

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../..");
const FIXTURE_DIR = path.join(REPO_ROOT, "tests/fixtures/issue-1138-b1");
const CACHE_DIR = path.join(FIXTURE_DIR, "source-cache");

/** The held-out word; it appears in no seed file. */
const HELD_OUT_WORD = "isogram";

/** The committed captures as a `url → { body, sha256 }` map. */
function loadCaptures() {
  const captures = new Map();
  for (const name of readdirSync(CACHE_DIR).filter((entry) => entry.endsWith(".meta"))) {
    const meta = readFileSync(path.join(CACHE_DIR, name), "utf8");
    const field = (key) => {
      const line = meta.split("\n").find((entry) => entry.startsWith(`${key}=`));
      return line ? line.slice(key.length + 1) : "";
    };
    const url = field("url");
    const sha256 = field("sha256");
    if (!url || !sha256) continue;
    const body = readFileSync(path.join(CACHE_DIR, "objects", `${sha256}.body`), "utf8");
    captures.set(url, { body, sha256, fetchedAt: field("fetched_at") });
  }
  return captures;
}

/**
 * A worker whose `fetch` serves the seed files and the committed captures, and
 * refuses everything else, so a test can never silently reach the network.
 */
async function bootWorker(captures, requested) {
  const context = createWorkerContext({
    fetch: (url) => {
      const target = String(url);
      const relative = target.split("?")[0].replace(/^\.?\//, "");
      if (!target.startsWith("http")) {
        const onDisk = relative.startsWith("seed/")
          ? path.join(REPO_ROOT, "data", relative)
          : path.join(REPO_ROOT, "src/web", relative);
        try {
          const text = readFileSync(onDisk, "utf8");
          return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
        } catch {
          return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
        }
      }
      if (requested) requested.push(target);
      const capture = captures.get(target);
      if (!capture) {
        return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
      }
      return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(capture.body) });
    },
  });
  await evaluate(context, "loadSeed()");
  return context;
}

/** Resolve one surface inside the worker realm and copy the senses out. */
async function lookup(context, surface, language = "en", preferences = {}) {
  context.__lookupSurface = surface;
  context.__lookupLanguage = language;
  context.__lookupPreferences = preferences;
  return plain(
    await evaluate(context, "lookupConceptSurface(__lookupSurface, __lookupLanguage, __lookupPreferences)"),
  );
}

test("the browser resolves an unknown word to the same senses as the native path", async () => {
  const captures = loadCaptures();
  const context = await bootWorker(captures);
  const outcome = await lookup(context, HELD_OUT_WORD);

  assert.ok(outcome.items.length > 0, "the committed captures must answer in the browser too");
  const expected = JSON.parse(readFileSync(path.join(FIXTURE_DIR, "expected-senses.json"), "utf8"));
  for (const sense of outcome.items) {
    assert.match(sense.sha256, /^[0-9a-f]{64}$/u);
    assert.equal(sense.sha256, captures.get(sense.sourceUrl).sha256);
    assert.ok(sense.licenseName.length > 0 && sense.licenseUrl.startsWith("http"));
    assert.ok(
      expected.some((row) => row.contentId === sense.contentId),
      `the browser sense ${sense.contentId} is not in the cross-runtime expectation`,
    );
  }
});

test("a settings opt-out silences a dictionary in the browser too", async () => {
  const context = await bootWorker(loadCaptures());
  const outcome = await lookup(context, HELD_OUT_WORD, "en", { externalServiceWiktionary: false });

  assert.ok(
    outcome.items.every((sense) => sense.sourceId !== "wiktionary"),
    "an opted-out dictionary contributes nothing",
  );
  const disabled = outcome.outcomes.find((row) => row.sourceId === "wiktionary");
  assert.equal(disabled.status, "disabled");
  assert.equal(disabled.items, 0);
});

test("a program composed from a retrieved meaning is labelled unverified", async () => {
  const context = await bootWorker(loadCaptures());
  context.__composePrompt =
    "Write a Python function is_isogram(word) that returns True when the word is an isogram.";
  const composed = plain(await evaluate(context, "composeFromConcepts(__composePrompt)"));

  assert.equal(composed.verified, false, "the browser has no runtime and must say so");
  assert.ok(
    composed.sources.length > 0,
    "an unverified program still names the sources it was built from",
  );
  for (const sense of composed.senses ?? []) {
    assert.ok(
      !composed.source.includes(sense.gloss),
      "a gloss may be quoted and attributed, never inlined into generated code",
    );
  }
});
