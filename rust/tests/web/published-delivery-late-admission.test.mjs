import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
import {observeEligibleAdmission as observeLateAdmission, isLatePublishedCompletion, recheckAdmission as lateRecheckAdmission} from '../../../scripts/admit-hive-original-delivery.mjs';
import {latePublishedBindingDecision} from '../../../scripts/lib/hive-delivery-release.mjs';
import {admissionPolicy as latePolicy, ciAdmission as lateCiAdmission, ciAdmissionDecision as lateCiDecision} from '../../../scripts/lib/hive-delivery-admission.mjs';
const lateRepository='owner/repo', checkedSource='a'.repeat(40), publishedSource='b'.repeat(40);
const lateTrigger=(head=checkedSource)=>({id:41,run_attempt:1,repository:{full_name:lateRepository},
 head_repository:{full_name:lateRepository},status:'completed',conclusion:'success',event:'push',head_branch:'main',
 path:'.github/workflows/coverage.yml',head_sha:head});
function lateAdmission(head=checkedSource){
 const trigger=lateTrigger(head);
 return {schema:latePolicy,admitted:true,selected:{repository:lateRepository,event:'published-release',
  ciSource:checkedSource,source:publishedSource},delta:{kind:'guarded-direct-prepared-version',
  ciSource:checkedSource,publishedSource},ci:head===checkedSource?[{id:trigger.id,attempt:1,path:trigger.path,source:head}]:[],
  publishedObservedCi:head===publishedSource?[{id:trigger.id,attempt:1,path:trigger.path,source:head}]:[]};
}
test('late authenticated checked and guarded published source completions bind independently',()=>{
 for(const head of [checkedSource,publishedSource]){
  assert.equal(isLatePublishedCompletion(lateTrigger(head),lateRepository),true);
  assert.equal(latePublishedBindingDecision(lateTrigger(head),lateAdmission(head)).admitted,true);
 }
 const exact=lateAdmission();exact.selected.source=checkedSource;
 exact.delta={kind:'exact-ci-source',ciSource:checkedSource,publishedSource:checkedSource};
 assert.equal(latePublishedBindingDecision(lateTrigger(),exact).admitted,true);
});
test('unrelated source, stale attempts, unguarded delta and absent authenticated CI refuse',()=>{
 for(const mutate of [x=>x.head_sha='c'.repeat(40),x=>x.run_attempt=2,x=>x.id=99,
  x=>x.path='.github/workflows/unrelated.yml',x=>x.event='workflow_run',x=>x.head_branch='other',
  x=>x.conclusion='failure',x=>x.repository.full_name='foreign/repo',x=>x.head_repository.full_name='foreign/repo']){
  const trigger=lateTrigger();mutate(trigger);
  if(isLatePublishedCompletion(trigger,lateRepository))assert.equal(latePublishedBindingDecision(trigger,lateAdmission()).admitted,false);
  else assert.equal(isLatePublishedCompletion(trigger,lateRepository),false);
 }
 for(const mutate of [x=>x.delta.kind='unguarded',x=>x.delta.ciSource='c'.repeat(40),x=>x.admitted=false]){
  const admission=lateAdmission();mutate(admission);assert.throws(()=>latePublishedBindingDecision(lateTrigger(),admission));
 }
});
test('all-CI pending or failed retains the original fail-closed admission semantics',()=>{
 for(const [status,conclusion]of [['in_progress',null],['completed','failure']]){
  assert.throws(()=>lateCiAdmission({source:checkedSource,expected:['.github/workflows/coverage.yml'],complete:true,
   runs:[{...lateTrigger(),status,conclusion}]}));
 }
});
function lateReader(trigger,latest){
 const calls=[];return {calls,
  read:async endpoint=>{
   calls.push(endpoint);
   if(endpoint==='repos/'+lateRepository+'/actions/runs/41')return trigger;
   assert.equal(endpoint,'repos/'+lateRepository+'/actions/workflows/verify-release-downloads.yml/runs?per_page=1');
   if(latest instanceof Error)throw latest;return {workflow_runs:Array.isArray(latest)?latest:[{
    id:77,repository:{full_name:lateRepository},head_repository:{full_name:lateRepository},
    path:'.github/workflows/verify-release-downloads.yml',...latest}]};
  },pages:async endpoint=>{assert.equal(endpoint,'repos/'+lateRepository+'/commits/'+trigger.head_sha+'/pulls');return [];}};
}
test('actual late route reaches latest producer only for a successful same-repository main push',async()=>{
 for(const mutate of [x=>x.event='workflow_run',x=>x.head_branch='other',x=>x.conclusion='failure']){
  const trigger=lateTrigger();mutate(trigger);const reader=lateReader(trigger,new Error('must not select release'));
  const result=await observeLateAdmission({repository:lateRepository,eventName:'workflow_run',event:{workflow_run:{id:41}},reader});
  assert.equal(result.admitted,false);assert(!reader.calls.some(value=>value.includes('/actions/workflows/')));
 }
 const reader=lateReader(lateTrigger(),new Error('actual authenticated latest-producer route reached'));
 await assert.rejects(observeLateAdmission({repository:lateRepository,eventName:'workflow_run',
  event:{workflow_run:{id:41}},reader}),/actual authenticated latest-producer route reached/u);
});
test('actual late route refuses latest pending or failed producer instead of selecting older success',async()=>{
 for(const [status,conclusion]of [['in_progress',null],['completed','failure']]){
  const reader=lateReader(lateTrigger(),{id:77,status,conclusion});
  const result=await observeLateAdmission({repository:lateRepository,eventName:'workflow_run',event:{workflow_run:{id:41}},reader});
  assert.equal(result.admitted,false);assert.match(result.reason,/latest-public-download-producer/u);
  assert(!reader.calls.some(value=>value.includes('/releases/')));
 }
});
test('late event concurrency preserves separate pending wakeups without changing direct release or PR groups',()=>{
 const expression=YAML.parse(readFileSync(new URL('../../../.github/workflows/hive-original-delivery.yml',import.meta.url),'utf8')).concurrency.group;
 // Evaluate only this maintained finite expression with controlled fixture values, never workflow input code.
 assert.match(expression,/format\('release-ci-\{0\}', github.event.workflow_run.id\)/u);
 const evaluate=(run)=>{
  const github={event:{workflow_run:run},ref_name:'scratch'};
  const format=(template,value)=>template.replace('{0}',String(value));
  const source=expression.replace(/^original-hive-\$\{\{\s*/u,'').replace(/\s*\}\}$/u,'');
  assert.match(source,/^[A-Za-z0-9_.\s'{}()=!&|,-]+$/u);
  return 'original-hive-'+Function('github','format','return ('+source+');')(github,format);
 };
 const first=evaluate({name:'Coverage',event:'push',head_branch:'main',conclusion:'success',id:41});
 const next=evaluate({name:'Layered CI',event:'push',head_branch:'main',conclusion:'success',id:42});
 assert.notEqual(first,next);
 assert.equal(evaluate({name:'Verify Public Release Downloads',id:77}),'original-hive-release-77');
 assert.equal(evaluate({name:'Coverage',event:'pull_request',head_branch:'topic',id:2}),'original-hive-candidate-topic');
});

test('absent latest producer defers without searching older releases',async()=>{
 const reader=lateReader(lateTrigger(),[]);
 const result=await observeLateAdmission({repository:lateRepository,eventName:'workflow_run',event:{workflow_run:{id:41}},reader});
 assert.deepEqual(result,{admitted:false,reason:'no-public-download-producer'});
 assert(!reader.calls.some(value=>value.includes('/releases/')));
});
test('normal source and current-CI mismatches defer while malformed identity remains an error',()=>{
 for(const mutate of [x=>x.head_sha='c'.repeat(40),x=>x.run_attempt=2,x=>x.id=99,x=>x.path='.github/workflows/unrelated.yml']){
  const trigger=lateTrigger();mutate(trigger);
  const result=latePublishedBindingDecision(trigger,lateAdmission());assert.equal(result.admitted,false);
 }
 for(const head of [checkedSource,publishedSource])assert.equal(latePublishedBindingDecision(lateTrigger(head),lateAdmission(head)).admitted,true);
 const missing=lateAdmission();missing.ci=[];assert.equal(latePublishedBindingDecision(lateTrigger(),missing).admitted,false);
 const admission=lateAdmission();admission.delta.kind='unguarded';assert.throws(()=>latePublishedBindingDecision(lateTrigger(),admission));
});
test('production CI decision catches only ordinary missing, pending or failed CI',()=>{
 const options={source:checkedSource,expected:['.github/workflows/coverage.yml'],complete:true,runs:[lateTrigger()]};
 assert.equal(lateCiDecision(options).admitted,true);
 for(const [status,conclusion,reason]of [['in_progress',null,'other CI pending'],['completed','failure','other CI not green']]){
  assert.deepEqual(lateCiDecision({...options,runs:[{...lateTrigger(),status,conclusion}]}),{admitted:false,reason,
   blockedBy:{run:41,attempt:1,path:'.github/workflows/coverage.yml',event:'push',status,conclusion}});
 }
 assert.equal(lateCiDecision({...options,expected:['missing.yml']}).admitted,false);
 for(const changed of [{...options,complete:false},{...options,runs:[{...lateTrigger(),head_sha:'c'.repeat(40)}]},
  {...options,runs:[{...lateTrigger(),status:'malformed'}]}])assert.throws(()=>lateCiDecision(changed));
});
test('source-bound published recheck refuses a deferred dependency before consuming success fields',async()=>{
 // Fault-inject only the observer dependency in the exact maintained function text.
 // This checks refusal ordering, not publication authentication or real effect authority.
 const source=lateRecheckAdmission.toString();assert.match(source,/assert.equal\(current.admitted, true,/u);
 const isolated=Function('observePublishedAdmission','assert','admissionPolicy','return ('+source+');')(
  async()=>({admitted:false,reason:'other CI pending',blockedBy:{path:'.github/workflows/desktop-release.yml'},
   get selected(){throw Error('success fields read before refusal');}}),assert,latePolicy);
 await assert.rejects(isolated({selection:{schema:latePolicy,admitted:true,selected:{event:'published-release'},
  publicDownloadVerification:{run:77}},repository:lateRepository,reader:{}}),{code:'ERR_ASSERTION',
   message:/published CI revalidation denied:.*other CI pending.*desktop-release/u});
});
