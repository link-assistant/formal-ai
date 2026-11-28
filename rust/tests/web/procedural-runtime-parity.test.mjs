import { readdirSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";
import { specificationCases } from "../../../scripts/lib/rust-specification-cases.mjs";
import { executeTypedProgram } from "../../../scripts/lib/rust-specification-programs.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { bookPage, bookSearch, procedureBooks, procedureBookCaptures } from "./support/procedure-publisher-captures.mjs";
import { everydayMethodReply, wikiHowArticleCapture, wikiHowMethodsCapture } from "./support/wiki-how-method-capture.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const CACHE_DIR = path.join(REPO_ROOT, "rust/tests/fixtures/issue-991/source-cache");

test("ranked Wikibooks procedures verify the page title, required words, and procedure section", async () => {
  const requested = [];
  const captures = new Map();
  for (const [index, book] of procedureBooks.entries()) {
    for (const [url, body] of procedureBookCaptures(book, index === 0)) captures.set(url, { body });
  }
  const pancakes = procedureBooks[1];
  const qualified = "prepare pancakes north american without milk";
  captures.set(bookPage("Prepare Pancakes North American Without Milk"), captures.get(pancakes.page));
  captures.set(bookSearch(qualified), captures.get(pancakes.search));
  captures.set(bookSearch("pancakes north american without milk", "&srnamespace=102"), captures.get(pancakes.scoped));
  const context = await bootWorker(captures, requested);
  const run = async (task, extra = {}) => {
    context.__bookPrompt = "How to " + task + "?";
    context.__bookPreferences = { externalServiceWikihow: false, ...extra };
    return plain(await evaluate(context, 'tryProceduralHowTo(__bookPrompt, "en", __bookPreferences)'));
  };
  for (const book of procedureBooks) {
    const start = requested.length;
    const result = await run(book.task);
    assert.ok(result.guide?.steps.length >= 2, result.content);
    assert.match(result.guide.steps[0].text, book.firstStep);
    assert.deepEqual(result.guide.steps.map(step => step.sourceUrl), result.guide.steps.map(() => book.page));
    assert.ok(result.guide.steps.every(step =>
      step.sha256 === createHash("sha256").update(captures.get(book.page).body).digest("hex")));
    assert.deepEqual(requested.slice(start).filter(url => new URL(url).hostname === "en.wikibooks.org"),
      [book.direct, book.search, book.scoped, book.page]);
    assert.doesNotMatch(result.content, /unrelated instruction/u);
  }
  const rejected = await run(qualified);
  assert.equal(rejected.guide, undefined, rejected.content);
  assert.ok(!rejected.evidence.includes("source:" + pancakes.page));
  const count = requested.length;
  const offline = await run(procedureBooks[0].task, { offline: true });
  assert.equal(offline.guide.steps[0].sourceUrl, procedureBooks[0].page);
  assert.equal(requested.length, count);
  const disabled = await run(procedureBooks[0].task, { offline: true, externalServiceMediawikiFamily: false });
  assert.equal(disabled.guide, undefined);
  assert.ok(!disabled.evidence.some(line => line === "source:" + procedureBooks[0].page));
});
const originals = [
  ["greeting_prefixed_russian_connect_how_to_composes_procedure", 7, 11],
  ["procedural_elaboration_followup_rebinds_to_prior_how_to", 4, 6],
  ["procedural_elaboration_followup_covers_supported_languages", 4, 16],
];
for (const [name, macroCount, executedCount] of originals) {
  test("native procedural assertions replay on JavaScript: " + name, async () => {
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
      const status = capture.status ?? 200;
      return Promise.resolve({ ok: status === 200, status, text: () => Promise.resolve(capture.body) });
    },
  });
  await evaluate(context, "loadSeed()");
  return context;
}

test("an unverified reply retains transport encoding and source-routing evidence", async () => {
  const requested = [];
  const context = await bootWorker(new Map(), requested);
  const result = plain(await evaluate(context, 'tryProceduralHowTo("как подключить redis к ruby", "ru", {})'));
  assert.equal(result.intent, "procedural_how_to");
  assert.equal(result.content, "Не удалось найти проверенные инструкции по запросу «подключить redis к ruby». Если у вас есть ссылка на руководство, пришлите её.");
  assert.ok(requested.some(url => url.includes("page=%D0%BF")));
  assert.ok(result.solverEvents.some(event => event.kind === "procedural_how_to:request"
    && event.payload === "подключить redis к ruby"));
  assert.ok(result.solverEvents.some(event => event.kind === "web_search:fusion_planned"));
  assert.ok(!result.solverEvents.some(event => /(?:capture|source):accepted/u.test(event.kind)));
});

test("an unverified Spanish reply gives the localized evidence gap", async () => {
  const context = await bootWorker(new Map());
  const answer = await evaluate(context, 'tryProceduralHowTo("Cómo puedo preparar té?", "es", { offline: true })');
  assert.equal(answer.content, "No pude verificar los pasos para «preparar té». Si tienes una guía, comparte el enlace y puedo ayudarte a seguirla.");
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

test("verified steps and citations reach the reply with full provenance in evidence", async () => {
  const captures = loadCaptures();
  const requested = [];
  const context = await bootWorker(captures, requested);
  const result = plain(await evaluate(context, 'tryProceduralHowTo("how to make pancakes", "en", {})'));
  assert.equal(result.intent, "procedural_how_to");
  assert.ok(result.guide.steps.length >= 2);
  assert.match(result.content, /Sources/u);
  assert.doesNotMatch(result.content, /sha256|max_depth=|page\(s\)/u);
  for (const step of result.guide.steps) {
    assert.equal(captures.get(step.sourceUrl)?.sha256, step.sha256);
    assert.ok(requested.includes(step.sourceUrl));
    assert.ok(result.evidence.some(line => line.startsWith("how_to:step ") && line.includes(step.sha256)));
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
const wikiSearch = (task) => `https://ru.wikihow.com/api.php?action=query&list=search&srsearch=${encodeURIComponent(task)}&srlimit=5&format=json&origin=*`;
const wikiParse = (title) => RUSSIAN_API.replace(RUSSIAN_SLUG, encodeURIComponent(title));
const publisherSearch = (query) => `https://ru.wikihow.com/Special:LSearch?fulltext=Search&search=${encodeURIComponent(query)}`;
const linkResult = (title, origin = "https://ru.wikihow.com") =>
  `<a class="result_link" href="${origin}/${encodeURIComponent(title.replaceAll(" ", "-"))}">Result</a>`;
const missingTitle = { body: JSON.stringify({ error: { code: "missingtitle" } }) };
const searchCapture = (...titles) => ({ body: JSON.stringify({ query: { search: titles.map(title => ({ title })) } }) });
const RUSSIAN_SHORT_TASK = "какать";
const RUSSIAN_DISCOVERED_TITLE = "хорошо покакать";
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

test("a contributing stale procedure retains its capture-age warning", async () => {
  const context = await bootWorker(new Map([[RUSSIAN_API, { body: RUSSIAN_CAPTURE }]]));
  const initial = plain(await answerRussian(context));
  assert.equal(initial.guide.steps.length, 2, initial.content);
  const retained = await evaluate(context, `
    for (const [url, capture] of sourceWalkCaptureCache) {
      sourceWalkCaptureCache.set(url, { ...capture, fetchedAt: "1" });
    }
    sourceWalkCaptureCache.size
  `);
  assert.ok(retained > 0);
  const stale = plain(await answerRussian(context, { offline: true }));
  assert.equal(stale.guide.steps.length, 2, stale.content);
  assert.ok(stale.guide.steps.every(step => step.fetchedAt === "1"));
  const outcome = stale.guide.outcomes.find(row => row.sourceId === "wikihow");
  assert.equal(outcome.status, "contributed");
  assert.match(outcome.detail, /stale_capture age_seconds=/u);
  assert.ok(outcome.detail.includes(RUSSIAN_API), outcome.detail);
});

test("a recovered procedure reports the accepted article after rejecting an unrelated capture", async () => {
  const unrelated = JSON.parse(RUSSIAN_CAPTURE);
  unrelated.parse.title = "Домашние растения";
  const title = "хорошо покакать";
  const matching = wikiParse(title);
  const captures = new Map([
    [RUSSIAN_API, { body: JSON.stringify(unrelated) }],
    [wikiSearch(title), searchCapture(title)],
    [matching, { body: RUSSIAN_CAPTURE }],
  ]);
  const answer = await answerRussian(await bootWorker(captures));
  assert.equal(answer.guide.steps.length, 2, answer.content);
  assert.ok(answer.guide.steps.every(step => step.sourceUrl === matching));
  assert.ok(!answer.content.includes(RUSSIAN_API), answer.content);
  const outcome = answer.guide.outcomes.find(row => row.sourceId === "wikihow");
  assert.equal(outcome.status, "contributed");
  assert.ok(outcome.detail.includes(matching), outcome.detail);
});

test("the browser uses a matching archival capture for the short Russian question after a live request fails", async () => {
  const requested = [];
  const body = `<link rel="canonical" href="${RUSSIAN_ARTICLE}"><ul><li>This unrelated navigation entry is long enough to look like a step.</li></ul>${WIKIHOW_STEPS}`;
  const captures = new Map([
    [wikiSearch(RUSSIAN_SHORT_TASK), searchCapture("Домашние растения", RUSSIAN_DISCOVERED_TITLE)],
    [WAYBACK_INDEX, { body: archiveIndex(RUSSIAN_ARTICLE) }],
    [WAYBACK_REPLAY, { body }],
  ]);
  const answer = await answerRussian(await bootWorker(captures, requested), {}, "Как какать?");
  assert.deepEqual(requested.filter((url) => [wikiParse(RUSSIAN_SHORT_TASK), wikiSearch(RUSSIAN_SHORT_TASK),
    wikiParse(RUSSIAN_DISCOVERED_TITLE), WAYBACK_INDEX, WAYBACK_REPLAY].includes(url)),
    [wikiParse(RUSSIAN_SHORT_TASK), wikiSearch(RUSSIAN_SHORT_TASK),
      wikiParse(RUSSIAN_DISCOVERED_TITLE), WAYBACK_INDEX, WAYBACK_REPLAY]);
  assert.equal(answer.guide.task, RUSSIAN_SHORT_TASK);
  assert.match(answer.content, /1\. Не откладывайте/u);
  assert.doesNotMatch(answer.content, /unrelated navigation entry/u);
  for (const step of answer.guide.steps) {
    assert.equal(step.sourceUrl, WAYBACK_REPLAY);
    assert.equal(step.sha256, createHash("sha256").update(body).digest("hex"));
  }
});

test("publisher search recovers the reported article when MediaWiki search returns 500", async () => {
  const requested = [];
  const search = publisherSearch("какать покакать");
  const exact = wikiParse(RUSSIAN_SHORT_TASK);
  const fullText = wikiSearch(RUSSIAN_SHORT_TASK);
  const article = wikiParse(RUSSIAN_DISCOVERED_TITLE);
  const body = wikiHowMethodsCapture();
  const captures = new Map([
    [exact, missingTitle],
    [fullText, { status: 500, body: "" }],
    [search, { body: linkResult("Домашние растения") + linkResult(RUSSIAN_DISCOVERED_TITLE) }],
    [article, { body }],
  ]);
  const context = await bootWorker(captures, requested);
  const run = (preferences) => answerRussian(context, preferences, "Как какать?");
  const answer = await run({});
  assert.deepEqual(requested.filter(url => new URL(url).hostname === "ru.wikihow.com"), [exact, fullText, search, article]);
  assert.equal(answer.content, everydayMethodReply(RUSSIAN_ARTICLE));
  assert.ok(answer.guide.steps.every(step => step.sourceUrl === article && step.depth === 2
    && step.sha256 === createHash("sha256").update(body).digest("hex")));
  const count = requested.length;
  const offline = await run({ offline: true });
  assert.equal(offline.content, answer.content);
  assert.ok(offline.guide.steps.every(step => step.cached));
  const disabled = await run({ offline: true, externalServiceWikihow: false });
  assert.equal(disabled.guide, undefined);
  assert.equal(requested.length, count);
});

test("publisher article recovery verifies canonical identity and replays the capture offline", { timeout: 30_000 }, async () => {
  const requested = [];
  const search = publisherSearch("какать покакать");
  const api = wikiParse(RUSSIAN_DISCOVERED_TITLE);
  const body = wikiHowArticleCapture(RUSSIAN_ARTICLE);
  const captures = new Map([
    [wikiParse(RUSSIAN_SHORT_TASK), missingTitle],
    [wikiSearch(RUSSIAN_SHORT_TASK), { status: 500, body: "" }],
    [search, { body: linkResult(RUSSIAN_DISCOVERED_TITLE) }],
    [api, { status: 503, body: "" }],
    [RUSSIAN_ARTICLE, { body }],
  ]);
  const context = await bootWorker(captures, requested);
  const ask = (worker, preferences = {}) => answerRussian(worker, preferences, "Как какать?");
  const answer = await ask(context);
  assert.equal(answer.content, everydayMethodReply(RUSSIAN_ARTICLE));
  assert.deepEqual(requested.filter(url => new URL(url).hostname === "ru.wikihow.com"), [
    wikiParse(RUSSIAN_SHORT_TASK), wikiSearch(RUSSIAN_SHORT_TASK), search, api, RUSSIAN_ARTICLE,
  ]);
  const digest = createHash("sha256").update(body).digest("hex");
  assert.ok(answer.solverEvents.some(event => event.kind === "how_to:source"
    && event.payload.includes("publisher_parse_failed") && event.payload.includes("503")));
  const steps = answer.solverEvents.filter(event => event.kind === "how_to:step");
  assert.equal(steps.length, 2);
  assert.ok(steps.every(event => event.payload.includes(RUSSIAN_ARTICLE) && event.payload.includes(digest)));
  const count = requested.length;
  const offline = await ask(context, { offline: true });
  assert.equal(offline.content, answer.content);
  assert.ok(offline.guide.steps.every(step => step.cached));
  assert.equal(requested.length, count);
  captures.set(RUSSIAN_ARTICLE, { body: wikiHowArticleCapture("https://ru.wikihow.com/unrelated") });
  const rejected = await ask(await bootWorker(captures));
  assert.match(rejected.content, /Не удалось найти проверенные инструкции/u);
  assert.ok(rejected.solverEvents.some(event => event.kind === "how_to:source"
    && event.payload.includes("publisher_article_failed")));
});

test("canonical wikiHow recovery renders the other registered languages", { timeout: 30_000 }, async () => {
  const cases = [
    { language: "en", host: "www", task: "swim", slug: "Swim", heading: "### Sources",
      steps: ["Practice putting your face in the water and breathing out gently.", "Hold the side of the pool while you practice kicking."] },
    { language: "hi", host: "hi", task: "रोटी बनाएं", slug: "घर-पर-रोटी-बनाएं", heading: "### स्रोत",
      steps: ["आटे में थोड़ा थोड़ा पानी डालकर मुलायम आटा गूंध लें और उसे ढक कर रख दें।", "आटे की एक लोई बेल लें और उसे गरम तवे पर दोनों तरफ से अच्छी तरह सेंकें।"] },
    { language: "zh", host: "zh", task: "烤面包", slug: "在家-烤面包", heading: "### 来源",
      steps: ["先把面粉和水倒进干净的碗里，用搅拌匙拌匀，然后用手轻轻揉成光滑的面团，盖起来放在温暖的地方发酵。",
        "面团发好后把它放进烤盘，等烤箱预热好再把烤盘放进去，烤至表面呈金黄色，然后取出并放在架子上冷却。"] },
    { language: "es", host: "es", task: "hornear pan", slug: "hornear-pan-en-casa", heading: "### Fuentes",
      steps: ["Mezcla la harina con el agua hasta obtener una masa uniforme.", "Hornea el pan hasta que la corteza quede firme y dorada."] },
  ];
  const captures = new Map();
  for (const item of cases) {
    const origin = `https://${item.host}.wikihow.com`;
    item.article = `${origin}/${encodeURIComponent(item.slug)}`;
    const search = `${origin}/Special:LSearch?fulltext=Search&search=${encodeURIComponent(item.task)}`;
    captures.set(search, { body: linkResult(item.slug, origin) });
    captures.set(item.article, { body: `<link rel="canonical" href="${item.article}">
      <ol class="steps_list_2">${item.steps.map(step => `<li>${step}</li>`).join("")}</ol>` });
  }
  const context = await bootWorker(captures);
  for (const { language, task, heading, article, steps } of cases) {
    const guide = await evaluate(context, `synthesizeHowToGuide(${JSON.stringify(task)}, {}, undefined, undefined, "${language}")`);
    assert.deepEqual(plain(guide.steps.map(step => step.text)), steps, language);
    assert.ok(guide.steps.every(step => step.sourceUrl === article && step.sourceId === "wikihow"), language);
    context.__localizedGuide = guide;
    const markdown = evaluate(context, `howToGuideMarkdown(__localizedGuide, "${language}", false)`);
    assert.ok(markdown.includes(heading), markdown);
    assert.ok(markdown.includes(article), markdown);
    const offline = await evaluate(context, `synthesizeHowToGuide(${JSON.stringify(task)}, { offline: true }, undefined, undefined, "${language}")`);
    assert.deepEqual(plain(offline.steps.map(step => step.text)), steps, language);
  }
});

test("publisher recovery works through the capture component when a browser cannot read the site", async () => {
  const context = await bootWorker(new Map());
  const search = publisherSearch("какать покакать");
  const article = wikiParse(RUSSIAN_DISCOVERED_TITLE);
  const captures = { [search]: linkResult(RUSSIAN_DISCOVERED_TITLE), [article]: wikiHowMethodsCapture() };
  evaluate(context, `self.FormalAIWebSearchComponent = { fetchUrl: async url => {
    const body = ${JSON.stringify(captures)}[url];
    return body ? { ok: true, text: async () => body } : { ok: false, status: 404 };
  } }`);
  const answer = await answerRussian(context, {}, "Как какать?");
  assert.match(answer.content, /1\. Не откладывайте[^\n]+\n2\. Сядьте/u);
  const offline = await answerRussian(context, { offline: true }, "Как какать?");
  assert.equal(offline.content, answer.content);
  assert.ok(offline.guide.steps.every(step => step.cached));
});

test("an explicit scope selects its method over the everyday one", async () => {
  const task = "какать быстро";
  const url = wikiParse(task.replaceAll(" ", "-"));
  const context = await bootWorker(new Map([[url, { body: wikiHowMethodsCapture(task) }]]));
  const answer = await answerRussian(context, {}, "Как какать быстро?");
  assert.match(answer.content, /1\. Сначала выберите[^\n]+\n2\. Уточните/u);
  assert.doesNotMatch(answer.content, /Не откладывайте/u);
});

test("publisher search discovers a held-out title and rejects foreign, negated and off-topic suggestions", async () => {
  const task = "читать вслух";
  const title = "Как почитать вслух с друзьями";
  const irrelevant = "Как читать по диагонали";
  const negated = "Как не читать вслух";
  const requested = [];
  const steps = "<ol class=\"steps_list_2\"><li>Выберите короткий текст и сначала прочтите его про себя.</li><li>Теперь прочтите текст группе, делая паузы между предложениями.</li></ol>";
  const captures = new Map([
    [wikiParse(task.replaceAll(" ", "-")), missingTitle],
    [wikiSearch(task), { status: 500, body: "" }],
    [publisherSearch(task + " почитать"), { body: linkResult(irrelevant) + linkResult(negated)
      + linkResult("Как почитать вслух на сцене", "https://foreign.invalid") + linkResult(title) }],
    [wikiParse(title), { body: JSON.stringify({ parse: { title, text: { "*": steps } } }) }],
  ]);
  const answer = await answerRussian(await bootWorker(captures, requested), {}, "Как читать вслух?");
  assert.equal(answer.guide.task, task);
  assert.match(answer.content, /1\. Выберите короткий текст/u);
  assert.ok(answer.guide.steps.every(step => step.sourceUrl === wikiParse(title)));
  for (const value of [irrelevant, negated, "Как почитать вслух на сцене"]) {
    assert.ok(!requested.includes(wikiParse(value)));
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

test("the capture component can discover and verify a source after browser transport fails", async () => {
  const context = await bootWorker(new Map());
  const body = `<link rel="canonical" href="${RUSSIAN_ARTICLE}">${WIKIHOW_STEPS}`;
  const captures = {
    [wikiSearch(RUSSIAN_SHORT_TASK)]: searchCapture("Домашние растения", RUSSIAN_DISCOVERED_TITLE).body,
    [RUSSIAN_ARTICLE]: body,
  };
  evaluate(context, `
    self.componentRequests = [];
    self.FormalAIWebSearchComponent = {
      fetchUrl: async (url) => {
        self.componentRequests.push(url);
        const body = ${JSON.stringify(captures)}[url];
        return body ? { ok: true, text: async () => body } : { ok: false, status: 404 };
      },
    };`);
  const answer = await answerRussian(context, {}, "Как какать?");
  assert.deepEqual(plain(evaluate(context, "self.componentRequests")), [
    wikiSearch(RUSSIAN_SHORT_TASK), RUSSIAN_ARTICLE,
  ]);
  assert.equal(answer.guide.task, RUSSIAN_SHORT_TASK);
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

test("the network opt-out covers provider recovery, indexed search and practical help", async () => {
  const requested = [];
  const context = await bootWorker(new Map(), requested);
  context.FormalAIWebSearchComponent = {
    fetchUrl(url) {
      requested.push(String(url));
      throw new Error("network disabled");
    },
  };
  for (const prompt of ["How to install Raku on Alpine Linux?", "How to buy extra baggage on IndiGo?"]) {
    context.__howToPrompt = prompt;
    const answer = plain(await evaluate(context, 'tryProceduralHowTo(__howToPrompt, "en", { allowNetwork: false })'));
    assert.equal(answer.intent, "procedural_how_to");
    assert.ok(answer.evidence.includes("how_to:insufficient_evidence steps=0 required=2"));
  }
  const help = plain(await evaluate(context, 'tryPracticalHelp("I need to poop", "en", { allowNetwork: false })'));
  assert.ok(help?.evidence.includes("practical_help:source_mode:unavailable"));
  const fact = plain(await evaluate(context, 'solve("What is the capital of Portugal?", [], { allowNetwork: false })'));
  assert.doesNotMatch(fact.content, /Lisbon/i);
  const unknown = plain(await evaluate(context, 'solve("What is the zigotted fliver?", [], { allowNetwork: false })'));
  assert.notEqual(unknown.intent, "explanation_research");
  assert.deepEqual(requested, []);
});
