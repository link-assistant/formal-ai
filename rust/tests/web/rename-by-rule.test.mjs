// The rename tool applies data/meta/rename-map.lino by rule (PR #1188,
// R1188-U5): it derives each rename's companions, finds every reference to an
// old path, leaves alone a token that names a different file with the same
// tail, renames Rust modules, and re-sorts generated lists.

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  companionMoves,
  findModuleReferences,
  findReferences,
  indexByBasename,
  parseRenameMap,
  renameRustModules,
  renamedToken,
  resortQuotedLists,
  tokenNames,
} from "../../../experiments/formal_ai_subagent/rename-by-rule.mjs";

const WORKER = { from: "js/worker/old_module.js", to: "js/worker/seed_module.js" };

test("the map states each rename once, with its tree, reason and exclusions", () => {
  const map = parseRenameMap([
    "# comment",
    "rename-map",
    '  exclude "dev/log/"',
    '  resort "js/worker-modules.js"',
    "  tree browser-worker-modules",
    "    rename",
    `      from "${WORKER.from}"`,
    `      to "${WORKER.to}"`,
    '      reason "Seed hydration."',
  ].join("\n"));
  assert.deepEqual(map.exclude, ["dev/log/"]);
  assert.deepEqual(map.resort, ["js/worker-modules.js"]);
  assert.deepEqual(map.renames, [{ tree: "browser-worker-modules", ...WORKER, reason: "Seed hydration.", review: "" }]);
});

test("a worker module brings its TypeScript twin and line-budget shard; a Rust source its census", () => {
  const existing = new Set(["ts/worker/old_module.ts", "data/meta/worker-line-budget/old_module.lino"]);
  assert.deepEqual(companionMoves(WORKER, (path) => existing.has(path)).map((move) => [move.to, move.companion]), [
    [WORKER.to, ""],
    ["ts/worker/seed_module.ts", "typescript twin"],
    ["data/meta/worker-line-budget/seed_module.lino", "worker line budget"],
  ]);
  const rust = { from: "rust/src/solver_x.rs", to: "rust/src/solver_routes.rs" };
  assert.deepEqual(companionMoves(rust, (path) => path === "data/meta/self-ast/src/solver_x.lino").map((move) => move.to), [
    "rust/src/solver_routes.rs",
    "data/meta/self-ast/src/solver_routes.lino",
  ]);
});

test("a reference is the whole path, a tail no other file shares, or a relative path that resolves to it", () => {
  const byBasename = indexByBasename([WORKER.from, "rust/tests/unit/a/issue_1.rs", "rust/tests/unit/issue_1.rs"]);
  assert.ok(tokenNames("js/worker/old_module.js", "docs/x.md", WORKER.from, byBasename));
  assert.ok(tokenNames("worker/old_module.js", "js/worker-modules.js", WORKER.from, byBasename));
  assert.ok(tokenNames("old_module.js", "data/meta/worker-line-budget/x.lino", WORKER.from, byBasename));
  assert.ok(tokenNames("../../../js/worker/old_module.js", "rust/tests/unit/x.rs", WORKER.from, byBasename));
  assert.ok(!tokenNames("src/web/worker/old_module.js", "experiments/x.mjs", WORKER.from, byBasename));
  // Two files share the tail `issue_1.rs`: only the one next to the reference is named.
  assert.ok(tokenNames("issue_1.rs", "rust/tests/unit/mod.rs", "rust/tests/unit/issue_1.rs", byBasename));
  assert.ok(!tokenNames("issue_1.rs", "rust/tests/unit/a/mod.rs", "rust/tests/unit/issue_1.rs", byBasename));
});

test("a reference keeps its own prefix and as many segments as it had", () => {
  assert.equal(renamedToken("../../../js/worker/old_module.js", WORKER.from, WORKER.to), "../../../js/worker/seed_module.js");
  assert.equal(renamedToken("old_module.js", WORKER.from, WORKER.to), "seed_module.js");
  const moved = { from: "js/worker/a.js", to: "js/browser/b.js" };
  assert.equal(renamedToken("worker/a.js", moved.from, moved.to), "browser/b.js");
  assert.equal(renamedToken("a.js", moved.from, moved.to), "b.js");
});

test("references are found with their boundaries, never inside a longer name", () => {
  const text = [
    "`js/worker/old_module.js:tryArithmetic`",
    "old_module.json",
    "my_old_module.js",
    "include_str!(\"../../../js/worker/old_module.js\")",
  ].join("\n");
  const found = findReferences(text, "rust/tests/unit/x.rs", [WORKER], indexByBasename([WORKER.from]));
  assert.deepEqual(found.map((reference) => reference.token), [
    "js/worker/old_module.js",
    "../../../js/worker/old_module.js",
  ]);
});

test("a Rust rename changes the parent's mod line and the module's paths", () => {
  const move = { from: "rust/tests/unit/issue_745.rs", to: "rust/tests/unit/file_write_routing.rs" };
  const byBasename = indexByBasename([move.from]);
  const parent = renameRustModules("mod issue_744;\nmod issue_745;\npub mod issue_7450;\n", "rust/tests/unit/mod.rs", [move], byBasename);
  assert.equal(parent.text, "mod issue_744;\nmod file_write_routing;\npub mod issue_7450;\n");
  const user = renameRustModules("use crate::issue_745::helper;\n", "rust/tests/unit/other.rs", [move], byBasename);
  assert.equal(user.text, "use crate::file_write_routing::helper;\n");
  const elsewhere = renameRustModules("mod issue_745;\n", "rust/tests/unit/ci/mod.rs", [move], byBasename);
  assert.equal(elsewhere.text, "mod issue_745;\n", "a namesake module in another directory keeps its name");
});

test("a #[path] module declaration follows its file", () => {
  const move = { from: "rust/tests/unit/area/issue_9.rs", to: "rust/tests/unit/area/fixture_area_rule.rs" };
  const byBasename = indexByBasename([move.from]);
  const declared = renameRustModules('#[path = "fixture_area_rule.rs"]\nmod issue_9;\nmod issue_90;\n', "rust/tests/unit/area/list.rs", [move], byBasename);
  assert.equal(declared.text, '#[path = "fixture_area_rule.rs"]\nmod fixture_area_rule;\nmod issue_90;\n');
});

test("a test path names a module outside Rust: unique, qualified by its parent, or at a binary root", () => {
  const root = { from: "rust/tests/unit/issue_9.rs", to: "rust/tests/unit/fixture_root_rule.rs" };
  const nested = { from: "rust/tests/unit/ci-cd/issue_9.rs", to: "rust/tests/unit/ci-cd/fixture_nested_rule.rs" };
  const byBasename = indexByBasename([root.from, nested.from]);
  const durations = 'test "issue_9::a"\ntest "ci_cd::issue_9::b"\ntest "issue_90::c"\n';
  const renamed = renameRustModules(durations, "data/meta/test-durations.lino", [root, nested], byBasename);
  assert.equal(renamed.text, 'test "fixture_root_rule::a"\ntest "ci_cd::fixture_nested_rule::b"\ntest "issue_90::c"\n');
  assert.deepEqual(findModuleReferences(renamed.text, "data/meta/test-durations.lino", [root, nested], byBasename), []);
  // Two namesakes below other modules and no qualifier: the occurrence is left alone.
  const left = { from: "rust/tests/unit/a/issue_8.rs", to: "rust/tests/unit/a/fixture_left.rs" };
  const right = { from: "rust/tests/unit/b/issue_8.rs", to: "rust/tests/unit/b/fixture_right.rs" };
  const ambiguous = findModuleReferences("see issue_8::case", "docs/x.md", [left, right], indexByBasename([left.from, right.from]));
  assert.deepEqual(ambiguous, []);
});

test("a generated one-entry-per-line list is re-sorted after a rename", () => {
  const list = 'self.X = [\n  "worker/zeta.js",\n  "worker/alpha.js",\n];\n';
  assert.equal(resortQuotedLists(list), 'self.X = [\n  "worker/alpha.js",\n  "worker/zeta.js",\n];\n');
});
