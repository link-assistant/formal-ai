import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,readFileSync,mkdirSync,chmodSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {planNativeResponseCaptures} from '../../../scripts/lib/native-response-scheduling.mjs';
import {executeBatches} from '../../../scripts/lib/coverage-execution.mjs';
const row=(caller,target='unit')=>({id:'source/'+target+'.rs::'+caller,target,caller});
const listing=cases=>cases.map(c=>c.target+'\t'+c.caller);
function plan(cases,weights=new Map(),fallback=.092) {
 return planNativeResponseCaptures(cases,listing(cases),new Map(cases.map(c=>[c.target,'fixture'])),weights,fallback);
}
test('recorded late selection starts first; singleton ties and fallback retain original identity',()=>{
 const original=[row('a_short'),row('z_long'),row('b_unknown'),row('c_zero')];
 const result=plan(original,new Map([['a_short',1],['z_long',9.3],['c_zero',0]]));
 assert.deepEqual(result.map(c=>c.caller),['z_long','a_short','b_unknown','c_zero']);
 assert.deepEqual(result.map(c=>c.selectionIndex),[1,0,2,3]);
 assert.deepEqual(result.map(c=>c.dispatchIndex),[0,1,2,3]);
 assert.deepEqual(result.map(c=>c.weight.origin),['recorded','recorded','fallback','recorded']);
 assert.deepEqual(result.map(c=>c.weight.seconds),[9.3,1,.092,0]);
 assert.ok(result.every(c=>c.batch.names.length===1));
 assert.deepEqual(new Set(result.map(c=>c.id)),new Set(original.map(c=>c.id)));
 const tied=plan([row('z_same'),row('a_same')],new Map([['z_same',2],['a_same',2]]));
 assert.deepEqual(tied.map(c=>c.batch.names),[['a_same'],['z_same']]);
});
test('same caller leaf in different registered targets retains distinct process identity',()=>{
 const cases=[row('nested::same','integration'),row('nested::same','unit')];
 const result=plan(cases,new Map([['nested::same',2]]));
 assert.deepEqual(result.map(c=>c.target),['integration','unit']);
 assert.equal(new Set(result.map(c=>c.id)).size,2);
});
test('duplicate source IDs, ambiguous qualified callers and invalid weights fail closed',()=>{
 assert.throws(()=>plan([row('same'),row('same')]),/duplicate original source identity/);
 assert.throws(()=>plan([{...row('same'),id:'one'}, {...row('same'),id:'two'}]),/ambiguous original qualified caller/);
 for(const value of [-1,NaN,Infinity])assert.throws(()=>plan([row('case')],new Map([['case',value]])),/invalid weight/);
 assert.throws(()=>plan([row('case')],new Map(),-1),/invalid fallback duration/);
 assert.throws(()=>planNativeResponseCaptures([row('unlisted')],[],new Map([['unit','fixture']]),new Map()),/not listed/);
 assert.throws(()=>planNativeResponseCaptures([row('case')],['unit\tcase'],new Map(),new Map()),/missing executable/);
});
for(const mode of ['success','first-failure','missing-completion'])test('real Node process dispatch preserves isolated whole bytes and '+mode,async()=>{
 const directory=mkdtempSync(join(tmpdir(),'native-capture-weighted-'));
 try {
  const executable=join(directory,'harness.mjs');
  const payload='Actual Node fixture answer: failed is source data\r\n😀\n';
  writeFileSync(executable,`#!/usr/bin/env node\nimport fs from 'node:fs';\nconst args=process.argv.slice(2);\nconst caller=args.at(-1);\nfs.appendFileSync(process.env.LAUNCH_LOG,caller+'\\n');\nfs.writeFileSync(process.env.FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR+'/native-'+process.pid+'.jsonl',JSON.stringify({caller,processId:process.pid,answer:${JSON.stringify(payload)}})+'\\n');\nconsole.error('raw stderr '+caller);\nconst fails=process.env.MODE==='first-failure'&&caller==='z_long';\nconst missing=process.env.MODE==='missing-completion'&&caller==='z_long';\nif(!missing)console.log('test '+caller+' ... '+(fails?'FAILED':'ok'));\nconsole.log('test result: '+(fails?'FAILED':'ok')+'. '+(fails?0:1)+' passed; '+(fails?1:0)+' failed; 0 ignored; 0 measured; 0 filtered out;');\nprocess.exitCode=fails?1:0;\n`);
  chmodSync(executable,0o755);
  const original=[row('a_short'),row('z_long'),row('b_unknown')];
  const executables=new Map([['unit',executable]]);
  const ordered=planNativeResponseCaptures(original,listing(original),executables,new Map([['z_long',8],['a_short',1]]),.092);
  const started=[],observations=[];let active=0,peak=0;
  for(const selected of ordered) {
   const folder=join(directory,String(selected.dispatchIndex));mkdirSync(folder);
   const stdout=[],stderr=[];
   const actual=await executeBatches([selected.batch],executables,{cwd:directory,threads:1,concurrency:1,nocapture:true,
    env:{...process.env,MODE:mode,LAUNCH_LOG:join(directory,'launch.log'),FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR:folder},
    onStart:batch=>{started.push({caller:batch.names[0],weight:batch.weight});active++;peak=Math.max(peak,active);},
    onFinish:()=>active--,onStdout:chunk=>stdout.push(chunk),onStderr:chunk=>stderr.push(chunk)});
   const execution=actual.batches[0];
   const bytes=readFileSync(join(folder,'native-'+execution.processId+'.jsonl'));
   const record=JSON.parse(bytes);assert.equal(record.answer,payload);assert.equal(record.caller,selected.caller);
   assert.equal(record.processId,execution.processId);assert.ok(bytes.subarray(-1).equals(Buffer.from('\n')));
   assert.equal(Buffer.concat(stderr).toString(),'raw stderr '+selected.caller+'\n');
   assert.ok(Buffer.concat(stdout).length>0);assert.ok(execution.elapsedSeconds>=0);
   observations.push({selected,execution,folder});
  }
  assert.deepEqual(started,[{caller:'z_long',weight:8},{caller:'a_short',weight:1},{caller:'b_unknown',weight:.092}]);
  assert.equal(readFileSync(join(directory,'launch.log'),'utf8'),'z_long\na_short\nb_unknown\n');
  assert.equal(peak,1);assert.equal(active,0);assert.equal(new Set(observations.map(o=>o.folder)).size,3);
  assert.equal(new Set(observations.map(o=>o.execution.processId)).size,3);
  assert.deepEqual(new Set(observations.map(o=>o.selected.id)),new Set(original.map(o=>o.id)));
  assert.ok(observations.slice(1).every(o=>o.execution.succeeded));
  assert.equal(observations[0].execution.succeeded,mode==='success');
  assert.equal(observations[0].execution.exitCode,mode==='first-failure'?1:0);
  if(mode==='missing-completion')assert.ok(observations[0].execution.problems.includes('missing completion: z_long'));
 } finally {rmSync(directory,{recursive:true,force:true});}
});
