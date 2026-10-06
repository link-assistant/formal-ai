// Issue #1176: a stated weekday offset ("100 days after Monday") shifts by the
// stated count mod 7 on the worker side, exactly as detect_offset /
// render_offset_answer in rust/src/solver_handlers/calendar.rs do. The prompts
// and expectations are the ones rust/tests/unit/issue_1176_quantities_dates.rs
// asserts against the native engine.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("a stated day offset shifts by the stated count", async () => {
  const response = await solve("100 days after Monday");
  assert.equal(response.intent, "calendar_weekday_relation");
  assert.equal(
    response.content,
    "100 days after Monday is Wednesday. 100 days = 14 weeks + 2 days, and Monday + 2 days = Wednesday in the seven-day calendar cycle.",
  );
  assert.ok(response.evidence.includes("calendar:offset:+100d"));
  assert.ok(response.evidence.includes("calendar:offset_derivation:100d = 14w + 2d"));
});

test("the question form answers Wednesday, not the +1 Tuesday", async () => {
  const response = await solve("What day is 100 days after Monday?");
  assert.equal(response.intent, "calendar_weekday_relation");
  assert.match(response.content, /is Wednesday/);
  assert.doesNotMatch(response.content, /Tuesday/);
});

test("a bare day-after keeps the plus-one reading", async () => {
  const response = await solve("the day after Monday");
  assert.equal(response.intent, "calendar_weekday_relation");
  assert.ok(response.content.includes("The day after Monday is Tuesday"), response.content);
});

test("a stated offset before shifts backward", async () => {
  const response = await solve("30 days before Friday");
  assert.ok(response.content.includes("Wednesday"), response.content);
  assert.ok(response.content.includes("4 weeks + 2 days"), response.content);
});

test("a whole-week offset leaves the weekday unchanged", async () => {
  const response = await solve("2 weeks after Monday");
  assert.ok(response.content.includes("Monday"), response.content);
  assert.ok(response.content.includes("14 days exactly"), response.content);
});

test("a CJK offset reads the day unit character", async () => {
  const response = await solve("星期一之后100天是星期几？");
  assert.equal(response.intent, "calendar_weekday_relation");
  assert.ok(response.content.includes("星期三"), response.content);
  assert.ok(response.content.includes("14周 + 2天"), response.content);
});

test("a Russian offset answers in Russian", async () => {
  const response = await solve("какой день будет через 100 дней после понедельника?");
  assert.equal(response.intent, "calendar_weekday_relation");
  assert.ok(response.content.includes("среда"), response.content);
  assert.ok(response.content.includes("14 недель + 2 дня"), response.content);
});
