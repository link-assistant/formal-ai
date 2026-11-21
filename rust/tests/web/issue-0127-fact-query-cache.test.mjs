// Issue #127 (R175, R176, R178): a recognised fact question is cached for a
// week under a per-language key, a fresh marker in any supported language
// bypasses the cache, and every pipeline step is recorded as a `fact_query:*`
// trace event. The worker's real `tryFactQuery` runs; Wikidata is a fixture
// fetch (the issue #1172 parity suite pins the live answer text itself).

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const AUSTRALIA = {
  entities: {
    Q408: {
      id: "Q408",
      labels: { en: { language: "en", value: "Australia" } },
      claims: { P36: [{ rank: "deprecated", mainsnak: { datavalue: { value: { id: "Q123" } } } }, { rank: "preferred", mainsnak: { datavalue: { value: { "entity-type": "item", id: "Q3114" }, type: "wikibase-entityid" } } }] },
    },
  },
};
const CANBERRA = { entities: { Q3114: { id: "Q3114", labels: { en: { language: "en", value: "Canberra" } } } } };

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");
const seedFetch = worker.fetch;
const wikidataRequests = [];
worker.fetch = (url, init) => {
  const text = String(url);
  if (!text.includes("wikidata.org")) return seedFetch(url, init);
  wikidataRequests.push(text);
  let body = null;
  if (text.includes("wbsearchentities")) {
    if (text.includes("search=Australia")) {
      body = { search: [{ id: "Q123", label: "Australia Day" }, { id: "Q408", label: "Australia" }] };
    } else if (text.includes("search=Springfield")) {
      body = { search: [{ id: "Q123", label: "Springfield" }, { id: "Q408", label: "Springfield" }] };
    } else if (text.includes("search=Canterbury")) {
      body = { search: [{ id: "Q123", label: "Canterbury" }, { id: "Q345", label: "Canterbury" }] };
    } else {
      body = { search: [] };
    }
  }
  else if (text.includes("ids=Q408")) body = AUSTRALIA;
  else if (text.includes("ids=Q345")) body = { entities: { Q345: { id: "Q345", claims: {} } } };
  else if (text.includes("ids=Q123")) body = { entities: { Q123: { ...AUSTRALIA.entities.Q408, id: "Q123", labels: {} } } };
  else if (text.includes("ids=Q3114")) body = CANBERRA;
  return Promise.resolve({
    ok: body !== null,
    status: body ? 200 : 404,
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
  });
};

async function call(expression) {
  await seeded;
  return plain(await evaluate(worker, expression));
}

const ask = (prompt, preferences = {}) => call(`tryFactQuery(${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}), ${JSON.stringify(preferences)})`);

test("R175: the cache keeps an answer for one week under a per-language key", async () => {
  assert.equal(await call("FACT_QUERY_TTL_MS"), 7 * 24 * 60 * 60 * 1000);
  assert.notEqual(await call('factCacheKey("capital", "Australia", "en")'), await call('factCacheKey("capital", "Australia", "ru")'));
  assert.equal(await call('factCacheKey("capital", "  Australia ", "EN")'), "capital:australia:en");
  await call('factCachePut("capital", "expired land", "en", { summary: "stale", ttlMs: -1 })');
  assert.equal(await call('factCacheGet("capital", "expired land", "en")'), null);
});

test("R175/R178: a miss resolves live and is stored; the repeat is served from the cache", async () => {
  const before = wikidataRequests.length;
  const first = await ask("What is the capital of Australia?");
  assert.equal(first.intent, "fact_query");
  for (const event of [
    "fact_query:request:What is the capital of Australia?",
    "fact_query:relation:capital",
    "fact_query:language:en",
    "fact_query:cache:check",
    "fact_query:cache:miss",
    "fact_query:wbsearchentities:resolved:Q408",
    "fact_query:label_resolve:Canberra",
    "fact_query:cache:store:capital:australia:en",
  ]) {
    assert.ok(first.trace.includes(event), `${event}\n${first.trace.join("\n")}`);
  }
  assert.ok(wikidataRequests.length > before, "the miss reached Wikidata");
  assert.equal(first.evidence.filter((entry) => /^source:http:.* sha256=[a-f0-9]{64}$/.test(entry)).length, 3);

  const requested = wikidataRequests.length;
  const second = await ask("What is the capital of Australia?");
  assert.equal(second.content, first.content);
  assert.ok(second.trace.includes("fact_query:cache:hit:runtime"), second.trace.join("\n"));
  assert.equal(wikidataRequests.length, requested, "a cached answer made a Wikidata request");
});

test("R176: a fresh marker bypasses the cache and resolves again", async () => {
  await ask("What is the capital of Australia?");
  const requested = wikidataRequests.length;
  const fresh = await ask("What is the capital of Australia? fresh");
  assert.ok(fresh.trace.includes("fact_query:force_fresh"), fresh.trace.join("\n"));
  assert.ok(fresh.trace.includes("fact_query:cache:bypass"), fresh.trace.join("\n"));
  assert.ok(wikidataRequests.length > requested, "the fresh request did not reach Wikidata");
});

test("R176: the fresh marker is recognised in every supported language", async () => {
  for (const prompt of ["Какая столица России? без кэша", "भारत की राजधानी क्या है? ताज़ा", "中国的首都是什么？刷新", "capital of France, no cache"]) {
    assert.equal(await call(`shouldForceFresh(normalizePrompt(${JSON.stringify(prompt)}), ${JSON.stringify(prompt)})`), true, prompt);
  }
  assert.equal(await call('shouldForceFresh(normalizePrompt("What is the capital of France?"), "What is the capital of France?")'), false);
});

test("fact search honors offline and source settings and refuses ambiguous labels", async () => {
  const requested = wikidataRequests.length;
  assert.equal(await ask("What is the capital of Springfield?", { offline: true }), null);
  assert.equal(await ask("What is the capital of Australia?", { externalServiceMediawikiFamily: false }), null);
  assert.equal(wikidataRequests.length, requested);

  assert.equal(await ask("What is the capital of Springfield?"), null);
  assert.equal(wikidataRequests.length, requested + 2);
  assert.match(wikidataRequests[requested], /search=Springfield/);
  assert.match(wikidataRequests[requested + 1], /ids=Q123/);

  const unique = await ask("What is the capital of Canterbury?");
  assert.ok(unique.trace.includes("fact_query:wbsearchentities:resolved:Q123"));
  assert.equal(unique.evidence.filter((entry) => /^source:http:.* sha256=[a-f0-9]{64}$/.test(entry)).length, 4);
});

test("fact formalization preserves the words within names across languages", async () => {
  for (const [prompt, subject] of [
    ["澳大利亚的首都是什么？", "澳大利亚"],
    ["¿Cuál es la capital de Trinidad y Tobago?", "trinidad y tobago"],
    ["What is the capital city of Trinidad and Tobago?", "trinidad and tobago"],
    ["Australia's capital?", "australia"],
  ]) {
    assert.deepEqual(await call(`factLiveQuestion(${JSON.stringify(prompt)})`), { relation: "capital", subjectTerm: subject });
  }
});
