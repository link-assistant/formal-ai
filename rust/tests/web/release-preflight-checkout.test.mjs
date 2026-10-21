import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import YAML from 'yaml';
const root=new URL('../../../',import.meta.url);
const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const read=file=>readFileSync(new URL(file,root),'utf8');
function validatePreflight(workflow){
 const job=workflow.jobs['release-preflight'],checkout=job.steps[0],probe=job.steps[1];
 assert.equal(job['timeout-minutes'],5);assert.equal(checkout['timeout-minutes'],2);
 assert.equal(checkout.uses,'actions/checkout@v7');
 assert.deepEqual(checkout.with,{'persist-credentials':false,'fetch-depth':1,filter:'blob:none',
  'sparse-checkout':'/scripts/preflight-credentials.sh','sparse-checkout-cone-mode':false});
 assert.equal(checkout.with.ref,undefined,'preserve actual triggering merge/event source');
 assert.deepEqual(checkout.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
 assert.equal(workflow.env.GIT_CONFIG_COUNT,'1');assert.equal(workflow.env.GIT_CONFIG_KEY_0,'init.defaultBranch');
 assert.equal(workflow.env.GIT_CONFIG_VALUE_0,'main');
 assert.equal(job.steps.length,2);assert.equal(probe.run,'bash scripts/preflight-credentials.sh');
 assert.equal(probe.if,undefined);assert.equal(job.if,undefined);assert.equal(probe['continue-on-error'],undefined);
 assert.deepEqual(job.permissions,{contents:'read',packages:'write'});
 return job;
}
test('canonical and deployed preflight retain the real credential gate with bounded sparse source checkout',()=>{
 const canonical=YAML.parse(read(evidence+'original-release-workflow.yml'));
 const expected=validatePreflight(canonical);
 for(const file of ['.github/workflows/release.yml',evidence+'candidate-release-caller.yml']){
  assert.deepEqual(validatePreflight(YAML.parse(read(file))),expected);
 }
 const script=read('scripts/preflight-credentials.sh');
 assert.ok(script.startsWith('#!/usr/bin/env bash\n'));
 assert.doesNotMatch(script,/^\s*(?:source|\.)\s+[^#\n]+/mu,'sparse closure must remain standalone');
});
test('broad checkout, lost transfer bounds, changed event source and bypassed probes are rejected',()=>{
 const original=YAML.parse(read(evidence+'original-release-workflow.yml'));
 const mutations=[
  x=>delete x.jobs['release-preflight'].steps[0].with['sparse-checkout'],
  x=>x.jobs['release-preflight'].steps[0].with['sparse-checkout']='/',
  x=>x.jobs['release-preflight'].steps[0].with['sparse-checkout-cone-mode']=true,
  x=>delete x.jobs['release-preflight'].steps[0].with.filter,
  x=>x.jobs['release-preflight'].steps[0].with.ref='main',
  x=>x.jobs['release-preflight'].steps[0].with['persist-credentials']=true,
  x=>delete x.jobs['release-preflight'].steps[0].env.GIT_HTTP_LOW_SPEED_TIME,
  x=>x.jobs['release-preflight'].steps[0].env.GIT_HTTP_LOW_SPEED_LIMIT='0',
  x=>x.jobs['release-preflight'].steps[0]['timeout-minutes']=5,
  x=>x.jobs['release-preflight'].steps[1].if='false',
  x=>x.jobs['release-preflight'].steps[1].run='true',
  x=>x.jobs['release-preflight']['timeout-minutes']=6
 ];
 for(const mutate of mutations){const changed=structuredClone(original);mutate(changed);assert.throws(()=>validatePreflight(changed));}
});
