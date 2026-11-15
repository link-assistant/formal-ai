import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import YAML from 'yaml';
import { requireExactCurrentGroupMember, compileContract, createStagedAuthority } from '../../../scripts/staged-caller-authority.mjs';
const root=new URL('../../../',import.meta.url),read=file=>fs.readFileSync(new URL(file,root),'utf8');
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const blob=bytes=>createHash('sha1').update(Buffer.concat([Buffer.from('blob '+Buffer.byteLength(bytes)+'\0'),Buffer.from(bytes)])).digest('hex');
const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
function sourceContract(){
 const canonical=read(evidence+'original-release-workflow.yml'),packetSource=read(evidence+'stage-source-coverage.json');
 const callerDraft=read(evidence+'candidate-release-caller.yml'),staged=read('.github/workflows/release-staged.yml');
 return {canonical,packetSource,callerDraft,staged,contract:compileContract({canonical,callerDraft,staged,packet:JSON.parse(packetSource),YAML,
  retainedSourceProof:{originalSource:canonical,packetSource}})};
}
function authorityFixture(){
 const {contract,canonical,packetSource,callerDraft,staged}=sourceContract();
 assert.equal(contract.routes.length,52);assert.equal(contract.bindings,104);assert.equal(contract.sources.length,4);
 const route=contract.routes[0],expected={repository:'link-assistant/formal-ai',head:'a'.repeat(40),run:'123',attempt:'1',runnerName:'synthetic-runner',
  caller:route.caller,ordinal:route.ordinal,stageJob:route.stageJob,operationSha256:route.operationSha256};
 const bodies=[callerDraft,staged,canonical,packetSource];const entries=contract.sources.map((source,index)=>({path:source.path,type:'blob',mode:'100644',sha:blob(bodies[index])}));
 const calls=[],job={id:42,run_id:123,run_attempt:1,head_sha:expected.head,name:route.descendantName,runner_name:expected.runnerName,runner_id:7,
  status:'in_progress',completed_at:null,started_at:'2026-10-11T00:00:00Z',url:'https://api.github.com/repos/'+expected.repository+'/actions/jobs/42'};
 const run={id:123,run_attempt:1,repository:{full_name:expected.repository},head_repository:{full_name:expected.repository},head_sha:expected.head,
   head_branch:'main',status:'in_progress',conclusion:null,path:'.github/workflows/release.yml',event:route.event,workflow_id:9};
 const group={group_name:contract.writerGroup,total_count:1,group_members:[{job_id:42,job_name:job.name,run_id:123,status:'in_progress',job_url:job.url,run_url:'https://api.github.com/repos/'+expected.repository+'/actions/runs/123'}]};
 const transport=async pathname=>{calls.push(pathname);let body;
  if(pathname.endsWith('/attempts/1'))body=run;
  else if(pathname.endsWith('/workflows/9'))body={path:'.github/workflows/release.yml',state:'active'};
  else if(pathname.includes('/git/trees/'))body={truncated:false,tree:entries};
  else if(pathname.includes('/git/blobs/')){const index=entries.findIndex(entry=>pathname.endsWith(entry.sha));assert.ok(index>=0);body={encoding:'base64',content:Buffer.from(bodies[index]).toString('base64')};}
  else if(pathname.endsWith('/jobs?per_page=100'))body={total_count:1,jobs:[job]};
  else {assert.equal(pathname,'/repos/'+expected.repository+'/actions/concurrency_groups/'+encodeURIComponent(contract.writerGroup));
   body=group;}
  return {status:200,body,observedAt:pathname.includes('/concurrency_groups/')?'2026-10-11T00:00:02Z':'2026-10-11T00:00:01Z'};
 };
 const authority=createStagedAuthority(contract,transport);
 return {authority,expected,route,calls,run,job,group};
}
test('legacy issued-receipt entry uses complete unfiltered exact membership with all source bytes',async()=>{
 const {authority,expected,route,calls}=authorityFixture(),receipt=await authority.observe(expected);
 assert.equal(authority.context(receipt,expected).job,route.caller);
 assert.equal(calls.length,9);assert.ok(calls.every(pathname=>!pathname.includes('ahead_of_')));
 assert.throws(()=>authority.context({...receipt},expected));
});
test('actual89a selector422 coexists with exact unfiltered membership; replay never promotes stale sampling',()=>{
 const raw=read('rust/tests/fixtures/release-fixture-membership-89a.json');
 assert.equal(digest(raw),'67e8bad7b76d74f461aa8629a66017c011ca46c80ae952c0389d3b90b09fe2ef');
 const captured=JSON.parse(raw),[group,jobs,physical,hinted,runHint]=captured.snapshots;
 assert.equal(hinted.status,422);assert.equal(runHint.status,422);
 const member=group.body.group_members.find(value=>value.status==='in_progress');
 assert.equal(member.job_id,physical.body.id);assert.equal(member.job_url,physical.body.url);
 assert.equal(member.job_id,119245645490);assert.equal(jobs.body.total_count,7);
 const expected={...captured.context,head:captured.context.eventHead,runnerName:'representative-runner'};
 // Runner display fields were redacted. Ordered observation times below are representative,
 // not a claim that this older captured sampling already satisfied the new ordered proof.
 const representativeJobs=structuredClone(jobs),representativeGroup=structuredClone(group);
 const child=representativeJobs.body.jobs.find(value=>value.id===member.job_id);
 child.runner_name=expected.runnerName;child.runner_id=7;
 assert.throws(()=>requireExactCurrentGroupMember(expected,member.job_name,group.body.group_name,representativeGroup,representativeJobs));
 representativeGroup.observedAt='2026-10-21T13:24:49.000Z';
 assert.equal(requireExactCurrentGroupMember(expected,member.job_name,group.body.group_name,representativeGroup,representativeJobs),undefined);
 for(const mutate of [
  (g,j)=>g.body.total_count++, (g,j)=>j.body.total_count++,
  (g,j)=>g.body.group_members.find(value=>value.status==='in_progress').job_id++,
  (g,j)=>g.body.group_members.find(value=>value.status==='in_progress').job_url='https://foreign.invalid',
  (g,j)=>g.body.group_members.find(value=>value.status==='in_progress').run_url='https://foreign.invalid',
  (g,j)=>j.body.jobs.find(value=>value.id===member.job_id).status='queued',
  (g,j)=>j.body.jobs.find(value=>value.id===member.job_id).run_attempt++
 ]){const g=structuredClone(representativeGroup),j=structuredClone(representativeJobs);mutate(g,j);assert.throws(()=>requireExactCurrentGroupMember(expected,member.job_name,g.body.group_name,g,j));}
});

// Main run 41873480573/job 126744977533 reported queued while the stage executed.
test('queued parent aggregate still requires the exact current staged writer',async()=>{
 for(const status of ['queued','in_progress','waiting','pending','requested']) {
  const fixture=authorityFixture();fixture.run.status=status;
  const receipt=await fixture.authority.observe(fixture.expected);
  assert.equal(fixture.authority.context(receipt,fixture.expected).job,fixture.route.caller);
  for(const mutate of [
   f=>{f.job.status='queued';},f=>{f.job.runner_name='foreign';},
   f=>{f.job.run_attempt=2;},f=>{f.group.group_members[0].job_id=43;},
   f=>{f.group.group_members=[];f.group.total_count=0;},
   f=>{f.group.group_members.push({...f.group.group_members[0]});f.group.total_count=2;}
  ]) {
   const invalid=authorityFixture();invalid.run.status=status;mutate(invalid);
   await assert.rejects(()=>invalid.authority.observe(invalid.expected));
  }
 }
 for(const status of ['completed','unknown',null,undefined,{private:'not a status'}]) {
  const fixture=authorityFixture();fixture.run.status=status;
  await assert.rejects(()=>fixture.authority.observe(fixture.expected),/unsupported_caller_aggregate_status:invalid/u);
  assert.equal(fixture.calls.length,1);
 }
 for(const conclusion of ['success','failure','cancelled','skipped','neutral','timed_out','action_required',undefined]) {
  const fixture=authorityFixture();fixture.run.status='queued';fixture.run.conclusion=conclusion;
  await assert.rejects(()=>fixture.authority.observe(fixture.expected),/terminal caller aggregate/u);
  assert.equal(fixture.calls.length,1);
 }
});
