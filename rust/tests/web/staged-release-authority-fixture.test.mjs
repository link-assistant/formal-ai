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
  {ref:'${{ github.sha }}','persist-credentials':false,'sparse-checkout':"/.github/workflows/release.yml\n/scripts/staged-release-authority-fixture.mjs\n",'sparse-checkout-cone-mode':false},
  {ref:'${{ github.sha }}',path:'_protocol','persist-credentials':false,'sparse-checkout':"/.github/\n/scripts/\n/experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/\n/package.json\n/bun.lock\n/.bun-version\n",'sparse-checkout-cone-mode':false}
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


test('declared sparse protocol inputs still compile the unchanged guarded operation inventory',async t=>{
 const fs=await import('node:fs'),path=await import('node:path'),os=await import('node:os'),url=await import('node:url');
 const {execFileSync}=await import('node:child_process');
 const root=path.resolve(url.fileURLToPath(new URL('../../../',import.meta.url)));
 const workspace=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-staged-sparse-')));
 t.after(()=>fs.rmSync(workspace,{recursive:true,force:true}));
 const workflow=YAML.parse(readFileSync(new URL('../../../.github/workflows/staged-release-authority-fixture.yml',import.meta.url),'utf8'));
 const checkout=workflow.jobs['staged-authority-refusal'].steps.filter(step=>step.uses==='actions/checkout@v7')[1];
 const selectors=checkout.with['sparse-checkout'].trim().split('\n').map(line=>line.slice(1).replace(/\/$/,''));
 const files=execFileSync('git',['ls-tree','-r','--name-only','HEAD','--',...selectors],{
  cwd:root,encoding:'utf8',timeout:5000,maxBuffer:1048576}).trim().split('\n');
 assert(files.length>0&&files.every(file=>selectors.some(selector=>file===selector||file.startsWith(selector+'/'))));
 for(const file of files){const target=path.join(workspace,file);fs.mkdirSync(path.dirname(target),{recursive:true});fs.linkSync(path.join(root,file),target);}
 fs.symlinkSync(path.join(root,'node_modules'),path.join(workspace,'node_modules'),'junction');
 const adapter=await import(url.pathToFileURL(path.join(workspace,'scripts/maintained-staged-authority.mjs')).href);
 const inventory=adapter.maintainedStagedInventory(adapter.compileDeployedStagedAuthority());
 assert.equal(inventory.operations,52);assert.equal(inventory.bindings,104);
 assert.equal(inventory.productionAuthority,false);
});
