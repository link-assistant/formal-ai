import {ownedPushRequest} from '../../../scripts/work-item-qualification/push-policy.mjs';
import {createCommandCapture} from '../../../scripts/work-item-qualification/command-capture.mjs';
// Focused hosted-effect contracts; actual hosted execution remains separate.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {startBroker} from '../../../scripts/work-item-qualification/transport-broker.mjs';
import {requireWorkItemOwner,cleanupWorkItemReceipt} from '../../../scripts/work-item-qualification/cleanup-contract.mjs';
import {ownedTaskBody} from '../../../scripts/work-item-qualification/fixtures.mjs';
import {explicitUnknownTerminal,cleanQualificationPass} from '../../../scripts/work-item-qualification/trace-oracle.mjs';
const digest=value=>createHash('sha256').update(value).digest('hex');
const hash='a'.repeat(40),head='b'.repeat(40),policyHash='c'.repeat(64),inputHash='d'.repeat(64);
const owner={schema:'formal-ai-hive-effect-owner/v1',taskMode:'work-item-1483',repository:'link-assistant/formal-ai',
  run:123,attempt:1,protocolCommit:hash,selectedSource:hash,scope:'pr-1495',policyHash,inputHash,tools:['agent','claude','codex']};
const run={id:123,run_attempt:1,head_sha:hash,path:'.github/workflows/work-item-qualification.yml',event:'workflow_dispatch',status:'completed',
  repository:{full_name:owner.repository},head_repository:{full_name:owner.repository}};
function fixture(){
  const expectedFixture={title:'Controlled task',body:'Observed contract without a final patch'};
  const receipt={schema:'formal-ai-work-item-owned-task/v1',owner:structuredClone(owner),repository:owner.repository,
    source:hash,tool:'agent',task:'original-documentation',baseBranch:'e2e/work-item/123/agent/original-documentation',
    baseCommit:hash,issueNumber:77,pullRequestNumber:78,head,job:321,complete:false,failure:'real test failed'};
  receipt.issueBodySha256=digest(ownedTaskBody(expectedFixture,owner,receipt.tool,receipt.task));
  const job={id:321,name:'Actual Hive agent work-item original-documentation',status:'completed',
    started_at:'2026-11-01T00:00:00Z',completed_at:'2026-11-01T00:05:00Z'};
  const issue={number:77,title:expectedFixture.title,body:ownedTaskBody(expectedFixture,owner,receipt.tool,receipt.task),
    user:{login:'github-actions[bot]'},state:'open',created_at:'2026-11-01T00:01:00Z',comments:0,
    labels:[{name:'e2e-task'},{name:'no-formal-ai-draft'}]};
  const pr={number:78,merged:false,merged_at:null,state:'open',base:{ref:receipt.baseBranch,repo:{full_name:owner.repository}},
    head:{ref:'issue-77-candidate',sha:head,repo:{full_name:owner.repository}}};
  const effects=[], saved=[];let protectedBranch=false,uncertainComment=false;
  const api=async(endpoint,{method='GET',body}={})=>{
    if(method!=='GET'){
      effects.push({endpoint,method,body});
      if(uncertainComment&&method==='POST')throw Error('uncertain write; must not retry');
      if(method==='PATCH'&&endpoint.endsWith('/issues/77'))issue.state='closed';
      if(method==='PATCH'&&endpoint.endsWith('/pulls/78'))pr.state='closed';
      return {};
    }
    if(endpoint.endsWith('/issues/77'))return structuredClone(issue);
    if(endpoint.endsWith('/pulls/78'))return structuredClone(pr);
    if(endpoint.includes('/comments'))return [];
    if(endpoint.includes('/pulls?'))return [structuredClone(pr)];
    if(endpoint.includes('/rules/branches/'))return protectedBranch?[{type:'deletion',ruleset_id:9}]:[];
    if(endpoint.includes('/git/ref/heads/'))return {object:{sha:endpoint.includes('issue-77')?head:hash}};
    throw Error('unexpected fake endpoint '+endpoint);
  };
  return {expectedFixture,receipt,job,issue,pr,effects,saved,api,
    protect(){protectedBranch=true;},uncertain(){uncertainComment=true;},
    options(){return {receipt,owner,run,jobs:[job],api,expectedFixture,save:row=>saved.push(structuredClone(row))};}};
}
test('cleanup accepts only this terminal exact-source workflow owner',()=>{
  assert.equal(requireWorkItemOwner({owner,run,policyHash,inputHash}),owner);
  for(const changed of [{status:'in_progress'},{head_sha:'e'.repeat(40)},{path:'.github/workflows/other.yml'},{run_attempt:2}])
    assert.throws(()=>requireWorkItemOwner({owner,run:{...run,...changed},policyHash,inputHash}));
});
test('failed actual task is closed with failed status and exact owned refs only',async()=>{
  const f=fixture(),result=await cleanupWorkItemReceipt(f.options());
  assert.equal(result.outcome,'failed');assert.equal(result.state,'complete');
  assert.match(f.effects[0].body.body,/outcome: failed/);
  assert.equal(f.effects.find(row=>row.method==='PATCH'&&row.endpoint.endsWith('/issues/77')).body.state_reason,'not_planned');
  assert.equal(f.effects.filter(row=>row.method==='DELETE').length,2);
  assert(f.effects.every(row=>!row.endpoint.endsWith('/main')));
  const passed=fixture();passed.receipt.complete=true;delete passed.receipt.failure;
  await cleanupWorkItemReceipt(passed.options());
  assert.equal(passed.effects.find(row=>row.method==='PATCH'&&row.endpoint.endsWith('/issues/77')).body.state_reason,'completed');
});
test('a human or changed requirement prevents every cleanup mutation',async()=>{
  for(const change of [f=>f.issue.user.login='person',f=>f.issue.body+=' changed',f=>f.pr.head.sha='e'.repeat(40),f=>f.pr.merged=true]){
    const f=fixture();change(f);await assert.rejects(cleanupWorkItemReceipt(f.options()));assert.deepEqual(f.effects,[]);
  }
});
test('protected owned branches remain accurately recorded residue',async()=>{
  const f=fixture();f.protect();const result=await cleanupWorkItemReceipt(f.options());
  assert.equal(result.state,'protected-residue');assert.equal(result.retained.length,2);
  assert.equal(f.effects.filter(row=>row.method==='DELETE').length,0);
});
test('an expected unsupported negative case stays unresolved even when its oracle passes',async()=>{
  const f=fixture();f.receipt.complete=true;f.receipt.taskDisposition='unresolved';delete f.receipt.failure;
  const result=await cleanupWorkItemReceipt(f.options());assert.equal(result.outcome,'unresolved');
  assert.match(f.effects[0].body.body,/outcome: unresolved/);
  assert.equal(f.effects.find(row=>row.method==='PATCH'&&row.endpoint.endsWith('/issues/77')).body.state_reason,'not_planned');
});
test('uncertain comment write is attempted once and does not proceed to closes or deletes',async()=>{
  const f=fixture();f.uncertain();await assert.rejects(cleanupWorkItemReceipt(f.options()),/uncertain write/);
  assert.equal(f.effects.length,1);assert.equal(f.effects[0].method,'POST');
});
test('unsupported oracle needs explicit final unknown evidence and false verification',()=>{
  const unknown=explicitUnknownTerminal(JSON.stringify({schema:'repository-work-item-controller/v1',
    terminalState:'evidence_unavailable',verified:false,missingEvidence:[{kind:'image',status:'unavailable'}]}));
  assert.deepEqual([unknown.disposition,unknown.verified,unknown.unresolvedEvidence],['gap',false,true]);
  assert.equal(explicitUnknownTerminal('planned not executed').disposition,'unknown');
  const legacy=explicitUnknownTerminal(JSON.stringify({schema:'repository-work-item-evidence/v1',reason:'MissingReadTool',
    authored:false,verified:false,requirements:[{text:'unknown image',status:'unbound'}]}));
  assert.deepEqual([legacy.disposition,legacy.verified,legacy.unresolvedEvidence],['gap',false,true]);
  assert.equal(explicitUnknownTerminal(JSON.stringify({verified:true,terminalState:'gap',missingEvidence:[]})).unresolvedEvidence,false);
});
async function brokerFixture(fn){
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'work-item-broker-test-')),rpc=directory+'/rpc';fs.mkdirSync(rpc);
  const calls=[],events=[],policy={repository:owner.repository,issueNumber:77,baseBranch:'e2e/work-item/123/agent/original-documentation',
    baseCommit:hash,mirror:directory+'/frozen.git',privateDirectory:directory+'/private',privateMirror:directory+'/private/git'};
  const broker=startBroker({directory:rpc,policy,observe:row=>events.push(row),execute:(file,args)=>{
    calls.push({file,args});return {status:0,stdout:Buffer.from('{}'),stderr:Buffer.alloc(0)};
  }});
  let count=0;
  async function request(value){
    const name=String(++count);fs.writeFileSync(rpc+'/'+name+'.request.json',JSON.stringify(value));
    const target=rpc+'/'+name+'.response.json',deadline=Date.now()+2000;
    while(!fs.existsSync(target)){assert(Date.now()<deadline);await new Promise(resolve=>setTimeout(resolve,10));}
    return JSON.parse(fs.readFileSync(target));
  }
  try{await fn({request,calls,events,policy});}finally{broker.stop();fs.rmSync(directory,{recursive:true,force:true});}
}
test('actual gh read transport freezes contents refs and denies current answer history',async()=>brokerFixture(async f=>{
  assert.equal((await f.request({kind:'gh',args:['api','repos/'+owner.repository+'/contents/docs/guide.md?ref=main']})).status,0);
  assert.match(f.calls[0].args.at(-1),new RegExp('ref='+hash+'$'));
  const count=f.calls.length;
  for(const endpoint of ['/commits/main','/contents/../README.md','/git/blobs/'+head])
    assert.equal((await f.request({kind:'gh',args:['api','repos/'+owner.repository+endpoint]})).status,1);
  assert.equal(f.calls.length,count);
}));
test('unscoped lists and foreign GitHub commands never reach an effect executor',async()=>brokerFixture(async f=>{
  for(const args of [['pr','list','--repo',owner.repository],['search','code','known-answer'],['api','repos/other/repo/contents/answer']])
    assert.equal((await f.request({kind:'gh',args})).status,1);
  assert.deepEqual(f.calls,[]);
}));
test('host file options, jq environment access and duplicate repo overrides are refused before gh',async()=>brokerFixture(async f=>{
  for(const args of [
    ['api','repos/'+owner.repository+'/contents/guide.md','--jq','env.GH_TOKEN'],
    ['api','repos/'+owner.repository+'/issues/77','-f','body=unexpected'],
    ['pr','comment','78','--repo',owner.repository,'--body-file','/private/grader.json'],
    ['pr','list','--repo',owner.repository,'--repo','foreign/repository','--base',f.policy.baseBranch],
    ['pr','list','--repo',owner.repository,'--repo=foreign/repository','--base',f.policy.baseBranch]
  ])assert.equal((await f.request({kind:'gh',args})).status,1);
  assert.deepEqual(f.calls,[]);
}));

test('full process streams and errors are retained; capture budgets fail explicitly',context=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'work-item-command-capture-'));
  context.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  const capture=createCommandCapture(directory,{maximumBytes:32,maximumCommands:1});
  capture({program:'controlled-tool',args:['--inspect'],status:2,error:{code:'EFAIL'}},Buffer.from('雪'),Buffer.from('failed'),Buffer.from('request'));
  const base=path.join(directory,'commands');
  assert.equal(fs.readFileSync(path.join(base,'000001.stdout.log'),'utf8'),'雪');
  assert.equal(fs.readFileSync(path.join(base,'000001.stderr.log'),'utf8'),'failed');
  assert.equal(fs.readFileSync(path.join(base,'000001.stdin.log'),'utf8'),'request');
  const record=JSON.parse(fs.readFileSync(path.join(base,'000001.json')));
  assert.equal(record.status,2);assert.equal(record.complete,false);assert.equal(record.error.code,'EFAIL');
  for(const member of record.members)assert.equal(digest(fs.readFileSync(path.join(base,member.path))),member.sha256);
  assert.throws(()=>capture({status:0},Buffer.from('more'),Buffer.alloc(0)),/CommandCaptureBudgetExceeded/u);
  assert.equal(JSON.parse(fs.readFileSync(path.join(base,'capture-incomplete.json'))).complete,false);
  assert.throws(()=>createCommandCapture(directory,{maximumBytes:Infinity}),/InvalidCommandCaptureBounds/u);
  const limited=createCommandCapture(path.join(directory,'limited'),{maximumBytes:1});
  assert.throws(()=>limited({status:0},Buffer.from('雪'),Buffer.alloc(0)),/CommandCaptureBudgetExceeded/u);
});

test('dry-run and alternate push semantics are refused before any mutation',()=>{
  const policy={repository:'link-assistant/formal-ai',issueNumber:77};
  for(const args of [[],['-u','origin','issue-77-change'],['origin','HEAD']])assert.doesNotThrow(()=>ownedPushRequest(args,policy));
  for(const args of [['--dry-run','origin','HEAD'],['-n','origin','HEAD'],['--delete','origin','issue-77-change'],
    ['--mirror','origin'],['--all','origin'],['--force','origin','HEAD'],['--force-with-lease','origin','HEAD'],
    ['origin','HEAD:other'],['origin',':issue-77-change'],['foreign','HEAD'],['origin','issue-78-change']])
    assert.throws(()=>ownedPushRequest(args,policy),/UnsupportedPush|UnownedPush/u);
});

test('qualification credit requires a clean terminated client, including negative tasks',()=>{
  const healthy={status:0,cleanupComplete:true,signal:null,timedOut:false,error:null};
  assert.equal(cleanQualificationPass(healthy),true);
  assert.equal(cleanQualificationPass({...healthy,status:5}),false);
  assert.equal(cleanQualificationPass({...healthy,status:5},{unsupported:true}),true);
  for(const change of [{status:1},{status:null},{signal:'SIGTERM'},{timedOut:true},{error:{code:'EIO'}},{cleanupComplete:false},{cleanupComplete:undefined},{timedOut:undefined}]){
    assert.equal(cleanQualificationPass({...healthy,...change}),false);
    assert.equal(cleanQualificationPass({...healthy,...change},{unsupported:true}),false);
  }
  assert.equal(cleanQualificationPass(null),false);
});


// Run41768761745 exhausted the old180s pull cap before any client launch.
import YAML from 'yaml';
import {createWorkItemPhaseBudget} from '../../../scripts/work-item-qualification/phase-budget.mjs';

function phaseFixture(elapsed = 5 * 60 * 1000) {
  let clock = 1000;
  const origin = Date.parse('2026-11-04T03:48:00Z');
  const job = {
    id: 321, run_id: 123,
    name: 'Actual Hive agent work-item original-documentation',
    status: 'in_progress', started_at: new Date(origin).toISOString()
  };
  const options = {
    job, expectedName: job.name, expectedRun: 123,
    wallNow: origin + elapsed, monotonicNow: () => clock
  };
  return {
    job, options, budget: createWorkItemPhaseBudget(options),
    advance: n => clock += n, set: n => clock = n
  };
}

test('progressing pull can exceed180s within the25m job and3m reserve', () => {
  const f = phaseFixture();
  assert.equal(f.budget.allocate(600000, 'command:docker').timeoutMilliseconds, 600000);
  f.advance(181000);
  assert.equal(f.budget.snapshot().remainingMilliseconds, 17 * 60000 - 181000);
  f.advance(419000);
  assert.equal(f.budget.allocate(540000, 'command:docker').timeoutMilliseconds, 7 * 60000);
  assert.equal(f.budget.snapshot().reserveMilliseconds, 180000);
});

test('exhaustion refuses before any command or hosted write starts', () => {
  const f = phaseFixture(22 * 60000);
  let effects = 0;
  const execute = () => {
    const allocation = f.budget.allocate(30000, 'command:gh');
    effects++;
    return allocation;
  };
  assert.throws(execute, /WorkItemPhaseBudgetExhausted/u);
  assert.equal(effects, 0);
  const last = phaseFixture(22 * 60000 - 1);
  assert.equal(last.budget.allocate(540000, 'command:docker').timeoutMilliseconds, 1);
  last.advance(1);
  assert.throws(() => last.budget.allocate(540000, 'command:docker'), /BudgetExhausted/u);
});

test('invalid provenance, timestamps, caps and regressing clocks fail closed', () => {
  const f = phaseFixture();
  const changes = [
    {id: 0}, {run_id: 124}, {name: 'foreign'}, {status: 'completed'},
    {started_at: null}, {started_at: 'invalid'}, {started_at: '2026-11-05T03:48:00Z'}
  ];
  for (const change of changes) {
    assert.throws(() => createWorkItemPhaseBudget({...f.options, job: {...f.job, ...change}}));
  }
  for (const cap of [0, -1, NaN, Infinity, 1.5, 1500001]) {
    assert.throws(() => f.budget.allocate(cap, 'command:gh'));
  }
  assert.throws(() => f.budget.allocate(1000, 'untrusted phase'));
  f.set(999);
  assert.throws(() => f.budget.snapshot(), /MonotonicClock/u);
});

test('budget owns immutable job scalars and matches the actual workflow clock', () => {
  const f = phaseFixture(), before = f.budget.snapshot();
  Object.assign(f.job, {id: 999, run_id: 999, started_at: '2099-01-01T00:00:00Z'});
  assert.deepEqual(f.budget.snapshot(), before);
  assert(Object.isFrozen(f.budget));
  assert(Object.isFrozen(before));
  const workflow = YAML.parse(fs.readFileSync(
    new URL('../../../.github/workflows/work-item-qualification.yml', import.meta.url), 'utf8'
  ));
  assert.equal(workflow.jobs.qualification['timeout-minutes'] * 60000, before.jobMilliseconds);
  assert.equal(workflow.jobs.qualification['timeout-minutes'], 25);
});

async function brokerBudgetFixture(context, elapsed, execute) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'work-item-budget-broker-'));
  const f = phaseFixture(elapsed);
  const broker = startBroker({
    directory,
    policy: {repository: owner.repository, issueNumber: 77, baseBranch: 'owned', baseCommit: hash},
    observe: () => {},
    timeoutFor: (cap, file) => f.budget.allocate(cap, 'broker:' + file).timeoutMilliseconds,
    execute
  });
  context.after(() => {
    broker.stop();
    fs.rmSync(directory, {recursive: true, force: true});
  });
  fs.writeFileSync(directory + '/1.request.json', JSON.stringify({
    kind: 'gh', args: ['api', 'repos/' + owner.repository + '/issues/77']
  }));
  const deadline = Date.now() + 2000;
  while (!fs.existsSync(directory + '/1.response.json')) {
    assert(Date.now() < deadline);
    await new Promise(resolve => setTimeout(resolve, 10));
  }
  return JSON.parse(fs.readFileSync(directory + '/1.response.json'));
}

test('broker exhaustion refuses before its injected command executor', async context => {
  const calls = [];
  const response = await brokerBudgetFixture(context, 22 * 60000, (...args) => {
    calls.push(args);
    return {status: 0, stdout: Buffer.from('{}'), stderr: Buffer.alloc(0)};
  });
  assert.equal(response.status, 1);
  assert.match(Buffer.from(response.stderr, 'base64').toString(), /WorkItemPhaseBudgetExhausted/u);
  assert.deepEqual(calls, []);
});

test('broker passes the positive remaining750ms clamp to its command executor', async context => {
  const calls = [];
  const response = await brokerBudgetFixture(context, 22 * 60000 - 750, (...args) => {
    calls.push(args);
    return {status: 0, stdout: Buffer.from('{}'), stderr: Buffer.alloc(0)};
  });
  assert.equal(response.status, 0);
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], 'gh');
  assert.deepEqual(calls[0][2], {timeout: 750});
});


import {workItemIdentity} from '../../../scripts/work-item-qualification/policy.mjs';

test('historical work-item roots do not request a later command-budget module', async () => {
  const policyPath = 'scripts/work-item-qualification/policy.mjs';
  const policy = "export const workItemRoots = ['legacy.js', '" + policyPath + "'];";
  const members = new Map([[policyPath, Buffer.from(policy)], ['legacy.js', Buffer.from('old bytes')]]);
  const reads = [];
  const identity = await workItemIdentity({read: async name => {
    reads.push(name);
    assert(members.has(name), 'historical source cannot contain a later helper');
    return members.get(name);
  }});
  const expected = ['legacy.js', policyPath].map(name => ({path: name, sha256: digest(members.get(name))}));
  assert.equal(identity.policyHash, digest(JSON.stringify(expected)));
  assert.deepEqual(reads, [policyPath, 'legacy.js']);
});

test('current source binds its newly declared phase helper without changing member order', async () => {
  const policyPath = 'scripts/work-item-qualification/policy.mjs';
  const phasePath = 'scripts/work-item-qualification/phase-budget.mjs';
  const policy = "export const workItemRoots = ['" + phasePath + "', '" + policyPath + "'];";
  const members = new Map([[policyPath, Buffer.from(policy)], [phasePath, Buffer.from('current helper')]]);
  const identity = await workItemIdentity({read: async name => {
    assert(members.has(name));
    return members.get(name);
  }});
  const expected = [phasePath, policyPath].map(name => ({path: name, sha256: digest(members.get(name))}));
  assert.equal(identity.policyHash, digest(JSON.stringify(expected)));
});

test('changed frozen policy cannot satisfy an existing owner seal; malformed lists never evaluate', async () => {
  const policyPath = 'scripts/work-item-qualification/policy.mjs';
  const original = "export const workItemRoots = ['legacy.js', '" + policyPath + "'];";
  const identity = text => workItemIdentity({read: async name => Buffer.from(name === policyPath ? text : 'old bytes')});
  const sealed = await identity(original), changed = await identity(original + '\n// changed policy');
  assert.notEqual(sealed.policyHash, changed.policyHash);
  assert.throws(() => requireWorkItemOwner({
    owner: {...owner, policyHash: sealed.policyHash, inputHash: sealed.inputHash}, run, ...changed
  }));
  for (const list of ["['legacy.js']", "['../escape', '" + policyPath + "']",
    "['legacy.js', 'legacy.js', '" + policyPath + "']", "[globalThis.unowned(), '" + policyPath + "']"]) {
    await assert.rejects(identity('export const workItemRoots = ' + list + ';'));
  }
});


import {knownProtectedWorkItemResidue}
  from '../../../scripts/work-item-qualification/cleanup-contract.mjs';
import {classifyCleanupResult, nonGatingCleanupState}
  from '../../../scripts/lib/generated-task-cleanup-journal.mjs';

async function protectedCleanupFixture() {
  const f = fixture();
  f.protect();
  const protectedRow = await cleanupWorkItemReceipt(f.options());
  const outcomes = owner.tools.flatMap(tool =>
    ['original-documentation', 'renamed-components', 'contradicted-documentation', 'unsupported']
      .map(task => ({tool, task, state: 'not-started'})));
  Object.assign(outcomes[0], protectedRow);
  const workItem = {
    schema: 'formal-ai-work-item-cleanup-outcome/v1', run: owner.run, attempt: owner.attempt,
    complete: false, deliveryCredit: false, retained: 1, outcomes
  };
  const result = {
    cleanupReceipt: {complete: false, retained: 1, workItem},
    applied: {results: outcomes.map(row => ({status: row.state}))}
  };
  return {f, protectedRow, workItem, result};
}

test('protected deferral requires actual post-close issue and PR observations', async () => {
  const {f, protectedRow, workItem, result} = await protectedCleanupFixture();
  assert.deepEqual(protectedRow.closure.issue, {number: 77, state: 'closed'});
  assert.deepEqual(protectedRow.closure.pullRequest, {number: 78, state: 'closed'});
  assert.equal(f.effects.filter(row => row.method === 'DELETE').length, 0);
  assert.equal(knownProtectedWorkItemResidue(workItem, owner), true);
  assert.equal(await classifyCleanupResult(result, owner), 'deferred-protected');
  assert.equal(result.cleanupReceipt.complete, false);
  assert.equal(workItem.deliveryCredit, false);
  assert.equal(workItem.retained, 1);
  assert.equal(nonGatingCleanupState('deferred-protected'), true);
  assert.equal(nonGatingCleanupState('retained'), false);
});

test('unknown, early, foreign or unobserved residue remains gating', async () => {
  const {workItem, result} = await protectedCleanupFixture();
  const changes = [
    row => delete row.closure,
    row => row.closure.issue.state = 'open',
    row => row.closure.pullRequest.state = 'open',
    row => row.retained[0].reason = 'unknown',
    row => row.retained[0].head = 'e'.repeat(40),
    row => row.retained[0].branch = 'foreign',
    row => row.retained = [row.retained[0].branch],
    row => row.state = 'retained-uncertain'
  ];
  for (const change of changes) {
    const changed = structuredClone(workItem);
    change(changed.outcomes[0]);
    assert.equal(knownProtectedWorkItemResidue(changed, owner), false);
    assert.equal(await classifyCleanupResult({
      ...result, cleanupReceipt: {...result.cleanupReceipt, workItem: changed}
    }, owner), 'retained');
  }
  for (const changed of [
    {...workItem, outcomes: workItem.outcomes.slice(1)},
    {...workItem, outcomes: [...workItem.outcomes.slice(0, -1), workItem.outcomes[0]]},
    {...workItem, retained: 0}, {...workItem, run: 999}
  ]) assert.equal(knownProtectedWorkItemResidue(changed, owner), false);
});

test('uncertain effects retain precedence over known protected deferral', async () => {
  const {result} = await protectedCleanupFixture();
  assert.equal(await classifyCleanupResult({
    ...result, applied: {results: [{status: 'mutation-uncertain'}]}
  }, owner), 'uncertain');
  assert.equal(await classifyCleanupResult(result, {...owner, taskMode: 'historical-1364'}), 'retained');
  assert.equal(nonGatingCleanupState('uncertain'), false);
});
