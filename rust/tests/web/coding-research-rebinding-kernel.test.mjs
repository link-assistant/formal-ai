import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {validateProgram} from '../../../js/agentic/host.mjs';
import {observeCodingResearchRebindingCandidate} from '../../../js/agentic/coding_research_observation.mjs';
import {codingResearchRebindingKernel,canonicalUniqueSourceIndex} from '../../../js/agentic/portable/coding_research_rebinding.mjs';
const fixtures=JSON.parse(fs.readFileSync(new URL('../fixtures/coding-research/rebinding-held-out.json',import.meta.url)));
await installNodeHost(new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}}));
const observed=(source,language='javascript')=>observeCodingResearchRebindingCandidate(source,language,'observed.mjs');
const render=candidate=>codingResearchRebindingKernel(candidate,'javascript','chosen',['input']);
test('actual catalog and tokenizer rebind unseen names while quoted content and Unicode remain lossless',()=>{
 for(const fixture of fixtures.accepted){
  const candidate=observed(fixture.source,fixture.language);
  const result=codingResearchRebindingKernel(candidate,fixture.language,fixture.name,fixture.parameters);
  assert.equal(result.$,'rendered',fixture.source);
  assert.equal(result.source,fixture.expected);
  assert.equal(validateProgram(fixture.language,result.source),true);
  assert.deepEqual(result.bindingsFrom,[candidate.name,...candidate.parameters]);
  assert.deepEqual(result.bindingsTo,[fixture.name,...fixture.parameters]);
 }
});
test('every catalog refusal and language/arity mismatch remains a rejection',()=>{
 const candidate=observed('function sampled(seed) { return seed; }');assert.equal(render(candidate).$,'rendered');
 for(const field of ['syntaxValid','moduleEffectsNone','moduleSyntaxSupported','contractSupported','callEffectsNone']){assert.deepEqual(render({...candidate,[field]:false}),{$:'rejected'},field);}
 for(const [field,value]of [['importsCount',1],['declarationsCount',0],['declarationsCount',2]])assert.deepEqual(render({...candidate,[field]:value}),{$:'rejected'});
 assert.deepEqual(render({...candidate,language:'typescript'}),{$:'rejected'});assert.deepEqual(codingResearchRebindingKernel(candidate,'javascript','chosen',[]),{$:'rejected'});
 assert.deepEqual(render({...candidate,pieces:[...candidate.pieces,'extra']}),{$:'rejected'});
});
test('binding identity collisions cannot silently shadow source or destination declarations',()=>{
 const candidate=observed('function sampled(seed) { return seed; }');
 assert.deepEqual(render({...candidate,name:'seed'}),{$:'rejected'});
 assert.deepEqual(codingResearchRebindingKernel(candidate,'javascript','input',['input']),{$:'rejected'});
 assert.deepEqual(codingResearchRebindingKernel(candidate,'javascript','',['input']),{$:'rejected'});
});
test('property keys, member collisions, comments and nested binding scopes are conservatively unsupported',()=>{
 for(const source of fixtures.rejected){const candidate=observed(source);assert.deepEqual(render(candidate),{$:'rejected'},source);
  // Even a falsely optimistic catalog cannot authorize renaming these lexical contexts.
  assert.deepEqual(render({...candidate,syntaxValid:true,moduleEffectsNone:true,moduleSyntaxSupported:true,contractSupported:true,callEffectsNone:true}),{$:'rejected'},source);
 }
});
test('a claimed identifier binding inside a quoted fragment is rejected instead of edited',()=>{
 const candidate=observed("function sampled(seed) { return 'seed'; }");const at=candidate.pieces.indexOf("'seed'");assert(at>=0);candidate.pieceBindings[at]='seed';assert.deepEqual(render(candidate),{$:'rejected'});
});
test('unique selection preserves no-result and ambiguity rather than picking the first proposal',()=>{
 for(const [valid,expected]of [[[],-1],[[false,false],-1],[[true],0],[[false,true,false],1],[[true,true],-2],[[false,true,false,true],-2]])assert.equal(canonicalUniqueSourceIndex(valid),expected);
});
