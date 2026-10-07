// Issue #1172 R9: which seeded facts.lino records the committed Wikidata
// captures could replace, measured rather than asserted.
//
// A seeded record is live-reproducible when its relation is grounded in a
// Wikidata property and it names a subject Q-id. Retiring such a record in
// favor of the cached capture pair (data/cache/wikidata/entity/<Q>.json and
// its .lino twin) needs every piece it serves to be readable from those
// captures: the relation's claim on the subject, each localized summary's
// subject and value labels, the subject aliases the R1172-2 label index
// resolves prompts with, and the localized source page (a sitelink). This
// audit reads the worker's own parsed seed and the committed captures and pins
// what is still missing, so a capture refresh that closes a gap fails here and
// the record can then be retired instead of silently kept.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const ENTITY_CACHE = path.join(REPO_ROOT, "data", "cache", "wikidata", "entity");

const worker = createWorkerContext({
  fetch: async (url) => {
    const relative = new URL(String(url), "http://localhost/").pathname.replace(/^\/+/u, "");
    const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
    try {
      const text = readFileSync(onDisk, "utf8");
      return { ok: true, status: 200, text: async () => text };
    } catch {
      return { ok: false, status: 404, text: async () => "" };
    }
  },
});
const ready = evaluate(worker, "loadSeed()");

/** The seeded records whose relation is grounded in a Wikidata property. */
async function reproducibleRecords() {
  await ready;
  return plain(
    await evaluate(
      worker,
      `FACTS.filter((record) => record.subjectQid && /^P\\d+$/u.test(factRelationProperty(record.relation)))
        .map((record) => ({ ...record, property: factRelationProperty(record.relation) }))`,
    ),
  );
}

/** The committed capture of `qid`, or null when none is committed. */
function capture(qid) {
  try {
    return JSON.parse(readFileSync(path.join(ENTITY_CACHE, `${qid}.json`), "utf8")).entities[qid] || null;
  } catch {
    return null;
  }
}

/** What of `record` the committed captures cannot supply. */
function gaps(record) {
  const subject = capture(record.subjectQid);
  const value = capture(record.valueQid);
  const claim = ((subject && subject.claims && subject.claims[record.property]) || []).some(
    (entry) => entry && entry.mainsnak && entry.mainsnak.datavalue && entry.mainsnak.datavalue.value && entry.mainsnak.datavalue.value.id === record.valueQid,
  );
  const label = (entity, language) => entity && entity.labels && entity.labels[language] && entity.labels[language].value;
  const surfaces = new Set(
    [
      ...Object.values((subject && subject.labels) || {}).map((entry) => entry.value),
      ...Object.values((subject && subject.aliases) || {}).flat().map((entry) => entry.value),
    ].map((surface) => surface.toLowerCase()),
  );
  return {
    hasCaptures: Boolean(subject && value),
    claim,
    // A summary is reproducible from a template only when it states both
    // captured labels verbatim (no inflected or respelled form).
    summaries: record.localized
      .filter((localized) => {
        const subjectLabel = label(subject, localized.language);
        const valueLabel = label(value, localized.language);
        return !(subjectLabel && valueLabel && localized.summary.includes(subjectLabel) && localized.summary.includes(valueLabel));
      })
      .map((localized) => localized.language),
    aliases: record.subjectAliases.filter((alias) => !surfaces.has(alias)).length,
    sources: record.localized
      .filter((localized) => !(subject && subject.sitelinks && subject.sitelinks[`${localized.language}wiki`]))
      .map((localized) => localized.language),
  };
}

test("#1172 R9: the live-reproducible records are the documented ten, each with a committed capture pair", async () => {
  const records = await reproducibleRecords();
  assert.deepEqual(
    records.map((record) => record.slug),
    [
      "fact_lotr_author",
      "fact_capital_japan",
      "fact_capital_russia",
      "fact_capital_france",
      "fact_capital_germany",
      "fact_capital_china",
      "fact_capital_india",
      "fact_capital_usa",
      "fact_capital_uk",
      "fact_capital_brazil",
    ],
  );
  for (const record of records) {
    assert.ok(gaps(record).hasCaptures, `${record.slug}: ${record.subjectQid}/${record.valueQid} capture pair is committed`);
  }
});

test("#1172 R9: no committed subject capture states the relation's claim", async () => {
  // The entity captures are trimmed to labels, descriptions, aliases (and, for
  // grounded entities, P17/P297 and sitelinks): the P36/P50 statement each
  // record exists for is not in the cache, so the triple itself lives only in
  // facts.lino today.
  const records = await reproducibleRecords();
  assert.deepEqual(
    records.filter((record) => gaps(record).claim).map((record) => record.slug),
    [],
  );
});

test("#1172 R9: localized summaries the captured labels cannot reproduce", async () => {
  // ru: the summaries decline the subject ("Столица <genitive> — <city>") or
  // the author (instrumental), and no capture carries case forms. zh/hi: the
  // captured label is a different script variant or spelling than the pinned
  // answer (traditional vs simplified, "東京都" vs "东京"), and China's en/hi
  // labels are the full state name. Only the USA's ru summary uses the
  // captured abbreviation verbatim.
  const records = await reproducibleRecords();
  assert.deepEqual(Object.fromEntries(records.map((record) => [record.slug, gaps(record).summaries])), {
    fact_lotr_author: ["ru", "hi", "zh"],
    fact_capital_japan: ["ru", "zh"],
    fact_capital_russia: ["ru"],
    fact_capital_france: ["ru", "hi", "zh"],
    fact_capital_germany: ["ru", "zh"],
    fact_capital_china: ["en", "ru", "hi", "zh"],
    fact_capital_india: ["ru"],
    fact_capital_usa: ["hi", "zh"],
    fact_capital_uk: ["ru", "zh"],
    fact_capital_brazil: ["ru"],
  });
});

test("#1172 R9: every record resolves prompts through aliases the captures lack, and most cite pages no sitelink records", async () => {
  // The label index needs inflected (ru) and possessive or colloquial (en)
  // aliases that Wikidata does not list; without them "Столица Германии" or
  // "japan's capital" would no longer resolve a subject.
  const records = await reproducibleRecords();
  assert.deepEqual(Object.fromEntries(records.map((record) => [record.slug, gaps(record).aliases])), {
    fact_lotr_author: 6,
    fact_capital_japan: 7,
    fact_capital_russia: 7,
    fact_capital_france: 4,
    fact_capital_germany: 4,
    fact_capital_china: 7,
    fact_capital_india: 5,
    fact_capital_usa: 10,
    fact_capital_uk: 12,
    fact_capital_brazil: 7,
  });
  assert.deepEqual(
    records.filter((record) => gaps(record).sources.length === 0).map((record) => record.slug),
    ["fact_capital_germany"],
  );
});

test("#1172 R9: no record is retirable from the committed captures yet", async () => {
  const records = await reproducibleRecords();
  const retirable = records.filter((record) => {
    const found = gaps(record);
    return found.claim && found.summaries.length === 0 && found.aliases === 0 && found.sources.length === 0;
  });
  assert.deepEqual(retirable.map((record) => record.slug), []);
});
