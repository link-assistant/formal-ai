// Issue #491 (R491-C1 to R491-C3): balanced binary task splits and
// least-action solution scoring. Pins js/agentic/crate/least_action.mjs with
// the cases of its Rust twin's test, rust/tests/unit/issue_491_least_action.rs.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  isLessActionThan,
  leastActionPlan,
  leastActionSolution,
  plan,
  rankByLeastAction,
} from "../../../js/agentic/crate/least_action.mjs";

const subtask = (title, atomic, solutionSteps) => ({ title, atomic, solutionSteps });
const subtasks = (count, prefix, atomic, steps) =>
  Array.from({ length: count }, (_, index) => subtask(`${prefix}${index}`, atomic(index), steps));
const cost = (steps, codeUnits, computeMs, memoryKb) => ({ steps, codeUnits, computeMs, memoryKb });
const candidate = (id, solvesEntireRange, actionCost) => ({ id, solvesEntireRange, cost: actionCost });
const ids = (candidates) => candidates.map((item) => item.id);

test("the highest abstraction is always 1, 2, 4, 8", () => {
  const eight = plan(subtasks(8, "t", () => true, 1));
  assert.deepEqual(eight.levelCounts, [1, 2, 4, 8]);
  assert.equal(eight.smallestSubtasks, 8);
  assert.equal(eight.depth, 3);
  assert.deepEqual(plan(subtasks(5, "t", () => true, 1)).levelCounts, [1, 2, 4, 5]);
  assert.deepEqual(plan([subtask("root", true, 1)]).levelCounts, [1]);
});

test("the plan counts smallest subtasks and leaves open work uncosted", () => {
  const planned = plan([
    subtask("parse inputs", true, 2),
    subtask("solve constraints", true, 3),
    subtask("render output", false, 0),
    subtask("log the run", false, 0),
    subtask("clean up", false, 0),
  ]);
  assert.equal(planned.smallestSubtasks, 5);
  assert.equal(planned.solved, 2);
  assert.equal(planned.plannedSteps, 5);
  assert.deepEqual(planned.unhandled, ["render output", "log the run", "clean up"]);
});

test("a plan with fewer planned steps is less action; different counts do not compare", () => {
  const coarse = plan(subtasks(8, "t", () => true, 3));
  const left = plan(subtasks(6, "a", () => true, 2));
  const right = plan(subtasks(6, "b", () => true, 4));
  assert.equal(leastActionPlan(coarse, left), null);
  assert.equal(leastActionPlan(left, right).plannedSteps, 12);
  const open = plan(subtasks(6, "o", (index) => index < 3, 2));
  const moreOpen = plan(subtasks(6, "m", (index) => index < 1, 2));
  assert.equal(leastActionPlan(open, moreOpen).unhandled.length, 5);
});

test("solutions rank by least action, steps first", () => {
  const candidates = [
    candidate("fast", true, cost(3, 90, 900, 40)),
    candidate("small", true, cost(5, 10, 10, 1)),
    candidate("slow", true, cost(7, 50, 500, 5)),
  ];
  assert.deepEqual(ids(rankByLeastAction(candidates)), ["fast", "small", "slow"]);
  assert.equal(leastActionSolution(candidates).id, "fast");
});

test("a millisecond saved by an extra step still loses", () => {
  const candidates = [candidate("fewer_steps", true, cost(2, 40, 900, 9)), candidate("fewer_ms", true, cost(3, 40, 1, 1))];
  assert.equal(leastActionSolution(candidates).id, "fewer_steps");
  assert.equal(isLessActionThan(cost(2, 40, 900, 9), cost(3, 40, 1, 1)), true);
  assert.equal(isLessActionThan(cost(3, 40, 1, 1), cost(3, 40, 1, 1)), false);
});

test("the shortest code that fails the input range never ranks", () => {
  const candidates = [
    candidate("short_but_narrow", false, cost(1, 1, 1, 1)),
    candidate("covers_all_inputs", true, cost(6, 60, 60, 6)),
  ];
  assert.deepEqual(ids(rankByLeastAction(candidates)), ["covers_all_inputs"]);
  assert.equal(leastActionSolution([candidates[0]]), null);
});

test("equal costs break stably by id", () => {
  const same = cost(4, 4, 4, 4);
  assert.deepEqual(ids(rankByLeastAction([candidate("b", true, same), candidate("a", true, same)])), ["a", "b"]);
});
