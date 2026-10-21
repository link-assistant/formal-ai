import test from 'node:test';
import assert from 'node:assert/strict';
import { observeActiveAncestor, validateQueueSnapshot, pollFixtureObservation, requireFixtureApiSuccess } from '../../../scripts/release-runtime-fixture.mjs';
const context={repository:'link-assistant/formal-ai',run:'123',attempt:'1',eventHead:'a'.repeat(40)};
const base='/repos/'+context.repository+'/actions',name='formal-ai-release-fixture-123-1';
const job={id:456,run_id:123,run_attempt:1,head_sha:context.eventHead,
 name:'Active reusable fixture / Fixture lease holder active',status:'in_progress',
 runner_name:'synthetic-runner',runner_id:7,started_at:'2026-10-11T00:00:00Z',completed_at:null,
 url:'https://api.github.com'+base+'/jobs/456'};
const member={run_id:123,job_id:456,job_name:job.name,status:'in_progress',job_url:job.url,
 run_url:'https://api.github.com'+base+'/runs/123'};
const state={status:200,observedAt:'2026-10-11T00:00:04Z',body:{group_name:name,total_count:1,group_members:[member]}};
const run={status:200,observedAt:'2026-10-11T00:00:03Z',body:{id:123,run_attempt:1,head_sha:context.eventHead,
 path:'.github/workflows/release-runtime-fixture.yml',status:'queued',conclusion:null}};
async function observe({jobs=[job],ancestor=state,runSnapshot=run,diagnose}={}) {
 const calls=[],snapshots=[];
 const get=async route=>{calls.push(route);
  if(route.includes('/jobs?'))return {status:200,observedAt:'2026-10-11T00:00:01Z',body:{total_count:jobs.length,jobs}};
  if(route===base+'/jobs/456')return {status:200,observedAt:'2026-10-11T00:00:02Z',body:job};
  if(route===base+'/runs/123')return runSnapshot;
  return ancestor;
 };
 let result,error;
 try {result=await observeActiveAncestor({get,context,name,label:'Fixture lease holder active',snapshots,diagnose,runnerName:'synthetic-runner'});}
 catch(value){error=value;}
 return {calls,snapshots,result,error};
}
test('checkpoint joins exact unfiltered member with independently executing descendant',async()=>{
 const x=await observe();assert.ifError(x.error);
 assert.equal(x.result.observation,'VerifiedPrivateFixtureMatchedExecutionCheckpoint');
 assert.equal(x.result.productionAuthority,false);
 assert.deepEqual(x.calls,[base+'/runs/123/jobs?filter=latest&per_page=100',base+'/jobs/456',base+'/runs/123',base+'/concurrency_groups/'+name]);
 assert.throws(()=>validateQueueSnapshot(state,{group:name,run:'123'}));
});
for(const [label,jobs] of [['missing',[]],['completed',[{...job,status:'completed'}]],
 ['foreign run',[{...job,run_id:124}]],['ambiguous',[job,{...job,id:457}]],['invalid ID',[{...job,id:NaN}]]]) {
 test(label+' descendant refuses',async()=>{const x=await observe({jobs});assert.ok(x.error);assert.equal(x.calls.length,1);});
}
for(const [label,ancestor] of [['422',{status:422,body:{message:'not in group'}}],
 ['empty',{...state,body:{...state.body,total_count:0,group_members:[]}}],
 ['foreign group',{...state,body:{...state.body,group_name:'other'}}],
 ['foreign run',{...state,body:{...state.body,group_members:[{...member,run_id:124}]}}],
 ['pending only',{...state,body:{...state.body,group_members:[{...member,status:'pending'}]}}],
 ['truncated',{...state,body:{...state.body,total_count:2}}],
 ['wrong URL',{...state,body:{...state.body,group_members:[{...member,job_url:'https://foreign.invalid'}]}}]]) {
 test(label+' group stays refused',async()=>{const x=await observe({ancestor});assert.ok(x.error);assert.equal(x.calls.length,4);});
}
test('separate queue obligation still requires actual pending contender',()=>{
 const queued=structuredClone(state);queued.body.group_members.push({run_id:123,status:'pending',job_name:'Fixture private queued contender',job_id:789});queued.body.total_count=2;
 assert.equal(validateQueueSnapshot(queued,{group:name,run:'123'}),true);
});
test('pending last member is not the selected active lease: exact active ID is independently required',async()=>{
 // Old selector-prefix at(-1) oracle was invalid for a complete unfiltered group.
 const ancestor=structuredClone(state);ancestor.body.group_members.push({run_id:123,status:'pending',job_id:789,job_name:'Fixture private queued contender'});ancestor.body.total_count=2;
 assert.ifError((await observe({ancestor})).error);
 ancestor.body.group_members[0].job_id=999;
 assert.ok((await observe({ancestor})).error);
});
test('preceding same-run member cannot substitute for ambiguous active or foreign execution',async()=>{
 const ancestor=structuredClone(state);ancestor.body.group_members.push({...member,job_id:789});ancestor.body.total_count=2;
 assert.ok((await observe({ancestor})).error);
 ancestor.body.group_members[1].status='pending';ancestor.body.group_members[0].run_id=124;
 assert.ok((await observe({ancestor})).error);
});
test('aggregate identity and observation freshness are independent obligations',async()=>{
 for(const change of [x=>x.body.run_attempt=2,x=>x.body.head_sha='b'.repeat(40),x=>x.body.path='foreign.yml',
 x=>x.body.status='completed',x=>x.body.conclusion='success',x=>x.observedAt='2026-10-11T00:01:00Z']) {
  const runSnapshot=structuredClone(run);change(runSnapshot);assert.ok((await observe({runSnapshot})).error);
 }
});
test('permanent API refusals never repeat; healthy pending can advance under the same deadline',async()=>{
 for(const status of [401,403,404,422]){
  let calls=0;const value=await pollFixtureObservation(async()=>{calls++;requireFixtureApiSuccess({status});},{deadline:100,now:()=>0,wait:async()=>assert.fail('permanent refusal must not wait')});
  assert.equal(calls,1);assert.equal(value.observation,'Unknown');
 }
 let calls=0,clock=0;
 const result=await pollFixtureObservation(async()=>++calls<3?null:{observation:'ready'},{deadline:100,now:()=>clock,wait:async()=>{clock++;}});
 assert.equal(calls,3);assert.equal(result.observation,'ready');
 calls=0;
 const unavailable=await pollFixtureObservation(async()=>{calls++;throw Error('transient');},{deadline:100,now:()=>0,wait:async()=>{}});
 assert.equal(calls,3);assert.equal(unavailable.observation,'Unknown');
});
