// Issue #1632 (R1): a transient wikiHow failure must not silence the priority
// how-to source for the seven-day accessibility TTL. Twin of
// rust/tests/unit/issue_1632_how_to_priority.rs; replays the committed
// issue-991 captures through the real worker.

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const CACHE_DIR = path.join(REPO_ROOT, "rust/tests/fixtures/issue-991/source-cache");

function loadCaptures() {
  const captures = new Map();
  for (const name of readdirSync(CACHE_DIR).filter((entry) => entry.endsWith(".meta"))) {
    const meta = readFileSync(path.join(CACHE_DIR, name), "utf8");
    const field = (key) => meta.split("\n").find((line) => line.startsWith(`${key}=`))?.slice(key.length + 1) || "";
    const url = field("url");
    const sha256 = field("sha256");
    if (url && sha256) {
      captures.set(url, readFileSync(path.join(CACHE_DIR, "objects", `${sha256}.body`), "utf8"));
    }
  }
  return captures;
}

async function bootWorker(captures) {
  const context = createWorkerContext({
    fetch: (url) => {
      const target = String(url);
      const parsed = new URL(target, "http://localhost/");
      const relative = parsed.pathname.replace(/^\/+/, "");
      if (parsed.origin === "http://localhost" || !target.startsWith("http")) {
        const onDisk = relative.startsWith("seed/")
          ? path.join(REPO_ROOT, "data", relative)
          : path.join(REPO_ROOT, "js", relative);
        try {
          const text = readFileSync(onDisk, "utf8");
          return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
        } catch {
          return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
        }
      }
      const body = captures.get(target);
      return Promise.resolve(body === undefined
        ? { ok: false, status: 404, text: () => Promise.resolve("") }
        : { ok: true, status: 200, text: () => Promise.resolve(body) });
    },
  });
  await evaluate(context, "loadSeed()");
  return context;
}

test("a cached wikiHow failure does not skip the priority how-to source", { timeout: 60_000 }, async () => {
  const context = await bootWorker(loadCaptures());
  const synthesize = async () => plain(await evaluate(context, 'synthesizeHowToGuide("make pancakes", {})'));
  const baseline = await synthesize();
  assert.equal(baseline.outcomes.find((outcome) => outcome.sourceId === "wikihow")?.status, "contributed");

  const keys = plain(evaluate(context, "Array.from(sourceWalkAccessibility.keys())"));
  assert.ok(keys.length > 0, "the walk records service accessibility");
  for (const key of keys) {
    context.__key = key;
    evaluate(context, 'howToObserveService(__key, "unreachable", "curl: (28) Operation timed out", howToNowSeconds())');
  }

  const guide = await synthesize();
  const wikihow = guide.outcomes.find((outcome) => outcome.sourceId === "wikihow");
  assert.equal(wikihow.status, "contributed", JSON.stringify(wikihow));
  assert.ok(guide.steps.length > 0, "the guide has real steps");
  for (const key of keys) {
    context.__key = key;
    const unreachable = evaluate(context, "sourceWalkServiceKnownUnreachable(__key, howToNowSeconds())");
    assert.equal(unreachable, !key.startsWith("wikihow"), `accessibility of ${key} after the retry`);
  }
});
