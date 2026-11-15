import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {requirePinnedLoadedSources} from '../../../js/provenance/pinned-source-comparison.mjs';
const root=path.resolve('synthetic-pinned-root'),originalRuntime=path.join(root,'capture'),freshRuntime=path.join(root,'replay');
const context={repository:root,originalRuntime,freshRuntime,bootstrapArtifacts:[{file:'entry.cjs'}]};
const entry=(file,format='module',blake3='a'.repeat(64))=>({url:pathToFileURL(path.join(root,file)).href,format,blake3});
test('exact source identities remain required while extra recomputation inputs confer no authority',()=>{
 const expected=entry('js/planner.mjs');
 requirePinnedLoadedSources([expected],[expected,entry('js/verification.mjs')],context);
 for(const actual of [[],[{...expected,format:'repository-read-buffer'}],[{...expected,blake3:'b'.repeat(64)}]])
  assert.throws(()=>requirePinnedLoadedSources([expected],actual,context),/PinnedReplayLoadedSourceMismatch/u);
});
test('only declared bootstrap source addresses remap to the fresh runtime',()=>{
 const expected=entry('capture/entry.cjs','commonjs'),actual=entry('replay/entry.cjs','commonjs');
 requirePinnedLoadedSources([expected],[actual],context);
 assert.throws(()=>requirePinnedLoadedSources([entry('capture/other.cjs')],[entry('replay/other.cjs')],context),/PinnedReplayLoadedSourceMismatch/u);
 assert.throws(()=>requirePinnedLoadedSources([expected],[{...actual,blake3:'b'.repeat(64)}],context),/PinnedReplayLoadedSourceMismatch/u);
});
test('missing input diagnostics retain bounded project identities and redact external source paths',()=>{
 const external={...entry('js/planner.mjs'),url:pathToFileURL(path.resolve(root,'../private-personal/file.mjs')).href};
 const expected=Array.from({length:20},(_,index)=>entry('js/source-'+index+'.mjs'));
 let failure;try{requirePinnedLoadedSources(expected,[],context);}catch(error){failure=error;}
 assert.equal(failure.message,'PinnedReplayLoadedSourceMismatch');assert.equal(failure.missingRuntimeInputCount,20);
 assert.equal(failure.missingRuntimeInputs.length,16);assert.deepEqual(failure.missingRuntimeInputs[0],{
  source:'js/source-0.mjs',format:'module',blake3:'a'.repeat(64)});
 assert.throws(()=>requirePinnedLoadedSources([external],[],context),error=>{
  assert.equal(error.missingRuntimeInputs[0].source,'external-source');
  assert.equal(JSON.stringify(error.missingRuntimeInputs).includes('private-personal'),false);return true;
 });
});
