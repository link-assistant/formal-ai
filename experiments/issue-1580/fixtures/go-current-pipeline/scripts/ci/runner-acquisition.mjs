// Classify GitHub Actions jobs so infrastructure failures are not reported as
// code failures (issue #14).
//
// When GitHub cannot hand out a hosted runner (usually macOS arm64 under
// capacity constraints) it gives up on the job with the annotation "The job
// was not acquired by Runner of type hosted even after multiple attempts".
// Such a job ends `cancelled` (sometimes `failure`) with no steps and an
// empty runner name: no code ran. A job cancelled by a newer push or by a
// maintainer before it started looks the same apart from that annotation, so
// the annotation is what marks a job as "Hosted runner not acquired".
import { execFileSync } from "node:child_process";

export const NOT_ACQUIRED = "was not acquired by Runner";

const UNSUCCESSFUL = new Set(["failure", "cancelled", "timed_out"]);

/** Whether a completed job ended without success. */
export function isUnsuccessful(job) {
  return job.status === "completed" && UNSUCCESSFUL.has(job.conclusion);
}

/** Whether a job never reached a runner: no runner name and no steps. */
export function neverStarted(job) {
  return !job.runner_name && !(job.steps || []).length;
}

/**
 * Classify an unsuccessful job.
 *
 * @param {object} job A job from the GitHub Actions jobs API.
 * @param {string[] | undefined} messages Its check-run annotation messages,
 *   or undefined when they could not be read.
 * @returns {"not-acquired" | "not-started" | "failed"}
 */
export function classifyJob(job, messages) {
  if (!neverStarted(job)) return "failed";
  if ((messages || []).some((message) => message.includes(NOT_ACQUIRED)))
    return "not-acquired";
  return "not-started";
}

export const REASONS = {
  "not-acquired":
    "Hosted runner not acquired: GitHub found no hosted runner, so no step ran (infrastructure, not code)",
  "not-started":
    "Cancelled before a runner picked it up (superseded, cancelled, or blocked by a failed dependency)",
  failed: "Failed while running its steps",
};

function gh(args) {
  return execFileSync("gh", args, {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

/** List jobs of a run attempt, or the latest attempt of each job. */
export function listJobs({ repository, runId, attempt, run = gh }) {
  const endpoint = attempt
    ? `repos/${repository}/actions/runs/${runId}/attempts/${attempt}/jobs?per_page=100`
    : `repos/${repository}/actions/runs/${runId}/jobs?filter=latest&per_page=100`;
  const pages = JSON.parse(run(["api", "--paginate", "--slurp", endpoint]));
  return pages.flatMap((page) => page.jobs || []);
}

/** Read the annotation messages of a job's check run. */
export function annotationMessages({ repository, job, run = gh }) {
  const annotations = JSON.parse(
    run([
      "api",
      "--paginate",
      "--slurp",
      `repos/${repository}/check-runs/${job.id}/annotations?per_page=100`,
    ])
  );
  return annotations.flat().map((annotation) => annotation.message || "");
}

/**
 * Classify every unsuccessful job. Annotations are only fetched for jobs that
 * never started; a lookup failure leaves the job "not-started", which is the
 * conservative choice because it is never rerun automatically.
 */
export function classifyJobs({
  repository,
  jobs,
  annotations = annotationMessages,
  onLookupError = () => {},
}) {
  return jobs.filter(isUnsuccessful).map((job) => {
    let messages;
    if (neverStarted(job)) {
      try {
        messages = annotations({ repository, job });
      } catch (error) {
        onLookupError(job, error);
      }
    }
    return { job, kind: classifyJob(job, messages) };
  });
}
