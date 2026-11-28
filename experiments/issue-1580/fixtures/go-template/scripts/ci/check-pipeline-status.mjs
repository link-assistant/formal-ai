#!/usr/bin/env bun

// Terminal status gate for the CI/CD pipeline (issue #14).
//
// The gate fails when any job it needs failed or was cancelled, and explains
// each unsuccessful job of the current run so the cause is visible on the run
// page: a job that GitHub never handed a hosted runner is reported as
// "Hosted runner not acquired" (infrastructure, rerun automatically by
// rerun-unacquired.yml), not as a code failure.
//
// Environment:
//   NEEDS_JSON      toJSON(needs) of the gate job (required)
//   REPOSITORY      owner/repo, RUN_ID and RUN_ATTEMPT of the current run
//   GH_TOKEN        token with actions: read and checks: read
//   CI_VERBOSE=true traces the job classification
import { appendFileSync } from "node:fs";
import { printUntrusted } from "../github-actions-log.mjs";
import { REASONS, classifyJobs, listJobs } from "./runner-acquisition.mjs";

const BLOCKING = new Set(["failure", "cancelled"]);

/** Names of the needed jobs whose result blocks the pipeline. */
export function blockingNeeds(needs) {
  return Object.entries(needs)
    .filter(([, value]) => BLOCKING.has(value?.result))
    .map(([name, value]) => `${name} (${value.result})`);
}

/**
 * Build the report for unsuccessful jobs.
 *
 * @param {{job: {name: string, conclusion: string, html_url?: string},
 *   kind: string}[]} classified
 */
export function describeJobs(classified) {
  const order = ["not-acquired", "failed", "not-started"];
  return [...classified]
    .sort((a, b) => order.indexOf(a.kind) - order.indexOf(b.kind))
    .map(({ job, kind }) => ({
      kind,
      title: kind === "not-acquired" ? "Hosted runner not acquired" : "Job did not succeed",
      line: `${job.name} (${job.conclusion}): ${REASONS[kind]}`,
    }));
}

/**
 * Evaluate the pipeline and return the exit code.
 *
 * @param {{needs: object, jobs?: () => object[], classify?: (jobs: object[]) => object[],
 *   summaryFile?: string, githubActions?: boolean,
 *   stdout?: {write: (text: string) => unknown}}} options
 */
export function evaluate({
  needs,
  jobs,
  classify,
  summaryFile,
  githubActions,
  stdout = process.stdout,
}) {
  const blocking = blockingNeeds(needs);
  const print = (text) => printUntrusted(text, { stream: stdout, githubActions });
  if (!blocking.length) {
    stdout.write("Every required job succeeded or was skipped.\n");
    return 0;
  }

  let details = [];
  try {
    details = describeJobs(classify(jobs()));
  } catch (error) {
    stdout.write("::warning::Could not read this run's jobs to explain the failure\n");
    print(error.message);
  }

  const infrastructure = details.filter(({ kind }) => kind === "not-acquired");
  const code = details.filter(({ kind }) => kind !== "not-acquired");
  if (infrastructure.length)
    stdout.write(
      `::error title=Hosted runner not acquired::${infrastructure.length} job(s) never got a hosted runner; ` +
        "this is GitHub runner capacity, not a code failure. rerun-unacquired.yml reruns it when no other job failed.\n"
    );
  if (code.length || !infrastructure.length)
    stdout.write("::error title=Pipeline failed::Required jobs did not succeed\n");
  stdout.write("Blocking needs:\n");
  print(blocking.join("\n"));
  if (details.length) {
    stdout.write("Unsuccessful jobs:\n");
    print(details.map(({ line }) => `- ${line}`).join("\n"));
  }

  if (summaryFile) {
    const clean = (text) => text.replaceAll("`", "'").replaceAll("\n", " ");
    const rows = details.length
      ? details.map(({ title, line }) => `- **${title}**: \`${clean(line)}\``)
      : blocking.map((name) => `- \`${clean(name)}\``);
    appendFileSync(
      summaryFile,
      `## Pipeline status: failed\n\n${rows.join("\n")}\n\n`
    );
  }
  return 1;
}

function main() {
  if (!process.env.NEEDS_JSON)
    throw new Error("NEEDS_JSON is required (pass toJSON(needs))");
  const repository = process.env.REPOSITORY;
  const runId = process.env.RUN_ID;
  const verbose = process.env.CI_VERBOSE === "true";
  return evaluate({
    needs: JSON.parse(process.env.NEEDS_JSON),
    // Latest attempt of each job: a rerun replaces the attempt it retried.
    jobs: () => listJobs({ repository, runId }),
    classify: (jobs) => {
      const classified = classifyJobs({ repository, jobs });
      if (verbose)
        for (const { job, kind } of classified)
          printUntrusted(`[pipeline-status] ${job.name}: ${job.conclusion}, ${kind}`);
      return classified;
    },
    summaryFile: process.env.GITHUB_STEP_SUMMARY,
  });
}

if (import.meta.main) {
  try {
    process.exitCode = main();
  } catch (error) {
    console.error("::error::Pipeline status check failed");
    printUntrusted(error.message, { stream: process.stderr });
    process.exitCode = 1;
  }
}
