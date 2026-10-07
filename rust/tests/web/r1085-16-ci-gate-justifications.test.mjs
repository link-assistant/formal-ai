// R1085-16: every CI gate must carry a `justification` that cites an issue/PR
// (#NNN) or a commit hash, and the pull-request pipeline's wall-clock ceiling
// must be recorded and respected.
//
// Pinned by rust/tests/web/r1085-16-ci-gate-justifications.test.mjs.

import { describe, test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import yaml from "yaml";

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
 * Return true if the workflow's `on:` section includes `pull_request`.
 * serde_yaml / js-yaml may parse the YAML `on:` key as either the string
 * "on" or the boolean true depending on the YAML 1.1 vs 1.2 parser.
 */
function workflowHasPrTrigger(wf) {
  const on = wf["on"] ?? wf[true] ?? null;
  if (on === null) return false;
  if (on === "pull_request") return true;
  if (Array.isArray(on)) return on.includes("pull_request");
  if (typeof on === "object") return "pull_request" in on;
  return false;
}

/**
 * Return true when a job's `if:` condition is restricted purely to non-PR
 * events and has no output-based escape that would allow it on a PR.
 */
function isJobExcludedOnPr(ifCondition) {
  if (!ifCondition) return false;
  const s = String(ifCondition);
  if (s.includes("pull_request")) return false;
  if (s.includes(".outputs.")) return false;
  const nonPr = [
    "github.event_name == 'push'",
    'github.event_name == "push"',
    "github.event_name == 'workflow_dispatch'",
    'github.event_name == "workflow_dispatch"',
    "github.event_name == 'release'",
    'github.event_name == "release"',
    "github.event_name == 'schedule'",
    'github.event_name == "schedule"',
  ];
  return nonPr.some((p) => s.includes(p));
}

/**
 * Find PR-reachable jobs in a workflow that have no timeout-minutes and are
 * not calling a reusable workflow. Returns an array of job names.
 */
function findUnboundedJobs(jobs) {
  const unbounded = [];
  for (const [name, job] of Object.entries(jobs ?? {})) {
    if (job == null) continue;
    if (isJobExcludedOnPr(job.if ?? "")) continue;
    if ("uses" in job) continue; // reusable workflow; timeout is in the callee
    if (!("timeout-minutes" in job)) {
      unbounded.push(name);
    } else {
      // Matrix expression like ${{ matrix.capmin }}: check if it can be
      // resolved from the static include list.
      const t = job["timeout-minutes"];
      if (typeof t === "string") {
        const m = t.match(/^\$\{\{\s*matrix\.(\w+)\s*\}\}$/);
        if (m) {
          const key = m[1];
          const include =
            job.strategy?.matrix?.include ?? [];
          const hasNumericValue = include.some(
            (row) => typeof row === "object" && typeof row[key] === "number",
          );
          if (!hasNumericValue) {
            unbounded.push(`${name}(DYNAMIC:${t})`);
          }
        }
      }
    }
  }
  return unbounded;
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
  test("data/meta/ci-wall-clock.lino is present and measured <= ceiling", () => {
    assert.ok(
      existsSync(WALL_CLOCK_FILE),
      "data/meta/ci-wall-clock.lino is missing",
    );
    const content = readFileSync(WALL_CLOCK_FILE, "utf8");
    const measured = extractField(content, "measured_minutes");
    const ceiling = extractField(content, "ceiling_minutes");
    assert.ok(measured !== null, "ci-wall-clock.lino: missing measured_minutes");
    assert.ok(ceiling !== null, "ci-wall-clock.lino: missing ceiling_minutes");
    const m = parseInt(measured, 10);
    const c = parseInt(ceiling, 10);
    assert.ok(
      !Number.isNaN(m) && !Number.isNaN(c),
      `measured_minutes and ceiling_minutes must be integers; got "${measured}", "${ceiling}"`,
    );
    assert.ok(
      m <= c,
      `measured_minutes (${m}) must not exceed ceiling_minutes (${c})`,
    );
  });

  test("ci-wall-clock.lino names a PR-triggered workflow that exists", () => {
    const content = readFileSync(WALL_CLOCK_FILE, "utf8");
    const wfField = extractField(content, "workflow");
    assert.ok(
      wfField !== null && wfField.length > 0,
      "ci-wall-clock.lino: missing workflow field",
    );
    const wfPath = join(REPO_ROOT, wfField);
    assert.ok(
      existsSync(wfPath),
      `ci-wall-clock.lino: workflow "${wfField}" does not exist at ${wfPath}`,
    );
    const wf = yaml.parse(readFileSync(wfPath, "utf8"));
    assert.ok(
      workflowHasPrTrigger(wf),
      `ci-wall-clock.lino: workflow "${wfField}" does not have a pull_request trigger`,
    );
  });

  test("no PR-triggered workflow has an unbounded job (no timeout-minutes)", () => {
    const wfFiles = readdirSync(WORKFLOWS_DIR).filter((f) =>
      f.endsWith(".yml"),
    );
    const unboundedByWorkflow = {};
    for (const filename of wfFiles) {
      let wf;
      try {
        wf = yaml.parse(
          readFileSync(join(WORKFLOWS_DIR, filename), "utf8"),
        );
      } catch {
        continue;
      }
      if (!workflowHasPrTrigger(wf)) continue;
      const unbounded = findUnboundedJobs(wf.jobs ?? {});
      if (unbounded.length > 0) {
        unboundedByWorkflow[filename] = unbounded;
      }
    }
    assert.deepStrictEqual(
      unboundedByWorkflow,
      {},
      `PR-triggered workflows have unbounded jobs (no timeout-minutes):\n${JSON.stringify(unboundedByWorkflow, null, 2)}`,
    );
  });
});
