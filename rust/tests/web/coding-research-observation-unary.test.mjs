// Fixed consumer oracle declared before the natural-language source transformation.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {parse} from '@babel/parser';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
await installNodeHost(new WorkerHost());
const {observeCodingResearchAcceptanceTokens:observe}=await import('../../../js/agentic/coding_research_observation.mjs');
const cases=[['const n = 3;', ['const','n','=','3',';']],['item.value + item.text',['item','.','value','+','item','.','text']],['["X", 7]', ['[','"X"',',','7',']']],['function f(x) { return x; }',['function','f','(','x',')','{','return','x',';','}']]];
for(const [source,texts] of cases)test('fixed lexical output: '+source,()=>{const result=observe(source);assert.equal(result.complete,true);assert.deepEqual(result.texts,texts);assert.equal(result.kinds.length,texts.length);});
test('actual source parses and retains the predeclared owner comment',()=>{
 const source=fs.readFileSync(new URL('../../../js/agentic/coding_research_observation.mjs',import.meta.url),'utf8');
 parse(source,{sourceType:'module'});
 assert(source.includes('// Lossless lexical/catalog observations; canonical kernels make research decisions.'));
});
