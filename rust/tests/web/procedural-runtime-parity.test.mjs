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
  assert.match(result.content, /ru\.wikihow\.com.*page=подключить-redis-к-ruby&/u);
  assert.ok(requested.some(url => url.includes("page=%D0%BF")));
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

test("typed discovery stages distinguish skipped, attempted and disabled wikiHow", async () => {
  const context = await bootWorker(new Map());
  const input = '{action:"install",task:"install cedar",object:"cedar"}';
  const args = input + ', "Install-Cedar", "https://www.wikihow.com/api.php?page=Install-Cedar", "cedar installation official documentation", "how to install cedar"';
  const skipped = plain(evaluate(context, 'proceduralDiscoveryEvents(' + args + ', true, false)'));
  assert.ok(!skipped.some(event => event.kind === "http_fetch:request"));
  assert.ok(!skipped.some(event => event.kind === "procedural_how_to:service_disabled"));
  assert.ok(skipped.some(event => event.kind === "procedural_how_to:source_gate"
    && event.payload === "official_documentation_first"));
  const attempted = plain(evaluate(context, 'proceduralDiscoveryEvents(' + args + ', true, true)'));
  assert.ok(attempted.some(event => event.kind === "http_fetch:request"
    && event.payload === "https://www.wikihow.com/api.php?page=Install-Cedar"));
  const disabled = plain(evaluate(context, 'proceduralDiscoveryEvents(' + args + ', false, false)'));
  assert.ok(!disabled.some(event => event.kind === "http_fetch:request"));
  assert.ok(disabled.some(event => event.kind === "procedural_how_to:service_disabled"
    && event.payload === "wikihow"));
});

const RUSSIAN_REQUEST = "Как хорошо покакать?";
const RUSSIAN_SLUG = encodeURIComponent("хорошо-покакать");
const RUSSIAN_ARTICLE = `https://ru.wikihow.com/${RUSSIAN_SLUG}`;
const RUSSIAN_API = `https://ru.wikihow.com/api.php?action=parse&page=${RUSSIAN_SLUG}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
const WAYBACK_INDEX = `https://archive.org/wayback/available?url=${encodeURIComponent(RUSSIAN_ARTICLE)}`;
const SNAPSHOT = "20241222160007";
const WAYBACK_REPLAY = `https://web.archive.org/web/${SNAPSHOT}id_/${RUSSIAN_ARTICLE}`;
const RUSSIAN_CAPTURE = readFileSync(path.join(REPO_ROOT, "rust/tests/fixtures/issue-1396/wiki-how-ru.json"), "utf8");
const WIKIHOW_STEPS = JSON.parse(RUSSIAN_CAPTURE).parse.text["*"];

function archiveIndex(article) {
  return JSON.stringify({ archived_snapshots: { closest: {
    available: true, status: "200", timestamp: SNAPSHOT,
    url: `http://web.archive.org/web/${SNAPSHOT}/${article}`,
  } } });
}

function answerRussian(context, preferences = {}, prompt = RUSSIAN_REQUEST) {
  return evaluate(context, `tryProceduralHowTo(${JSON.stringify(prompt)}, "ru", ${JSON.stringify(preferences)})`);
}

test("an unregistered Russian how-to uses the localized CORS API and quotes its ordered steps", async () => {
  const requested = [];
  const context = await bootWorker(new Map([[RUSSIAN_API, { body: RUSSIAN_CAPTURE }]]), requested);
  const answer = await answerRussian(context);
  assert.match(answer.content, /## Как хорошо покакать/u);
  assert.match(answer.content, /1\. Не откладывайте/u);
  assert.equal(answer.guide.steps.length, 2);
  assert.deepEqual(new Set(answer.guide.steps.map((step) => step.sourceUrl)), new Set([RUSSIAN_API]));
  assert.ok(requested.includes(RUSSIAN_API));
  assert.ok(!requested.includes(WAYBACK_INDEX));
});

test("the browser uses a matching archival capture for the short Russian question after a live request fails", async () => {
  const requested = [];
  const body = `<link rel="canonical" href="${RUSSIAN_ARTICLE}"><ul><li>This unrelated navigation entry is long enough to look like a step.</li></ul>${WIKIHOW_STEPS}`;
  const captures = new Map([
    [WAYBACK_INDEX, { body: archiveIndex(RUSSIAN_ARTICLE) }],
    [WAYBACK_REPLAY, { body }],
  ]);
  const answer = await answerRussian(await bootWorker(captures, requested), {}, "Как какать?");
  assert.deepEqual(requested.filter((url) => [RUSSIAN_API, WAYBACK_INDEX, WAYBACK_REPLAY].includes(url)),
    [RUSSIAN_API, WAYBACK_INDEX, WAYBACK_REPLAY]);
  assert.match(answer.content, /1\. Не откладывайте/u);
  assert.doesNotMatch(answer.content, /unrelated navigation entry/u);
  for (const step of answer.guide.steps) {
    assert.equal(step.sourceUrl, WAYBACK_REPLAY);
    assert.equal(step.sha256, createHash("sha256").update(body).digest("hex"));
  }
});

test("a mismatched parse or archive cannot become numbered instructions", async () => {
  const requested = [];
  const captures = new Map([
    [RUSSIAN_API, { body: JSON.stringify({ parse: { title: "Tió de Nadal", text: { "*": WIKIHOW_STEPS } } }) }],
    [WAYBACK_INDEX, { body: archiveIndex("https://ru.wikihow.com/another-page") }],
  ]);
  const context = await bootWorker(captures, requested);
  evaluate(context, `runWebSearchQuery = async () => ({
    content: "Tió de Nadal: unrelated ranked Wikipedia fragment",
    confidence: 0.8, evidence: [], diagnostics: { fused: [{ title: "Tió de Nadal" }] },
  })`);
  const answer = await answerRussian(context);
  assert.doesNotMatch(answer.content, /Tió de Nadal|1\. Не откладывайте/u);
  assert.ok(answer.evidence.some((line) => line.includes("wayback:no_matching_snapshot")));
  assert.ok(!requested.some((url) => url.startsWith("https://web.archive.org/")));
  assert.ok(answer.evidence.some((line) => line.startsWith("how_to:insufficient_evidence")));
});

test("the browser tries the web capture component when the archive is unavailable", async () => {
  const context = await bootWorker(new Map());
  const body = `<link rel="canonical" href="${RUSSIAN_ARTICLE}">${WIKIHOW_STEPS}`;
  evaluate(context, `self.FormalAIWebSearchComponent = {
    fetchUrl: async (url) => url === ${JSON.stringify(RUSSIAN_ARTICLE)}
      ? { ok: true, text: async () => ${JSON.stringify(body)} } : { ok: false, status: 404 },
  }`);
  const answer = await answerRussian(context);
  assert.equal(answer.guide.steps.length, 2);
  assert.ok(answer.guide.steps.every((step) => step.sourceUrl === RUSSIAN_ARTICLE));
});

test("offline and wikiHow opt-out prevent both live and archived wikiHow requests", async () => {
  const requested = [];
  const context = await bootWorker(new Map(), requested);
  await answerRussian(context, { offline: true });
  assert.equal(requested.length, 0);
  await answerRussian(context, { externalServiceWikihow: false });
  assert.ok(!requested.some((url) => url.includes("wikihow.com") || url.includes("/wayback/available")));
});
