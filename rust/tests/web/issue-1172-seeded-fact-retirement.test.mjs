// Issue #1172 R9: the facts the committed Wikidata captures can reproduce are
// no longer written in data/seed/facts.lino; both runtimes derive them.
//
// The ten retired records (the capitals of Japan, Russia, France, Germany,
// China, India, the USA, the UK and Brazil, and the author of The Lord of the
// Rings) are now subject captures in data/cache/wikidata/fact-claims, projected
// into data/seed/fact-captures.lino by scripts/ground-fact-captures.py, and
// realized by data/seed/fact-realization.lino: the value is the subject's
// truthy claim for the property the relation is grounded in, the labels and
// sources are the captures' labels and sitelinks, and each sentence is the
// relation's template with Russian case forms, the English article and the
// recorded preferred label variants. This test pins that every answer the
// retired records gave is reproduced verbatim, that every surface their label
// index resolved still resolves, and that nothing derived is written down.
// The Rust twin is rust/tests/unit/issue_1172_fact_derivation.rs.

import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { installNodeHost } from "../../../js/agentic/node-host.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const FACT_CLAIMS = path.join(REPO_ROOT, "data", "cache", "wikidata", "fact-claims");

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function call(expression) {
  await ready;
  return plain(await evaluate(worker, expression));
}

async function derived() {
  return call("derivedCaptureFacts(SEED_RAW)");
}

/** The answers the retired facts.lino records gave, per subject and language. */
const RETIRED_SUMMARIES = {
  Q17: { en: "The capital of Japan is Tokyo.", ru: "Столица Японии — Токио.", hi: "जापान की राजधानी टोक्यो है।", zh: "日本的首都是东京。" },
  Q159: { en: "The capital of Russia is Moscow.", ru: "Столица России — Москва.", hi: "रूस की राजधानी मास्को है।", zh: "俄罗斯的首都是莫斯科。" },
  Q142: { en: "The capital of France is Paris.", ru: "Столица Франции — Париж.", hi: "फ्रांस की राजधानी पेरिस है।", zh: "法国的首都是巴黎。" },
  Q183: { en: "The capital of Germany is Berlin.", ru: "Столица Германии — Берлин.", hi: "जर्मनी की राजधानी बर्लिन है।", zh: "德国的首都是柏林。" },
  Q148: { en: "The capital of China is Beijing.", ru: "Столица Китая — Пекин.", hi: "चीन की राजधानी बीजिंग है।", zh: "中国的首都是北京。" },
  Q668: { en: "The capital of India is New Delhi.", ru: "Столица Индии — Нью-Дели.", hi: "भारत की राजधानी नई दिल्ली है।", zh: "印度的首都是新德里。" },
  Q30: { en: "The capital of the United States is Washington, D.C.", ru: "Столица США — Вашингтон.", hi: "संयुक्त राज्य अमेरिका की राजधानी वाशिंगटन, डी.सी. है।", zh: "美国的首都是华盛顿哥伦比亚特区。" },
  Q145: { en: "The capital of the United Kingdom is London.", ru: "Столица Великобритании — Лондон.", hi: "यूनाइटेड किंगडम की राजधानी लंदन है।", zh: "英国的首都是伦敦。" },
  Q155: { en: "The capital of Brazil is Brasília.", ru: "Столица Бразилии — Бразилиа.", hi: "ब्राज़ील की राजधानी ब्रासीलिया है।", zh: "巴西的首都是巴西利亚。" },
  Q15228: { en: "The Lord of the Rings was written by J. R. R. Tolkien.", ru: "«Властелин колец» был написан Дж. Р. Р. Толкином.", hi: "द लॉर्ड ऑफ द रिंग्स को जे. आर. आर. टॉल्किन ने लिखा था।", zh: "《魔戒》由 J·R·R·托爾金 创作。" },
};

/** The label-index surfaces the retired records listed. */
const RETIRED_ALIASES = {
  Q17: ["japan", "japan's", "japans", "япония", "японии", "японию", "японией", "япония's", "जापान", "जापानी", "日本"],
  Q159: ["russia", "russia's", "russias", "russian federation", "россия", "россии", "россию", "россией", "российская федерация", "российской федерации", "रूस", "रूसी संघ", "俄罗斯", "俄羅斯"],
  Q142: ["france", "france's", "frances", "french republic", "франция", "франции", "францию", "французская республика", "फ्रांस", "फ़्रांस", "法国", "法國"],
  Q183: ["germany", "germany's", "germanys", "federal republic of germany", "германия", "германии", "германию", "федеративная республика германия", "जर्मनी", "德国", "德國"],
  Q148: ["china", "china's", "chinas", "people's republic of china", "prc", "китай", "китая", "китаю", "китаем", "китае", "китайская народная республика", "चीन", "中国", "中華人民共和國"],
  Q668: ["india", "india's", "indias", "republic of india", "индия", "индии", "индию", "индией", "भारत", "इंडिया", "印度"],
  Q30: ["united states", "united states of america", "usa", "us", "america", "the united states", "сша", "соединенные штаты", "соединённые штаты", "соединенных штатов", "соединённых штатов", "америка", "америки", "америку", "संयुक्त राज्य अमेरिका", "अमेरिका", "美国", "美利坚合众国", "美國", "美利堅合眾國"],
  Q145: ["united kingdom", "uk", "britain", "great britain", "england", "the united kingdom", "великобритания", "великобритании", "британия", "британии", "англия", "англии", "यूनाइटेड किंगडम", "ब्रिटेन", "इंग्लैंड", "英国", "大不列颠", "英國", "大不列顛"],
  Q155: ["brazil", "brazil's", "brazils", "brasil", "federative republic of brazil", "бразилия", "бразилии", "бразилию", "бразилией", "ब्राज़ील", "ब्राजील", "巴西", "巴西聯邦共和國"],
  Q15228: ["lord of the rings", "the lord of the rings", "властелин колец", "властелина колец", "властелину колец", "властелином колец", "властелине колец", "द लॉर्ड ऑफ द रिंग्स", "द लॉर्ड ऑफ़ द रिंग्स", "魔戒", "魔戒之王", "指環王"],
};

test("#1172 R9: no written record states a fact the captures derive", async () => {
  const written = await call("self.FormalAiSeed.extractFacts(self.FormalAiSeed.parse(SEED_RAW[\"seed/facts.lino\"]))");
  assert.deepEqual(
    written.map((record) => record.slug),
    [
      "fact_spider_man_films_release_order",
      "fact_air_india_infant_stroller_allowance",
      "fact_formal_ai_creator",
      "fact_rust_creator",
      "fact_eiffel_tower_built",
      "fact_speed_of_light",
      "fact_mona_lisa_painter",
    ],
  );
  assert.deepEqual(written.filter((record) => record.subjectQid).map((record) => record.slug), []);
  // The worker's fact store: the written records (the two fact_comparison_*
  // entries are meanings-facts.lino meanings the loader also visits), then the
  // derived ones in capture order.
  const facts = await call("FACTS.map((record) => record.slug)");
  assert.deepEqual(facts.slice(7, 9), ["fact_comparison_request", "fact_comparison_joiner"]);
  assert.deepEqual(facts.slice(9), [
    "fact_capital_q17",
    "fact_capital_q159",
    "fact_capital_q142",
    "fact_capital_q183",
    "fact_capital_q148",
    "fact_capital_q668",
    "fact_capital_q30",
    "fact_capital_q145",
    "fact_capital_q155",
    "fact_author_of_book_q15228",
  ]);
});

test("#1172 R9: the derived records, their claims and their evidence order", async () => {
  const records = await derived();
  assert.deepEqual(
    records.map((record) => [record.slug, record.relation, record.subjectQid, record.valueQid, record.wikidata.join(" "), record.category]),
    [
      ["fact_capital_q17", "capital", "Q17", "Q1490", "Q17 Q1490", "geography"],
      ["fact_capital_q159", "capital", "Q159", "Q649", "Q159 Q649", "geography"],
      ["fact_capital_q142", "capital", "Q142", "Q90", "Q142 Q90", "geography"],
      ["fact_capital_q183", "capital", "Q183", "Q64", "Q183 Q64", "geography"],
      ["fact_capital_q148", "capital", "Q148", "Q956", "Q148 Q956", "geography"],
      ["fact_capital_q668", "capital", "Q668", "Q987", "Q668 Q987", "geography"],
      ["fact_capital_q30", "capital", "Q30", "Q61", "Q30 Q61", "geography"],
      ["fact_capital_q145", "capital", "Q145", "Q84", "Q145 Q84", "geography"],
      ["fact_capital_q155", "capital", "Q155", "Q2844", "Q155 Q2844", "geography"],
      ["fact_author_of_book_q15228", "author_of_book", "Q15228", "Q892", "Q892 Q15228", "literature"],
    ],
  );
  // Each value is the subject capture's truthy claim, never a seed value.
  for (const record of records) {
    const capture = JSON.parse(readFileSync(path.join(FACT_CLAIMS, `${record.subjectQid}.json`), "utf8"));
    const claims = Object.values(capture.entities[record.subjectQid].claims)[0];
    assert.equal(claims[0].mainsnak.datavalue.value.id, record.valueQid, record.slug);
  }
});

test("#1172 R9: every retired answer is realized verbatim from the captures", async () => {
  const records = await derived();
  const realized = Object.fromEntries(
    records.map((record) => [record.subjectQid, Object.fromEntries(record.localized.map((entry) => [entry.language, entry.summary]))]),
  );
  assert.deepEqual(realized, RETIRED_SUMMARIES);
});

test("#1172 R9: the localized labels and sources come from the captures", async () => {
  const records = await derived();
  const usa = records.find((record) => record.subjectQid === "Q30");
  assert.deepEqual(
    usa.localized.map((entry) => [entry.language, entry.subjectLabel, entry.valueLabel, entry.source]),
    [
      ["en", "the United States", "Washington, D.C.", "https://en.wikipedia.org/wiki/Washington,_D.C."],
      ["ru", "США", "Вашингтон", "https://ru.wikipedia.org/wiki/Вашингтон"],
      ["hi", "संयुक्त राज्य अमेरिका", "वाशिंगटन, डी.सी.", "https://hi.wikipedia.org/wiki/वॉशिंगटन,_डी॰_सी॰"],
      ["zh", "美国", "华盛顿哥伦比亚特区", "https://zh.wikipedia.org/wiki/华盛顿哥伦比亚特区"],
    ],
  );
  const book = records.find((record) => record.subjectQid === "Q15228");
  assert.deepEqual(
    book.localized.map((entry) => [entry.language, entry.subjectLabel, entry.valueLabel, entry.source]),
    [
      ["en", "The Lord of the Rings", "J. R. R. Tolkien", "https://en.wikipedia.org/wiki/The_Lord_of_the_Rings"],
      ["ru", "Властелин колец", "Дж. Р. Р. Толкин", "https://ru.wikipedia.org/wiki/Властелин_колец"],
      ["hi", "द लॉर्ड ऑफ द रिंग्स", "जे. आर. आर. टॉल्किन", "https://hi.wikipedia.org/wiki/द_लॉर्ड_ऑफ़_द_रिंग्स"],
      ["zh", "魔戒", "J·R·R·托爾金", "https://zh.wikipedia.org/wiki/魔戒"],
    ],
  );
});

test("#1172 R9: Russian case forms and the English article are rules, not stored strings", async () => {
  const rules = "factRealizationRules(factSeedRoot(seedRawText(SEED_RAW, FACT_REALIZATION_FILE), \"fact_realization\"))";
  const forms = await call(`(() => { const rules = ${rules}; return [
    factInflect(rules, "Япония", "ru", "genitive"),
    factInflect(rules, "Китай", "ru", "genitive"),
    factInflect(rules, "США", "ru", "genitive"),
    factInflect(rules, "Дж. Р. Р. Толкин", "ru", "instrumental"),
    factInflect(rules, "Властелин колец", "ru", "prepositional"),
    factInflect(rules, "Москва", "ru", "genitive"),
    factInflect(rules, "Нью-Дели", "ru", "genitive"),
    factDefinite(rules, "United Kingdom", "en"),
    factDefinite(rules, "Japan", "en"),
    factDefinite(rules, "the United States", "en"),
  ]; })()`);
  assert.deepEqual(forms, [
    "Японии",
    "Китая",
    "США",
    "Дж. Р. Р. Толкином",
    "Властелине колец",
    "Москвы",
    "Нью-Дели",
    "the United Kingdom",
    "Japan",
    "the United States",
  ]);
});

test("#1172 R9: every surface the retired records resolved still resolves its subject", async () => {
  const records = await derived();
  for (const [qid, surfaces] of Object.entries(RETIRED_ALIASES)) {
    const record = records.find((candidate) => candidate.subjectQid === qid);
    assert.deepEqual(surfaces.filter((surface) => !record.subjectAliases.includes(surface)), [], qid);
  }
  // How many of each subject's surfaces are recorded rather than generated
  // from its captured labels and the morphology.
  const recorded = await call(
    "(() => { const rules = factRealizationRules(factSeedRoot(seedRawText(SEED_RAW, FACT_REALIZATION_FILE), \"fact_realization\")); return Object.fromEntries([...rules.surfaces].map(([qid, list]) => [qid, list.length])); })()",
  );
  assert.deepEqual(recorded, { Q17: 2, Q159: 5, Q142: 3, Q183: 2, Q148: 3, Q668: 2, Q30: 14, Q145: 13, Q155: 4, Q15228: 2 });
});

test("#1172 R9: a preferred label marked as a Wikidata alias is one of the entity's captured aliases", async () => {
  const realization = readFileSync(path.join(REPO_ROOT, "data", "seed", "fact-realization.lino"), "utf8");
  const origins = { wikidata_alias: [], recorded: [] };
  for (const block of realization.split(/\n(?=  preferred_label )/u).slice(1)) {
    const qid = block.match(/^  preferred_label (Q\d+)/u)[1];
    const capture = JSON.parse(readFileSync(path.join(FACT_CLAIMS, `${qid}.json`), "utf8")).entities[qid];
    const aliases = Object.values(capture.aliases || {}).flat().map((entry) => entry.value);
    for (const match of block.matchAll(/\n {4}([a-z-]+)\n {6}text "([^"]+)"\n {6}origin "([a-z_]+)"/gu)) {
      origins[match[3]].push(`${qid} ${match[1]} ${match[2]}`);
      assert.equal(aliases.includes(match[2]), match[3] === "wikidata_alias", `${qid} ${match[1]} ${match[2]}`);
    }
  }
  assert.deepEqual(origins.recorded, [
    "Q148 en China",
    "Q956 zh 北京",
    "Q61 hi वाशिंगटन, डी.सी.",
    "Q15228 hi द लॉर्ड ऑफ द रिंग्स",
    "Q892 hi जे. आर. आर. टॉल्किन",
  ]);
});

test("#1172 R9: the seed projection is exactly what the committed captures encode", () => {
  const quote = (items) => `(${items.map((item) => JSON.stringify(item)).join(" ")})`;
  const byteOrder = (left, right) => (Buffer.from(left[0]) < Buffer.from(right[0]) ? -1 : Buffer.from(left[0]) > Buffer.from(right[0]) ? 1 : 0);
  const projection = readFileSync(path.join(REPO_ROOT, "data", "seed", "fact-captures.lino"), "utf8");
  const entities = [...projection.matchAll(/^  entity (Q\d+)$/gmu)].map((match) => match[1]);
  assert.deepEqual(
    entities.slice().sort(),
    readdirSync(FACT_CLAIMS).filter((name) => name.endsWith(".json")).map((name) => name.slice(0, -5)).sort(),
  );
  let expected = projection.slice(0, projection.indexOf("  entity "));
  for (const qid of entities) {
    const entity = JSON.parse(readFileSync(path.join(FACT_CLAIMS, `${qid}.json`), "utf8")).entities[qid];
    expected += `  entity ${qid}\n`;
    for (const [language, label] of Object.entries(entity.labels || {}).sort(byteOrder)) expected += `    label ${quote([language, label.value])}\n`;
    for (const [language, list] of Object.entries(entity.aliases || {}).sort(byteOrder)) expected += `    alias ${quote([language, ...list.map((alias) => alias.value)])}\n`;
    for (const [site, link] of Object.entries(entity.sitelinks || {}).sort(byteOrder)) expected += `    sitelink ${quote([site, link.title])}\n`;
    for (const [property, claims] of Object.entries(entity.claims || {}).sort(byteOrder)) {
      for (const claim of claims) expected += `    claim ${quote([property, claim.mainsnak.datavalue.value.id, claim.rank])}\n`;
    }
  }
  assert.equal(projection, expected);
});

test("#1172 R9: the derived records answer in every language they realize", async () => {
  await ready;
  const answer = async (prompt) => plain(await worker.solve(prompt, [], {}, {}, [], {}));
  const cases = [
    ["What is the capital of Japan?", "The capital of Japan is Tokyo."],
    ["What is the capital of the USA?", "The capital of the United States is Washington, D.C."],
    ["Столица Германии?", "Столица Германии — Берлин."],
    ["Кто написал «Властелин колец»?", "«Властелин колец» был написан Дж. Р. Р. Толкином."],
    ["中国的首都是什么？", "中国的首都是北京。"],
  ];
  for (const [prompt, expected] of cases) {
    const solved = await answer(prompt);
    assert.equal(solved.content, expected, prompt);
    assert.equal(solved.intent, "fact_lookup", prompt);
  }
});

test("#1172 R9: the agentic port derives the same records and resolves the retired subjects", async () => {
  await installNodeHost(new WorkerHost());
  const { derivedFacts } = await import("../../../js/agentic/crate/fact_derivation.mjs");
  const { factStoreResolves } = await import("../../../js/agentic/crate/solver_handlers_benchmark_prompts.mjs");
  assert.deepEqual(JSON.parse(JSON.stringify(derivedFacts())), await derived());
  assert.deepEqual(
    [
      "What is the capital of Japan?",
      "Столица Германии?",
      "Who wrote The Lord of the Rings?",
      "What is the capital of Australia?",
    ].map((prompt) => factStoreResolves(prompt)),
    [true, true, true, false],
  );
});
