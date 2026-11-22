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
import { wikidataFixture } from "./support/fact-fixtures.mjs";

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
  else if (text.includes("wbsearchentities") &&
    Object.values(WAR_AND_PEACE.entities.Q161531.labels)
      .some(({ value }) => value === new URL(text).searchParams.get("search"))) {
    const label = new URL(text).searchParams.get("search");
    body = { search: [{ id: "Q161531", label }] };
  }
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
  assert.deepEqual(await call(`factLiveQuestion(${literal("What is the speed of light?")})`), {
    relation: "physical_constant",
    subjectTerm: "speed of light",
  });
  assert.deepEqual(await call(`factLiveQuestion(${literal("澳大利亚的首都是什么？")})`), {
    relation: "capital",
    subjectTerm: "澳大利亚",
  });
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

test("quantities and painting attribution require the subject, property and labeled value from the publisher", async () => {
  const fresh = createWorkerContext();
  const sourceFetch = fresh.fetch;
  const requests = [];
  const subjects = new Map([
    ["speed of light", "Q2111"], ["mona lisa", "Q12418"], ["мона лиза", "Q12418"],
  ]);
  fresh.fetch = async (url, options) => {
    const target = new URL(String(url), "http://localhost");
    if (target.hostname !== "www.wikidata.org") return sourceFetch(url, options);
    requests.push(target.href);
    const term = target.searchParams.get("search");
    const subject = subjects.get(term?.toLowerCase());
    const body = term && !subject ? { search: [] } : wikidataFixture(target, subject, "live-entities");
    return { ok: Boolean(body), status: body ? 200 : 404, text: async () => JSON.stringify(body) };
  };
  await evaluate(fresh, "loadSeed()");
  const ask = async (prompt, preferences = {}) => plain(await fresh.solve(prompt, [], preferences));
  const speed = "What is the speed of light?";
  const painter = "Who painted the Mona Lisa?";
  for (const prompt of [speed, painter]) {
    const cold = await ask(prompt, { offline: true });
    assert.notEqual(cold.intent, "fact_query", prompt);
    assert.doesNotMatch(cold.content, /299792458|Leonardo da Vinci/u, prompt);
  }
  assert.deepEqual(requests, []);
  const quantity = await ask(speed);
  assert.equal(quantity.intent, "fact_query");
  assert.match(quantity.content, /299792458 (metre|meter) per second/u);
  assert.ok(quantity.evidence.some((link) => /Q182429.*sha256=[0-9a-f]{64}/u.test(link)), quantity.evidence.join("\n"));
  const attributed = await ask(painter);
  assert.equal(attributed.intent, "fact_query");
  assert.match(attributed.content, /Leonardo da Vinci/u);
  const russian = await ask("Кто написал Мону Лизу?");
  assert.equal(russian.intent, "fact_query", JSON.stringify(requests));
  assert.match(russian.content, /Леонардо да Винчи/u);
  assert.ok(russian.evidence.includes("fact_query:relation:painter_of_painting"));
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
