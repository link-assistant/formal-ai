// Issue #1172 R3 JS parity: the browser worker's live fact path, the twin of
// rust/tests/unit/issue_1172_live_fact_answer.rs. The question is formalized
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

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");
const seedFetch = worker.fetch;
worker.fetch = (url, init) => {
  const text = String(url);
  if (!text.includes("wikidata.org")) return seedFetch(url, init);
  let body = null;
  if (text.includes("wbsearchentities") && text.includes("search=Australia")) body = { search: [{ id: "Q408", label: "Australia" }] };
  else if (text.includes("wbsearchentities")) body = { search: [] };
  else if (text.includes("ids=Q408")) body = AUSTRALIA;
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
