// R1188-U29: the requirement-status ledger cites the JavaScript test of a row
// that names tests in both roots (scripts/generate-requirement-status.mjs
// `testPath`, twin of `test_path` in scripts/generate-requirement-status.rs).
import assert from "node:assert/strict";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { testPath } from "../../../scripts/generate-requirement-status.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const RUST = "rust/tests/unit/minimal_core_and_seed_metadata.rs";
const JAVASCRIPT = "rust/tests/web/issue-0918-handler-inventory.test.mjs";

test("a row naming a Rust test and then a JavaScript test cites the JavaScript one", () => {
  assert.equal(testPath(`| R1-1 | x | Pinned by \`${RUST}\` and \`${JAVASCRIPT}\`. |`, ROOT), JAVASCRIPT);
});

test("a row naming only Rust tests cites the first of them", () => {
  assert.equal(testPath(`| R1-1 | x | Pinned by \`${RUST}\` and \`rust/tests/unit/concise_lexemes.rs\`. |`, ROOT), RUST);
});

test("a named test that is not on disk is not cited", () => {
  assert.equal(testPath("| R1-1 | x | Pinned by `rust/tests/web/no-such.test.mjs`. |", ROOT), "");
  assert.equal(testPath(`| R1-1 | x | \`rust/tests/web/no-such.test.mjs\`, \`${RUST}\` |`, ROOT), RUST);
});
