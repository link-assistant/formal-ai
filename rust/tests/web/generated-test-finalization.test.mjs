import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {finalizeGeneratedRun} from '../../../scripts/finalize-generated-test-issues.mjs';
const head = 'a'.repeat(40);
function fixture({status = 'completed', conclusion = 'success', log = null} = {}) {
  const calls = [], cleanupCalls = [];
  const run = {id: 40, head_sha: head, run_attempt: 1, status, conclusion,
    repository: {full_name: 'link-assistant/formal-ai'}, path: '.github/workflows/hive-original-delivery.yml'};
  const jobs = [{id: 41, name: 'Actual Hive claude original delivery', status: 'completed', conclusion}];
  const observation = log ?? {ok: true, streamComplete: true, tailComplete: true,
    text: 'Final issue URL: https://github.com/link-assistant/formal-ai/issues/42\n'};
  const client = {async json(endpoint) { calls.push(endpoint); return endpoint.includes('/jobs?')
    ? {total_count: jobs.length, jobs} : run; },
  async request(endpoint) { calls.push(endpoint); return observation; }};
  const cleanup = async options => {
    cleanupCalls.push(options);
    return options.apply ? {results: [{number: 42, status: 'closed'}]}
      : {candidates: options.plan.candidates, results: []};
  };
  return {run, jobs, client, cleanup, calls, cleanupCalls};
}
test('nonfinite owner identifiers fail before any API IO', async () => {
  for (const runId of [NaN, Infinity, 0, -1, 1.5]) {
    const f = fixture(); await assert.rejects(finalizeGeneratedRun({runId, ...f}));
    assert.equal(f.calls.length, 0);
  }
});
test('in-run always cleanup cannot establish terminal custody', async () => {
  const f = fixture({status: 'in_progress'});
  await assert.rejects(finalizeGeneratedRun({runId: 40, ...f}), /completed owner/u);
  assert.equal(f.cleanupCalls.length, 0);
});
test('foreign workflow and changed run identities are refused', async () => {
  for (const change of [{id: 99}, {path: '.github/workflows/other.yml'}, {repository: {full_name: 'other/repo'}}]) {
    const f = fixture(); Object.assign(f.run, change);
    await assert.rejects(finalizeGeneratedRun({runId: 40, ...f}));
    assert.equal(f.cleanupCalls.length, 0);
  }
});
test('incomplete or still-active job listings never reach cleanup', async () => {
  const f = fixture(); f.jobs[0].status = 'in_progress';
  await assert.rejects(finalizeGeneratedRun({runId: 40, ...f}), /terminal owner jobs/u);
  assert.equal(f.cleanupCalls.length, 0);
});
test('cancelled finalization derives only exact own-run candidates and delegates full proof', async () => {
  const f = fixture({conclusion: 'cancelled'});
  const result = await finalizeGeneratedRun({runId: 40, ...f});
  assert.equal(result.apply, false); assert.equal(f.cleanupCalls.length, 1);
  const option = f.cleanupCalls[0]; assert.equal(option.allowOwnedCancelled, true);
  assert.deepEqual(option.plan.candidates[0].proof, {run: 40, job: 41, head});
  assert.equal(option.plan.candidates[0].number, 42);
});
test('interrupted logs and multiple final owners remain retained', async () => {
  for (const log of [{ok: true, streamComplete: false, tailComplete: true,
    text: 'Final issue URL: https://github.com/link-assistant/formal-ai/issues/42\n'},
  {ok: true, streamComplete: true, tailComplete: true,
    text: 'Final issue URL: https://github.com/link-assistant/formal-ai/issues/42\n'
      + 'Final issue URL: https://github.com/other/repo/issues/43\n'}]) {
    const f = fixture({log}); const result = await finalizeGeneratedRun({runId: 40, ...f});
    assert.equal(result.discovery.length, 1); assert.deepEqual(f.cleanupCalls[0].plan.candidates, []);
  }
});
test('apply reuses only freshly prepared candidates and does not retry uncertain writes', async () => {
  const f = fixture(); f.cleanup = async options => {
    f.cleanupCalls.push(options);
    return options.apply ? {results: [{number: 42, status: 'mutation-uncertain'}]}
      : {candidates: [{number: 42, expectedIssue: {bodySha256: 'reviewed'}}]};
  };
  const result = await finalizeGeneratedRun({runId: 40, apply: true, ...f});
  assert.equal(f.cleanupCalls.length, 2);
  assert.deepEqual(f.cleanupCalls[1].plan, result.prepared);
  assert.equal(result.applied.results[0].status, 'mutation-uncertain');
});
test('repeated terminal events repeat fresh proof rather than reuse a prior close decision', async () => {
  const f = fixture(); let count = 0;
  f.cleanup = async options => { f.cleanupCalls.push(options); count++;
    return {candidates: [], results: [{number: 42, status: 'retained', reason: 'already-closed'}]}; };
  for (let i = 0; i < 2; i++) await finalizeGeneratedRun({runId: 40, apply: true, ...f});
  assert.equal(count, 4); assert.equal(f.calls.filter(row => row.endsWith('/runs/40')).length, 2);
});

test('dry-run never certifies cleanup completion', async () => {
  const f = fixture(); const result = await finalizeGeneratedRun({runId: 40, ...f});
  assert.equal(result.cleanupReceipt.complete, false);
  assert.equal(result.cleanupReceipt.closed, 0);
  assert.equal(result.cleanupReceipt.sourceRun, 40);
  assert.equal(result.cleanupReceipt.sourceAttempt, 1);
  assert.equal(result.cleanupReceipt.protocolHead, head);
  assert.equal(Object.hasOwn(result.cleanupReceipt, 'sourceHead'), false);
});
test('proof receipt requires all three confirmed closures and recorded outcome comments', async () => {
  const f = fixture();
  f.jobs.push({...f.jobs[0], id: 43, name: 'Actual Hive codex original delivery'},
    {...f.jobs[0], id: 45, name: 'Actual Hive agent original delivery'});
  f.client.request = async endpoint => { const job = Number(/jobs\/(\d+)/u.exec(endpoint)[1]);
    return {ok: true, streamComplete: true, tailComplete: true,
      text: `Final issue URL: https://github.com/link-assistant/formal-ai/issues/${job + 1}\n`}; };
  let uncertain = false;
  f.cleanup = async options => options.apply ? {results: options.plan.candidates.map(row =>
    ({number: row.number, status: 'closed', comment: uncertain && row.number === 44 ? 'uncertain-not-retried' : 'recorded'}))}
    : {candidates: options.plan.candidates, results: []};
  const good = await finalizeGeneratedRun({runId: 40, apply: true, ...f});
  assert.equal(good.cleanupReceipt.complete, true); assert.equal(good.cleanupReceipt.closed, 3);
  assert.deepEqual(good.cleanupReceipt.perTool.map(row => row.tool), ['claude', 'codex', 'agent']);
  uncertain = true;
  const refused = await finalizeGeneratedRun({runId: 40, apply: true, ...f});
  assert.equal(refused.cleanupReceipt.complete, false); assert.equal(refused.cleanupReceipt.closed, 3);
  assert.equal(refused.cleanupReceipt.perTool[1].complete, false);
});

test('API protocol SHA is not a separately selected or claimed tested PR identity', async () => {
  const f = fixture(); f.run.head_sha = 'b'.repeat(40);
  f.run.selected_tested_head = 'c'.repeat(40);
  const result = await finalizeGeneratedRun({runId: 40, ...f});
  assert.equal(result.cleanupReceipt.protocolHead, 'b'.repeat(40));
  assert.equal(Object.hasOwn(result.cleanupReceipt, 'sourceHead'), false);
  assert.equal(Object.hasOwn(result.cleanupReceipt, 'selectedTestedHead'), false);
  assert.equal(result.cleanupReceipt.complete, false);
});

test('authenticated owner attempt and protocol cannot race into a newer cleanup', async () => {
  for (const expected of [{expectedRunAttempt: 2}, {expectedProtocolHead: 'd'.repeat(40)}]) {
    const f = fixture();
    await assert.rejects(finalizeGeneratedRun({runId: 40, apply: true, ...expected, ...f}),
      error => error.code === 'OWNER_CHANGED_BEFORE_CLEANUP');
    assert.equal(f.calls.length, 1); assert.equal(f.cleanupCalls.length, 0);
  }
});

const preEffectReplay = JSON.parse(fs.readFileSync(new URL('../fixtures/hive-pre-effect-failure.json', import.meta.url)));
function replayFixture() {
  const f = fixture({conclusion: 'failure', log: {ok: true, streamComplete: true, tailComplete: true,
    text: 'Credential preflight failed before the effect claim.\n'}});
  f.jobs.splice(0, f.jobs.length, ...structuredClone(preEffectReplay.jobs));
  return f;
}
test('actual pre-claim credential failure and cancelled image loads need no generated issue closure', async () => {
  const f = replayFixture(), result = await finalizeGeneratedRun({runId: 40, apply: true, ...f});
  assert.equal(result.cleanupReceipt.noEffects, true);
  assert.equal(result.cleanupReceipt.complete, false, 'a no-op never certifies three successful solves');
  assert.equal(result.cleanupReceipt.closed, 0); assert.equal(result.cleanupReceipt.retained, 0);
  assert.equal(result.discovery.length, 3);
  assert(result.discovery.every(row => row.reason === 'all-effect-gates-skipped'));
  assert(f.cleanupCalls.every(call => (call.plan.candidates ?? []).length === 0));
});
test('missing, duplicated, reordered, active or executed gates cannot prove a no-op', async () => {
  for (const change of [
    steps => steps.splice(steps.findIndex(step => step.name.startsWith('Claim one bounded')), 1),
    steps => steps.push({...steps.find(step => step.name.startsWith('Persist effect claim'))}),
    steps => steps.reverse(),
    steps => { steps.find(step => step.name.startsWith('Run actual Hive')).status = 'in_progress'; },
    ...['Claim one bounded', 'Persist effect claim', 'Run actual Hive'].map(prefix => steps => {
      steps.find(step => step.name.startsWith(prefix)).conclusion = 'success'; })
  ]) {
    const f = replayFixture(); change(f.jobs[0].steps);
    if (f.jobs[0].steps.some(step => step.status !== 'completed')) {
      const result = await finalizeGeneratedRun({runId: 40, apply: true, ...f});
      assert.equal(result.cleanupReceipt.noEffects, false); continue;
    }
    const result = await finalizeGeneratedRun({runId: 40, apply: true, ...f});
    assert.equal(result.cleanupReceipt.noEffects, false); assert.equal(result.cleanupReceipt.retained, 1);
  }
});
test('truncated logs or a final issue URL retain cleanup even when every effect gate was skipped', async () => {
  for (const log of [{ok: true, streamComplete: true, tailComplete: false, text: ''},
    {ok: true, streamComplete: true, tailComplete: true,
      text: 'Final issue URL: https://github.com/link-assistant/formal-ai/issues/42\n'}]) {
    const f = replayFixture(); f.client.request = async endpoint => ({...log,
      text: log.text.replace('/issues/42', '/issues/' + /jobs\/(\d+)/u.exec(endpoint)[1])});
    const result = await finalizeGeneratedRun({runId: 40, apply: true, ...f});
    assert.equal(result.cleanupReceipt.noEffects, false);
  }
});
