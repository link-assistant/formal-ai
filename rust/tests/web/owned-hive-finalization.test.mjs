import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import vm from 'node:vm';
import YAML from 'yaml';
import {prepareOwnedHiveRuns, applyOwnedHiveClaim, writeCleanupPreparation, writeCleanupOutcome,
  writeCleanupRecheck} from '../../../scripts/finalize-owned-hive-delivery-runs.mjs';
import {prepareCleanupClaim, revalidateCleanupClaim, readScanCursor, makeScanProgress, cleanupClaimName,
  effectOwnerArtifact} from '../../../scripts/lib/generated-task-cleanup-journal.mjs';
const protocol = 'a'.repeat(40), cleanupProtocol = 'b'.repeat(40), selected = 'c'.repeat(40);
const deliveryHash = 'd'.repeat(64), inputHash = 'e'.repeat(64), cleanupHash = 'f'.repeat(64);
const producer = {run: 900, attempt: 1, protocolCommit: cleanupProtocol};
function fixture({additionalProducers = []} = {}) {
  const calls = [], effects = [], owners = new Map(), artifacts = new Map(), documents = new Map();
  let claims = [], progress = [], listingOverride = null;
  function addOwner(run = 101, artifact = 11) {
    const owner = {schema: 'formal-ai-hive-effect-owner/v1', repository: 'link-assistant/formal-ai',
      run, attempt: 1, protocolCommit: protocol, selectedSource: selected, scope: 'pr-1329',
      policyHash: deliveryHash, inputHash, taskMode: 'historical-1364', tools: ['claude']};
    const metadata = {id: artifact, name: effectOwnerArtifact, expired: false, size_in_bytes: 100,
      digest: 'sha256:' + '1'.repeat(64), workflow_run: {id: run, head_sha: protocol}};
    owners.set(artifact, owner); artifacts.set(artifact, metadata); documents.set(artifact, owner); return metadata;
  }
  const metadata = addOwner();
  const reader = {
    async read(endpoint) {
      calls.push(endpoint);
      if (endpoint.includes('/actions/artifacts?name=hive-cleanup-scan-progress-')) {
        const name = new URLSearchParams(endpoint.split('?')[1]).get('name');
        const values = progress.filter(row => row.name === name);
        return {total_count: values.length, artifacts: values.slice(0,100)};
      }
      if (endpoint.includes('/actions/artifacts?')) {
        if (listingOverride) return listingOverride(endpoint);
        return {total_count: owners.size, artifacts: [...owners.keys()].map(id => artifacts.get(id))};
      }
      if (/actions\/artifacts\/\d+$/u.test(endpoint)) {
        const value = artifacts.get(Number(endpoint.split('/').at(-1))); if (!value) throw Error('missing artifact'); return value;
      }
      if (endpoint.endsWith('/git/ref/heads/main')) return {object: {sha: cleanupProtocol}};
      if (endpoint.includes('/compare/')) return {status: 'ahead', merge_base_commit: {sha: endpoint.split('/compare/')[1].split('...')[0]}};
      const match = /runs\/(\d+)\/attempts\/(\d+)/u.exec(endpoint);
      if (match) {
        const id = Number(match[1]), attempt = Number(match[2]);
        const isCleanup = [producer, ...additionalProducers].some(value => value.run === id);
        return {id, run_attempt: attempt, head_sha: isCleanup ? cleanupProtocol : protocol,
          repository: {full_name: 'link-assistant/formal-ai'}, head_repository: {full_name: 'link-assistant/formal-ai'},
          path: isCleanup ? '.github/workflows/hive-original-delivery-finalize.yml'
            : '.github/workflows/hive-original-delivery.yml', event: isCleanup ? 'schedule' : 'workflow_run',
          status: 'completed', conclusion: 'success'};
      }
      throw Error('unexpected synthetic endpoint');
    },
    async pages(endpoint) {
      calls.push(endpoint);
      if (endpoint.includes('name=hive-cleanup-attempt-')) return claims;
      if (endpoint.includes('name=hive-cleanup-scan-progress-')) return progress.filter(row => endpoint.endsWith(row.name));
      const match = /runs\/(\d+)\/artifacts/u.exec(endpoint);
      if (match) return [...owners.keys()].map(id => artifacts.get(id)).filter(row => row.workflow_run.id === Number(match[1]));
      throw Error('unexpected synthetic pages');
    },
  };
  const dependencies = {readArtifact: async (_reader, _repository, artifact) => {
    calls.push('artifact-json/' + artifact.id); return documents.get(artifact.id); },
  deliveryPolicy: async () => ({policyHash: deliveryHash, inputHash}),
  cleanupPolicy: async () => ({policyHash: cleanupHash})};
  const finalize = async options => { effects.push(options); return {applied: {results: [{status: 'closed', comment: 'recorded'}]},
    cleanupReceipt: {schema: 'generated-e2e-cleanup-proof/v1', sourceRun: options.runId,
      sourceAttempt: options.expectedRunAttempt, protocolHead: options.expectedProtocolHead, complete: true}}; };
  function publishClaim(claim, id = 21) {
    const value = {id, name: cleanupClaimName(claim.sourceRun, claim.sourceAttempt), expired: false,
      size_in_bytes: 100, digest: 'sha256:' + '2'.repeat(64), workflow_run: {id: producer.run, head_sha: cleanupProtocol}};
    artifacts.set(id, value); documents.set(id, claim); claims.push(value); return value;
  }
  return {reader, metadata, producer, dependencies, finalize, calls, effects, owners, artifacts, documents,
    addOwner, publishClaim, setClaims: value => { claims = value; }, setProgress: value => { progress = value; },
    setListing: value => { listingOverride = value; }};
}
async function prepare(f) {
  return prepareCleanupClaim({reader: f.reader, metadata: f.metadata, producer, ...f.dependencies});
}
test('nonfinite scan bounds and invalid cursors reject before IO', async () => {
  for (const options of [{maxPages: Infinity}, {maxOwners: NaN}, {cursor: {page: 0, offset: 0}}, {now: Infinity}]) {
    const f = fixture(); await assert.rejects(prepareOwnedHiveRuns({reader: f.reader, producer,
      cursor: {page: 1, offset: 0}, ...f.dependencies, ...options})); assert.equal(f.calls.length, 0);
  }
});
test('prepare creates source-bound claim data without any close', async () => {
  const f = fixture(), result = await prepare(f);
  assert.equal(result.claimName, 'hive-cleanup-attempt-101-1');
  assert.equal(result.outcomeName, 'hive-cleanup-outcome-101-1');
  assert.equal(result.claim.cleanupPolicyHash, cleanupHash);
  assert.equal(result.claim.protocolHead, protocol); assert.equal(result.claim.selectedSourceFromOwner, selected);
  assert.equal(f.effects.length, 0);
});
test('no remote durable claim means no cleanup effect', async () => {
  const f = fixture(); await assert.rejects(applyOwnedHiveClaim({reader: f.reader, producer,
    claimArtifactId: 21, finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
});
test('interruption after claim upload permanently quarantines automatic repetition', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  const again = await prepare(f); assert.equal(again.eligible, false);
  assert.equal(again.reason, 'prior-claim-quarantined'); assert.equal(f.effects.length, 0);
});
test('apply authenticates exact claim and delegates expected owner attempt and protocol', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  const result = await applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    finalize: f.finalize, ...f.dependencies});
  assert.equal(result.state, 'completed'); assert.equal(f.effects.length, 1);
  assert.equal(f.effects[0].expectedRunAttempt, 1); assert.equal(f.effects[0].expectedProtocolHead, protocol);
  assert.equal(result.cleanupReceipt.protocolHead, protocol);
});
test('claim cannot migrate to another cleanup producer or policy', async () => {
  for (const variant of ['producer', 'policy']) {
    const f = fixture(), prepared = await prepare(f);
    if (variant === 'policy') prepared.claim.cleanupPolicyHash = '0'.repeat(64);
    f.publishClaim(prepared.claim);
    await assert.rejects(applyOwnedHiveClaim({reader: f.reader,
      producer: variant === 'producer' ? {...producer, attempt: 2} : producer,
      claimArtifactId: 21, finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
  }
});
test('concurrent immutable claims refuse all issue mutations', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim); f.publishClaim(prepared.claim, 22);
  await assert.rejects(applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
});
test('a queued second producer rechecks after the first claim and never publishes a duplicate', async () => {
  const other = {...producer, run: 901}, f = fixture({additionalProducers: [other]});
  const first = await prepare(f), queued = await prepareCleanupClaim({reader: f.reader,
    metadata: f.metadata, producer: other, ...f.dependencies});
  assert.equal(first.eligible, true); assert.equal(queued.eligible, true);
  const fresh = await revalidateCleanupClaim({reader: f.reader, producer, claim: first.claim, ...f.dependencies});
  assert.deepEqual(fresh.claim, first.claim); assert.equal(f.effects.length, 0); f.publishClaim(fresh.claim);
  const blocked = await revalidateCleanupClaim({reader: f.reader, producer: other, claim: queued.claim, ...f.dependencies});
  assert.equal(blocked.eligible, false); assert.equal(blocked.reason, 'prior-claim-quarantined');
  assert.equal((await f.reader.pages('name=hive-cleanup-attempt-101-1')).length, 1);
  const applied = await applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    finalize: f.finalize, ...f.dependencies});
  assert.equal(applied.state, 'completed'); assert.equal(f.effects.length, 1);
});
test('queued claim identity is revalidated even when another claim already quarantines the owner', async () => {
  for (const field of ['cleanupPolicyHash', 'selectedSourceFromOwner', 'protocolHead', 'ownerArtifactDigest']) {
    const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
    const claim = structuredClone(prepared.claim); claim[field] = field === 'ownerArtifactDigest'
      ? 'sha256:' + '0'.repeat(64) : '0'.repeat(field === 'cleanupPolicyHash' ? 64 : 40);
    await assert.rejects(revalidateCleanupClaim({reader: f.reader, producer, claim, ...f.dependencies}));
    assert.equal(f.effects.length, 0);
  }
});
test('queued claim from another invocation rejects before any transport or mutation', async () => {
  const f = fixture(), prepared = await prepare(f); f.calls.length = 0;
  await assert.rejects(revalidateCleanupClaim({reader: f.reader, producer: {...producer, attempt: 2},
    claim: prepared.claim, ...f.dependencies}));
  assert.equal(f.calls.length, 0); assert.equal(f.effects.length, 0);
});
test('an incomplete cleanup receipt cannot be promoted to a completed outcome', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  const result = await applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    ...f.dependencies, finalize: async () => ({applied: {results: [{status: 'retained'}]}, cleanupReceipt: {complete: false}})});
  assert.equal(result.state, 'retained'); assert.equal(result.cleanupReceipt.complete, false);
});
test('pre-claim materialization retains a blocked decision without creating another durable claim', async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-preclaim-'));
  try {
    const blocked = writeCleanupRecheck({eligible: false, run: 101, attempt: 1, reason: 'prior-claim-quarantined'}, directory);
    assert.equal(blocked.schema, 'owned-hive-cleanup-preclaim/v1'); assert.equal(blocked.claimPath, null);
    assert.deepEqual(fs.readdirSync(directory), []);
    const f = fixture(), prepared = await prepare(f), fresh = writeCleanupRecheck(prepared, directory);
    assert.deepEqual(JSON.parse(fs.readFileSync(fresh.claimPath)), prepared.claim);
    assert.throws(() => writeCleanupRecheck({...prepared, run: '../escape'}, directory));
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});
test('actual cleanup workflow rechecks inside owner serialization and gates every effect on fresh eligibility', () => {
  const workflow = YAML.parse(fs.readFileSync('.github/workflows/hive-original-delivery-finalize.yml', 'utf8'));
  const job = workflow.jobs.sweep, steps = job.steps, recheck = steps.findIndex(step => step.id === 'recheck');
  assert.match(job.concurrency.group, /matrix\.owner\.run.*matrix\.owner\.attempt/u);
  assert.equal(job.concurrency['cancel-in-progress'], false); assert(recheck >= 0);
  assert.match(steps[recheck].run, /finalize-owned-hive-delivery-runs\.mjs recheck/u);
  const effects = [steps.find(step => step.id === 'claim'),
    steps.find(step => step.run?.includes('finalize-owned-hive-delivery-runs.mjs apply')),
    steps.find(step => step.with?.name?.startsWith('hive-cleanup-outcome-'))];
  for (const step of effects) {
    assert(steps.indexOf(step) > recheck); assert.equal(typeof step.if, 'string');
    const expression = step.if.replace(/^\$\{\{\s*|\s*\}\}$/gu, '');
    for (const eligible of ['true', 'false']) assert.equal(vm.runInNewContext(expression,
      {steps: {recheck: {outputs: {eligible}}}, always: () => true}, {timeout: 100}), eligible === 'true');
  }
});
test('changed owner artifact identity after claim is refused', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  f.metadata.digest = 'sha256:' + '3'.repeat(64);
  await assert.rejects(applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
});
test('uncertain mutation outcome remains a claim and is never retried', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  const finalize = async options => { f.effects.push(options); throw Error('transport interrupted after effect'); };
  const result = await applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21, finalize, ...f.dependencies});
  assert.equal(result.state, 'uncertain'); assert.equal((await prepare(f)).eligible, false);
  assert.equal(f.effects.length, 1);
});
test('bounded exact-name enumeration exposes continuation instead of losing older owners', async () => {
  const f = fixture(); f.addOwner(102, 12);
  const result = await prepareOwnedHiveRuns({reader: f.reader, producer, maxOwners: 1,
    cursor: {page: 1, offset: 0}, ...f.dependencies});
  assert.equal(result.owners.length, 1); assert.deepEqual(result.continuation, {page: 1, offset: 1});
  assert.equal(result.incomplete, true); assert.equal(f.effects.length, 0);
  assert(f.calls.some(endpoint => endpoint.includes('/actions/artifacts?name=hive-real-effect-owner')));
});
test('active and untrusted protocol owners never produce claims', async () => {
  for (const variant of ['active', 'untrusted']) {
    const f = fixture(), original = f.reader.read;
    f.reader.read = async endpoint => {
      const value = await original(endpoint);
      if (variant === 'active' && endpoint.includes('/runs/101/')) return {...value, status: 'in_progress'};
      if (variant === 'untrusted' && endpoint.includes('/compare/')) return {...value, status: 'diverged'};
      return value;
    };
    await assert.rejects(prepare(f)); assert.equal(f.effects.length, 0);
  }
});
test('authenticated durable progress advances scheduled discovery beyond its previous bounded window', async () => {
  const f = fixture();
  const progress = await makeScanProgress({reader: f.reader, producer, now: Date.parse('2026-10-21T12:00:00Z'),
    continuation: {page: 8, offset: 17}, ...f.dependencies});
  const metadata = {id: 31, name: progress.name, expired: false, size_in_bytes: 100,
    created_at: '2026-10-21T12:00:00Z', workflow_run: {id: producer.run, head_sha: cleanupProtocol}};
  f.documents.set(31, progress.document); f.setProgress([metadata]);
  assert.deepEqual(await readScanCursor({reader: f.reader, now: Date.parse('2026-10-21T13:00:00Z'), ...f.dependencies}),
    {page: 8, offset: 17});
  progress.document.cleanupPolicyHash = '0'.repeat(64);
  await assert.rejects(readScanCursor({reader: f.reader, now: Date.parse('2026-10-21T13:00:00Z'), ...f.dependencies}));
});

test('progress journals larger than the old daily cap retain authenticated continuation with bounded reads', async () => {
  const f=fixture(),now=Date.parse('2026-10-21T13:00:00Z');
  const progress=await makeScanProgress({reader:f.reader,producer,now,continuation:{page:8,offset:17},...f.dependencies});
  const values=Array.from({length:1000},(_,index)=>({id:2000-index,name:progress.name,expired:false,size_in_bytes:100,
    created_at:new Date(now-index*1000).toISOString(),workflow_run:{id:producer.run,head_sha:cleanupProtocol}}));
  for(const metadata of values)f.documents.set(metadata.id,structuredClone(progress.document));
  f.setProgress(values);
  assert.deepEqual(await readScanCursor({reader:f.reader,now,...f.dependencies}),{page:8,offset:17});
  const reads=f.calls.filter(value=>value.includes('name=hive-cleanup-scan-progress-'));
  assert.equal(reads.length,2);assert.ok(reads.every(value=>value.endsWith('&per_page=100&page=1')));
  assert.equal(f.calls.filter(value=>value.startsWith('artifact-json/')).length,1);
  assert.equal(f.effects.length,0);
  f.documents.get(values[0].id).schema='foreign-progress';
  const beforeFallback=f.calls.filter(value=>value.startsWith('artifact-json/')).length;
  assert.deepEqual(await readScanCursor({reader:f.reader,now,...f.dependencies}),{page:8,offset:17});
  assert.equal(f.calls.filter(value=>value.startsWith('artifact-json/')).length-beforeFallback,2);
  for(const metadata of values)f.documents.get(metadata.id).cleanupPolicyHash='0'.repeat(64);
  await assert.rejects(readScanCursor({reader:f.reader,now,...f.dependencies}),/without an authenticated cursor/u);
  assert.equal(f.effects.length,0);
});

test('malformed progress windows cannot masquerade as absent history',async()=>{
  for(const listing of [{total_count:1,artifacts:[]},{total_count:0,artifacts:[{id:1}]},
    {total_count:2,artifacts:[{id:1},{id:1}]}]) {
    const f=fixture(),read=f.reader.read;
    f.reader.read=endpoint=>endpoint.includes('name=hive-cleanup-scan-progress-')?listing:read(endpoint);
    await assert.rejects(readScanCursor({reader:f.reader,now:Date.parse('2026-10-21T13:00:00Z'),...f.dependencies}));
    assert.equal(f.effects.length,0);
  }
});

test('cancelled no-op admissions do not enqueue cleanup scans; schedules and failed deliveries still do',()=>{
  const workflow=YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/hive-original-delivery-finalize.yml',import.meta.url),'utf8'));
  const expression=workflow.jobs.prepare.if.replace(/^\$\{\{\s*|\s*\}\}$/gu,'');
  const eligible=(event_name,conclusion,full_name='link-assistant/formal-ai')=>vm.runInNewContext(expression,
    {github:{event_name,repository:'link-assistant/formal-ai',event:{workflow_run:{conclusion,head_repository:{full_name}}}}},{timeout:100});
  assert.equal(eligible('workflow_run','cancelled'),false);
  assert.equal(eligible('workflow_run','failure'),true);
  assert.equal(eligible('workflow_run','success'),true);
  assert.equal(eligible('workflow_run','success','foreign/repo'),false);
  assert.equal(eligible('schedule','cancelled'),true);
});

test('published claim and outcome filenames match exact sole artifact members', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'cleanup-artifact-members-'));
  try {
    const claim = {schema: 'synthetic-local-file-only', sourceRun: 101, sourceAttempt: 1};
    const prepared = writeCleanupPreparation({owners: [{run: 101, attempt: 1, claim}], progress: {continuation: null}}, directory);
    assert.equal(path.basename(prepared.owners[0].claimPath), 'claim.json');
    assert.deepEqual(JSON.parse(fs.readFileSync(prepared.owners[0].claimPath)), claim);
    const outcome = {sourceRun: 101, sourceAttempt: 1, state: 'completed'};
    const published = writeCleanupOutcome(outcome, directory);
    assert.equal(path.basename(published.outcomePath), 'outcome.json');
    assert.deepEqual(JSON.parse(fs.readFileSync(published.outcomePath)), outcome);
    assert.throws(() => writeCleanupOutcome(outcome, directory));
    assert.throws(() => writeCleanupPreparation({owners: [{run: '../other', attempt: 1, claim}], progress: {}}, directory));
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});
