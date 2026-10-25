import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
import {ciAdmissionDecision,isGatingCiRun,applicableWorkflows} from '../../../scripts/lib/hive-delivery-admission.mjs';
import {precheckPublishedCi} from '../../../scripts/lib/hive-delivery-release.mjs';

// Minimal replay of release run 40173640886: proposal run 40176454890 started
// after admission; Agent's claim failed before creating any task resources.
const repository='link-assistant/formal-ai',source='43fa87e4a0dd7745737be9e41237fabde05fef8b';
const publisher={id:40170491754,run_attempt:1,head_sha:source,event:'workflow_run',
 path:'.github/workflows/desktop-release.yml',status:'completed',conclusion:'success',
 repository:{full_name:repository},head_repository:{full_name:repository}};
const proposal={...publisher,id:40176454890,event:'schedule',path:'.github/workflows/learning-cycle.yml',
 status:'in_progress',conclusion:null};
const options=runs=>({source,expected:[publisher.path],runs,complete:true});
const reader=runs=>({pages:async()=>runs});

test('derived PR obligations and observed CI agree that cleanup is outside delivery admission',()=>{
 const validator={path:'.github/workflows/release.yml',document:{on:{pull_request:{}}}};
 const cleanup={path:'.github/workflows/hive-original-delivery-finalize.yml',document:
  YAML.parse(readFileSync(new URL('../../../.github/workflows/hive-original-delivery-finalize.yml',import.meta.url),'utf8'))};
 const expected=applicableWorkflows([validator,cleanup],{event:'pull_request',baseBranch:'main',
  files:['scripts/finalize-generated-test-issues.mjs']});
 assert.deepEqual(expected,[validator.path]);
 const result=ciAdmissionDecision({source,expected,complete:true,runs:[{...publisher,path:validator.path,event:'pull_request'},
  {...proposal,path:cleanup.path,event:'pull_request'}]});
 assert.equal(result.admitted,true);
});

test('a later proposal-only cycle cannot invalidate authenticated release CI',async()=>{
 for(const event of ['schedule','workflow_dispatch']){
  for(const [status,conclusion] of [['in_progress',null],['completed','success'],['completed','failure']]){
   const run={...proposal,event,status,conclusion};
   assert.equal(isGatingCiRun(run),false);
   const checked=ciAdmissionDecision(options([publisher,run]));
   assert.equal(checked.admitted,true);
   assert.deepEqual(checked.ci.map(record=>record.id),[publisher.id]);
   const precheck=await precheckPublishedCi(reader([publisher,run]),repository,source);
   assert.equal(precheck.admitted,true);
   assert.deepEqual(precheck.ci,checked.ci);
  }
 }
 assert.deepEqual(await precheckPublishedCi(reader([proposal]),repository,source),{admitted:true,ci:[]});
});

test('real checks, unknown schedules and publisher retries still fail closed with diagnostics',async()=>{
 for(const run of [
  {...proposal,event:'push'},
  {...proposal,event:'pull_request'},
  {...proposal,path:'.github/workflows/security.yml'},
  {...publisher,id:publisher.id+1,run_attempt:2,status:'in_progress',conclusion:null},
  {...publisher,id:publisher.id+1,run_attempt:2,conclusion:'failure'}
 ]){
  const result=ciAdmissionDecision(options([publisher,run]));
  assert.equal(result.admitted,false);
  assert.equal(result.blockedBy.run,run.id);
  assert.equal(result.blockedBy.path,run.path);
  assert.equal(result.blockedBy.status,run.status);
  assert.equal((await precheckPublishedCi(reader([publisher,run]),repository,source)).admitted,false);
 }
 const missing=ciAdmissionDecision({...options([proposal]),expected:[publisher.path]});
 assert.equal(missing.admitted,false);
 assert.match(missing.reason,/missing applicable CI/u);
});

test('proposal classification grants no foreign source or repository authority',async()=>{
 const foreign={...proposal,head_sha:'f'.repeat(40)};
 assert.throws(()=>ciAdmissionDecision(options([publisher,foreign])),/wrong-source/u);
 await assert.rejects(precheckPublishedCi(reader([publisher,foreign]),repository,source),/precheck CI source/u);
 await assert.rejects(precheckPublishedCi(reader([publisher,{...proposal,
  head_repository:{full_name:'foreign/repository'}}]),repository,source));
});

test('the excluded workflow remains a read-only proposal generator with no push or PR trigger',()=>{
 const workflow=YAML.parse(readFileSync(new URL('../../../.github/workflows/learning-cycle.yml',import.meta.url),'utf8'));
 assert.deepEqual(Object.keys(workflow.on).sort(),['schedule','workflow_dispatch']);
 assert(Object.values(workflow.permissions).every(permission=>permission==='read'));
 const commands=workflow.jobs.cycle.steps.map(step=>step.run??'').join('\n');
 assert.match(commands,/--dry-run --proposals/u);
 assert.match(commands,/human_gated/u);
 assert.match(commands,/git diff --exit-code -- data\/seed/u);
});
