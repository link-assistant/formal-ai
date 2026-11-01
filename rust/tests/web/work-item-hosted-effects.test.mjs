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
import {explicitUnknownTerminal} from '../../../scripts/work-item-qualification/trace-oracle.mjs';
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
