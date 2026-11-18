import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import YAML from 'yaml';
import {cleanupOwnerInput, waitForCleanupOwner, requestOwnedHiveCleanup} from '../../../scripts/request-owned-hive-cleanup.mjs';
import {createEvidenceClient} from '../../../scripts/collect-test-replays.mjs';
const repository = 'link-assistant/formal-ai', head = 'a'.repeat(40);
const environment = {GITHUB_REPOSITORY: repository, GITHUB_REF: 'refs/heads/main',
  GITHUB_RUN_ID: '101', GITHUB_RUN_ATTEMPT: '1', GITHUB_SHA: head};
const owner = cleanupOwnerInput('101', '1', head);
const run = {...owner, repository: {full_name: repository}, head_repository: {full_name: repository},
  path: '.github/workflows/hive-original-delivery.yml', event: 'workflow_run', status: 'in_progress'};
const artifact = {name: 'hive-real-effect-owner', expired: false, workflow_run: {id: 101, head_sha: head}};

function fixture({present = true, dispatchOk = true} = {}) {
  const calls = [];
  const client = createEvidenceClient({bounds: {retries: 0}, maxRequests: 3, transport: async argv => {
    calls.push(argv);
    if (argv.includes('POST')) return {ok: dispatchOk, text: '', bytesReceived: 0};
    return {ok: true, text: JSON.stringify(argv[1].includes('/artifacts?')
      ? {total_count: present ? 1 : 0, artifacts: present ? [artifact] : []} : run)};
  }});
  return {client, calls};
}
test('default-token dispatch requests only the exact owner on trusted main and grants no cleanup credit', async () => {
  const f = fixture(), result = await requestOwnedHiveCleanup({environment, client: f.client});
  assert.equal(result.dispatched, true); assert.equal(result.cleanupCredit, false);
  const mutation = f.calls.filter(argv => argv.includes('POST'));
  assert.equal(mutation.length, 1);
  assert.equal(mutation[0][1], `repos/${repository}/actions/workflows/hive-original-delivery-finalize.yml/dispatches`);
  for (const value of ['ref=main', 'inputs[mode]=checks', 'inputs[cleanup]=true', 'inputs[owner-run]=101',
    'inputs[owner-attempt]=1', `inputs[owner-head]=${head}`]) assert(mutation[0].includes(value));
});
test('no effect owner produces no dispatch; an uncertain dispatch is never retried', async () => {
  const absent = fixture({present: false});
  assert.equal((await requestOwnedHiveCleanup({environment, client: absent.client})).dispatched, false);
  assert.equal(absent.calls.length, 2);
  const failed = fixture({dispatchOk: false});
  await assert.rejects(requestOwnedHiveCleanup({environment, client: failed.client}), /do not automatically retry/u);
  assert.equal(failed.calls.filter(argv => argv.includes('POST')).length, 1);
});
test('foreign sources and repeated owner attempts reject before any request', async () => {
  for (const patch of [{GITHUB_REPOSITORY: 'foreign/repo'}, {GITHUB_REF: 'refs/heads/other'},
    {GITHUB_RUN_ATTEMPT: '2'}, {GITHUB_RUN_ID: '9007199254740992'}, {GITHUB_SHA: 'bad'}]) {
    const f = fixture();
    await assert.rejects(requestOwnedHiveCleanup({environment: {...environment, ...patch}, client: f.client}));
    assert.equal(f.calls.length, 0);
  }
});
test('cleanup polling allows the dispatching owner to finish, but is finite and read-only', async () => {
  let reads = 0, sleeps = 0;
  const reader = {read: async () => ({...run, status: ++reads === 3 ? 'completed' : 'in_progress'})};
  const value = await waitForCleanupOwner({reader, owner, sleep: async () => {sleeps++;}});
  assert.equal(value.status, 'completed'); assert.equal(reads, 3); assert.equal(sleeps, 2);
  reads = 0;
  await assert.rejects(waitForCleanupOwner({owner, maximumPolls: 3, intervalMs: 0,
    reader: {read: async () => {reads++; return run;}}, sleep: async () => {}}), /finite polling budget/u);
  assert.equal(reads, 3);
  for (const patch of [{head_sha: 'b'.repeat(40)}, {run_attempt: 2}, {path: 'other.yml'},
    {head_repository: {full_name: 'foreign/repo'}}]) {
    await assert.rejects(waitForCleanupOwner({owner, reader: {read: async () => ({...run, ...patch})}}));
  }
});
test('only a final actual owner can dispatch after every delivery and receipt job, including failure', () => {
  const workflow = YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/hive-original-delivery.yml', import.meta.url), 'utf8'));
  const job = workflow.jobs['cleanup-request'];
  for (const dependency of ['admission', 'image', 'delivery', 'verification', 'ledger', 'acceptance']) assert(job.needs.includes(dependency));
  assert.match(job.if, /always\(\)/u); assert.match(job.if, /outputs.execute == 'true'/u);
  assert.match(job.if, /github.run_attempt == 1/u);
  assert.deepEqual(job.permissions, {contents: 'read', actions: 'write'});
  assert(job.steps.filter(step => step.uses?.startsWith('actions/checkout@')).every(step =>
    step.with.ref === '${{ github.workflow_sha }}' && step.with['persist-credentials'] === false));
  assert(job.steps.some(step => step.run === 'node scripts/request-owned-hive-cleanup.mjs'
    && step.env.GH_TOKEN === '${{ github.token }}'));
});
