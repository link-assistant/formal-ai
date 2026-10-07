// R1085-16: every CI gate must carry a `justification` that cites an issue/PR
// (#NNN) or a commit hash, and the pull-request pipeline's wall-clock ceiling
// must be recorded and respected.
//
// Pinned by rust/tests/web/r1085-16-ci-gate-justifications.test.mjs.

import { describe, test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { REPO_ROOT } from "./support/browser-runtime.mjs";

const GATES_DIR = join(REPO_ROOT, "data/meta/ci-gates");
const WALL_CLOCK_FILE = join(REPO_ROOT, "data/meta/ci-wall-clock.lino");
const WORKFLOW_FILE = join(REPO_ROOT, ".github/workflows/release.yml");

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

describe("R1085-16 CI gate justifications", () => {
  const gateFiles = readdirSync(GATES_DIR).filter((f) => f.endsWith(".lino"));

  test("at least 64 gate files exist", () => {
    assert.ok(gateFiles.length >= 64, `expected >=64 gates, found ${gateFiles.length}`);
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
  test("data/meta/ci-wall-clock.lino is present and has required fields", () => {
    const content = readFileSync(WALL_CLOCK_FILE, "utf8");
    const measured = extractField(content, "measured_minutes");
    const ceiling = extractField(content, "ceiling_minutes");
    const jobs = extractField(content, "critical_path_jobs");
    assert.ok(measured !== null, "ci-wall-clock.lino: missing measured_minutes");
    assert.ok(ceiling !== null, "ci-wall-clock.lino: missing ceiling_minutes");
    assert.ok(jobs !== null, "ci-wall-clock.lino: missing critical_path_jobs");
    assert.ok(
      parseInt(measured, 10) <= parseInt(ceiling, 10),
      `measured_minutes (${measured}) must not exceed ceiling_minutes (${ceiling})`,
    );
  });

  test("every critical_path_job exists in release.yml with a timeout-minutes", () => {
    const wallContent = readFileSync(WALL_CLOCK_FILE, "utf8");
    const jobsField = extractField(wallContent, "critical_path_jobs");
    const jobs = jobsField.split(/\s+/).filter(Boolean);

    const workflowContent = readFileSync(WORKFLOW_FILE, "utf8");
    // Extract job-level timeout-minutes: first `timeout-minutes:` seen under each job.
    const timeouts = {};
    let currentJob = null;
    let inJobs = false;
    for (const line of workflowContent.split("\n")) {
      if (line === "jobs:") { inJobs = true; continue; }
      if (!inJobs) continue;
      if (/^  [a-zA-Z_][a-zA-Z0-9_-]*:$/.test(line)) {
        currentJob = line.trim().replace(/:$/, "");
      }
      if (currentJob && !(currentJob in timeouts)) {
        const m = line.match(/^\s{4}timeout-minutes:\s*(\d+)/);
        if (m) timeouts[currentJob] = parseInt(m[1], 10);
      }
    }

    const ceilingMinutes = parseInt(extractField(wallContent, "ceiling_minutes"), 10);
    let total = 0;
    for (const job of jobs) {
      assert.ok(
        job in timeouts,
        `critical_path_job "${job}" not found in release.yml with a timeout-minutes`,
      );
      total += timeouts[job];
    }
    assert.ok(
      total <= ceilingMinutes,
      `critical path sum ${total} min exceeds ceiling ${ceilingMinutes} min`,
    );
  });
});
