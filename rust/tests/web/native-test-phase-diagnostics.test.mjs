// Synthetic filesystem and callback observations only; no native commands or repository-wide identity.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {observeNativeTestPhase} from '../../../scripts/lib/native-response-identity.mjs';
import {writeReceipt,verifyReceipt} from '../../../scripts/native-test-artifact.mjs';

function states(events){return events.map(event=>[event.phase,event.state]);}
function finiteTimes(events){
 for(const event of events){
  assert.deepEqual(Object.keys(event).sort(),['elapsedMs','phase','state']);
  assert(Number.isFinite(event.elapsedMs)&&event.elapsedMs>=0);
 }
}
test('started is delivered before bounded work and completed preserves its return value',()=>{
 const events=[],value={synthetic:true};
 const actual=observeNativeTestPhase('native-inputs',()=>{
  assert.deepEqual(states(events),[['native-inputs','started']]);
  const began=performance.now();while(performance.now()-began<2){}
  return value;
 },event=>events.push(event));
 assert.equal(actual,value);
 assert.deepEqual(states(events),[['native-inputs','started'],['native-inputs','completed']]);
 finiteTimes(events);assert(events[1].elapsedMs>=events[0].elapsedMs);
});
test('failed preserves the original exception and reports no error contents',()=>{
 const events=[],failure=new Error('private fixture detail must not enter telemetry');
 assert.throws(()=>observeNativeTestPhase('tracked-test-inputs',()=>{
  assert.equal(events[0].state,'started');throw failure;
 },event=>events.push(event)),error=>error===failure);
 assert.deepEqual(states(events),[['tracked-test-inputs','started'],['tracked-test-inputs','failed']]);
 finiteTimes(events);assert(!JSON.stringify(events).includes(failure.message));
});
function fixture(t){
 const root=mkdtempSync(join(tmpdir(),'native-phase-fixture-')),directory=join(root,'tests'),binary=join(root,'formal-ai');
 mkdirSync(directory);
 for(const name of ['unit','integration','source'])writeFileSync(join(directory,name),'synthetic '+name+' bytes');
 writeFileSync(binary,'synthetic binary bytes');
 t.after(()=>rmSync(root,{recursive:true,force:true}));
 return{root,directory,binary,identity:{fixture:'mocked identity only'}};
}
test('receipt diagnostics preserve actual physical hashes and reject tampered bytes',t=>{
 const f=fixture(t),events=[],observe=event=>events.push(event);
 const receipt=writeReceipt(f.directory,f.identity,f.binary,observe);
 assert.deepEqual(verifyReceipt(f.directory,f.identity,f.binary,{observe}),receipt);
 assert.deepEqual(states(events),[
  ['write-receipt','started'],['write-receipt','completed'],
  ['verify-receipt','started'],['verify-receipt','completed']]);
 events.length=0;writeFileSync(join(f.directory,'unit'),'tampered actual bytes');
 assert.throws(()=>verifyReceipt(f.directory,f.identity,f.binary,{observe}));
 assert.deepEqual(states(events),[['verify-receipt','started'],['verify-receipt','failed']]);
 finiteTimes(events);assert(!JSON.stringify(events).includes('tampered actual bytes'));
});
test('imported APIs stay quiet without an observer and preserve full-member custody',t=>{
 const f=fixture(t),saved=console.log,printed=[];
 try{
  console.log=(...args)=>printed.push(args);
  const receipt=writeReceipt(f.directory,f.identity,f.binary);
  assert.deepEqual(verifyReceipt(f.directory,f.identity,f.binary),receipt);
  writeFileSync(join(f.directory,'integration'),'foreign member');
  assert.throws(()=>verifyReceipt(f.directory,f.identity,f.binary));
  assert.deepEqual(printed,[]);
 }finally{console.log=saved;}
});
test('unknown phase labels refuse before work or telemetry',()=>{
 let ran=false;const events=[];
 assert.throws(()=>observeNativeTestPhase('/private/unsupported',()=>{ran=true},event=>events.push(event)));
 assert.equal(ran,false);assert.deepEqual(events,[]);
});
