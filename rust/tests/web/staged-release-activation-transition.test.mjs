import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {pathToFileURL,fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {buildStagedReleaseProjection} from '../../../scripts/generate-staged-release.mjs';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const YAML=createRequire(import.meta.url)('yaml');
function fixture() {
 const directory=fs.mkdtempSync(path.join(tmpdir(),'staged-release-transition-'));
 const files=['scripts/checked-release-operation-view.mjs','scripts/generate-staged-release.mjs','scripts/maintained-staged-authority.mjs',
 'scripts/staged-caller-authority.mjs','scripts/governed-github-command-provider.mjs',
 'scripts/release-source-freshness.mjs','scripts/lib/ci-speed-workflows.mjs',
 ...['stage-source-coverage.json','original-release-workflow.yml'].map(name=>evidence+name)];
 for(const file of files){const destination=path.join(directory,file);fs.mkdirSync(path.dirname(destination),{recursive:true});fs.copyFileSync(path.join(root,file),destination);}
 fs.mkdirSync(path.join(directory,'.github/workflows'),{recursive:true});
 fs.writeFileSync(path.join(directory,'package.json'),'{}');fs.symlinkSync(path.join(root,'node_modules'),path.join(directory,'node_modules'));
 const original=fs.readFileSync(path.join(root,evidence,'original-release-workflow.yml'),'utf8');
 const packet=JSON.parse(fs.readFileSync(path.join(root,evidence,'stage-source-coverage.json'),'utf8'));
 const projection=buildStagedReleaseProjection(original,packet);
 for(const [file,bytes] of projection.outputs)fs.writeFileSync(path.join(directory,file),bytes);
 fs.writeFileSync(path.join(directory,'.github/workflows/release.yml'),original);
 return {directory,original,packet,projection};
}
function check(directory,mode) {
 return spawnSync(process.execPath,[path.join(directory,'scripts/generate-staged-release.mjs'),mode],{encoding:'utf8',timeout:10000});
}
test('separate ordinary and deployed compiler modes conserve original source and all bindings',async()=>{
 const value=fixture();try {
  assert.equal(value.projection.operations,52);assert.equal(value.projection.binding.length,104);
  assert.equal(check(value.directory,'--check').status,0);
  assert.notEqual(check(value.directory,'--check-deployed').status,0);
  const caller=value.projection.outputs.get(evidence+'candidate-release-caller.yml');
  fs.writeFileSync(path.join(value.directory,'.github/workflows/release.yml'),caller);
  assert.equal(check(value.directory,'--check-deployed').status,0);
  assert.notEqual(check(value.directory,'--check').status,0);
  const adapter=await import(pathToFileURL(path.join(value.directory,'scripts/maintained-staged-authority.mjs')));
  assert.equal(adapter.maintainedStagedInventory(adapter.compileDeployedStagedAuthority()).operations,52);
  for(const mutate of [text=>text+'\n# drift\n',text=>text.replace('      actions: read','      actions: write')]){
   fs.writeFileSync(path.join(value.directory,'.github/workflows/release.yml'),mutate(caller));
   assert.notEqual(check(value.directory,'--check-deployed').status,0);
   assert.throws(()=>adapter.compileDeployedStagedAuthority());
  }
 }finally{fs.rmSync(value.directory,{recursive:true,force:true});}
});
test('Actions read is confined to operation-owning prepare leaves and caller ceilings',()=>{
 const value=fixture();try {
  const original=YAML.parse(value.original);
  const caller=YAML.parse(value.projection.outputs.get(evidence+'candidate-release-caller.yml'));
  const staged=YAML.parse(value.projection.outputs.get('.github/workflows/release-staged.yml'));
  assert.deepEqual(staged.permissions,{contents:'read'});assert.equal(staged.concurrency,undefined);
  for(const [id,job] of Object.entries(original.jobs)) {
   if(!['auto-release','manual-release'].includes(id)){assert.deepEqual(caller.jobs[id],job);continue;}
   assert.deepEqual(caller.jobs[id].permissions,{contents:'write',packages:'write',actions:'read'});
   assert.deepEqual(caller.jobs[id].concurrency,job.concurrency);
  }
  for(const [id,job] of Object.entries(staged.jobs)) {
   assert.equal(job.concurrency,undefined);assert.ok(job['timeout-minutes']<=30);
   if(id.endsWith('prepare-source'))assert.deepEqual(job.permissions,{contents:'write',actions:'read'});
   else assert.equal(job.permissions?.actions,undefined);
  }
 }finally{fs.rmSync(value.directory,{recursive:true,force:true});}
});

test('current physical view rejects source, permissions, order, output, compiler and artifact drift',async()=>{
 const value=fixture();try {
  const callerPath=path.join(value.directory,'.github/workflows/release.yml');
  const caller=value.projection.outputs.get(evidence+'candidate-release-caller.yml');
  fs.writeFileSync(callerPath,caller);
  const module=await import(pathToFileURL(path.join(value.directory,'scripts/checked-release-operation-view.mjs')));
  const view=module.readCheckedReleaseOperationView();assert.equal(view.mode,'deployed');
  assert.equal(view.originalSource,value.original);assert.equal(view.inventory.operations,52);assert.equal(view.inventory.bindings,104);
  assert.equal(view.productionAuthority,false);
  assert.equal(view.physicalCaller.jobs['auto-release'].permissions.actions,'read');
  assert.equal(view.physicalStages.jobs['auto_prepare-source'].permissions.actions,'read');
  assert.throws(()=>module.readCheckedReleaseOperationView({root:'untrusted'}));
  const stagePath=path.join(value.directory,'.github/workflows/release-staged.yml');
  const stageBytes=fs.readFileSync(stagePath,'utf8');
  for(const mutate of [
   doc=>delete doc.jobs['auto_prepare-source'].permissions.actions,
   doc=>doc.jobs['auto_prepare-source'].permissions.actions='write',
   doc=>doc.jobs['auto_compile-release'].permissions={actions:'read'},
   doc=>doc.jobs['auto_compile-release'].env.EXPECT_COMPILER='foreign',
   doc=>doc.jobs['auto_verify-package'].env.EXPECT_SOURCE='foreign',
   doc=>doc.jobs['auto_create-release'].needs.pop(),
   doc=>doc.jobs['auto_create-release'].outputs['container-tag']='fake',
   doc=>doc.jobs['auto_prepare-source'].steps.reverse(),
   doc=>doc.jobs['auto_compile-release']['timeout-minutes']=31,
   doc=>doc.jobs['auto_compile-release'].concurrency={group:'formal-ai-repository-writes'},
   doc=>doc.jobs['auto_compile-release'].steps.find(step=>step.name==='Download immutable selected source artifact').with['artifact-ids']='forged'
  ]) {
   const changed=YAML.parse(stageBytes);mutate(changed);fs.writeFileSync(stagePath,YAML.stringify(changed));
   assert.throws(()=>module.readCheckedReleaseOperationView());fs.writeFileSync(stagePath,stageBytes);
  }
  for(const mutate of [text=>text+'\n# caller drift\n',text=>text.replace('      actions: read','      actions: write')]){
   fs.writeFileSync(callerPath,mutate(caller));assert.throws(()=>module.readCheckedReleaseOperationView());fs.writeFileSync(callerPath,caller);
  }
  const retainedPath=path.join(value.directory,evidence,'original-release-workflow.yml');
  fs.writeFileSync(retainedPath,value.original+'\n# retained drift\n');assert.throws(()=>module.readCheckedReleaseOperationView());
  fs.writeFileSync(retainedPath,value.original);
  const alternate=path.join(value.directory,'same-caller.yml');fs.writeFileSync(alternate,caller);
  fs.rmSync(callerPath);fs.symlinkSync(alternate,callerPath);assert.throws(()=>module.readCheckedReleaseOperationView());
 }finally{fs.rmSync(value.directory,{recursive:true,force:true});}
});
