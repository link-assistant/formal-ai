// Issue #12 (R71): every current and tracked feature the holistic vision names
// is pinned by an executable specification test under
// rust/tests/unit/specification/, and none of those tests is parked as an
// ignored placeholder for a feature still to come. The only ignores allowed
// there are opt-in runs that download or execute external benchmark payloads.

import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { REPO_ROOT } from "./support/browser-runtime.mjs";

const SPECIFICATION = path.join(REPO_ROOT, "rust/tests/unit/specification");

const FEATURES = [
  "chat_surface",
  "code_generation",
  "multilingual",
  "openai_compatibility",
  "telegram_surface",
  "links_network",
  "reasoning_loop",
  "source_cache",
  "agent_isolation",
  "translation_via_links",
  "network_visualization",
  "transparent_state",
];

/** Every Rust file of a specification module: `name.rs` plus a `name/` directory. */
function moduleSources(name) {
  const files = [];
  const file = path.join(SPECIFICATION, `${name}.rs`);
  if (existsSync(file)) files.push(file);
  const directory = path.join(SPECIFICATION, name);
  if (existsSync(directory) && statSync(directory).isDirectory()) {
    for (const entry of readdirSync(directory, { recursive: true })) {
      if (String(entry).endsWith(".rs")) files.push(path.join(directory, String(entry)));
    }
  }
  return files;
}

function everySpecificationFile(directory = SPECIFICATION) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) return everySpecificationFile(full);
    return entry.name.endsWith(".rs") ? [full] : [];
  });
}

test("R71: each vision feature is a compiled specification module with tests", () => {
  const declared = readFileSync(path.join(SPECIFICATION, "mod.rs"), "utf8");
  for (const feature of FEATURES) {
    assert.match(declared, new RegExp(`^mod ${feature};$`, "mu"), `${feature} is not declared in specification/mod.rs`);
    const tests = moduleSources(feature)
      .map((file) => (readFileSync(file, "utf8").match(/#\[test\]/gu) || []).length)
      .reduce((sum, count) => sum + count, 0);
    assert.ok(tests > 0, `${feature} has no #[test]`);
  }
});

test("R71: no specification test is ignored as a placeholder for a pending feature", () => {
  const ignores = everySpecificationFile().flatMap((file) =>
    [...readFileSync(file, "utf8").matchAll(/#\[ignore(?: = "([^"]*)")?\]/gu)].map((match) => ({
      file: path.relative(REPO_ROOT, file),
      reason: match[1] ?? "",
    })),
  );
  for (const { file, reason } of ignores) {
    assert.match(reason, /download|network|fetch|execut/iu, `${file}: #[ignore] must name an opt-in external run, got "${reason}"`);
    assert.doesNotMatch(reason, /\b(?:tracked|pending|todo|not yet|unimplemented)\b/iu, `${file}: "${reason}"`);
  }
});
