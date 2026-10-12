// R1188-U29 / R1188-U30: Rust specification tests of the simple shape carried
// to JavaScript (scripts/lib/rust-specification-cases.mjs) and asked of the
// browser worker (scripts/check-specification-in-javascript.mjs).
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";

import { caseOf, checkFailure, promptHelper, testFunctions, tokenize } from "../../../scripts/lib/rust-specification-cases.mjs";

const SOURCE = `
fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn greets() {
    let response = answer("Hi \\u{1F600}");
    assert_eq!(response.intent, "greeting", "got {}", response.intent);
    assert!(
        response.answer.to_lowercase().contains("hello"),
    );
    assert!(!response.answer.contains("error")); // a comment
    assert!(response.answer.contains("a") || response.answer.contains("b"));
    assert!(response.evidence_links.iter().any(|link| link.starts_with("rule:")));
}

#[test]
#[ignore = "downloads a payload"]
fn downloads() {}

#[test]
fn loops() {
    for prompt in ["a", "b"] {
        let _ = answer(prompt);
    }
}
`;

test("a test of the simple shape becomes a case with its prompt and checks", () => {
  const [greets, loops] = testFunctions(tokenize(SOURCE));
  assert.equal(promptHelper(SOURCE), "answer");
  assert.deepEqual(caseOf(greets.body, "answer").case, {
    asks: [{ binding: "response", prompt: "Hi 😀" }],
    checks: [
      { binding: "response", field: "intent", value: "greeting", lower: false, op: "equals" },
      { binding: "response", field: "content", value: "hello", lower: true, op: "includes" },
      { binding: "response", field: "content", value: "error", lower: false, op: "excludes" },
      {
        op: "any",
        checks: [
          { binding: "response", field: "content", value: "a", lower: false, op: "includes" },
          { binding: "response", field: "content", value: "b", lower: false, op: "includes" },
        ],
      },
      { binding: "response", field: "evidence", value: "rule:", lower: false, op: "has-evidence-prefix" },
    ],
  });
  assert.equal(loops.name, "loops", "an ignored test is left out");
  assert.equal(caseOf(loops.body, "answer").reason, "for prompt in [");
});

test("a solver bound once and asked through solve is carried too", () => {
  const [solves] = testFunctions(tokenize(`#[test]
fn solves() {
    let solver = UniversalSolver::default();
    let text = solver.solve("2 + 2").answer;
    assert!(text.contains("4"));
}`));
  assert.deepEqual(caseOf(solves.body, null).case.asks, [{ binding: "text", prompt: "2 + 2" }]);
});

test("a check reports what was expected and what came", () => {
  const responses = new Map([["r", { content: "Hello there", intent: "greeting", evidence: ["rule:greeting"] }]]);
  const leaf = (field, op, value, lower = false) => ({ binding: "r", field, op, value, lower });
  assert.equal(checkFailure(leaf("content", "includes", "hello", true), responses), null);
  assert.equal(checkFailure(leaf("evidence", "has-evidence", "rule:greeting"), responses), null);
  assert.equal(checkFailure(leaf("intent", "equals", "farewell"), responses), 'intent equals "farewell"; got "greeting"');
  const either = { op: "any", checks: [leaf("content", "includes", "Bye"), leaf("content", "includes", "there")] };
  assert.equal(checkFailure(either, responses), null);
  const both = { op: "all", checks: [leaf("content", "includes", "Bye"), leaf("content", "includes", "there")] };
  assert.equal(checkFailure(both, responses), 'content includes "Bye"; got "Hello there"');
});

test("the worker passes every carried case the record does not list as a gap", () => {
  execFileSync(process.execPath, ["scripts/check-specification-in-javascript.mjs", "--check"], { stdio: "pipe" });
});
