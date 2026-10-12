import test from 'node:test';
import assert from 'node:assert/strict';
import {validateDiagnosticDescendant,validateRunAncestorDiagnostic,collectLeaseDiagnostics} from '../../../scripts/release-fixture-lease-diagnostics.mjs';
const expected={repository:'owner/repo',run:'123',attempt:'2',eventHead:'a'.repeat(40),id:42,name:'Declared caller / Declared stage',runnerName:'Declared runner',group:'private-fixture-123-2'};
const job=()=>({status:200,body:{id:42,run_id:123,run_attempt:2,head_sha:expected.eventHead,name:expected.name,runner_name:expected.runnerName,runner_id:7,status:'in_progress',url:'https://api.github.com/repos/owner/repo/actions/jobs/42'}});
const group=()=>({status:200,body:{group_name:expected.group,group_members:[{run_id:123,status:'in_progress'}]}});
test('actual executing descendant identity requires every independently bound operand',()=>{
 assert.equal(validateDiagnosticDescendant(job(),expected),true);
 for(const [key,value] of [['id',43],['run_id',124],['run_attempt',3],['head_sha','b'.repeat(40)],['name','foreign'],['runner_name','foreign'],['status','completed'],['runner_id',0],['url','https://foreign.invalid']]){
  const changed=job();changed.body[key]=value;assert.throws(()=>validateDiagnosticDescendant(changed,expected));
 }
});
test('run-scoped diagnostic never certifies a descendant or production authority',()=>{
 const observation=validateRunAncestorDiagnostic(group(),expected);
 assert.deepEqual(observation,{observation:'ObservedRunScopedGroup',descendantLeaseProof:false,productionAuthority:false});
 for(const mutate of [x=>x.status=422,x=>x.body.group_name='foreign',x=>x.body.group_members=[],x=>x.body.group_members[0].run_id=124,x=>x.body.group_members[0].status='pending']){
  const changed=group();mutate(changed);assert.throws(()=>validateRunAncestorDiagnostic(changed,expected));
 }
});
test('diagnostic GET inventory is bounded by the unchanged caller observation window',async()=>{
 const calls=[];
 const observation=await collectLeaseDiagnostics({context:expected,child:{id:42,name:expected.name},group:expected.group,runnerName:expected.runnerName,deadline:Date.now()+1000,get:async(route,budget)=>{calls.push({route,budget});return route.includes('/jobs/')?job():group();}});
 assert.equal(observation.observation,'ObservedRunScopedGroup');assert.equal(observation.descendantLeaseProof,false);
 assert.deepEqual(calls.map(call=>call.route),['/repos/owner/repo/actions/jobs/42','/repos/owner/repo/actions/concurrency_groups/private-fixture-123-2?ahead_of_run=123']);
 assert.ok(calls.every(call=>call.budget>0 && call.budget<=1000));
 const expired=await collectLeaseDiagnostics({context:expected,child:{id:42,name:expected.name},group:expected.group,runnerName:expected.runnerName,deadline:Date.now()-1,get:async()=>assert.fail('expired diagnostic must not query')});
 assert.equal(expired.observation,'Unknown');assert.equal(expired.descendantLeaseProof,false);
});
test('unavailable ancestor API remains Unknown without weakening original lease proof',async()=>{
 const observation=await collectLeaseDiagnostics({context:expected,child:{id:42,name:expected.name},group:expected.group,runnerName:expected.runnerName,deadline:Date.now()+1000,get:async(route)=>route.includes('/jobs/')?job():{status:422,body:{message:'specified run is not in group'}}});
 assert.equal(observation.observation,'Unknown');assert.equal(observation.descendantLeaseProof,false);assert.equal(observation.snapshots.at(-1).status,422);
});
