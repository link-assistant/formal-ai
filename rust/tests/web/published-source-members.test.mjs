// Standalone focused test snippet; import owner unchanged when appended to the existing test.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {orderedSourceMembers,readWorkflowPolicy} from '../../../scripts/lib/hive-delivery-source.mjs';
const memberCommit='a'.repeat(40),memberRepository='link-assistant/formal-ai';
function sourceMember(text){
 const bytes=Buffer.from(text);
 return {type:'file',size:bytes.length,encoding:'base64',content:bytes.toString('base64'),
  sha:createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex')};
}
test('immutable source member waves preserve order and bound four physical reads',async()=>{
 const names=Array.from({length:9},(_,index)=>'source-'+index),pending=[];
 let active=0,maximum=0,calls=0;
 const reader={read:async endpoint=>{
  const name=endpoint.split('/contents/')[1].split('?')[0];calls++;active++;maximum=Math.max(maximum,active);
  const result=await new Promise(resolve=>pending.push(()=>resolve(sourceMember(name))));active--;return result;
 }};
 const result=orderedSourceMembers(reader,memberRepository,memberCommit,names);
 for(const size of [4,4,1]){
  assert.equal(pending.length,size);pending.splice(0).reverse().forEach(resolve=>resolve());
  // Let the completed sourceBytes/Promise.all wave schedule the next one.
  for(let turn=0;turn<12;turn++)await Promise.resolve();
 }
 assert.deepEqual((await result).map(bytes=>bytes.toString()),names);assert.equal(maximum,4);assert.equal(calls,9);
});
test('source member failure refuses identity and never starts another wave',async()=>{
 let calls=0;const reader={read:async()=>{calls++;return {...sourceMember('source'),sha:'b'.repeat(40)};}};
 await assert.rejects(orderedSourceMembers(reader,memberRepository,memberCommit,Array(9).fill('source')),/source Git blob/);
 assert.equal(calls,4);
 await assert.rejects(orderedSourceMembers(reader,memberRepository,memberCommit,Array(129).fill('source')));
 assert.equal(calls,4);
});
test('workflow policy preserves directory order and ignores non-workflow members',async()=>{
 const entries=[{name:'z.yml',path:'.github/workflows/z.yml'},{name:'notes.txt',path:'.github/workflows/notes.txt'},
  {name:'a.yaml',path:'.github/workflows/a.yaml'}];
 const seen=[];const reader={read:async endpoint=>{
  if(endpoint.includes('/contents/.github/workflows?'))return entries;
  seen.push(endpoint);return sourceMember('name: '+(endpoint.includes('z.yml')?'last':'first')+'\n');
 }};
 const workflows=await readWorkflowPolicy(reader,memberRepository,memberCommit);
 assert.deepEqual(workflows.map(value=>[value.path,value.document.name]),[
  ['.github/workflows/z.yml','last'],['.github/workflows/a.yaml','first']]);
 assert.equal(seen.length,2);
});
