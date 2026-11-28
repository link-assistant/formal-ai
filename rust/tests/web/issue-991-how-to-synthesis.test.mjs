// Issue #991: the browser worker must run the *same* bounded multi-source
// synthesis contract as the Rust solver.
//
// These tests boot the real worker (`js/worker/formal_ai_worker.js`, which loads
// `js/worker/formal_ai_worker_how_to_guide.js`) with the real seed registry and
// replay the *same* committed real-service captures under
// `rust/tests/fixtures/issue-991/` that `rust/tests/unit/issue_991_how_to_synthesis.rs`
// replays. Nothing here is a hand-written fixture: every byte the worker parses
// came from wikiHow, Stack Exchange, or a Wikimedia wiki through the production
// fetch path, and its sha256 is recorded in `capture-manifest.lino`.

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";
import { indexedProcedure, indexedProcedureBody, unrelatedProcedure } from "./support/indexed-procedure-capture.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const FIXTURE_DIR = path.join(REPO_ROOT, "rust/tests/fixtures/issue-991");
const CACHE_DIR = path.join(FIXTURE_DIR, "source-cache");

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

/** Synthesise a guide inside the worker realm and copy it out. */
async function synthesize(context, task, preferences = {}) {
  context.__howToTask = task;
  context.__howToPreferences = preferences;
  const guide = await evaluate(
    context,
    "synthesizeHowToGuide(__howToTask, __howToPreferences)",
  );
  return plain(guide);
}

test("the browser worker reads the how-to services out of the seed registry", async () => {
  const context = await bootWorker(loadCaptures());
  const registry = plain(evaluate(context, "howToSourceRegistry()"));
  assert.ok(registry.length > 0, "the seed registry must reach the worker");
  const wikihow = registry.find((record) => record.id === "wikihow");
  assert.equal(wikihow.howToRole, "primary");
  assert.equal(wikihow.settingsKey, "externalServiceWikihow");
  assert.equal(wikihow.licenseName, "CC BY-NC-SA 3.0");

  // Every service the registry marks procedural is eligible; nothing is
  // hardcoded, so enabling a service in the seed is enough for it to appear.
  const eligible = plain(evaluate(context, 'howToSelectSources("make pancakes", {})')).map(
    (record) => record.id,
  );
  assert.ok(eligible.includes("wikihow"), `expected wikihow among ${eligible.join(", ")}`);
  assert.ok(eligible.length <= 4, "the max_services bound must hold");
});

test("settings opt-outs stay authoritative in the browser", async () => {
  const context = await bootWorker(loadCaptures());
  const selected = plain(
    evaluate(context, 'howToSelectSources("make pancakes", { externalServiceWikihow: false })'),
  ).map((record) => record.id);
  assert.ok(!selected.includes("wikihow"), "a disabled service must not be consulted");

  const skipped = plain(
    evaluate(context, 'howToSkippedSources("make pancakes", { externalServiceWikihow: false })'),
  );
  const disabled = skipped.find((outcome) => outcome.sourceId === "wikihow");
  assert.equal(disabled.status, "disabled");
  assert.equal(disabled.detail, "externalServiceWikihow");
});

test("the committed captures synthesise a provenance-carrying guide", async () => {
  const captures = loadCaptures();
  const requested = [];
  const context = await bootWorker(captures, requested);
  const guide = await synthesize(context, "make pancakes");

  assert.ok(guide.steps.length >= 2, `expected a procedure, got ${guide.steps.length} step(s)`);
  assert.ok(evaluate(context, "howToGuideIsSufficient")(guide) !== false);
  for (const step of guide.steps) {
    // Exact provenance on every accepted step, which is what makes the guide
    // auditable: the URL, the digest of the bytes, when they were fetched, the
    // tier they were weighed at, and the license they are quoted under.
    assert.match(step.sha256, /^[0-9a-f]{64}$/u);
    assert.equal(step.sha256, captures.get(step.sourceUrl).sha256);
    assert.ok(step.sourceUrl.startsWith("http"));
    assert.ok(Number.parseInt(step.fetchedAt, 10) > 0);
    assert.ok(step.licenseName.length > 0 && step.licenseUrl.startsWith("http"));
    assert.ok(step.depth <= guide.bounds.maxDepth, "steps stay inside the depth bound");
  }

  // The bounds are declared, not incidental.
  const perService = new Map();
  for (const url of requested) {
    const host = new URL(url).host;
    perService.set(host, (perService.get(host) || 0) + 1);
  }
  for (const [host, count] of perService) {
    assert.ok(count <= guide.bounds.maxPagesPerService, `${host} fetched ${count} page(s)`);
  }
  // `maxSteps` became `maxItems` when issue #1138 plan 01 L2 made one bounds
  // vocabulary for every need kind; the worker moved and this line did not, so
  // the comparison was against `undefined` and passed nothing.
  assert.ok(guide.steps.length <= guide.bounds.maxItems);

  const markdown = evaluate(context, "howToGuideMarkdown")(guide);
  assert.match(markdown, /## How to make pancakes/u);
  assert.match(markdown, /### Sources/u);
});

test("insufficient evidence is reported instead of invented", async () => {
  const context = await bootWorker(loadCaptures());
  const guide = await synthesize(context, "build a nonexistent quantum flux capacitor");
  assert.ok(guide.steps.length < 2, "no service documents this task");
  assert.equal(evaluate(context, "howToGuideIsSufficient")(guide), false);
  const events = plain(evaluate(context, "howToGuideEvents")(guide));
  assert.ok(events.some(({ kind }) => kind === "how_to:insufficient_evidence"));
  assert.match(evaluate(context, "howToGuideMarkdown")(guide), /Insufficient evidence/u);

  // The refusal is still explained per service rather than silent.
  assert.ok(guide.outcomes.length > 0);
  assert.ok(guide.outcomes.some((outcome) => outcome.detail.includes("no_relevant_result")));
});

test("the browser follows ranked URLs and takes steps from the matching publisher document", async () => {
  const requested = [];
  const privateUrl = "https://127.0.0.1/guide";
  const captures = new Map([
    [indexedProcedure.page, { body: indexedProcedureBody() }],
    [unrelatedProcedure.page, { body: unrelatedProcedure.body }],
  ]);
  const context = await bootWorker(captures, requested);
  context.__indexedResults = [privateUrl, unrelatedProcedure.page, indexedProcedure.page].map(url => ({
    url, title: indexedProcedure.task, excerpt: "A search excerpt is not an instruction.",
  }));
  context.__indexedPrompt = `How to ${indexedProcedure.task}?`;
  evaluate(context, "runWebSearchQuery = async () => ({ evidence: [], diagnostics: { fused: __indexedResults } })");
  const answer = plain(await evaluate(context, "tryProceduralHowTo(__indexedPrompt, 'en')"));
  assert.equal(answer.intent, "procedural_how_to");
  assert.match(answer.guide.steps[0].text, indexedProcedure.firstStep);
  assert.match(answer.content, indexedProcedure.finalAction);
  assert.doesNotMatch(answer.content, /archive it|search excerpt/iu);
  assert.ok(answer.guide.steps.every(step => step.sourceUrl === indexedProcedure.page
    && step.sha256 === indexedProcedure.sha256));
  assert.ok(requested.includes(unrelatedProcedure.page));
  assert.ok(requested.includes(indexedProcedure.page));
  assert.ok(!requested.includes(privateUrl));
  requested.length = 0;
  const replay = plain(await evaluate(context, "tryProceduralHowTo(__indexedPrompt, 'en', { offline: true })"));
  assert.match(replay.guide.steps[0].text, indexedProcedure.firstStep);
  assert.equal(requested.length, 0);
});

test("distinct ordered procedures under one matching heading require a narrower task", async () => {
  const context = await bootWorker(new Map());
  context.__ambiguousPage = `<main><h1>Assembling a garden bench</h1>
    <h2>Metal kit</h2><ol><li>Position the feet.</li><li>Bolt on the seat.</li></ol>
    <h2>Wooden kit</h2><ol><li>Oil the planks.</li><li>Nail on the seat.</li></ol></main>`;
  const steps = (task) => plain(evaluate(context, "indexedProcedureSteps")(context.__ambiguousPage, task, "en", 5));
  assert.deepEqual(steps("assemble a garden bench"), []);
  assert.deepEqual(steps("assemble a garden bench from a metal kit"), ["Position the feet.", "Bolt on the seat."]);
});

test("per-service accessibility is remembered for at least seven days", async () => {
  const context = await bootWorker(loadCaptures());
  const ttl = evaluate(context, "SERVICE_ACCESSIBILITY_TTL_SECONDS");
  assert.ok(ttl >= 7 * 24 * 60 * 60, `TTL must be at least seven days, got ${ttl}`);

  evaluate(context, 'howToInvalidateAllServices(); howToObserveService("wikihow", "unreachable", "http_503", 1000)');
  // Inside the TTL the failure is authoritative, so the service is skipped.
  assert.equal(evaluate(context, 'howToServiceKnownUnreachable("wikihow", 1000 + 6 * 86400)'), true);
  assert.equal(evaluate(context, 'howToServiceNeedsRefresh("wikihow", 1000 + 6 * 86400)'), false);
  // Past the TTL the record must be refreshed rather than trusted.
  assert.equal(evaluate(context, 'howToServiceNeedsRefresh("wikihow", 1000 + 8 * 86400)'), true);
  assert.equal(evaluate(context, 'howToServiceKnownUnreachable("wikihow", 1000 + 8 * 86400)'), false);
  // Explicit invalidation forgets it immediately.
  assert.equal(evaluate(context, 'howToInvalidateService("wikihow")'), true);
  assert.equal(evaluate(context, 'howToServiceNeedsRefresh("wikihow", 1000)'), true);

  const lino = evaluate(
    context,
    'howToObserveService("wikihow", "reachable", "captured", 42), howToServiceAccessibilityLino()',
  );
  assert.match(lino, /^service_accessibility\n/u);
  assert.match(lino, /ttl_seconds 604800/u);
});

test("copies and contradictions follow the issue #709 source-tier policy", async () => {
  const context = await bootWorker(loadCaptures());
  // Identical bytes under two URLs: the higher tier keeps them, the copy is
  // reported and contributes nothing.
  const copied = plain(
    evaluate(
      context,
      `(() => {
         const guide = { copies: [], conflicts: [] };
         const steps = [
           { text: "a", sha256: "d", sourceUrl: "https://original/", sourceId: "wikinews", tier: "original_journalism" },
           { text: "a", sha256: "d", sourceUrl: "https://mirror/", sourceId: "mirror", tier: "unoriginal" },
         ];
         const kept = howToApplyCopiedSourcePolicy(steps, guide);
         return { kept: kept.map((step) => step.sourceUrl), copies: guide.copies };
       })()`,
    ),
  );
  assert.deepEqual(copied.copies, ["https://mirror/"]);
  assert.deepEqual(copied.kept, ["https://original/"]);

  // Two services describing the same action differently is a contradiction:
  // the higher tier wins and the disagreement is recorded, not averaged.
  const conflicted = plain(
    evaluate(
      context,
      `(() => {
         const guide = { copies: [], conflicts: [] };
         const steps = [
           { text: "Heat the pan slowly over medium heat until water beads.", sha256: "x", sourceUrl: "https://low/", sourceId: "low", tier: "independent_corroboration", position: 1, depth: 0 },
           { text: "Heat the pan quickly over the highest heat you have available.", sha256: "y", sourceUrl: "https://high/", sourceId: "high", tier: "original_first_party", position: 2, depth: 0 },
         ];
         const kept = howToApplyConflictPolicy(steps, guide);
         return { kept: kept.map((step) => step.sourceId), conflicts: guide.conflicts };
       })()`,
    ),
  );
  assert.deepEqual(conflicted.kept, ["high"]);
  assert.equal(conflicted.conflicts.length, 1);
  assert.equal(conflicted.conflicts[0].keptSource, "high");
  assert.equal(conflicted.conflicts[0].droppedSource, "low");
});

test("the browser worker and the Rust solver synthesise the same guide", async () => {
  // `examples/issue_991_how_to_parity.rs` replays these captures through the
  // production Rust path and writes what it got. Requiring the browser to
  // reproduce it byte-for-byte is what makes "the same contract" checkable
  // rather than asserted: relevance, recursion, tier ordering, depth ordering,
  // step compaction, and the sufficiency threshold all have to agree.
  const expected = JSON.parse(readFileSync(path.join(FIXTURE_DIR, "expected-guides.json"), "utf8"));
  const context = await bootWorker(loadCaptures());
  for (const [task, want] of Object.entries(expected)) {
    const guide = await synthesize(context, task);
    assert.equal(
      evaluate(context, "howToGuideIsSufficient")(guide),
      want.sufficient,
      `sufficiency disagrees for "${task}"`,
    );
    assert.deepEqual(
      guide.steps.map((step) => ({
        source: step.sourceId,
        depth: step.depth,
        tier: step.tier,
        text: step.text,
      })),
      want.steps,
      `the browser guide for "${task}" differs from the Rust guide`,
    );
  }
});

test("the English procedural answer presents a complete primary procedure with its citation", async () => {
  const context = await bootWorker(loadCaptures());
  const answer = plain(
    await evaluate(context, 'tryProceduralHowTo("how to make pancakes?", "en", {})'),
  );
  assert.equal(answer.intent, "procedural_how_to");
  assert.ok(answer.evidence.includes("procedural_how_to:stage:multi_source_synthesis"));
  assert.ok(answer.evidence.some((line) => line.startsWith("how_to:bounds ")));
  assert.ok(answer.evidence.some((line) => line.startsWith("how_to:step rank=1 ")));
  assert.match(answer.content, /## How to make pancakes/u);
  assert.match(answer.content, /^1\. Crack the eggs/mu);
  assert.doesNotMatch(answer.content, /Checking if input|Valid==False|2 cups \(18oz\/510g\)/u);

  // A task with no verified steps still returns a useful reply and trace.
  const fallback = plain(
    await evaluate(
      context,
      'tryProceduralHowTo("how to build a nonexistent quantum flux capacitor?", "en", {})',
    ),
  );
  assert.equal(fallback.intent, "procedural_how_to");
  assert.ok(fallback.evidence.some((line) => line.startsWith("how_to:insufficient_evidence")));
  assert.ok(fallback.evidence.includes("procedural_how_to:stage:recursive_fetch_check"));
});

// Issue #1632 (R1): a transient wikiHow failure must not silence the priority
// how-to source for the seven-day accessibility TTL. Twin of
// rust/tests/source/source_tests/how_to_guide/production_tests.rs.
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

// Issue #1632 (R2): the degraded web-search reply names only what was used, so
// it no longer lists the unexecuted provider plan in any supported language.
const UNAVAILABLE_CASES = [
  { language: "en", marker: "No captured provider response" },
  { language: "ru", marker: "нет захваченного ответа провайдера" },
  { language: "hi", marker: "प्रदाता प्रतिक्रिया" },
  { language: "zh", marker: "已捕获提供商响应" },
  { language: "es", marker: "respuesta capturada del proveedor" },
];

test("the web-search-unavailable reply lists no unused sources in any language", { timeout: 60_000 }, async () => {
  const context = await bootWorker(new Map());
  for (const { language, marker } of UNAVAILABLE_CASES) {
    context.__language = language;
    const answer = plain(evaluate(
      context,
      'guardCannedWebSearchAnswer({ content: "Web search requested", query: "как какать" }, __language)',
    ));
    assert.ok(answer.content.includes(marker), `${language}: ${answer.content}`);
    assert.ok(answer.content.includes("как какать"), `${language} names the query`);
    for (const unused of ["DuckDuckGo", "Internet Archive", "Wikidata", "Wiktionary", "Wikinews"]) {
      assert.ok(!answer.content.includes(unused), `${language} lists unused source ${unused}: ${answer.content}`);
    }
  }
});
