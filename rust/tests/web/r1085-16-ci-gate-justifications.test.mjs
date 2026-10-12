// R1085-16: every CI gate must carry a `justification` that cites an issue/PR
// (#NNN) or a commit hash, and the pull-request pipeline's wall-clock ceiling
// must be recorded and respected. The ceiling itself is measured by
// scripts/check-ci-wall-clock.rs (the longest `needs:` path over every
// PR-triggered workflow, reusable workflows included, failing on any
// unbounded job), a registered rust-stage gate; this suite pins the record
// and that registration, with no installed dependency.

import { describe, test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, existsSync } from "node:fs";
import { join } from "node:path";

import { REPO_ROOT } from "./support/browser-runtime.mjs";

const GATES_DIR = join(REPO_ROOT, "data/meta/ci-gates");
const WALL_CLOCK_FILE = join(REPO_ROOT, "data/meta/ci-wall-clock.lino");
const WORKFLOWS_DIR = join(REPO_ROOT, ".github/workflows");

/** Return the unquoted value of a key at 2-space indent in a lino record. */
function extractField(content, key) {
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("#") || trimmed === "") continue;
    if (line.startsWith(`  ${key} `)) {
      const raw = line.slice(`  ${key} `.length).trim();
      return raw.startsWith('"') && raw.endsWith('"') ? raw.slice(1, -1) : raw;
    }
  }
  return null;
}

/** True when text contains `#NNN` (issue/PR citation). */
function hasIssueCitation(text) {
  return /#\d+/.test(text);
}

/** True when text contains a standalone run of 7-40 hex chars (commit hash). */
function hasCommitHash(text) {
  return /(?<![0-9a-fA-F])[0-9a-fA-F]{7,40}(?![0-9a-fA-F])/.test(text);
}

function hasCitation(text) {
  return hasIssueCitation(text) || hasCommitHash(text);
}

/**
 * Whether a workflow's `on:` block names the `pull_request` event, read
 * without a YAML parser: this tier installs no dependency.
 */
function workflowHasPrTrigger(text) {
  const inline = /^on:\s*\[([^\]]*)\]/mu.exec(text);
  if (inline) return /\bpull_request\b/u.test(inline[1]);
  if (/^on:\s*pull_request\s*$/mu.test(text)) return true;
  const block = /^on:\s*\n((?:[ \t#].*\n|\s*\n)*)/mu.exec(text);
  return block !== null && /^ {2}pull_request:?\s*$/mu.test(block[1]);
}

describe("R1085-16 CI gate justifications", () => {
  const gateFiles = readdirSync(GATES_DIR).filter((f) => f.endsWith(".lino"));

  test("at least 65 gate files exist", () => {
    assert.ok(
      gateFiles.length >= 65,
      `expected >=65 gates, found ${gateFiles.length}`,
    );
  });

  for (const filename of gateFiles) {
    test(`${filename} has a justification citing #NNN or a commit`, () => {
      const content = readFileSync(join(GATES_DIR, filename), "utf8");
      const justification = extractField(content, "justification");
      assert.ok(
        justification !== null && justification.length > 0,
        `${filename}: missing justification field`,
      );
      assert.ok(
        hasCitation(justification),
        `${filename}: justification must cite an issue/PR (#NNN) or a commit hash; got: "${justification}"`,
      );
    });
  }
});

describe("R1085-16 wall-clock ceiling", () => {
  const record = readFileSync(WALL_CLOCK_FILE, "utf8");

  test("data/meta/ci-wall-clock.lino records measured <= ceiling", () => {
    const measured = Number.parseInt(extractField(record, "measured_minutes"), 10);
    const ceiling = Number.parseInt(extractField(record, "ceiling_minutes"), 10);
    assert.ok(Number.isInteger(measured) && Number.isInteger(ceiling), record);
    assert.ok(measured <= ceiling, `measured_minutes (${measured}) exceeds ceiling_minutes (${ceiling})`);
  });

  test("the record names the PR-triggered workflow its critical path comes from", () => {
    const workflow = extractField(record, "workflow");
    assert.ok(workflow, "ci-wall-clock.lino: missing workflow field");
    const path = join(REPO_ROOT, workflow);
    assert.ok(existsSync(path), `${workflow} does not exist`);
    assert.ok(workflowHasPrTrigger(readFileSync(path, "utf8")), `${workflow} has no pull_request trigger`);
    assert.ok(extractField(record, "chain"), "ci-wall-clock.lino: missing chain field");
  });

  test("the measuring script is a registered rust-stage gate", () => {
    const gate = readFileSync(join(GATES_DIR, "check-ci-wall-clock.lino"), "utf8");
    assert.match(gate, /^ {2}stage rust$/mu);
    assert.match(gate, /rust-script --test scripts\/check-ci-wall-clock\.rs && rust-script scripts\/check-ci-wall-clock\.rs/u);
    assert.ok(existsSync(join(REPO_ROOT, "scripts/check-ci-wall-clock.rs")));
  });

  test("the pull_request trigger reader sees the release pipeline and skips a push-only workflow", () => {
    assert.ok(workflowHasPrTrigger(readFileSync(join(WORKFLOWS_DIR, "release.yml"), "utf8")));
    assert.ok(!workflowHasPrTrigger("on:\n  push:\n    branches: [main]\njobs: {}\n"));
    assert.ok(workflowHasPrTrigger("on: [push, pull_request]\n"));
  });
});
