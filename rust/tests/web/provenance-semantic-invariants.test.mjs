import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import {withEvidenceContext,activeDriveCapture} from '../../../js/provenance/capture-context.mjs';
import {EventLog} from '../../../js/agentic/crate/event_log.mjs';
import {persist,load,recordFor} from '../../../js/agentic/crate/derivation.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {host} from '../../../js/agentic/host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
before(async()=>installNodeHost(new WorkerHost({WebAssembly:{compile(){throw Error('LocalProductWasmForbidden');},instantiate(){throw Error('LocalProductWasmForbidden');}}})));
test('capture requires a real object with explicit active state',async()=>{
 for(const value of [null,undefined,false,'active',1,{}, {active:1},{active:false}]){
  assert.equal(await withEvidenceContext(value,()=>activeDriveCapture()),null);
 }
 const context={active:true};assert.equal(await withEvidenceContext(context,()=>activeDriveCapture()),context);
});
test('event log owns its events field and retains exact IDs, key order and first/last behavior',()=>{
 const initial=[{kind:'impulse',payload:'held-out input'},{kind:'render:emit',payload:'held-out output'}];
 const constructed=new EventLog(initial),incremental=new EventLog();
 for(const event of initial)incremental.append(event.kind,event.payload);
 assert.deepEqual(Object.keys(constructed),['events']);
 assert.deepEqual(Object.getOwnPropertyDescriptor(constructed,'events'),{value:constructed.events,writable:true,enumerable:true,configurable:true});
 assert.deepEqual(constructed.events,incremental.events);
 assert.equal(constructed.firstOf('impulse'),constructed.events[0]);
 assert.equal(constructed.lastOf('render:emit'),constructed.events[1]);assert.equal(constructed.firstOf('missing'),null);
});
test('derivation IO capabilities refuse missing/noncallable slots and preserve real write/read order',()=>{
 const record=recordFor([{kind:'render:emit',payload:'held-out'}],'answer_held_out');
 assert.deepEqual(persist(record,'/scratch',{}),{ok:false,error:'derivation_store_has_no_writer'});
 assert.deepEqual(persist(record,'/scratch',{writeText:42}),{ok:false,error:'derivation_store_has_no_writer'});
 assert.equal(load('/scratch','answer_held_out',{}),null);assert.equal(load('/scratch','answer_held_out',{readText:42}),null);
 const calls=[];let stored='';const io={createDirAll:directory=>calls.push(['mkdir',directory]),writeText:(file,text)=>{calls.push(['write',file]);stored=text;},readText:file=>{calls.push(['read',file]);return stored;}};
 const written=persist(record,'/scratch',io);assert.equal(written.ok,true);assert.deepEqual(load('/scratch','answer_held_out',io),record);
 assert.deepEqual(calls.map(row=>row[0]),['mkdir','write','read']);
});
test('noncallable observer slots never execute and ordinary production planning remains unchanged',async()=>{
 const blocked={WebAssembly:{compile(){throw Error('LocalProductWasmForbidden');},instantiate(){throw Error('LocalProductWasmForbidden');}}};
 await installNodeHost(new WorkerHost(blocked));
 const messages=[{role:'user',content:'Create file invariant.mjs containing export const invariant = 59;'}];
 const expected=await planChatStep(messages,['read','write']);
 const actual=await withEvidenceContext({active:true,planningObservation:'not-callable',engineObservation:7,engineGenesis:{}},()=>planChatStep(messages,['read','write']));
 assert.deepEqual(actual,expected);
 const answer=await withEvidenceContext({active:true,engineObservation:'not-callable'},()=>host().solve('What is 23 + 31?',[]));
 assert(answer.answer.includes('54'));
});

test('derivation capability containers preserve primitive refusals and callable/prototype IO',()=>{
 const record=recordFor([{kind:'render:emit',payload:'independent capability case'}],'answer_capability_case');
 for(const io of [null,undefined,false,0,41,'text',Symbol('io')]){
  assert.deepEqual(persist(record,'/scratch',io),{ok:false,error:'derivation_store_has_no_writer'});
  assert.equal(load('/scratch','answer_capability_case',io),null);
 }
 const calls=[];let stored='';const callable=function(){};
 callable.writeText=(file,text)=>{calls.push(['function-write',file]);stored=text;};
 callable.readText=file=>{calls.push(['function-read',file]);return stored;};
 assert.equal(persist(record,'/scratch',callable).ok,true);assert.deepEqual(load('/scratch','answer_capability_case',callable),record);
 assert.deepEqual(calls.map(row=>row[0]),['function-write','function-read']);
 const observed=[];const prototype={
  get writeText(){observed.push('writer-get');return function(file,text){assert.equal(this,container);observed.push('write');stored=text;};},
  get readText(){observed.push('reader-get');return function(){assert.equal(this,container);observed.push('read');return stored;};}
 };
 const container=Object.create(prototype);
 assert.equal(persist(record,'/scratch',container).ok,true);assert.deepEqual(load('/scratch','answer_capability_case',container),record);
 assert.deepEqual(observed,['writer-get','writer-get','write','reader-get','reader-get','read']);
 const refused=Object.create({get writeText(){return 7;},get readText(){return false;}});
 assert.deepEqual(persist(record,'/scratch',refused),{ok:false,error:'derivation_store_has_no_writer'});assert.equal(load('/scratch','answer_capability_case',refused),null);
 const thrown=new Error('held-out accessor failure');const throwing=Object.create({get writeText(){throw thrown;}});
 assert.throws(()=>persist(record,'/scratch',throwing),error=>error===thrown);
});
