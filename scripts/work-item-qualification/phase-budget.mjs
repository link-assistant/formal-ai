// One authenticated job deadline; command caps cannot extend its usable window.
import {performance} from 'node:perf_hooks';

export function createWorkItemPhaseBudget({
  job,
  expectedName,
  expectedRun,
  wallNow = Date.now(),
  monotonicNow = () => performance.now()
}) {
  const timestamp = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:[.]\d{1,3})?Z$/u;
  if (!job || !Number.isSafeInteger(job.id) || job.id <= 0
    || job.name !== expectedName || job.run_id !== expectedRun
    || !Number.isSafeInteger(expectedRun) || expectedRun <= 0
    || job.status !== 'in_progress' || typeof job.started_at !== 'string'
    || !timestamp.test(job.started_at)) {
    throw Error('InvalidWorkItemJobBudget');
  }
  const startedAt = Date.parse(job.started_at);
  const start = monotonicNow();
  if (!Number.isFinite(wallNow) || !Number.isFinite(start)
    || !Number.isFinite(startedAt) || startedAt > wallNow) {
    throw Error('InvalidWorkItemJobClock');
  }
  const jobId = job.id, startedAtText = job.started_at, runId = expectedRun;
  const jobMilliseconds = 25 * 60 * 1000;
  const reserveMilliseconds = 3 * 60 * 1000;
  const deadline = start + startedAt + jobMilliseconds - reserveMilliseconds - wallNow;
  let previous = start;

  function remaining() {
    const current = monotonicNow();
    if (!Number.isFinite(current) || current < previous) {
      throw Error('InvalidWorkItemMonotonicClock');
    }
    previous = current;
    return Math.max(0, Math.floor(deadline - current));
  }
  const snapshot = () => Object.freeze({
    jobId, runId, startedAt: startedAtText,
    jobMilliseconds, reserveMilliseconds,
    remainingMilliseconds: remaining()
  });
  return Object.freeze({
    snapshot,
    allocate(requestedMilliseconds, phase) {
      if (!Number.isSafeInteger(requestedMilliseconds) || requestedMilliseconds <= 0
        || requestedMilliseconds > jobMilliseconds || typeof phase !== 'string'
        || !/^[a-z][a-z0-9:-]{0,79}$/u.test(phase)) {
        throw Error('InvalidWorkItemPhaseCap');
      }
      const state = snapshot();
      if (state.remainingMilliseconds <= 0) throw Error('WorkItemPhaseBudgetExhausted');
      return Object.freeze({
        ...state, phase, requestedMilliseconds,
        timeoutMilliseconds: Math.min(requestedMilliseconds, state.remainingMilliseconds)
      });
    }
  });
}
