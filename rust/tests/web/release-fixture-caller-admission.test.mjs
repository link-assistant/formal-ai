import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { validateCallerAdmission, observeCallerAdmission, requireNonterminalFixtureRun } from '../../../scripts/release-fixture-caller-admission.mjs';
import { validateDiagnosticDescendant } from '../../../scripts/release-fixture-lease-diagnostics.mjs';

const context = Object.freeze({repository: 'owner/repo', run: '123', attempt: '2', eventHead: 'a'.repeat(40)});
const base = '/repos/owner/repo/actions';
const groupName = 'formal-ai-release-fixture-123-2';
const childName = 'Active reusable fixture / Fixture lease holder active';
function syntheticSnapshots() {
  const job = {id: 42, name: childName, run_id: 123, run_attempt: 2, head_sha: context.eventHead,
    status: 'queued', completed_at: null, url: 'https://api.github.com' + base + '/jobs/42'};
  return [
    {status: 200, route: base + '/runs/123', observedAt: '2026-10-11T00:00:00Z',
      body: {id: 123, run_attempt: 2, head_sha: context.eventHead,
        path: '.github/workflows/release-runtime-fixture.yml', status: 'in_progress', conclusion: null}},
    {status: 200, route: base + '/runs/123/jobs?filter=latest&per_page=100',
      observedAt: '2026-10-11T00:00:01Z', body: {total_count: 1, jobs: [job]}},
    {status: 200, route: base + '/concurrency_groups/' + groupName,
      observedAt: '2026-10-11T00:00:02Z', body: {group_name: groupName, total_count: 1,
        group_members: [{run_id: 123, job_id: 42, job_name: childName, status: 'in_progress',
          job_url: job.url, run_url: 'https://api.github.com' + base + '/runs/123'}]}}
  ];
}

test('synthetic admission permits queuing but refuses executing-lease promotion', () => {
  const snapshots = syntheticSnapshots();
  const receipt = validateCallerAdmission(...snapshots, context);
  assert.equal(receipt.observation, 'ObservedCallerAdmissionForContender');
  assert.equal(receipt.descendantStatus, 'queued');
  assert.equal(receipt.executingLeaseProof, false);
  assert.equal(receipt.productionAuthority, false);
  assert.throws(() => validateDiagnosticDescendant({status: 200, body: snapshots[1].body.jobs[0]},
    {...context, id: 42, name: childName, runnerName: 'synthetic-runner'}));
});

test('synthetic admitted executing status still carries no lease proof', () => {
  const snapshots = syntheticSnapshots();
  snapshots[1].body.jobs[0].status = 'in_progress';
  assert.equal(validateCallerAdmission(...snapshots, context).executingLeaseProof, false);
});

test('synthetic foreign, ambiguous, truncated, terminal and stale admissions refuse', () => {
  const mutations = [
    values => values[0].status = 403,
    values => values[0].route += '/foreign',
    values => values[0].body.id = 124,
    values => values[0].body.run_attempt = 1,
    values => values[0].body.head_sha = 'b'.repeat(40),
    values => values[0].body.path = '.github/workflows/release.yml',
    values => values[0].body.status = 'completed',
    values => values[1].body.jobs.push(structuredClone(values[1].body.jobs[0])),
    values => values[1].body.total_count = 2,
    values => values[1].body.jobs[0].name = 'foreign caller',
    values => values[1].body.jobs[0].run_id = 124,
    values => values[1].body.jobs[0].run_attempt = 1,
    values => values[1].body.jobs[0].head_sha = 'b'.repeat(40),
    values => values[1].body.jobs[0].status = 'completed',
    values => values[1].body.jobs[0].completed_at = '2026-10-11T00:00:01Z',
    values => values[1].body.jobs[0].url = 'https://foreign.invalid',
    values => values[2].body.group_name = 'formal-ai-release-fixture-123-1',
    values => values[2].body.group_members[0].status = 'pending',
    values => values[2].body.group_members[0].job_id = 43,
    values => values[2].body.group_members[0].run_id = 124,
    values => values[2].body.group_members[0].job_url = 'https://foreign.invalid',
    values => values[2].body.group_members.push(structuredClone(values[2].body.group_members[0])),
    values => values[2].body.total_count = 2,
    values => values[2].body.group_members.push({job_name: 'Fixture private queued contender'}),
    values => values[2].observedAt = '2026-10-11T00:00:06Z',
    values => values[1].observedAt = '2026-10-10T23:59:59Z'
  ];
  for (const mutate of mutations) {
    const values = syntheticSnapshots();
    mutate(values);
    assert.throws(() => validateCallerAdmission(...values, context));
  }
});

test('synthetic bounded GET orchestration preserves routes and never dispatches work', async () => {
  const values = syntheticSnapshots();
  const calls = [];
  const receipt = await observeCallerAdmission(async (route, timeout) => {
    calls.push({route, timeout});
    return values[calls.length - 1];
  }, context, Date.now() + 1000);
  assert.deepEqual(calls.map(value => value.route), values.map(value => value.route));
  assert.ok(calls.every(value => value.timeout > 0 && value.timeout <= 1000));
  assert.equal(receipt.executingLeaseProof, false);
  await assert.rejects(observeCallerAdmission(() => {throw Error('must not call');}, context, Date.now() - 1));
});

import { observeActiveAncestor } from '../../../scripts/release-runtime-fixture.mjs';
function syntheticPhysicalReceipts() {
  const physical = {id: 42, name: childName, run_id: 123, run_attempt: 2, head_sha: context.eventHead,
    status: 'in_progress', runner_name: 'synthetic-runner', runner_id: 7,
    completed_at:null,started_at:'2026-10-11T00:00:00Z',
    url: 'https://api.github.com' + base + '/jobs/42'};
  return [{status: 200, observedAt:'2026-10-11T00:00:01Z', body: {total_count:1,jobs: [structuredClone(physical)]}},
    {status: 200, observedAt:'2026-10-11T00:00:02Z', body: physical},
    {status: 200, observedAt:'2026-10-11T00:00:04Z', body: {group_name: groupName,total_count:1,
      group_members: [{run_id:123,job_id:42,job_name:childName,status:'in_progress',job_url:physical.url,
        run_url:'https://api.github.com'+base+'/runs/123'}]}}];
}
async function syntheticPhysicalAttempt(values, runnerName = 'synthetic-runner') {
  const calls = [];
  const receipt = await observeActiveAncestor({
    get: async route => {calls.push(route);
      if(route===base+'/runs/123')return {...syntheticSnapshots()[0],observedAt:'2026-10-11T00:00:03Z'};
      return values[route.includes('/jobs?')?0:route.includes('/jobs/')?1:2];},
    context, name: groupName, label: 'Fixture lease holder active', snapshots: [], runnerName
  });
  return {calls, receipt};
}

test('synthetic physical execution guard binds exact current runner, source, run and attempt', async () => {
  const result = await syntheticPhysicalAttempt(syntheticPhysicalReceipts());
  assert.equal(result.receipt.descendantJobId, 42);
  assert.deepEqual(result.calls, [base + '/runs/123/jobs?filter=latest&per_page=100',
    base + '/jobs/42', base + '/runs/123', base + '/concurrency_groups/' + groupName]);
});

test('synthetic queued, foreign and missing physical execution cannot use admission as a witness', async () => {
  for (const mutate of [
    values => values[1].status = 403,
    values => values[1].body.id = 43,
    values => values[1].body.run_id = 124,
    values => values[1].body.run_attempt = 1,
    values => values[1].body.head_sha = 'b'.repeat(40),
    values => values[1].body.name = 'unrelated descendant',
    values => values[1].body.runner_name = 'foreign-runner',
    values => values[1].body.runner_id = 0,
    values => values[1].body.status = 'queued',
    values => values[1].body.url = 'https://foreign.invalid'
  ]) {
    const values = syntheticPhysicalReceipts();
    mutate(values);
    await assert.rejects(syntheticPhysicalAttempt(values));
  }
  await assert.rejects(syntheticPhysicalAttempt(syntheticPhysicalReceipts(), null));
  await assert.rejects(syntheticPhysicalAttempt(syntheticPhysicalReceipts(), ''));
});

test('production queue call site supplies missing-runner refusal rather than optional omission', () => {
  const source = readFileSync(new URL('../../../scripts/release-runtime-fixture.mjs', import.meta.url), 'utf8');
  assert.ok(source.includes('runnerName: process.env.RUNNER_NAME ?? null'));
  assert.ok(source.includes("assert.equal(typeof runnerName, 'string')"));
});

test('queued aggregate requires authentic matching admitted group without execution promotion',()=>{
 const values=syntheticSnapshots();values[0].body.status='queued';
 const observed=validateCallerAdmission(...values,context);
 assert.equal(observed.descendantStatus,'queued');assert.equal(observed.executingLeaseProof,false);assert.equal(observed.productionAuthority,false);
 for(const mutate of [
  x=>x[0].body.conclusion='success',
  x=>delete x[0].body.conclusion,
  x=>x[0].body.status='unrecognized',
  x=>x[2].body.group_members[0].status='queued',
  x=>x[2].body.group_members[0].job_id=43,
  x=>x[2].observedAt='2026-10-11T00:00:06Z'
 ]){
  const x=structuredClone(values);
  mutate(x);
  assert.throws(()=>validateCallerAdmission(...x,context));
 }
});

test('admission follows the authenticated source-declared active callee phase', () => {
 for (const phase of ['Fixture artifact consumer active', 'Fixture final active']) {
  const values=syntheticSnapshots(),old=structuredClone(values[1].body.jobs[0]);
  old.id=41;old.status='completed';old.completed_at='2026-10-11T00:00:00Z';
  values[1].body.jobs.unshift(old);values[1].body.total_count=2;
  values[1].body.jobs[1].name='Active reusable fixture / '+phase;
  values[1].body.jobs[1].status='in_progress';
  values[2].body.group_members[0].job_name=values[1].body.jobs[1].name;
  const admission=validateCallerAdmission(...values,context);
  assert.equal(admission.descendantJobId,42);assert.equal(admission.executingLeaseProof,false);
  assert.equal(admission.productionAuthority,false);
  const foreign=structuredClone(values);foreign[1].body.jobs[1].name='Inactive reusable fixture / '+phase;
  foreign[2].body.group_members[0].job_name=foreign[1].body.jobs[1].name;
  assert.throws(()=>validateCallerAdmission(...foreign,context));
  for(const mutate of [x=>x[2].body.group_members[0].status='pending',x=>x[1].body.jobs[1].status='completed',
   x=>x[1].body.jobs[1].run_attempt=1,x=>x[1].body.jobs[1].head_sha='b'.repeat(40),
   x=>x[2].body.group_members[0].job_name='foreign phase']){
   const changed=structuredClone(values);mutate(changed);
   assert.throws(()=>validateCallerAdmission(...changed,context));
  }
 }
});

test('admission refuses duplicate selected phase names even with distinct job ids', () => {
 for(const phase of ['Fixture lease holder active','Fixture artifact consumer active','Fixture final active']) {
  const values=syntheticSnapshots();values[1].body.jobs[0].name='Active reusable fixture / '+phase;
  values[1].body.jobs[0].status='in_progress';
  values[2].body.group_members[0].job_name=values[1].body.jobs[0].name;
  const duplicate=structuredClone(values[1].body.jobs[0]);duplicate.id=43;
  duplicate.url='https://api.github.com'+base+'/jobs/43';
  values[1].body.jobs.push(duplicate);values[1].body.total_count=2;
  assert.throws(()=>validateCallerAdmission(...values,context));
 }
});


test('captured source-bound admission accepts the actual queued consumer without execution authority', async () => {
  const fixture=JSON.parse(readFileSync(new URL('../fixtures/release-fixture-admission-788.json',import.meta.url)));
  assert.equal(fixture.snapshots[1].body.total_count,7);
  assert.equal(fixture.snapshots[1].body.jobs.length,7);
  const admission=validateCallerAdmission(...fixture.snapshots,fixture.context);
  assert.equal(admission.descendantJobId,119233027993);
  assert.equal(admission.descendantStatus,'queued');
  assert.equal(admission.executingLeaseProof,false);
  assert.equal(admission.productionAuthority,false);
  const child=fixture.snapshots[1].body.jobs.find(job=>job.id===admission.descendantJobId);
  const physical={...child,runner_id:7,runner_name:'synthetic-heldout-runner'};
  assert.throws(()=>validateDiagnosticDescendant({status:200,body:physical},
    {...fixture.context,id:physical.id,name:physical.name,runnerName:physical.runner_name}),/in_progress/u);
  await assert.rejects(observeActiveAncestor({get:async()=>fixture.snapshots[1],context:fixture.context,
    name:fixture.snapshots[2].body.group_name,label:'Fixture artifact consumer active',snapshots:[],
    runnerName:physical.runner_name}),/executing descendant/u);
});
test('all three admitted queued phases retain strict group identity and reject physical execution promotion', async () => {
  for(const phase of ['Fixture lease holder active','Fixture artifact consumer active','Fixture final active']) {
    const values=syntheticSnapshots();const child=values[1].body.jobs[0];child.name='Active reusable fixture / '+phase;
    values[2].body.group_members[0].job_name=child.name;
    const admission=validateCallerAdmission(...values,context);
    assert.equal(admission.descendantStatus,'queued');assert.equal(admission.executingLeaseProof,false);
    assert.equal(admission.productionAuthority,false);
    const physical={...child,runner_id:7,runner_name:'synthetic-heldout-runner'};
    assert.throws(()=>validateDiagnosticDescendant({status:200,body:physical},
      {...context,id:child.id,name:child.name,runnerName:physical.runner_name}),/in_progress/u);
    await assert.rejects(observeActiveAncestor({get:async()=>values[1],context,name:groupName,label:phase,
      snapshots:[],runnerName:physical.runner_name}),/executing descendant/u);
    for(const mutate of [x=>x[2].body.group_members[0].status='pending',
      x=>x[2].body.group_members[0].job_id=43,x=>x[1].body.jobs[0].run_attempt=1,
      x=>x[1].body.jobs[0].head_sha='b'.repeat(40),x=>x[1].body.jobs[0].completed_at='2026-10-11T00:00:01Z']) {
      const changed=structuredClone(values);mutate(changed);assert.throws(()=>validateCallerAdmission(...changed,context));
    }
  }
});


test('documented nonterminal aggregate states retain null conclusion and never confer execution authority',()=>{
 for(const status of ['queued','in_progress','requested','waiting','pending']){
  const values=syntheticSnapshots();values[0].body.status=status;
  assert.equal(requireNonterminalFixtureRun(values[0].body),true);
  const admission=validateCallerAdmission(...values,context);
  assert.equal(admission.executingLeaseProof,false);assert.equal(admission.productionAuthority,false);
  for(const conclusion of ['success','failure','cancelled','neutral',undefined]){
   const changed=structuredClone(values);changed[0].body.conclusion=conclusion;
   assert.throws(()=>validateCallerAdmission(...changed,context));
  }
 }
 for(const status of ['completed','success','unknown','',null,undefined]){
  assert.throws(()=>requireNonterminalFixtureRun({status,conclusion:null}));
 }
});

test('captured cbd pending aggregate is nonterminal while stale prior-phase membership remains no final proof',()=>{
 const raw=readFileSync(new URL('../fixtures/release-fixture-pending-aggregate-cbd3.json',import.meta.url));
 assert.equal(createHash('sha256').update(raw).digest('hex'),'35ec856b7ddc20966680c05c00eca67246f2e6d44ff5000a72363b2fea1b2b25');
 const capture=JSON.parse(raw),[group,jobs,physical,run]=capture.snapshots;
 assert(capture.snapshots.every(value=>value.status===200));
 assert.equal(run.body.id,39617489419);assert.equal(run.body.head_sha,capture.context.eventHead);
 assert.equal(String(run.body.run_attempt),capture.context.attempt);
 assert.equal(run.body.status,'pending');assert.equal(run.body.conclusion,null);
 assert.equal(requireNonterminalFixtureRun(run.body),true);
 const child=jobs.body.jobs.find(value=>value.id===physical.body.id);assert(child);
 assert.equal(child.id,119288808781);assert.equal(child.status,'in_progress');assert.equal(physical.body.status,'in_progress');
 assert.equal(child.head_sha,run.body.head_sha);assert.equal(child.run_attempt,run.body.run_attempt);
 const times=capture.snapshots.map(value=>Date.parse(value.observedAt));
 assert.deepEqual(times,[Date.parse('2026-10-21T14:46:33.607Z'),Date.parse('2026-10-21T14:46:34.097Z'),
  Date.parse('2026-10-21T14:46:34.368Z'),Date.parse('2026-10-21T14:46:34.655Z')]);
 assert.ok(Math.max(...times)-Math.min(...times)<5000);
 // This initial group still named the prior consumer; no later final-group snapshot was captured.
 const active=group.body.group_members.find(value=>value.status==='in_progress');
 assert.equal(active.job_id,119287510771);assert.notEqual(active.job_id,child.id);
 // Runner facts were redacted. This capture establishes aggregate classification only.
 assert.throws(()=>validateDiagnosticDescendant(physical,{...capture.context,id:child.id,name:child.name,runnerName:'unproven'}));
 for(const change of [x=>x.status='completed',x=>x.status='unknown',x=>x.conclusion='success']){
  const changed=structuredClone(run.body);change(changed);assert.throws(()=>requireNonterminalFixtureRun(changed));
 }
});
