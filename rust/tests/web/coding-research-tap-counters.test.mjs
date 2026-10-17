import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {verificationReceiptGap} from '../../../js/agentic/coding_research.mjs';
import {tapCounterSyntaxValid} from '../../../js/agentic/portable/coding_research_kernel.mjs';
const vectors=JSON.parse(fs.readFileSync(new URL('../fixtures/coding-research/tap-counter-lines.json',import.meta.url),'utf8'));
await installNodeHost(new WorkerHost({WebAssembly:{
 instantiate(){throw Error('LocalWasmExecutionForbidden');},
 compile(){throw Error('LocalWasmExecutionForbidden');},
}}));
test('recognized malformed or duplicate fields fail closed across existing receipt frames',()=>{
 for(const row of vectors){
  const raw='Output: '+row.payload+'\nExit Code: 0';
  const structured=JSON.stringify({output:row.payload,exit_code:0});
  assert.equal(verificationReceiptGap(raw),row.expected,row.id);
  assert.equal(verificationReceiptGap(structured),row.expected,row.id);
 }
});
test('canonical syntax policy distinguishes absence from malformed and duplicate evidence',()=>{
 for(const [recognized,valid,expected]of [[0,0,true],[1,1,true],[2,2,false],[1,0,false],[0,1,false],[-1,-1,false],[Infinity,Infinity,false],[NaN,0,false]]){
  assert.equal(tapCounterSyntaxValid(recognized,valid),expected);
 }
});
