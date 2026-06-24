// Issue #541 (R5/R6): reproducible unit test for the message-reveal pacing math.
//
// The real implementation lives inside src/web/app.js as the browser-only React
// hooks `normalizeAnimationBudgetMs` / `useMessageReveal`. Those depend on
// `window`, `useState`, `useEffect` and so cannot be imported under plain Node.
// This test pins the *pure logic* both functions encode so a future refactor
// that changes the timing curve fails loudly here. The constants and formulas
// are mirrored verbatim from app.js; if you change them there, change them here.
//
// Run: node --test experiments/reveal-budget-logic.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";

const PREFERENCE_DEFAULT_BUDGET_MS = 2000;
const MIN_MESSAGE_ANIMATION_MAX_MS = 8000;

// Mirror of normalizeAnimationBudgetMs (app.js).
function normalizeAnimationBudgetMs(value) {
  const number = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(number)) return PREFERENCE_DEFAULT_BUDGET_MS;
  const clamped = Math.min(Math.max(number, 0), MIN_MESSAGE_ANIMATION_MAX_MS);
  return Math.round(clamped);
}

// Mirror of the scheduling inside useMessageReveal (app.js). Returns the ordered
// list of {atMs, kind, value} events that would fire for a given step count and
// budget, so we can assert ordering and the under-budget guarantee.
function scheduleReveal(stepCount, budgetMs, reducedMotion = false) {
  const active = budgetMs > 0 && stepCount > 0 && !reducedMotion;
  if (!active) {
    return { active, events: [{ atMs: 0, kind: "all", value: stepCount }] };
  }
  const stepsWindow = budgetMs * 0.72;
  const perStep = stepsWindow / stepCount;
  const events = [{ atMs: 0, kind: "steps", value: 1 }];
  for (let index = 1; index < stepCount; index += 1) {
    events.push({
      atMs: Math.round(perStep * index),
      kind: "steps",
      value: index + 1,
    });
  }
  events.push({ atMs: Math.round(budgetMs), kind: "body", value: true });
  return { active, events };
}

test("normalizeAnimationBudgetMs clamps, rounds, and defaults", () => {
  assert.equal(normalizeAnimationBudgetMs(2000), 2000);
  assert.equal(normalizeAnimationBudgetMs(0), 0, "0 = immediate is preserved");
  assert.equal(normalizeAnimationBudgetMs(-500), 0, "negatives clamp to 0");
  assert.equal(normalizeAnimationBudgetMs(99999), MIN_MESSAGE_ANIMATION_MAX_MS);
  assert.equal(normalizeAnimationBudgetMs(1234.6), 1235, "rounds to int ms");
  assert.equal(normalizeAnimationBudgetMs("1500"), 1500, "coerces strings");
  assert.equal(
    normalizeAnimationBudgetMs("nonsense"),
    PREFERENCE_DEFAULT_BUDGET_MS,
    "non-finite falls back to the default",
  );
  assert.equal(normalizeAnimationBudgetMs(undefined), PREFERENCE_DEFAULT_BUDGET_MS);
});

test("budget 0 is an immediate no-op (everything shown at once)", () => {
  const { active, events } = scheduleReveal(4, 0);
  assert.equal(active, false);
  assert.deepEqual(events, [{ atMs: 0, kind: "all", value: 4 }]);
});

test("reduced motion forces an immediate no-op even with a budget", () => {
  const { active } = scheduleReveal(4, 2000, true);
  assert.equal(active, false);
});

test("no steps => no-op (cannot reveal a trace that does not exist)", () => {
  const { active } = scheduleReveal(0, 2000);
  assert.equal(active, false);
});

test("R6: steps reveal before the body, body strictly last", () => {
  const { events } = scheduleReveal(3, 2000);
  const bodyEvent = events.find((e) => e.kind === "body");
  const stepEvents = events.filter((e) => e.kind === "steps");
  assert.equal(stepEvents.length, 3, "one event per step");
  // Steps revealed in strictly increasing order 1,2,3.
  assert.deepEqual(stepEvents.map((e) => e.value), [1, 2, 3]);
  // Every step is revealed at or before the body appears.
  for (const s of stepEvents) {
    assert.ok(s.atMs <= bodyEvent.atMs, `step@${s.atMs} <= body@${bodyEvent.atMs}`);
  }
});

test("R5/R6: the whole sequence stays within the budget", () => {
  for (const budget of [250, 1000, 2000, 6000]) {
    for (const steps of [1, 2, 5, 12]) {
      const { events } = scheduleReveal(steps, budget);
      const last = events[events.length - 1];
      assert.ok(
        last.atMs <= budget,
        `final event @${last.atMs}ms must be <= budget ${budget}ms (steps=${steps})`,
      );
      assert.equal(last.kind, "body", "the body is always the final event");
    }
  }
});

test("R5: even a single step gets a perceptible beat before the body", () => {
  // 2s budget, 1 step: step shows at 0, body at 2000 -> a full 2s 'thinking' beat.
  const { events } = scheduleReveal(1, 2000);
  assert.deepEqual(events, [
    { atMs: 0, kind: "steps", value: 1 },
    { atMs: 2000, kind: "body", value: true },
  ]);
});
