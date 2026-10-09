import { readdirSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";
import { specificationCases } from "../../../scripts/lib/rust-specification-cases.mjs";
import { executeTypedProgram } from "../../../scripts/lib/rust-specification-programs.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const CACHE_DIR = path.join(REPO_ROOT, "rust/tests/fixtures/issue-991/source-cache");
const originals = [
  ["greeting_prefixed_russian_connect_how_to_composes_procedure", 7, 11],
  ["procedural_elaboration_followup_rebinds_to_prior_how_to", 4, 6],
  ["procedural_elaboration_followup_covers_supported_languages", 4, 16],
];
for (const [name, macroCount, executedCount] of originals) {
  test("unchanged native procedural assertions: " + name, async () => {
    const native = specificationCases(REPO_ROOT).cases.find(item => item.id.endsWith("::" + name));
    assert.ok(native?.program, "the complete native body must parse");
    assert.equal(native.program.nativeAssertions, macroCount);
    const result = await executeTypedProgram(new WorkerHost(), native.program);
    assert.equal(result.status, "passed", JSON.stringify(result.failure));
    assert.equal(result.assertions, executedCount);
  });
}

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
    assert.equal(createHash("sha256").update(body).digest("hex"), sha256);
    captures.set(url, { body, sha256, fetchedAt: field("fetched_at") });
  }
  return captures;
}

/**
 * A worker whose `fetch` serves the seed files and the committed captures, and
 * refuses everything else — so a test can never silently reach the network and
 * an unexpected request shows up as an explicit failure in the guide trace.
 */
async function bootWorker(captures, requested) {
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

test("no-source plans retain transport encoding and producer-owned typed stages", async () => {
  const requested = [];
  const context = await bootWorker(new Map(), requested);
  const result = plain(await evaluate(context, 'tryProceduralHowTo("как подключить redis к ruby", "ru", {})'));
  assert.equal(result.intent, "procedural_how_to");
  assert.match(result.content, /page=Подключить-Redis-К-Ruby&/u);
  assert.ok(requested.some(url => url.includes("page=%D0%9F")));
  assert.ok(result.solverEvents.some(event => event.kind === "procedural_how_to:request"
    && event.payload === "подключить redis к ruby"));
  assert.ok(result.solverEvents.some(event => event.kind === "web_search:fusion_planned"));
  assert.ok(!result.solverEvents.some(event => /(?:capture|source):accepted/u.test(event.kind)));
});

test("disabled source services keep their actual outcome and skip wikiHow requests", async () => {
  const requested = [];
  const context = await bootWorker(new Map(), requested);
  const result = plain(await evaluate(context,
    'tryProceduralHowTo("how to connect redis to ruby", "en", { externalServiceWikihow: false })'));
  assert.equal(result.intent, "procedural_how_to");
  assert.ok(!requested.some(url => new URL(url).hostname.endsWith("wikihow.com")));
  assert.ok(!result.solverEvents.some(event => event.kind === "http_fetch:request"
    && event.payload.includes("wikihow.com")));
  assert.ok(result.evidence.includes("procedural_how_to:service_disabled:wikihow"));
  assert.ok(result.solverEvents.some(event => event.kind === "procedural_how_to:service_disabled"
    && event.payload === "wikihow"));
});

test("a procedural followup requires an actual prior procedural dialogue", async () => {
  const context = await bootWorker(new Map());
  assert.equal(await evaluate(context, 'tryProceduralHowToFollowup("tell me more", "en", [], {})'), null);
});

test("genuine captured source steps survive discovery-plan rendering", async () => {
  const captures = loadCaptures();
  const requested = [];
  const context = await bootWorker(captures, requested);
  const result = plain(await evaluate(context, 'tryProceduralHowTo("how to make pancakes", "en", {})'));
  assert.equal(result.intent, "procedural_how_to");
  assert.ok(result.guide.steps.length >= 2);
  assert.match(result.content, /Sources/u);
  for (const step of result.guide.steps) {
    assert.equal(captures.get(step.sourceUrl)?.sha256, step.sha256);
    assert.ok(requested.includes(step.sourceUrl));
  }
});
