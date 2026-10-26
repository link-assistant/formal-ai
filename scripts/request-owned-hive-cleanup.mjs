#!/usr/bin/env node
// Explicit dispatch breaks GitHub's three-level workflow_run chain limit.
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {createEvidenceClient} from './collect-test-replays.mjs';
import {cleanupWorkflow, effectOwnerArtifact} from './lib/generated-task-cleanup-journal.mjs';
const repository = 'link-assistant/formal-ai';
const ownerWorkflow = '.github/workflows/hive-original-delivery.yml';
export function cleanupOwnerInput(run, attempt, head) {
  assert.match(run ?? '', /^[1-9]\d*$/u);
  assert.match(attempt ?? '', /^[1-9]\d*$/u);
  const value = {id: Number(run), run_attempt: Number(attempt), head_sha: head};
  assert.ok(Number.isSafeInteger(value.id) && Number.isSafeInteger(value.run_attempt));
  assert.match(head ?? '', /^[a-f0-9]{40}$/u);
  return value;
}
function requireOwnerRun(run, expected) {
  assert.equal(run.id, expected.id); assert.equal(run.run_attempt, expected.run_attempt);
  assert.equal(run.head_sha, expected.head_sha);
  assert.equal(run.repository?.full_name, repository);
  assert.equal(run.head_repository?.full_name, repository);
  assert.equal(run.path?.split('@')[0], ownerWorkflow);
  assert.ok(['workflow_run', 'workflow_dispatch'].includes(run.event));
}
export async function waitForCleanupOwner({reader, owner, maximumPolls = 13, intervalMs = 10000,
  sleep = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds))}) {
  cleanupOwnerInput(String(owner.id), String(owner.run_attempt), owner.head_sha);
  assert.ok(Number.isSafeInteger(maximumPolls) && maximumPolls >= 1 && maximumPolls <= 13);
  assert.ok(Number.isSafeInteger(intervalMs) && intervalMs >= 0 && intervalMs <= 10000);
  for (let poll = 1; poll <= maximumPolls; poll++) {
    const run = await reader.read(`repos/${repository}/actions/runs/${owner.id}/attempts/${owner.run_attempt}`);
    requireOwnerRun(run, owner);
    if (run.status === 'completed') return run;
    assert.ok(['queued', 'in_progress', 'waiting', 'pending', 'requested'].includes(run.status));
    assert.ok(poll < maximumPolls, 'cleanup owner did not become terminal within the finite polling budget');
    await sleep(intervalMs);
  }
}
export async function requestOwnedHiveCleanup({environment = process.env,
  client = createEvidenceClient({bounds: {retries: 0}, maxRequests: 3, aggregateBytes: 3145728})} = {}) {
  assert.equal(environment.GITHUB_REPOSITORY, repository);
  assert.equal(environment.GITHUB_REF, 'refs/heads/main');
  const owner = cleanupOwnerInput(environment.GITHUB_RUN_ID, environment.GITHUB_RUN_ATTEMPT, environment.GITHUB_SHA);
  assert.equal(owner.run_attempt, 1, 'automatic dispatch never repeats an owner attempt');
  const run = await client.json(`repos/${repository}/actions/runs/${owner.id}/attempts/${owner.run_attempt}`);
  requireOwnerRun(run, owner);
  const listing = await client.json(`repos/${repository}/actions/runs/${owner.id}/artifacts?name=${effectOwnerArtifact}&per_page=100`);
  assert.ok(Number.isSafeInteger(listing.total_count) && listing.total_count >= 0 && listing.total_count <= 100);
  assert.ok(Array.isArray(listing.artifacts) && listing.artifacts.length === listing.total_count);
  if (listing.total_count === 0) return {dispatched: false, reason: 'no-immutable-effect-owner', owner};
  assert.equal(listing.total_count, 1, 'ambiguous immutable effect owners must not dispatch');
  const artifact = listing.artifacts[0];
  assert.equal(artifact.name, effectOwnerArtifact); assert.equal(artifact.expired, false);
  assert.equal(artifact.workflow_run?.id, owner.id); assert.equal(artifact.workflow_run.head_sha, owner.head_sha);
  const result = await client.request(`repos/${repository}/actions/workflows/${cleanupWorkflow.split('/').at(-1)}/dispatches`, {
    mutation: ['--method', 'POST', '-f', 'ref=main', '-f', 'inputs[mode]=checks', '-f', 'inputs[cleanup]=true',
      '-f', `inputs[owner-run]=${owner.id}`, '-f', `inputs[owner-attempt]=${owner.run_attempt}`,
      '-f', `inputs[owner-head]=${owner.head_sha}`],
  });
  assert.ok(result.ok && !result.truncated && !result.downloadTruncated,
    'cleanup dispatch failed or is uncertain; do not automatically retry the mutation');
  return {dispatched: true, owner, cleanupCredit: false};
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.stdout.write(JSON.stringify(await requestOwnedHiveCleanup()) + '\n');
}
