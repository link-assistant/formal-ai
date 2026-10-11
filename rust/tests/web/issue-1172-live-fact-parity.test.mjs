// Issue #1172 R3 JS parity: the browser worker's live fact path, the twin of
// rust/tests/unit/seed/issue_1172_live_fact_answer.rs. The question is formalized
// from seed cues alone (factLiveQuestion, same expectations as the native
// live_fact_question), and a Wikidata resolution answers through the seeded
// fact_live_answer template citing the claim's reference URL, else the
// subject's snapshot URL. Wikidata is a fixture fetch, so no test reaches the
// network.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REFERENCE_URL = "https://www.example.gov.au/about/capital";
const AUSTRALIA = {
  entities: {
    Q408: {
      id: "Q408",
      labels: { en: { language: "en", value: "Australia" } },
      claims: {
        P36: [{
          mainsnak: { datavalue: { value: { "entity-type": "item", id: "Q3114" }, type: "wikibase-entityid" } },
          references: [{ snaks: { P854: [{ datavalue: { value: REFERENCE_URL, type: "string" } }] } }],
        }],
        P1082: [{ mainsnak: { datavalue: { value: { amount: "+27122411", unit: "1" }, type: "quantity" } } }],
      },
    },
  },
};
const CANBERRA = { entities: { Q3114: { id: "Q3114", labels: { en: { language: "en", value: "Canberra" } } } } };
const labels = (byLanguage) =>
  Object.fromEntries(Object.entries(byLanguage).map(([language, value]) => [language, { language, value }]));
const WAR_AND_PEACE = {
  entities: {
    Q161531: {
      id: "Q161531",
      labels: labels({ en: "War and Peace", ru: "Война и мир", hi: "युद्ध और शान्ति", zh: "战争与和平" }),
      claims: { P50: [{ mainsnak: { datavalue: { value: { "entity-type": "item", id: "Q7243" }, type: "wikibase-entityid" } } }] },
    },
  },
};
const TOLSTOY = {
  entities: {
    Q7243: { id: "Q7243", labels: labels({ en: "Leo Tolstoy", ru: "Лев Толстой", hi: "लेव तोलस्तोय", zh: "列夫·托尔斯泰" }) },
  },
};

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");
const seedFetch = worker.fetch;
worker.fetch = (url, init) => {
  const text = String(url);
  if (!text.includes("wikidata.org")) return seedFetch(url, init);
  let body = null;
  if (text.includes("wbsearchentities") && text.includes("search=Australia")) body = { search: [{ id: "Q408", label: "Australia" }] };
  else if (text.includes("wbsearchentities") && /search=(War|%D0%92%D0%BE%D0%B9|%E0%A4%AF|%E6%88%98)/.test(text)) body = { search: [{ id: "Q161531", label: "War and Peace" }] };
  else if (text.includes("wbsearchentities")) body = { search: [] };
  else if (text.includes("ids=Q408")) body = AUSTRALIA;
  else if (text.includes("ids=Q3114")) body = CANBERRA;
  else if (text.includes("ids=Q161531")) body = WAR_AND_PEACE;
  else if (text.includes("ids=Q7243")) body = TOLSTOY;
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

const literal = (value) => JSON.stringify(value);

test("the question is formalized from seed cues, as the native twin", async () => {
  assert.deepEqual(await call(`factLiveQuestion(${literal("What is the capital of Australia?")})`), {
    relation: "capital",
    subjectTerm: "australia",
  });
  assert.deepEqual(await call(`factLiveQuestion(${literal("What is the population of New Zealand?")})`), {
    relation: "population",
    subjectTerm: "new zealand",
  });
  assert.equal(await call(`factLiveQuestion(${literal("What is the speed of light?")})`), null);
  assert.equal(await call(`factLiveQuestion(${literal("澳大利亚的首都是什么？")})`), null);
});

test("R3: a live capital answer cites the claim's reference URL", async () => {
  const prompt = "What is the capital of Australia?";
  const answer = await call(`tryFactQuery(${literal(prompt)}, normalizePrompt(${literal(prompt)}), {})`);
  assert.equal(answer.intent, "fact_query");
  assert.equal(
    answer.content,
    "Wikidata states that the capital of Australia is Canberra. Source: https://www.example.gov.au/about/capital",
  );
  assert.ok(answer.evidence.includes("wikidata:Q3114"), answer.evidence.join("\n"));
});

test("R3: a claim citing no reference points at the subject snapshot", async () => {
  const prompt = "What is the population of Australia?";
  const answer = await call(`tryFactQuery(${literal(prompt)}, normalizePrompt(${literal(prompt)}), {})`);
  assert.equal(answer.intent, "fact_query");
  assert.equal(
    answer.content,
    "Wikidata states that the population of Australia is 27122411. Source: https://www.wikidata.org/wiki/Special:EntityData/Q408.json",
  );
});

test("R3: an authorship answer reads through the relation's own seeded phrasing", async () => {
  const snapshot = "https://www.wikidata.org/wiki/Special:EntityData/Q161531.json";
  const cases = [
    ["Who wrote War and Peace?", `War and Peace was written by Leo Tolstoy. Source: ${snapshot}`],
    ["Кто написал «Войну и мир»?", `Автор произведения «Война и мир»: Лев Толстой. Источник: ${snapshot}`],
    ["युद्ध और शान्ति किसने लिखी?", `युद्ध और शान्ति को लेव तोलस्तोय ने लिखा था। स्रोत: ${snapshot}`],
    ["战争与和平是谁写的?", `《战争与和平》由列夫·托尔斯泰创作。来源：${snapshot}`],
  ];
  for (const [prompt, expected] of cases) {
    const answer = await call(`tryFactQuery(${literal(prompt)}, normalizePrompt(${literal(prompt)}), {})`);
    assert.equal(answer && answer.intent, "fact_query", prompt);
    assert.equal(answer.content, expected, prompt);
    assert.ok(answer.evidence.includes("fact_query:relation:author_of_book"), answer.evidence.join("\n"));
    assert.ok(answer.evidence.includes("wikidata:Q7243"), answer.evidence.join("\n"));
  }
});

test("explicit seed namespaces exclude sibling captures and preserve registry order", async () => {
  const seed = [
    "captures", "  source impostor", '    api "https://impostor.invalid/{id}"',
    "registry", "  source first", '    api "https://first.invalid/{id}"',
    "registry", "  source second", '    api "https://second.invalid/{id}"',
  ].join("\n");
  assert.deepEqual(await call("pageSeedRecords(" + literal(seed) + ', "registry").map(row => [row.name, row.value, childValue(row, "api")])'), [
    ["source", "first", "https://first.invalid/{id}"],
    ["source", "second", "https://second.invalid/{id}"],
  ]);
  assert.deepEqual(await call("pageSeedRecords(" + literal(seed) + ', "missing")'), []);
});

test("the real multi-namespace registry supplies endpoints and page primacy", async () => {
  const registry = await call('pageSeedRecords(seedRawText(SEED_RAW, "sources-registry.lino"), "sources_registry").filter(row => row.name === "source").map(row => [row.value, childValue(row, "api")])');
  assert.equal(registry.find(([id]) => id === "wikidata")[1], "https://www.wikidata.org/wiki/Special:EntityData/{id}.json");
  assert.equal(await call('pageRegistryPrimacy("wikidata.org")'), "editorial_synthesis");
  assert.equal(await call('pageRegistryPrimacy("unregistered.invalid")'), null);
});

test("unnamed seed records retain wrapper and bare-record behavior", async () => {
  assert.deepEqual(await call('pageSeedRecords("wrapper\\n  record first\\n  record second").map(row => row.value)'), ["first", "second"]);
  assert.deepEqual(await call('pageSeedRecords("record first\\nrecord second").map(row => row.value)'), ["first", "second"]);
});
