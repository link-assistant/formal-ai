import assert from 'node:assert/strict';
import {test}from 'node:test';
import {splitResponseFamilies}from '../../../scripts/lib/split-response-families.mjs';
const record=(id,intent,language,text)=>`  response ${id}\n    intent ${intent}\n    language ${language}\n    text "${text}"\n`;
const source='multilingual_responses\n'+record('a-en','a','en','{value}|first')+record('a-es','a','es','{value}|primero')+record('b-en','b','en','second\\nline')+record('b-es','b','es','segunda');
test('complete intent families retain exact ordered record bytes and placeholders',()=>{
 const result=splitResponseFamilies(source,9);
 assert.equal(result.shards.length,2);
 assert.deepEqual(result.shards.map(s=>s.split('\n').length-1),[9,9]);
 assert.equal(result.shards.map(s=>s.slice(result.header.length)).join(''),source.slice(result.header.length));
 assert.ok(result.shards[0].includes('{value}|primero'));
 assert.ok(result.shards[1].includes('second\\nline'));
});
test('unknown aggregation children, incomplete fields and ambiguous lookup ownership are refused',()=>{
 for(const candidate of [source.replace('  response a-en','  meaning a-en'),source.replace('    language en\n',''),source+record('a-en','different','ru','duplicate id'),source+record('extra','a','en','duplicate key'),source+record('extra','a','ru','noncontiguous family'),source.replace('multilingual_responses','meanings')])assert.throws(()=>splitResponseFamilies(candidate,9));
});
test('a single indivisible family never crosses or raises the supplied line ceiling',()=>{
 assert.throws(()=>splitResponseFamilies(source,8),/complete family exceeds/);
 assert.throws(()=>splitResponseFamilies(source.slice(0,-1),9));
 assert.throws(()=>splitResponseFamilies(source,0));
});
