// Issue #1064 (no silent release deferral): a push to main that carries a
// changelog fragment but changes no code used to skip lint, test and the
// binary build (all gated on `any-code-changed`), so `build` and Auto Release
// skipped too while Pipeline Status stayed green. Merge #1150 left
// `20260926_215107_issue-1149-coverage-budget.md` unreleased that way.
//
// The release workflow now asks scripts/detect-code-changes.rs to count a
// waiting fragment on main as a reason to run the pipeline. This pins the
// chain: the detector folds `release-pending` into `any-code-changed`, the
// release workflow opts in, and every job Auto Release needs to succeed is
// gated on `any-code-changed`.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");
const RELEASE = read(".github/workflows/release.yml");

function job(name) {
  const lines = RELEASE.split("\n");
  const start = lines.indexOf(`  ${name}:`);
  assert.ok(start >= 0, `release.yml has a ${name} job`);
  const end = lines.findIndex((line, index) => index > start && /^ {2}[^ #][^:]*:\s*$/.test(line));
  return lines.slice(start, end < 0 ? lines.length : end).join("\n");
}

test("the detector counts a waiting fragment on main as a pipeline change", () => {
  const detector = read("scripts/detect-code-changes.rs");
  const pending = detector.slice(detector.indexOf("fn release_pending("), detector.indexOf("fn changelog_fragments("));
  assert.match(pending, /event_name == "push"/);
  assert.match(pending, /git_ref == "refs\/heads\/main"/);
  assert.match(pending, /name != "README\.md"/);
  assert.match(detector, /fs::read_dir\("changelog\.d"\)/);
  assert.match(detector, /env::var\("RELEASE_PIPELINE"\)/);
  assert.match(detector, /if flags\.any_code_changed \|\| pending \{/);
  assert.match(detector, /set_output\("release-pending"/);
});

test("the release workflow opts in, and Auto Release's prerequisites follow any-code-changed", () => {
  const detect = job("detect-changes");
  assert.match(detect, /RELEASE_PIPELINE: 'true'/);
  assert.match(detect, /run: rust-script scripts\/detect-code-changes\.rs/);
  const auto = job("auto-release");
  assert.match(auto, /needs\.build\.result == 'success'/);
  assert.match(job("build"), /needs\.lint\.result == 'success' && needs\.test\.result == 'success'/);
  assert.match(job("build"), /needs: \[[^\]]*build-artifacts[^\]]*\]/);
  for (const name of ["lint", "test", "build-artifacts"]) {
    assert.match(job(name), /needs\.detect-changes\.outputs\.any-code-changed == 'true'/, `${name} runs when any-code-changed is true`);
  }
});

test("the other workflows that run the detector do not opt in", () => {
  for (const workflow of ["coverage.yml", "layered-ci.yml"]) {
    assert.doesNotMatch(read(`.github/workflows/${workflow}`), /RELEASE_PIPELINE/, `${workflow} keeps its own gating`);
  }
});
