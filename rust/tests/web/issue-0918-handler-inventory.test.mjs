// R918-2: every compiled handler source is inventoried with a disposition
// (migrate, promote or delete) and a reason (data/meta/core-boundary-ledger.lino,
// read by scripts/check-minimal-core-boundary.mjs).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parseLedger, sourceFiles } from "../../../scripts/check-minimal-core-boundary.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const ledger = parseLedger(readFileSync(join(ROOT, "data/meta/core-boundary-ledger.lino"), "utf8"));
const byPath = new Map(ledger.entries.map((entry) => [entry.path, entry]));

test("every handler source under the recursive walk has a ledger entry", () => {
  const missing = [...sourceFiles(ROOT).keys()].filter((path) => !byPath.has(path));
  assert.deepEqual(missing, []);
});

test("every entry records migrate, promote or delete with a reason", () => {
  for (const entry of ledger.entries) {
    assert.match(entry.disposition, /^(migrate|promote|delete)$/u, entry.path);
    assert.notEqual(entry.reason, "", `${entry.path} has no reason`);
  }
});

test("a migrate entry names its data target and a promote entry its core component", () => {
  for (const entry of ledger.entries) {
    if (entry.disposition === "migrate") {
      assert.notEqual(entry.dataTarget, "", `${entry.path} has no data_target`);
    }
    if (entry.disposition === "promote") {
      assert.notEqual(entry.coreComponent, "", `${entry.path} has no core_component`);
    }
  }
});
