import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {generateRecords} from '../../../scripts/lib/native-record-registry.mjs';
import {tokenize,testFunctions} from '../../../scripts/lib/rust-specification-cases.mjs';
import {typedProgramOf,executeTypedProgram} from '../../../scripts/lib/rust-specification-programs.mjs';
import {evaluate} from '../../../scripts/lib/rust-specification-values.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const root=resolve(import.meta.dirname,'../../..');
const sourceFor=(owner='Flags',method='ready',first='first',second='second')=>
 '#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)] pub struct '+owner+'{pub '+first+':bool,pub '+second+':bool} impl '+owner+
 '{#[must_use] pub const fn '+method+'(self)->bool{self.'+first+' && self.'+second+'}}';
const compile=source=>generateRecords(source,'rust/src/fixture.rs','fixture','pub mod fixture;');
test('complete unchanged original by-value record case executes all four assertions',async()=>{
 const file=root+'/rust/tests/unit/specification/skill_ledger.rs',source=readFileSync(file,'utf8');
 const body=testFunctions(tokenize(source)).find(item=>item.name==='a_proposed_skill_cannot_be_promoted_without_tests_and_a_benchmark_delta').body;
 const parsed=typedProgramOf(body,{source,file,root});assert.ok(parsed.program,parsed.reason);assert.equal(parsed.program.nativeAssertions,4);
 const result=await executeTypedProgram(new WorkerHost(),parsed.program);assert.equal(result.status,'passed',result.failure);assert.equal(result.assertions,4);
});
test('arbitrary Copy-owned record fields and methods retain exact bool matrix and pure defaults',async()=>{
 for(const source of [sourceFor(),sourceFor('Other','accepts','tested','measured')]){
  const programs=compile(source).programs,constructor=programs.find(item=>item.kind==='nativeRecordConstructor'),method=programs.find(item=>item.kind==='nativeRecordCopyRead');
  assert.ok(constructor&&method);assert.equal(constructor.copy,true);assert.equal(method.copy,true);
  const fields=Object.keys(constructor.fields);
  for(const [first,second,expected]of [[false,false,false],[true,false,false],[false,true,false],[true,true,true]]){
   const owned={kind:'record',binding:constructor,entries:[[fields[0],{kind:'literal',value:first}],[fields[1],{kind:'literal',value:second}]]};
   const value=await evaluate(owned,new Map(),{observations:[]}),environment=new Map([['value',value]]);
   for(let iteration=0;iteration<2;iteration++)assert.equal(await evaluate({kind:'call',binding:method,args:[{kind:'reference',name:'value'}]},environment,{observations:[]}),expected);
  }
  const defaults=programs.find(item=>item.kind==='nativeRecordDefault');assert.ok(defaults);
  const result=await evaluate({kind:'call',binding:method,args:[{kind:'call',binding:defaults,args:[]}]},new Map(),{observations:[]});assert.equal(result,false);
 }
});
test('non-Copy, Clone-only, custom derives, move fields and side effects cannot mint by-value readers',()=>{
 const original=sourceFor();
 for(const source of [original.replace(',Copy',''),original.replace('Clone,Copy','Clone'),original.replace('Copy','"Copy"'),
  original.replace('bool','String'),original.replace('self.first && self.second','effect()'),
  original.replace('(self)','(mut self)').replace('self.first && self.second','self.first=false;true'),
  '#[cfg(feature="unknown")] '+original]){
  let programs=[];try{programs=compile(source).programs;}catch{}
  assert.ok(!programs.some(item=>item.kind==='nativeRecordCopyRead'));
 }
 for(const source of ['use custom::Copy; '+original,original+' impl Copy for Flags{}',original+original])assert.throws(()=>compile(source));
});
test('runtime JSON spoof and erased Copy capability are refused despite matching primitive shape',async()=>{
 const programs=compile(sourceFor()).programs,constructor=programs.find(item=>item.kind==='nativeRecordConstructor'),method=programs.find(item=>item.kind==='nativeRecordCopyRead');
 await assert.rejects(()=>evaluate({kind:'call',binding:method,args:[{kind:'literal',value:{first:true,second:true}}]},new Map(),{observations:[]}),/unowned/);
 const weak={...constructor,copy:false},owned={kind:'record',binding:weak,entries:[['first',{kind:'literal',value:true}],['second',{kind:'literal',value:true}]]};
 await assert.rejects(()=>evaluate({kind:'call',binding:method,args:[owned]},new Map(),{observations:[]}),/Copy capability/);
});

import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {recordTestImportContract} from '../../../scripts/lib/native-record-registry.mjs';
function nominalFixture(run){
 const temporary=mkdtempSync(join(tmpdir(),'record-nominal-'));
 try{
  mkdirSync(temporary+'/rust/src/bridge',{recursive:true});
  const library='pub mod fixture;pub mod bridge;',native=sourceFor();
  writeFileSync(temporary+'/rust/src/lib.rs',library);writeFileSync(temporary+'/rust/src/fixture.rs',native);
  const binding=generateRecords(native,'rust/src/fixture.rs','fixture',library).programs.find(item=>item.kind==='nativeRecordCopyRead');
  const source='use formal_ai::fixture::Flags;use formal_ai::bridge::Unused;';
  const qualify=()=>recordTestImportContract(source,temporary,binding,tokenize('Flags'));
  run(temporary,qualify);
 }finally{rmSync(temporary,{recursive:true,force:true});}
}
test('unused declaration identities follow exact builtin attributes and renamed grouped public reexports',()=>{
 nominalFixture((temporary,qualify)=>{
  writeFileSync(temporary+'/rust/src/bridge.rs','#[must_use]pub fn Unused(){unknown_effect();}');
  assert.ok(qualify().some(item=>item.path==='rust/src/bridge.rs'));
  writeFileSync(temporary+'/rust/src/bridge.rs','pub mod inner;pub use inner::{Actual as Unused,Other};');
  writeFileSync(temporary+'/rust/src/bridge/inner.rs','#[must_use]pub fn Actual(){unknown_effect();}pub struct Other;');
  const witnesses=qualify();assert.ok(witnesses.some(item=>item.path==='rust/src/bridge/inner.rs'));
  assert.ok(witnesses.every(item=>item.sha256?.length===64));
 });
});
test('unused nominal reexport cycles, wildcards, aliases, lexical counterfeits and custom attributes refuse',()=>{
 nominalFixture((temporary,qualify)=>{
  writeFileSync(temporary+'/rust/src/bridge/inner.rs','#[must_use]pub fn Actual(){}pub fn Other(){}');
  for(const source of ['#[proc]pub fn Unused(){}','#["must_use"]pub fn Unused(){}',
   'pub "fn" Unused(){}','pub mod inner;pub use inner::*;',
   'pub mod inner;pub use inner::{Actual as Unused,Other as Unused};',
   'use custom::Thing as inner;pub mod inner;pub use inner::Actual as Unused;',
   '#[cfg(feature="unknown")]pub mod inner;pub use inner::Actual as Unused;',
   'pub "mod" inner;pub use inner::Actual as Unused;']){
   writeFileSync(temporary+'/rust/src/bridge.rs',source);assert.throws(qualify);
  }
  writeFileSync(temporary+'/rust/src/bridge.rs','pub mod inner;pub use inner::Unused;');
  writeFileSync(temporary+'/rust/src/bridge/inner.rs','pub use crate::bridge::Unused;');assert.throws(qualify,/cyclic/);
 });
});

test('nominal builtin attribute namespaces, malformed path groups and module effects refuse',()=>{
 nominalFixture((temporary,qualify)=>{
  writeFileSync(temporary+'/rust/src/bridge/inner.rs','pub fn Actual(){}');
  for(const source of ['use custom::must_use;#[must_use]pub fn Unused(){}',
   'use custom::Other as must_use;#[must_use]pub fn Unused(){}',
   'pub use custom::Other as must_use;#[must_use]pub fn Unused(){}',
   'use custom::{Other as must_use};#[must_use]pub fn Unused(){}',
   'pub mod inner;pub use inner:::{Actual as Unused};',
   'pub mod inner;pub use inner:{Actual as Unused};',
   'static EFFECT:usize=effect();pub fn Unused(){}',
   'expanded!();pub fn Unused(){}',
   '#![no_implicit_prelude]pub fn Unused(){}']){
   writeFileSync(temporary+'/rust/src/bridge.rs',source);assert.throws(qualify);
  }
 });
});
