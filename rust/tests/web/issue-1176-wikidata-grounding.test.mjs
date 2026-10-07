// Issue #1176 R1176-1 browser twin: the unit converter resolves each unit of
// a linear conversion to its Wikidata item, derives the factor at answer time
// from the two items' captured P2370 statements
// (data/seed/wikidata-conversion-to-si.lino) and cites them. Native pins:
// rust/tests/unit/issue_1176_wikidata_grounding.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("R1176-1: an English conversion cites both items and the P2370 quotient", async () => {
  const answer = await solve("How many meters are 5 feet?");
  assert.equal(answer.intent, "unit_conversion");
  assert.equal(
    answer.content,
    "5 feet is 1.524 meters. 5 × 0.3048 = 1.524, because 1 feet = 0.3048 meters. Wikidata grounds the factor: feet is Q3710 and meters is Q11573, whose conversion to SI unit (P2370, captured 2026-10-07) is 0.3048 and 1 of Q11573, so 0.3048 ÷ 1 = 0.3048.",
  );
  for (const item of ["Q3710", "Q11573"]) {
    assert.ok(answer.evidence.includes(`unit_conversion:wikidata_item:${item}`), JSON.stringify(answer.evidence));
  }
});

test("R1176-1: a Russian conversion cites the items in Russian", async () => {
  const answer = await solve("Сколько километров в 26.2 милях?");
  assert.equal(
    answer.content,
    "26.2 милях — это 42.1648128 километров. 26.2 × 1.609344 = 42.1648128, потому что 1 милях = 1.609344 километров. Коэффициент из Викиданных: милях — это Q253276, километров — это Q828224; их перевод в единицу СИ (P2370, снимок от 2026-10-07) равен 1609.344 и 1000 от Q11573, поэтому 1609.344 ÷ 1000 = 1.609344.",
  );
});

test("R1176-1: an amount past the exact bounds keeps the seed factor and cites nothing", async () => {
  const answer = await solve("How many grams are 2 ounces?");
  assert.equal(
    answer.content,
    "2 ounces is 56.69904625 grams. 2 × 28.349523125 = 56.69904625, because 1 ounces = 28.349523125 grams.",
  );
  assert.ok(!answer.evidence.some((entry) => entry.startsWith("unit_conversion:wikidata_item:")));
});

test("R1176-1: the factor is the capture's quotient, so a moved capture amount moves the answer", async () => {
  const factor = await evaluate(worker, "exactDecimalRender(unitWikidataFactor('mile', 'kilometer').factor)");
  assert.equal(factor, "1.609344");
  assert.equal(await evaluate(worker, "unitWikidataFactor('celsius', 'kelvin')"), null);
});

test("R1176-1: Hindi and Chinese conversions cite the items in their language", async () => {
  assert.equal((await solve("26.2 मील कितने किलोमीटर हैं?")).content, "26.2 मील = 42.1648128 किलोमीटर। 26.2 × 1.609344 = 42.1648128, क्योंकि 1 मील = 1.609344 किलोमीटर। गुणांक विकिडेटा से: मील Q253276 है और किलोमीटर Q828224 है; SI इकाई में उनका रूपांतरण (P2370, 2026-10-07 को लिया गया) Q11573 का 1609.344 और 1000 है, इसलिए 1609.344 ÷ 1000 = 1.609344।");
  assert.equal((await solve("26.2英里是多少公里？")).content, "26.2 英里 是 42.1648128 公里。26.2 × 1.609344 = 42.1648128，因为 1 英里 = 1.609344 公里。系数来自维基数据：英里 是 Q253276，公里 是 Q828224；它们换算为国际单位（P2370，2026-10-07 采集）分别是 Q11573 的 1609.344 和 1000，所以 1609.344 ÷ 1000 = 1.609344。");
});
