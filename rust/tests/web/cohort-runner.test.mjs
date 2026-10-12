import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, realpathSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { digest, validateManifest, admitManifest, executeAdmittedCase, partitionMeasurements, deriveDeclaredVariants } from '../../../experiments/formal_ai_subagent/cohort-runner.mjs';

function setup() {
  const dir = mkdtempSync(join(tmpdir(), 'cohort-control-'));
  const workspace = join(dir, 'workspace'); mkdirSync(workspace);
  const source = join(dir, 'source.mjs'); writeFileSync(source, 'export const immutable = 1;\n');
  const oracle = join(dir, 'oracle.test.mjs');
  writeFileSync(oracle, "import assert from 'node:assert/strict'; import {readFileSync} from 'node:fs'; import {join} from 'node:path'; assert.equal(readFileSync(join(process.env.COHORT_WORKSPACE,'result.mjs'),'utf8'),'export const answer = 42;\\n');\n");
  const binding = path => ({path, sha256:digest(readFileSync(path))});
  const task = 'Add the requested independently accepted module.';
  const item = {runId:'control',taskKind:'coding',category:'feature-implementation',expectedRelation:'output-larger',
    task,taskSHA256:digest(task),workspace,allowedEffects:[join(workspace,'result.mjs')],oracle:binding(oracle),timeoutMilliseconds:1000};
  const manifest = {schemaVersion:1,cohortId:'instrumentation-control-only',bindings:[binding(source),binding(oracle)],cases:[item]};
  return {dir,workspace,source,oracle,item,manifest,journal:join(dir,'journal.jsonl'),cleanup:()=>rmSync(dir,{recursive:true,force:true})};
}
const rows = fixture => readFileSync(fixture.journal,'utf8').trim().split('\n').map(JSON.parse);
const controlDriver = async (_, workspace) => {writeFileSync(join(workspace,'result.mjs'),'export const answer = 42;\n'); return {stop:'final',transcript:[{control:true}]};};

test('exclusive physical admission and independent control acceptance preserve ordered journal',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);
  assert.equal(rows(f)[0].kind,'admitted');
  assert.throws(()=>admitManifest(f.manifest,f.journal),/EEXIST/);
  const result=await executeAdmittedCase(admission,'control','first',controlDriver);
  assert.equal(result.accepted,true);assert.equal(result.usage.status,'Unknown');
  assert.deepEqual(rows(f).map(row=>row.kind),['admitted','started','finished']);
  await assert.rejects(executeAdmittedCase(admission,'control','first',controlDriver),/duplicate attempt/);
 }finally{f.cleanup();}
});
test('undeclared task and source drift refuse execution',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);let calls=0;
  await assert.rejects(executeAdmittedCase(admission,'missing','first',()=>{calls++;}),/undeclared/);
  writeFileSync(f.source,'drift');
  await assert.rejects(executeAdmittedCase(admission,'control','first',()=>{calls++;}),/drift/);
  assert.equal(calls,0);assert.equal(rows(f).length,1);
 }finally{f.cleanup();}
});
test('no-plan actual-driver disposition remains durable measured failure',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);
  const result=await executeAdmittedCase(admission,'control','first',async()=>({stop:'no-plan'}));
  assert.equal(result.accepted,false);assert.match(result.failure.message,/did not complete/);
  assert.equal(rows(f).at(-1).kind,'finished');
 }finally{f.cleanup();}
});
test('wrong behavioral output is independently refused',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);
  const result=await executeAdmittedCase(admission,'control','first',async(_,workspace)=>{writeFileSync(join(workspace,'result.mjs'),'wrong');return {stop:'final'};});
  assert.equal(result.accepted,false);assert.notEqual(result.check.exitCode,0);
 }finally{f.cleanup();}
});
test('out-of-scope effect cannot gain acceptance',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);
  const result=await executeAdmittedCase(admission,'control','first',async(_,workspace)=>{await controlDriver(_,workspace);writeFileSync(join(workspace,'extra.mjs'),'padding');return {stop:'final'};});
  assert.equal(result.accepted,false);assert.match(result.failure.message,/scope/);assert.equal(rows(f).at(-1).kind,'finished');
 }finally{f.cleanup();}
});
test('immutable oracle mutation remains durable failure',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);
  const result=await executeAdmittedCase(admission,'control','first',async()=>{writeFileSync(f.oracle,'process.exit(0);');return {stop:'final'};});
  assert.equal(result.accepted,false);assert.equal(rows(f).at(-1).kind,'finished');
 }finally{f.cleanup();}
});
test('manifest category, identity and external oracle guards are independent',()=>{
 const f=setup();try {
  assert.throws(()=>validateManifest({...f.manifest,cases:[{...f.item,taskSHA256:'0'.repeat(64)}]}),/task/);
  assert.throws(()=>validateManifest({...f.manifest,cases:[{...f.item,expectedRelation:'unrestricted'}]}),/relation/);
  assert.throws(()=>validateManifest({...f.manifest,cases:[f.item,f.item]}),/duplicate/);
  assert.throws(()=>validateManifest({...f.manifest,cases:[{...f.item,oracle:{path:join(f.workspace,'oracle.mjs'),sha256:'0'.repeat(64)}}]}),/outside/);
 }finally{f.cleanup();}
});
test('tampered journal admission identity refuses the actual callback',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);const record=rows(f)[0];record.manifest.cases[0].task='changed';writeFileSync(f.journal,JSON.stringify(record)+'\n');
  await assert.rejects(executeAdmittedCase(admission,'control','first',controlDriver),/identity/);
 }finally{f.cleanup();}
});
test('declared generator is reproducible and refuses unsupported substitutions',()=>{
 const f=setup();try {
  const contract={source:f.manifest.bindings[0],taskTemplate:'Add {{exportName}} in {{destination}}.',allowedFields:['exportName','destination']};
  const variants=[{runId:'heldout',parameters:{exportName:'caption',destination:'nested/module.mjs'}}];
  assert.deepEqual(deriveDeclaredVariants(contract,variants),deriveDeclaredVariants(contract,variants));
  assert.throws(()=>deriveDeclaredVariants(contract,[{runId:'bad',parameters:{...variants[0].parameters,unexpected:'x'}}]),/undeclared/);
  assert.throws(()=>deriveDeclaredVariants(contract,[variants[0],variants[0]]),/duplicate/);
 }finally{f.cleanup();}
});
test('optional actual receipt remains captured unnormalized and partitions preserve combined API',async()=>{
 const f=setup();try {
  const admission=admitManifest(f.manifest,f.journal);
  const result=await executeAdmittedCase(admission,'control','first',async(...args)=>({...await controlDriver(...args),usageReceipt:{instrumentationControl:true,inputTokens:1}}));
  assert.equal(result.usage.status,'CapturedUnnormalized');assert.equal(result.usage.rawSHA256,digest(JSON.stringify(result.usage.raw)));
  const all=[{taskKind:'coding'},{taskKind:'self-coding'}];const parts=partitionMeasurements(all);
  assert.equal(parts.combined,all);assert.equal(parts.ordinaryCoding.length,1);assert.equal(parts.selfCoding.length,1);
 }finally{f.cleanup();}
});


test('incomplete and throwing drivers retain exact failed-attempt source effects',async()=>{
 for(const disposition of ['no-plan','steps','throw','throw-null']){
  const f=setup();try{
   const admission=admitManifest(f.manifest,f.journal);
   const result=await executeAdmittedCase(admission,'control','first',async(_,workspace)=>{
    writeFileSync(join(workspace,'result.mjs'),'export const answer = 42;\n');
    if(disposition==='throw-null')throw null;
    if(disposition==='throw')throw new Error('actual driver failed after write');
    return {stop:disposition};
   });
   assert.equal(result.accepted,false);assert.equal(result.sourceEffects.length,1);
   assert.equal(result.sourceEffects[0].path,join(realpathSync(f.workspace),'result.mjs'));
   assert.equal(result.sourceEffects[0].before,null);
   assert.equal(result.sourceEffects[0].after.content,'export const answer = 42;\n');
   assert.deepEqual(rows(f).at(-1).result.sourceEffects,result.sourceEffects);
   assert.ok(result.failure);
  }finally{f.cleanup();}
 }
});
