// Issue #16 follow-up (R105): `js/seed/` is a deploy artefact rebuilt from
// the canonical `data/seed/`. It is gitignored, never tracked, and every
// surface that serves the web app regenerates it first with
// scripts/sync-seed.sh. The ignore entry once named the pre-move
// `rust/src/web/seed/`, which left the regenerated mirror untracked but not
// ignored; this suite fails if the entry and the sync target drift apart.

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { REPO_ROOT } from "./support/browser-runtime.mjs";

const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");
const git = (...args) => execFileSync("git", args, { cwd: REPO_ROOT, encoding: "utf8" });

test("R105: scripts/sync-seed.sh copies data/seed into js/seed", () => {
  const script = read("scripts/sync-seed.sh");
  assert.match(script, /^SRC_DIR="\$ROOT_DIR\/data\/seed"$/mu);
  assert.match(script, /^DEST_DIR="\$ROOT_DIR\/js\/seed"$/mu);
  assert.match(script, /--check/u, "the script offers a divergence check");
});

test("R105: the mirror is gitignored and has no tracked copy", () => {
  assert.match(git("check-ignore", "-v", "js/seed/agent-info.lino"), /^\.gitignore:\d+:js\/seed\/\t/u);
  assert.equal(git("ls-files", "js/seed").trim(), "");
});

test("R105: every web-serving surface regenerates the mirror before serving js/", () => {
  for (const config of ["rust/tests/e2e/playwright.local.config.js", "rust/tests/e2e/playwright.adhoc.config.js"]) {
    assert.match(read(config), /sync-seed\.sh && npx serve \.\.\/\.\.\/\.\.\/js /u, config);
  }
  // The Pages bundle is built by the reusable pages-artifact.yml, which
  // release.yml calls (PR #1188 R1188-U9 moved the build out of release.yml).
  for (const workflow of [".github/workflows/e2e-local.yml", ".github/workflows/pages-artifact.yml"]) {
    assert.match(read(workflow), /run: scripts\/sync-seed\.sh$/mu, workflow);
  }
  assert.match(read(".github/workflows/release.yml"), /uses: \.\/\.github\/workflows\/pages-artifact\.yml$/mu);
});
