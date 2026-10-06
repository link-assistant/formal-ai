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

test("a month offset from a month name is modulo-12 arithmetic", async () => {
  const response = await solve("2 months after January");
  assert.equal(response.intent, "calendar_month_relation");
  assert.equal(
    response.content,
    "2 months after January is March. January is month 1; 1 + 2 = 3, and 3 ≡ 3 (mod 12), which is March in the twelve-month calendar cycle.",
  );
  assert.ok(response.evidence.includes("calendar:offset:+2m"));
  assert.ok(response.evidence.includes("calendar:offset_derivation:1 + 2 = 3 ≡ 3 (mod 12)"));
});

test("a month offset wraps around the year in both directions", async () => {
  const forward = await solve("What month is 5 months after November?");
  assert.ok(forward.content.includes("11 + 5 = 16, and 16 ≡ 4 (mod 12), which is April"), forward.content);
  const backward = await solve("3 months before February");
  assert.ok(backward.content.startsWith("3 months before February is November."), backward.content);
  assert.ok(backward.content.includes("2 - 3 = -1, and -1 ≡ 11 (mod 12)"), backward.content);
});

test("a month offset from a weekday asks for the starting date and says why", async () => {
  const response = await solve("3 months after Monday");
  assert.equal(response.intent, "calendar_month_offset_clarification");
  assert.equal(
    response.content,
    "3 months after Monday has no single weekday answer: 3 months span 89 to 92 days depending on the starting date, because calendar months run 28 to 31 days, so the weekday it lands on is not fixed. Tell me the starting date, or state the offset in days or weeks.",
  );
  assert.ok(response.evidence.includes("calendar:offset_span:89d..92d"));
});

test("month offsets answer in the prompt language", async () => {
  const russian = await solve("через 2 месяца после января");
  assert.ok(russian.content.startsWith("Через 2 месяца после месяца «январь» наступает март."), russian.content);
  const spanish = await solve("¿Qué mes es 2 meses después de enero?");
  assert.ok(spanish.content.startsWith("2 meses después de enero es marzo."), spanish.content);
  const chinese = await solve("三月之后2个月是几月？");
  assert.ok(chinese.content.startsWith("三月之后2个月是五月。"), chinese.content);
  const clarification = await solve("星期一之后3个月是星期几？");
  assert.equal(clarification.intent, "calendar_month_offset_clarification");
  assert.ok(clarification.content.includes("89到92天"), clarification.content);
});
