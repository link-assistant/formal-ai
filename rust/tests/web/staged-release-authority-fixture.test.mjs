import test from 'node:test';
import assert from 'node:assert/strict';
import {requireFixtureRefusal} from '../../../scripts/staged-release-authority-fixture.mjs';
test('guarded fixture proves only explicit refusal without production output',()=>{
 const valid={status:1,signal:null,stderr:'deployed caller differs from checked original projection',stdout:''};
 requireFixtureRefusal(valid);
 requireFixtureRefusal({...valid,stderr:'unique source-declared operation in current stage required'});
 for(const change of [{status:0},{signal:'SIGTERM'},{stderr:'failed download'},{stdout:'production receipt'}])assert.throws(()=>requireFixtureRefusal({...valid,...change}));
});

import {readFileSync} from 'node:fs';
import YAML from 'yaml';
test('separate staged probe keeps the original runtime fixture checkout and operation bindings',()=>{
 const read=name=>YAML.parse(readFileSync(new URL('../../../.github/workflows/'+name,import.meta.url),'utf8'));
 const workflow=read('staged-release-authority-fixture.yml');
 const ordinary=read('release-runtime-fixture.yml');
 assert.equal(ordinary.jobs['staged-authority-refusal'],undefined);
 assert.deepEqual(workflow.permissions,{contents:'read',actions:'read'});
 assert.deepEqual(workflow.on.pull_request['branches-ignore'],['e2e/**']);
 assert.ok(workflow.on.workflow_dispatch);
 assert.deepEqual(Object.keys(workflow.jobs),['staged-authority-refusal']);
 const job=workflow.jobs['staged-authority-refusal'];
 assert.equal(job.if,"github.event_name == 'pull_request'");
 assert.equal(job['timeout-minutes'],5);
 assert.equal(job.concurrency.group,'formal-ai-staged-authority-fixture-${{ github.run_id }}-${{ github.run_attempt }}');
 assert.equal(job.concurrency['cancel-in-progress'],false);
 const checkouts=job.steps.filter(step=>step.uses==='actions/checkout@v7');
 assert.deepEqual(checkouts.map(step=>step.with),[
  {ref:'${{ github.sha }}','persist-credentials':false},
  {ref:'${{ github.sha }}',path:'_protocol','persist-credentials':false}
 ]);
 const install=job.steps.find(step=>step.name==='Install immutable protocol dependencies');
 assert.equal(install.run,'bun install --frozen-lockfile --ignore-scripts');
 assert.equal(install['working-directory'],'_protocol');
 assert.equal(install['timeout-minutes'],3);
 const probe=job.steps.find(step=>step.name==='Exercise actual current-process guard refusal');
 assert.equal(probe.run,'node scripts/staged-release-authority-fixture.mjs');
 assert.equal(probe['timeout-minutes'],1);
 for(const existing of Object.values(ordinary.jobs))for(const step of existing.steps??[]) {
  if(step.uses==='actions/checkout@v7')assert.deepEqual(step.with,{ref:'${{ github.sha }}','persist-credentials':false});
 }
});

import {compileDeployedStagedAuthority,compileMaintainedStagedAuthority,maintainedStagedInventory} from '../../../scripts/maintained-staged-authority.mjs';
test('fixture inventory uses real deployed source proof while ordinary compiler still refuses deployment',()=>{
 const inventory=maintainedStagedInventory(compileDeployedStagedAuthority());
 assert.equal(inventory.operations,52);
 assert.equal(inventory.bindings,104);
 assert.throws(()=>compileMaintainedStagedAuthority());
 assert.throws(()=>maintainedStagedInventory({}));
 const fixture=readFileSync(new URL('../../../scripts/staged-release-authority-fixture.mjs',import.meta.url),'utf8');
 assert.ok(fixture.includes('adapter.maintainedStagedInventory(adapter.compileDeployedStagedAuthority())'));
 assert.ok(!fixture.includes('adapter.compileMaintainedStagedAuthority()'));
});
