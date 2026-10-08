// R1188-U28 / R1188-U29: requirements on resolution levels, JavaScript first
// (scripts/render-progressive-plan.mjs, docs/progressive-delivery.md).
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";

import { ledgerRecords, levelOf, renderPlan, rootOf } from "../../../scripts/render-progressive-plan.mjs";

const LEDGER = [
  "requirement_status_ledger_shard",
  '  shard "docs/requirements/issue-1.md"',
  '  verdict "implemented"',
  "  requirement",
  '    id "R1-1"',
  '    automated_test "rust/tests/web/a.test.mjs"',
  "  requirement",
  '    id "R1-2"',
  '    verdict "partial"',
  '    automated_test "rust/tests/unit/b.rs"',
  "  requirement",
  '    id "R1-3"',
  '    verdict "not-delivered"',
  "",
].join("\n");

test("records inherit the file's values unless they state their own", () => {
  assert.deepEqual(ledgerRecords(LEDGER), [
    { id: "R1-1", shard: "docs/requirements/issue-1.md", verdict: "implemented", test: "rust/tests/web/a.test.mjs" },
    { id: "R1-2", shard: "docs/requirements/issue-1.md", verdict: "partial", test: "rust/tests/unit/b.rs" },
    { id: "R1-3", shard: "docs/requirements/issue-1.md", verdict: "not-delivered", test: "" },
  ]);
});

test("a level is the verdict, and a test lifts an undelivered row to measured", () => {
  assert.equal(levelOf({ verdict: "implemented", test: "x.rs" }), 4);
  assert.equal(levelOf({ verdict: "partial", test: "" }), 3);
  assert.equal(levelOf({ verdict: "not-delivered", test: "x.test.mjs" }), 2);
  assert.equal(levelOf({ verdict: "not-delivered", test: "" }), 1);
  assert.equal(levelOf({ verdict: "superseded", test: "" }), 0);
  assert.equal(levelOf({ verdict: "withdrawn", test: "x.rs" }), 0);
});

test("the pinning root is read from the test file", () => {
  assert.equal(rootOf({ test: "rust/tests/web/a.test.mjs" }), "javascript");
  assert.equal(rootOf({ test: "rust/tests/unit/b.rs" }), "rust");
  assert.equal(rootOf({ test: "" }), "none");
});

test("the next pass is the lowest non-empty level, listed by shard", () => {
  const page = renderPlan(ledgerRecords(LEDGER));
  assert.match(page, /\| 1 recorded \| the row exists, nothing measures it yet \| 1 \|/u);
  assert.match(page, /\| a JavaScript test \| 1 \|\n\| a Rust test only \| 1 \|\n\| no test \| 1 \|/u);
  assert.match(page, /Level 1 \(recorded\): 1 requirement\(s\) to raise/u);
  assert.match(page, /- `docs\/requirements\/issue-1\.md`: R1-3/u);
});

test("the committed plan matches the ledger", () => {
  execFileSync(process.execPath, ["scripts/render-progressive-plan.mjs", "--check"], { stdio: "pipe" });
});
