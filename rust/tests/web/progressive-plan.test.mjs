// R1188-U28 / R1188-U29: requirements on resolution levels, JavaScript first
// (scripts/render-progressive-plan.mjs, docs/progressive-delivery.md).
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";

import {
  holdRatchet,
  ledgerRecords,
  levelOf,
  measure,
  ratchetValues,
  renderPlan,
  renderRatchet,
  rootOf,
} from "../../../scripts/render-progressive-plan.mjs";

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

const RATCHET = {
  requirements: 3,
  "recorded-ceiling": 1,
  "measured-ceiling": 0,
  "partial-ceiling": 1,
  "rust-only-ceiling": 1,
};

test("the ratchet measures each open level and the Rust-only count", () => {
  assert.deepEqual(measure(ledgerRecords(LEDGER)), RATCHET);
  assert.deepEqual(ratchetValues(renderRatchet(RATCHET)), RATCHET);
});

test("a lower level that grows while a higher one is refined fails", () => {
  const raised = { ...RATCHET, "recorded-ceiling": 2, "partial-ceiling": 0 };
  const { grown, next } = holdRatchet(raised, RATCHET);
  assert.equal(grown.length, 1);
  assert.match(grown[0], /level 1 \(recorded\) grew by 1 beyond the 0 requirement/u);
  assert.equal(next["recorded-ceiling"], 1);
  assert.equal(next["partial-ceiling"], 0);
});

test("a newly recorded requirement may enter a low level, and a fallen ceiling is lowered", () => {
  const recorded = { ...RATCHET, requirements: 4, "recorded-ceiling": 2, "partial-ceiling": 0 };
  assert.deepEqual(holdRatchet(recorded, RATCHET), {
    grown: [],
    next: { ...RATCHET, requirements: 4, "recorded-ceiling": 2, "partial-ceiling": 0 },
  });
});

test("the Rust-only count never grows, even with new requirements", () => {
  const rustOnly = { ...RATCHET, requirements: 4, "rust-only-ceiling": 2 };
  assert.match(holdRatchet(rustOnly, RATCHET).grown.join("\n"), /1 more requirement\(s\) pinned only by a Rust test/u);
});

test("the committed plan matches the ledger", () => {
  execFileSync(process.execPath, ["scripts/render-progressive-plan.mjs", "--check"], { stdio: "pipe" });
});
