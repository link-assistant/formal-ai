// Issue #1187: GitHub credentials are optional in every workflow.
//
// R1187-3 / R1187-7: a dispatch of release.yml runs the checks unless a release
// mode is chosen, and every job that writes (release, publish, tag, Pages) can
// run on a dispatch only for an explicit release mode on `refs/heads/main`.
// The JavaScript twin of `release_dispatch_needs_an_explicit_mode_and_main` in
// rust/tests/unit/issue_1187_credentials.rs, over the same structural rule.
//
// R1187-8 (mechanism): with no AUTOMATION_* secret the resolver's own script
// reports layer `default` in its output and job summary, and the dispatcher's
// own script, run against this repository's workflows with a recording `gh`,
// dispatches every pull_request workflow in checks mode on the bot branch, so
// no check waits for an approval. The acceptance run itself needs an
// `issues: labeled` event, which runs the default branch's workflow file, so it
// can only happen after the merge.

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");

// The `on:` block of a workflow.
function triggerBlock(body) {
  const lines = body.split("\n");
  const start = lines.findIndex((line) => /^on:(\s|$)/.test(line));
  if (start < 0) return "";
  const block = [];
  for (const line of lines.slice(start + 1)) {
    if (line && !line.startsWith(" ") && !line.startsWith("#")) break;
    block.push(line);
  }
  return block.join("\n");
}

// The jobs of a workflow as [name, body].
function workflowJobs(body) {
  const jobs = [];
  let inside = false;
  for (const line of body.split("\n")) {
    if (line && !line.startsWith(" ") && !line.startsWith("#")) {
      inside = line.trimEnd() === "jobs:";
      continue;
    }
    if (!inside) continue;
    if (/^ {2}[^ #][^:]*:\s*$/.test(line)) jobs.push([line.trim().replace(/:$/, ""), ""]);
    else if (jobs.length) jobs[jobs.length - 1][1] += `${line}\n`;
  }
  return jobs;
}

// A job's own `if:` condition joined onto one line; empty when it has none.
function jobCondition(job) {
  const lines = job.split("\n");
  const start = lines.findIndex((line) => line.startsWith("    if:"));
  if (start < 0) return "";
  let condition = lines[start].slice("    if:".length).trim();
  for (const line of lines.slice(start + 1)) {
    if (line && !line.startsWith("      ")) break;
    condition += ` ${line.trim()}`;
  }
  return condition;
}

const guarded = (text) => text.split("||")
  .filter((alternative) => alternative.includes("workflow_dispatch"))
  .every((alternative) =>
    (alternative.includes("release_mode == 'instant'") || alternative.includes("release_mode == 'changelog-pr'"))
    && alternative.includes("github.ref == 'refs/heads/main'"));

test("R1187-3: release.yml dispatch defaults to checks", () => {
  const triggers = triggerBlock(read(".github/workflows/release.yml"));
  const releaseMode = triggers.slice(triggers.indexOf("release_mode:"), triggers.indexOf("bump_type:"));
  assert.match(releaseMode, /default: checks/);
  assert.match(releaseMode, /- checks/);
  assert.match(triggers, /options: \[checks\]/, "the checks-mode input the dispatcher passes");
});

test("R1187-3 / R1187-7: every writing job needs an explicit mode and main on a dispatch", () => {
  const writers = [];
  for (const [name, job] of workflowJobs(read(".github/workflows/release.yml"))) {
    const writes = job.split("\n").some((line) => !line.trim().startsWith("#") && /: write\s*$/.test(line));
    if (!writes) continue;
    writers.push(name);
    const condition = jobCondition(job);
    if (!condition) {
      assert.ok(job.includes("workflow_dispatch") && guarded(job), `${name} writes without an if: and an unguarded dispatch`);
    } else if (condition.includes("workflow_dispatch")) {
      assert.ok(guarded(condition), `${name} runs on a dispatch without an explicit mode and main: ${condition}`);
    } else {
      assert.ok(/github\.event_name == '(push|pull_request)'/.test(condition), `${name} does not exclude a dispatch: ${condition}`);
    }
  }
  for (const expected of ["release-preflight", "auto-release", "manual-release", "changelog-pr", "deploy-pages"]) {
    assert.ok(writers.includes(expected), `the job parser found ${expected}: ${writers}`);
  }
  const manifest = read("docs/integration-manifest.md");
  assert.ok(!manifest.split("\n- ").filter((row) => row.startsWith("`pending:`"))
    .some((row) => row.includes("`.github/workflows/release.yml` — #1187 R3")),
  "the R3 guard is applied, so no pending R3 row remains");
});

test("R1187-7: the rule rejects an unguarded dispatch", () => {
  assert.equal(guarded("github.event_name == 'workflow_dispatch' && github.ref == 'refs/heads/main'"), false);
  assert.equal(guarded("github.event_name == 'workflow_dispatch' && github.event.inputs.release_mode == 'instant'"), false);
  assert.equal(guarded("github.event_name == 'workflow_dispatch' && github.event.inputs.release_mode == 'instant' && github.ref == 'refs/heads/main'"), true);
});

// The `run: |` script of the composite-action step with the given id.
function stepScript(action, id) {
  const lines = action.split("\n");
  const step = lines.findIndex((line) => line.trim() === `- id: ${id}`);
  assert.ok(step >= 0, `step ${id} exists`);
  const run = lines.findIndex((line, index) => index > step && line.trim() === "run: |");
  const indent = lines[run].indexOf("run:") + 2;
  const body = [];
  for (const line of lines.slice(run + 1)) {
    if (line.trim() && line.search(/\S/) < indent) break;
    body.push(line.slice(indent));
  }
  return body.join("\n");
}

function scratch() {
  return mkdtempSync(path.join(tmpdir(), "formal-ai-1187-"));
}

test("R1187-8 mechanism: with no AUTOMATION_* secret the resolver reports layer default", () => {
  const dir = scratch();
  try {
    const output = path.join(dir, "output");
    const summary = path.join(dir, "summary");
    writeFileSync(output, "");
    writeFileSync(summary, "");
    const log = execFileSync("bash", ["-c", stepScript(read(".github/actions/automation-token/action.yml"), "resolve")], {
      env: {
        PATH: process.env.PATH,
        APP_ID: "",
        APP_PRIVATE_KEY: "",
        AUTOMATION_TOKEN: "",
        FALLBACK_TOKEN: "ghs_default_job_token",
        REPOSITORIES: "",
        GITHUB_OUTPUT: output,
        GITHUB_STEP_SUMMARY: summary,
      },
      encoding: "utf8",
    });
    assert.match(log, /automation-token: layer=default/);
    const outputs = readFileSync(output, "utf8");
    assert.match(outputs, /^layer=default$/m);
    assert.match(outputs, /^token=ghs_default_job_token$/m);
    assert.match(readFileSync(summary, "utf8"), /layer: `default`/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("R1187-8 mechanism: the self-authored workflow dispatches checks at layer default", () => {
  const workflow = read(".github/workflows/self-authored-pull-request.yml");
  assert.match(workflow, /issues:\n\s+types: \[labeled\]/);
  assert.match(workflow, /github\.event\.label\.name == 'formal-ai-solve'/);
  assert.match(workflow, /if: steps\.automation\.outputs\.layer == 'default' && steps\.author\.outputs\.authored == 'true'\n\s+uses: \.\/\.github\/actions\/dispatch-checks/);
  assert.match(workflow, /pull-request: \$\{\{ steps\.author\.outputs\.pull-request \}\}/);
  assert.match(workflow, /actions: write/);
  const author = read(".github/actions/author-with-formal-ai/action.yml");
  const outputs = author.slice(author.indexOf("\noutputs:"), author.indexOf("\nruns:"));
  assert.match(outputs, /\n {2}pull-request:/);
  assert.match(outputs, /\n {2}authored:/);
});

test("R1187-8 mechanism: the dispatcher starts every pull_request workflow in checks mode", () => {
  const dir = scratch();
  try {
    const calls = path.join(dir, "calls");
    writeFileSync(calls, "");
    const gh = path.join(dir, "gh");
    writeFileSync(gh, `#!/usr/bin/env bash
case "$1 $2" in
  "pr view")
    case "$*" in
      *headRefName*) echo "formal-ai/self-authored-42" ;;
      *headRefOid*) echo "0123456789abcdef" ;;
    esac ;;
  "workflow run") printf '%s\\n' "$*" >> "${calls}" ;;
esac
`);
    chmodSync(gh, 0o755);
    const output = path.join(dir, "output");
    writeFileSync(output, "");
    execFileSync("bash", ["-c", stepScript(read(".github/actions/dispatch-checks/action.yml"), "dispatch")], {
      cwd: REPO_ROOT,
      env: {
        PATH: `${dir}:${process.env.PATH}`,
        PR_NUMBER: "https://github.com/link-assistant/formal-ai/pull/42",
        WORKFLOWS: "",
        GH_TOKEN: "ghs_default_job_token",
        REPOSITORY: "link-assistant/formal-ai",
        GITHUB_OUTPUT: output,
      },
      encoding: "utf8",
    });
    const dispatched = new Map(readFileSync(calls, "utf8").trim().split("\n").filter(Boolean)
      .map((line) => [line.split(" ")[2], line]));
    for (const line of dispatched.values()) {
      assert.match(line, /--ref formal-ai\/self-authored-42 /);
      assert.match(line, /-f mode=checks -f pull-request=/);
    }
    // Every pull_request workflow is dispatched except the three whose
    // dispatch means something else (a release-asset publish, a credentialed
    // benchmark, the authoring workflow itself).
    const excluded = new Set(["desktop-release.yml", "external-benchmarks.yml", "self-authored-pull-request.yml"]);
    const pullRequestWorkflows = readdirSync(path.join(REPO_ROOT, ".github", "workflows"))
      .filter((name) => name.endsWith(".yml"))
      .filter((name) => /\n {2}pull_request:/.test(triggerBlock(read(`.github/workflows/${name}`)).replace(/^/, "\n")));
    assert.ok(pullRequestWorkflows.length > 5, "the repository has pull_request workflows");
    for (const name of pullRequestWorkflows) {
      assert.equal(dispatched.has(name), !excluded.has(name), `${name} dispatched: ${dispatched.has(name)}`);
    }
    assert.ok(dispatched.has("release.yml"), "release.yml is dispatched in checks mode");
    assert.match(readFileSync(output, "utf8"), /dispatched<<EOF\n[\s\S]*release\.yml/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
