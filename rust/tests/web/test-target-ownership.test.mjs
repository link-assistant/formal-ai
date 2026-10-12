import assert from "node:assert/strict";
import { before, test } from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { installNodeHost } from "../../../js/agentic/node-host.mjs";
import { planTestExpectationQuestion } from "../../../js/agentic/function_expectation.mjs";
import { classifyRepositoryOperation } from "../../../js/repository-workspace/observation.mjs";
before(async () => {
  await installNodeHost(new WorkerHost());
});
for (const prompt of [
  "Implement a resolver in output.mjs with meaningful tests.",
  "Write a program in result.py with regressions.",
  "Implement a method in folder/方法-α.mjs with tests.",
  "Create output.mjs. Run node --test checks.test.mjs.",
])
  test(
    "acceptance artifact does not rebind the named source: " + prompt,
    () => {
      assert.equal(planTestExpectationQuestion(prompt), null);
    },
  );
for (const prompt of [
  "Create a Python test for add in test_m.py and run it.",
  "Create the test in missing.test.mjs.",
  "Write tests in checks.mjs.",
])
  test("owned underspecified test retains clarification: " + prompt, () => {
    assert.equal(
      planTestExpectationQuestion(prompt).result.disposition,
      "clarification",
    );
  });
test("explicit expectations and quoted source retain their original guards", () => {
  assert.equal(
    planTestExpectationQuestion(
      "Create tests in checks.mjs that add(2,3) returns 5.",
    ),
    null,
  );
  assert.equal(
    planTestExpectationQuestion(
      "Create checks.mjs containing «const tests = 1;».",
    ),
    null,
  );
});
test("plural inspection support remains bound to the query goal", () => {
  assert.equal(
    classifyRepositoryOperation("Where do unit tests live in this repository?")
      .kind,
    "test-targets",
  );
  assert.equal(
    classifyRepositoryOperation("Where are regressions?").kind,
    "test-targets",
  );
});
