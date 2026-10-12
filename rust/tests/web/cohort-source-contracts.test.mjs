import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { digest } from '../../../experiments/formal_ai_subagent/cohort-runner.mjs';
import { readBoundSource, extractExportedLiteral, extractArithmeticAssertions, validateHeldOutContract } from '../../../experiments/formal_ai_subagent/cohort-source-contracts.mjs';
function fixture(source) {
 const directory=mkdtempSync(join(tmpdir(),'static-contract-controls-'));const path=join(directory,'contract.mjs');writeFileSync(path,source);
 return {binding:{path,sha256:digest(readFileSync(path))},cleanup:()=>rmSync(directory,{recursive:true,force:true})};
}
test('static contract extraction retains source spans and is deterministic',()=>{
 const f=fixture("export const variants = [{id:'held', names:['caption'], enabled:true, count:2, absent:null}];");try{
  const first=extractExportedLiteral(f.binding,'variants');assert.equal(first.value[0].id,'held');assert.equal(first.value[0].count,2);
  assert.equal(first.value[0].absent,null);assert.deepEqual(first,extractExportedLiteral(f.binding,'variants'));
  assert.match(readBoundSource(f.binding).slice(first.span.start,first.span.end),/^\[/u);
 }finally{f.cleanup();}
});
test('calls computed expressions and duplicate exports never execute or fabricate literal contracts',()=>{
 for(const source of ["export const variants = [(() => { throw new Error('must not execute'); })()];",'export const variants=[1+2];','export const variants=[]; export const variants=[];']){
  const f=fixture(source);try{assert.throws(()=>extractExportedLiteral(f.binding,'variants'),/nonliteral|syntax|one explicit/);}finally{f.cleanup();}
 }
});
test('duplicate and prototype keys are refused and observed source drift is rejected',()=>{
 for(const source of ["export const variants=[{id:'a',id:'b'}];","export const variants=[{__proto__:'x'}];"]){
  const f=fixture(source);try{assert.throws(()=>extractExportedLiteral(f.binding,'variants'),/unsafe/);}finally{f.cleanup();}
 }
 const f=fixture('export const variants=[];');try{writeFileSync(f.binding.path,'drift');assert.throws(()=>readBoundSource(f.binding),/drift/);}finally{f.cleanup();}
});
test('binary numeric fixture assertions generalize callable names and preserve independent expected values',()=>{
 const f=fixture("test('fixture',async()=>{assert.equal(module.combine(7,-3),4);assert.equal(module.combine(-6,2),-4);});");try{
  const result=extractArithmeticAssertions(f.binding,'fixture');assert.equal(result.exportName,'combine');
  assert.deepEqual(result.assertions.map(({first,second,expected})=>({first,second,expected})),[{first:7,second:-3,expected:4},{first:-6,second:2,expected:-4}]);
 }finally{f.cleanup();}
});
test('insufficient dynamic and ambiguous numeric oracles are refused',()=>{
 for(const source of ["test('fixture',()=>{assert.equal(module.sum(1,2),3);});","test('fixture',()=>{assert.equal(module.sum(1,-2),-1);assert.equal(module.other(-1,2),1);});","test('fixture',()=>{assert.equal(module.sum(value,-2),-1);assert.equal(module.sum(-1,2),1);});"]){
  const f=fixture(source);try{assert.throws(()=>extractArithmeticAssertions(f.binding,'fixture'),/insufficient|ambiguous|numeric/);}finally{f.cleanup();}
 }
});
test('heldout declaration boundary rejects escapes unknown identifiers and alias collisions',()=>{
 const entry={id:'held',destination:'nested/module.mjs',exportName:'caption',parameterName:'choice',sourceBindings:{selector:'pick',summary:'describe'},fixtureNames:['case']};
 assert.equal(validateHeldOutContract([entry])[0],entry);
 for(const invalid of [{...entry,
   destination:'../escape.mjs'},
   {...entry,
   destination:'C:/escape.mjs'},
   {...entry,
   destination:'..\\escape.mjs'},
   {...entry,
   exportName:undefined},
   {...entry,
   sourceBindings:{}},
   {...entry,
   sourceBindings:{selector:'same',
   summary:'same'}}])assert.throws(()=>validateHeldOutContract([invalid]),
   /heldout/);

});
test('maintained archive exposes two heldouts and original request remains955 bytes, not seven tasks',()=>{
 const root=resolve(fileURLToPath(new URL('../../../',import.meta.url)));
 const archive=join(root,'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/module-observation/original-selected-summary.test.mjs');
 const request=join(root,'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/g132/T1804-request.json');
 const source=readFileSync(archive);const contract=extractExportedLiteral({path:archive,sha256:digest(source)},'heldOutCases');
 assert.equal(validateHeldOutContract(contract.value).length,2);
 assert.equal(Buffer.byteLength(JSON.parse(readFileSync(request,'utf8')).prompt),955);
});


import { discoverSourceContracts, compositionVariantTask, checkSourceEvaluation } from '../../../experiments/formal_ai_subagent/evaluate-source-contracts.mjs';
test('source-owned catalog generation is deterministic and does not invent broad cohorts',()=>{
 const root=resolve(fileURLToPath(new URL('../../../',import.meta.url)));const first=discoverSourceContracts(root);
 assert.deepEqual(first,discoverSourceContracts(root));assert.equal(first.independentFamilies,2);assert.equal(first.representativeBaseline,false);
 assert.equal(first.heldOut.value.length,2);assert.equal(Buffer.byteLength(first.original.task),955);
});
test('catalog check refuses task reclassification dropped cases and source tampering',()=>{
 const root=resolve(fileURLToPath(new URL('../../../',import.meta.url))),catalog=discoverSourceContracts(root);
 const dir=mkdtempSync(join(tmpdir(),'catalog-check-controls-')),path=join(dir,'manifest.json');
 const cases=[{runId:'g132-original',
   taskKind:'self-coding',
   task:catalog.original.task},
   ...catalog.heldOut.value.map(item=>({runId:'g132-'+item.id,
   taskKind:'self-coding',
   task:compositionVariantTask(item)})),
   {runId:'ordinary-arithmetic',
   taskKind:'coding',
   task:`Add ${catalog.arithmetic.exportName}(first,second) in result.mjs.`}].map(item=>({...item,
   category:'feature-implementation',
   expectedRelation:'output-larger',
   taskSHA256:digest(item.task)}));

 const manifest={catalogSHA256:digest(JSON.stringify(catalog)),cases,bindings:[catalog.original.source,catalog.heldOut.source,catalog.arithmetic.source]};
 try {
  writeFileSync(path,JSON.stringify(manifest));assert.equal(checkSourceEvaluation(root,path).attemptedTasks,4);
  writeFileSync(path,JSON.stringify({...manifest,cases:cases.slice(1)}));assert.throws(()=>checkSourceEvaluation(root,path),/missing/);
  writeFileSync(path,JSON.stringify({...manifest,cases:cases.map((item,index)=>index?item:{...item,category:'repair'})}));assert.throws(()=>checkSourceEvaluation(root,path),/changed/);
  writeFileSync(path,JSON.stringify({...manifest,catalogSHA256:'0'.repeat(64)}));assert.throws(()=>checkSourceEvaluation(root,path),/drift/);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
