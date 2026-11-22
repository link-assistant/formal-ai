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

test("facts require the subject, property and labeled value from the publisher", async () => {
  const fresh = createWorkerContext();
  const sourceFetch = fresh.fetch;
  const requests = [];
  const subjects = new Map([
    ["speed of light", "Q2111"], ["mona lisa", "Q12418"], ["мона лиза", "Q12418"],
    ["rust", "Q575650"],
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
  const history = [{ role: "user", content: "What is Rust?" }];
  const followup = "Who created it?";
  const cold = plain(await fresh.solve(followup, history, { offline: true }));
  assert.doesNotMatch(cold.content, /Graydon Hoare/u);
  const creator = plain(await fresh.solve(followup, history, {}));
  assert.equal(creator.intent, "fact_query", JSON.stringify(requests));
  assert.match(creator.content, /Graydon Hoare/u);
  assert.ok(creator.evidence.includes("coreference:resolved:it=Rust"));
  assert.ok(creator.evidence.includes("wikidata:Q28937563"));
  assert.ok(requests.some((url) => {
    const { searchParams } = new URL(url);
    return searchParams.get("ids") === "Q28937563" &&
      searchParams.get("languages").split("|").includes("mul");
  }));
  for (const prompt of ["Who created Rust?", "Кто создал Rust?", "Rust किसने बनाया?", "谁创建 Rust?"]) {
    const direct = await ask(prompt);
    assert.equal(direct.intent, "fact_query", prompt);
    assert.ok(direct.evidence.includes("wikidata:Q28937563"), prompt);
  }
});

test("dates come from ranked publisher claims for any matched subject", async () => {
  const fresh = createWorkerContext();
  const seedOnly = fresh.fetch;
  const requests = [];
  const subjects = new Map([
    ["eiffel tower", { id: "Q243", label: "Eiffel Tower" }],
    ["эйфелева башня", { id: "Q243", label: "Эйфелева башня" }],
    ["एफिल टावर", { id: "Q243", label: "एफ़िल टावर" }],
    ["埃菲尔铁塔", { id: "Q243", label: "埃菲尔铁塔" }],
    ["unesco", { id: "Q7809", label: "UNESCO" }],
  ]);
  fresh.fetch = async (url, options) => {
    const target = new URL(String(url), "http://localhost");
    if (target.hostname !== "www.wikidata.org") return seedOnly(url, options);
    requests.push(target.href);
    const term = target.searchParams.get("search");
    const subject = term && subjects.get(term.toLowerCase());
    const body = term ? { search: subject ? [subject] : [] } : wikidataFixture(target, null, "live-entities");
    return { ok: Boolean(body), status: body ? 200 : 404, text: async () => JSON.stringify(body) };
  };
  await evaluate(fresh, "loadSeed()");
  const ask = async (prompt, preferences = {}) => plain(await fresh.solve(prompt, [], preferences));
  const construction = "When was the Eiffel Tower built?";
  const opening = "When did the Eiffel Tower open?";
  const cold = await ask(construction, { offline: true });
  assert.doesNotMatch(cold.content, /1887|1889/u);
  assert.deepEqual(requests, []);
  for (const prompt of [construction, "What year did construction of the Eiffel Tower start?", "Когда построили Эйфелеву башню?", "एफिल टॉवर कब बनी?", "埃菲尔铁塔建于何时?"]) {
    const answer = await ask(prompt);
    assert.equal(answer.intent, "fact_query", `${prompt}\n${answer.content}\n${requests.join("\n")}`);
    assert.match(answer.content, /\b1887\b/u, prompt);
    assert.doesNotMatch(answer.content, /1889|1887-01/u, prompt);
    assert.ok(answer.evidence.some((value) => /Q243.*sha256=[0-9a-f]{64}/u.test(value)), prompt);
  }
  const dates = await ask(opening);
  assert.equal(dates.intent, "fact_query", dates.content);
  assert.match(dates.content, /1889-03-31, 1889-05-15/u);
  assert.doesNotMatch(dates.content, /1887/u);
  const heldOut = await ask("When was UNESCO founded?");
  assert.equal(heldOut.intent, "fact_query", heldOut.content);
  assert.match(heldOut.content, /UNESCO.*1945/u);
  assert.ok(heldOut.evidence.includes("wikidata:Q7809"));
  const fetched = requests.length;
  assert.equal((await ask(opening, { offline: true })).content, dates.content);
  assert.equal(requests.length, fetched);
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

test("project origin comes from an authorized declaration and an empty offline worker has no copy", async () => {
  const api = "https://api.github.com/repos/link-assistant/formal-ai/issues/comments/4526577668";
  const source = "https://github.com/link-assistant/formal-ai/issues/157#issuecomment-4526577668";
  const declaration = {
    id: 4526577668,
    html_url: source,
    issue_url: "https://api.github.com/repos/link-assistant/formal-ai/issues/157",
    author_association: "CONTRIBUTOR",
    user: { login: "konard" },
    body: "Formal AI was created by github.com/konard using github.com/link-assistant/hive-mind.",
  };
  const secondSource = "https://github.com/acme/example/issues/2#issuecomment-21";
  const secondApi = "https://api.github.com/repos/acme/example/issues/comments/21";
  const secondDeclaration = {
    id: 21,
    html_url: secondSource,
    issue_url: "https://api.github.com/repos/acme/example/issues/2",
    author_association: "OWNER",
    user: { login: "example-author" },
    body: "Example Widget was created by github.com/example-author.",
  };
  const fresh = createWorkerContext();
  await evaluate(fresh, "loadSeed()");
  evaluate(fresh, `PROJECTS.push(${literal({
    org: "acme", name: "example", displayName: "Example Widget", aliases: ["example widget"],
    url: "https://github.com/acme/example", originDeclaration: secondSource,
  })})`);
  const seedFetch = fresh.fetch;
  let requests = 0;
  fresh.fetch = async (url, init) => {
    const body = String(url) === api ? declaration : String(url) === secondApi ? secondDeclaration : null;
    if (body) {
      requests++;
      return { ok: true, status: 200, text: async () => JSON.stringify(body) };
    }
    if (String(url).startsWith("https://")) return { ok: false, status: 404 };
    return seedFetch(url, init);
  };
  const ask = async (prompt, preferences = {}) => plain(await fresh.solve(prompt, [], preferences));
  const cold = await ask("Who created you?", { offline: true });
  assert.ok(!cold.content.includes("github.com/konard"), cold.content);
  const denied = await ask("Who created you?", { externalServiceGithub: false });
  assert.ok(!denied.content.includes("github.com/konard"), denied.content);
  assert.equal(requests, 0);
  for (const prompt of ["Who created you?", "Кто тебя создал?", "तुम्हें किसने बनाया?", "谁创建了你？"]) {
    const response = await ask(prompt);
    assert.equal(response.intent, "fact_lookup", JSON.stringify(response));
    assert.match(response.content, /github\.com\/konard/);
    assert.match(response.content, /github\.com\/link-assistant\/hive-mind/);
    assert.ok(response.evidence.includes(`source:${source}`));
    assert.ok(response.evidence.some((link) => link.startsWith(`source:http:${api} sha256=`)));
  }
  assert.equal(requests, 1);
  assert.equal((await ask("Who created you?", { offline: true })).intent, "fact_lookup");
  const other = await ask("Who created Example Widget?");
  assert.equal(other.content, "Example Widget was created by github.com/example-author.");
  assert.equal(requests, 2);
  assert.equal(await evaluate(fresh, `originCreatorStatement(PROJECTS.at(-1), ${literal({
    ...secondDeclaration, author_association: "NONE",
  })})`), null);
  assert.equal(await evaluate(fresh, `originCreatorStatement(PROJECTS.at(-1), ${literal({
    ...secondDeclaration, body: "Another Widget was created by github.com/example-author.",
  })})`), null);
});

test("registered carrier policies use the checked section from a verified publisher capture", async () => {
  const source = "https://www.airindia.com/in/en/travel-information/baggage-guidelines/checked-baggage-allowance.html";
  const excerpt = "Infants are entitled to one collapsible stroller/carrycot/infant car seat.";
  const prompt = "Does Air India allow a free stroller for an infant?";
  const policyPage = `<h1>Checked Baggage</h1><h2>Domestic flights</h2><ul><li>${excerpt}</li></ul>`;
  const makeWorker = async (page) => {
    const worker = createWorkerContext();
    await evaluate(worker, "loadSeed()");
    const seedFetch = worker.fetch;
    const requests = [];
    worker.fetch = async (url, init) => {
      if (String(url) === source) {
        requests.push(String(url));
        return { ok: true, status: 200, url: source, text: async () => page };
      }
      if (String(url).startsWith("https://")) return { ok: false, status: 404 };
      return seedFetch(url, init);
    };
    return { worker, requests, ask: (question, options = {}) => worker.solve(question, [], options) };
  };
  const fixture = await makeWorker(policyPage);
  const offline = await fixture.ask(prompt, { offline: true });
  assert.equal(fixture.requests.length, 0);
  assert.doesNotMatch(offline.content, /collapsible stroller|not allowed onboard/, offline.content);
  for (const question of [prompt, "Даёт airindia бесплатную детскую коляску к багажу?",
    "क्या एयर इंडिया शिशु के लिए मुफ्त बेबी स्ट्रॉलर सामान में देती है?", "印度航空允许婴儿推车免费作为行李吗?"]) {
    const response = await fixture.ask(question);
    assert.equal(response.intent, "fact_lookup", JSON.stringify(response));
    assert.ok(response.content.includes(excerpt), response.content);
    assert.ok(response.content.includes(source), response.content);
    assert.ok(response.evidence.some(link => link.startsWith(`source:http:${source} fetched_at=`) && link.includes("sha256=")));
    assert.ok(response.evidence.includes("wikidata:Q69906"));
  }
  assert.equal(fixture.requests.length, 1);
  assert.ok((await fixture.ask(prompt, { offline: true })).content.includes(excerpt));
  assert.equal(fixture.requests.length, 1);
  const changed = await makeWorker(policyPage.replace("one collapsible", "two collapsible"));
  assert.match((await changed.ask(prompt)).content, /two collapsible stroller/);
  const unrelated = await makeWorker(`<h1>Promotions</h1><p>${excerpt}</p><h1>Checked Baggage</h1><p>Read the full list of allowances.</p>`);
  assert.doesNotMatch((await unrelated.ask(prompt)).content, /collapsible stroller|not allowed onboard/);
  const outOfScope = await fixture.ask("What is Air India's baggage allowance?");
  assert.doesNotMatch(outOfScope.content, /collapsible stroller/);

  const secondSource = "https://carrier.example/travel/rules";
  const worker = fixture.worker;
  const originalFetch = worker.fetch;
  worker.fetch = async (url, init) => {
    if (String(url) !== secondSource) return originalFetch(url, init);
    return {
      ok: true, status: 200, url: secondSource,
      text: async () => "<h2>Sports gear</h2><p>Each bicycle must be packed in a hard case.</p>",
    };
  };
  evaluate(worker, `FACTS.push(${literal({
    slug: "fact_bicycle_policy",
    subjectAliases: ["blue coast air"],
    questionKeywords: ["bicycle"],
    sourceKind: "official-docs",
    source: secondSource,
    publisherHeading: "Sports gear",
    publisherTerms: ["bicycle", "hard case"],
  })})`);
  const second = await fixture.ask("Can I check my bicycle on Blue Coast Air?");
  assert.equal(second.intent, "fact_lookup");
  assert.match(second.content, /Each bicycle must be packed in a hard case/);
  assert.ok(second.evidence.includes(`source:${secondSource}`));
});
