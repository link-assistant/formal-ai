import assert from 'node:assert/strict';
export function dispatchedProducer(environment, event) {
 const runId = event.inputs?.['producer-run'] ?? '';
 const attempt = event.inputs?.['producer-attempt'] ?? '';
 if (runId === '' && attempt === '') return null;
 assert.equal(environment.EVENT ?? environment.GITHUB_EVENT_NAME, 'workflow_dispatch');
 assert.equal(environment.REFERENCE ?? environment.GITHUB_REF, 'refs/heads/main');
 assert.match(runId, /^[1-9][0-9]*$/u);
 assert.match(attempt, /^[1-9][0-9]*$/u);
 const repository = environment.REPOSITORY ?? environment.GITHUB_REPOSITORY;
 assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
 return {repository, runId, attempt};
}
export function awaitCompletedProducer(expected, github, options = {}) {
 const now = options.now ?? Date.now;
 const sleep = options.sleep ?? (milliseconds => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds));
 const deadline = options.deadline ?? now() + 160000;
 for (;;) {
  const run = github.json('repos/' + expected.repository + '/actions/runs/' + expected.runId + '/attempts/' + expected.attempt);
  assert.equal(String(run.id), expected.runId);
  assert.equal(String(run.run_attempt), expected.attempt);
  assert.equal(run.repository.full_name, expected.repository);
  assert.equal(run.head_repository.full_name, expected.repository);
  assert.equal(run.head_branch, 'main');
  assert.equal(run.path, '.github/workflows/desktop-release.yml');
  assert.match(run.head_sha, /^[a-f0-9]{40}$/u);
  if (run.status === 'completed') return run;
  assert.ok(['queued', 'in_progress', 'waiting', 'pending'].includes(run.status));
  const remaining = deadline - now();
  assert.ok(remaining > 0, 'Producer completion remains Unknown at the existing resolver deadline');
  sleep(Math.min(1000, remaining));
 }
}
