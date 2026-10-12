import {lex} from '../../../scripts/self-translation/lexer.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {generateOptions,TEMPORARY_OPTION_WORKAROUND} from '../../../scripts/lib/native-option-registry.mjs';
import {tokenize,testFunctions} from '../../../scripts/lib/rust-specification-cases.mjs';
import {typedProgramOf,executeTypedProgram} from '../../../scripts/lib/rust-specification-programs.mjs';
import {evaluate,nativeEquality} from '../../../scripts/lib/rust-specification-values.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const root=resolve(import.meta.dirname,'../../..');
const sourceFor=(owner='Toggle',producer='label',parser='read_label',parameter='input')=>
 '#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)] pub enum '+owner+'{#[default] Off,On} impl '+owner+
 '{#[must_use] pub const fn '+producer+'(self)->&\'static str{match self{Self::Off=>"off",Self::On=>"on"}}'+
 '#[must_use] pub fn '+parser+'('+parameter+':&str)->Option<Self>{match '+parameter+'.trim().to_ascii_lowercase().as_str(){"off"=>Some(Self::Off),"on"=>Some(Self::On),_=>None}}}';
const compile=source=>generateOptions(source,'rust/src/fixture.rs','fixture','pub mod fixture;');
function dropProducerArm(source){
 const producer=compile(source).programs.find(program=>program.kind==='nativeEnumStringMatch');assert.ok(producer);
 const tokens=lex(source.slice(producer.sourceSpan.start,producer.sourceSpan.end),'Rust').filter(token=>token.type!=='comment');
 const functionAt=tokens.findIndex((token,index)=>token.text==='fn'&&tokens[index+1]?.text===producer.name);assert.ok(functionAt>=0);
 const matchAt=tokens.findIndex((token,index)=>index>functionAt&&token.text==='match');
 const armAt=tokens.findIndex((token,index)=>index>matchAt&&token.text==='Self'&&tokens[index+1]?.text==='::');
 const valueAt=armAt+4;assert.equal(tokens[armAt+3]?.text,'=>');assert.equal(tokens[valueAt]?.type,'string');
 const start=producer.sourceSpan.start+tokens[armAt].start;
 const end=producer.sourceSpan.start+(tokens[valueAt+1]?.text===','?tokens[valueAt+1].end:tokens[valueAt].end);
 const changed=source.slice(0,start)+source.slice(end);assert.notEqual(changed,source);return changed;
}
const parsed=body=>{const source='use formal_ai::meta_construction::RecursionMode; #[test] fn check(){'+body+'}';return typedProgramOf(testFunctions(tokenize(source))[0].body,{source,root});};
test('four unchanged original native case bodies preserve every assertion and loop',async()=>{
 const worker=new WorkerHost();let count=0;
 for(const namespace of ['meta_construction','meta_self_improvement','selection','skill_ledger']){
  const file=root+'/rust/tests/unit/specification/'+namespace+'.rs',source=readFileSync(file,'utf8');
  const body=testFunctions(tokenize(source)).find(item=>item.name==='modes_round_trip_through_their_slugs').body;
  const result=typedProgramOf(body,{source,file,root});assert.ok(result.program,result.reason);assert.equal(result.program.nativeAssertions,3);
  const actual=await executeTypedProgram(worker,result.program);assert.equal(actual.status,'passed',actual.failure);count+=actual.assertions;
 }
 assert.equal(count,17);
});
test('renamed enum methods and parameters compile finite producer/parser inverses',async()=>{
 for(const source of [sourceFor(),sourceFor('Different','describe','decode','text')]){
  const generated=compile(source).programs,parser=generated.find(item=>item.kind==='nativeOptionStringParse');
  assert.ok(parser);assert.equal(generated.filter(item=>item.kind==='nativeEnumStringMatch').length,1);
  for(const whitespace of ['',' ','\t','\n','\v','\f','\r','\t\v\f\r\n ']){
   const some=await evaluate({kind:'call',binding:parser,args:[{kind:'literal',value:whitespace+'ON'+whitespace}]},new Map(),{observations:[]});
   const other=await evaluate({kind:'call',binding:parser,args:[{kind:'literal',value:'on'}]},new Map(),{observations:[]});
   assert.equal(nativeEquality(some,other,parser.returnType),true);
  }
 }
});
test('private Option equality separates variants, None and nominal foreign enum payloads',async()=>{
 const generated=compile(sourceFor()).programs,parser=generated.find(item=>item.kind==='nativeOptionStringParse');
 const run=value=>evaluate({kind:'call',binding:parser,args:[{kind:'literal',value}]},new Map(),{observations:[]});
 const on=await run('on'),off=await run('off'),none=await run('absent');
 assert.equal(nativeEquality(on,off,parser.returnType),false);assert.equal(nativeEquality(on,none,parser.returnType),false);
 assert.equal(nativeEquality(none,await run('absent'),parser.returnType),true);
 for(const spoof of [null,undefined,{},Object.create(null),{Some:'On'},{nativeEnum:parser.enumPath,variant:'On'}])assert.throws(()=>nativeEquality(on,spoof,parser.returnType),/unowned/);
 const foreign=compile(sourceFor('Other')).programs.find(item=>item.kind==='nativeOptionStringParse');
 const value=await evaluate({kind:'call',binding:foreign,args:[{kind:'literal',value:'on'}]},new Map(),{observations:[]});
 assert.throws(()=>nativeEquality(on,value,parser.returnType),/foreign/);
});
test('non-ASCII and malformed UTF16 inputs refuse statically and at runtime',async()=>{
 const parser=compile(sourceFor()).programs.find(item=>item.kind==='nativeOptionStringParse');
 for(const value of ['\u00a0on','\u2003on','\ufeffon','ÖN','on\ud800','on\udfff']){
  await assert.rejects(()=>evaluate({kind:'call',binding:parser,args:[{kind:'literal',value}]},new Map(),{observations:[]}),/non-ASCII/);
  const result=parsed('assert_eq!(RecursionMode::from_slug('+JSON.stringify(value)+'),Some(RecursionMode::Up));');assert.ok(!result.program);
 }
});
test('source enum ownership, arm coverage, constructor namespaces and effects refuse',()=>{
 const original=sourceFor();
 const refused=source=>{try{return !compile(source).programs.some(item=>item.nativeOptionQualified);}catch{return true;}};
 for(const source of [original.replace(',Copy',''),original.replace(',PartialEq',''),original.replace('Clone,Copy','Clone,"Copy"'),
  original.replace('Self::On=>"on"','Self::Off=>"on"'),dropProducerArm(original),
  original.replace('"on"=>Some(Self::On)','"off"=>Some(Self::On)'),original.replace('_=>None','_=>Some(Self::Missing)'),
  original.replace('_=>None','_=>effect()'),original.replace('Option<Self>','Foreign<Self>'),
  original.replace('input.trim()','effect().trim()'),original.replace('Self::On),_=>None','Self::On),_=>None};effect()'),
  'use custom::Option; '+original,'use custom::Thing as Some; '+original,'use custom::*; '+original,
  original+' impl Toggle{pub fn read_label(input:&str)->Option<Self>{None}}']){assert.notEqual(source,original,'negative control must change source');assert.ok(refused(source),source);}
 assert.equal(TEMPORARY_OPTION_WORKAROUND.upstreamIssue,null);assert.match(TEMPORARY_OPTION_WORKAROUND.blocker,/carried/);
});
test('full connected reader refuses borrowed, unknown, collection and shadowed Option domains',()=>{
 for(const body of ['assert_eq!(Some(&RecursionMode::Up),Some(RecursionMode::Up));',
  'assert_eq!(Some(1),Some(1));','assert_eq!(None,None);',
  'assert_eq!([Some(RecursionMode::Up)],[None]);','assert_eq!((Some(RecursionMode::Up),),(None,));',
  'let Some=1; assert_eq!(RecursionMode::from_slug("up"),Some(RecursionMode::Up));',
  'let None=1; assert_eq!(RecursionMode::from_slug("unknown"),None);',
  'let input="up".to_owned(); assert_eq!(RecursionMode::from_slug(input),Some(RecursionMode::Up));',
  'for mode in &[RecursionMode::Up]{assert_eq!(RecursionMode::from_slug("up"),Some(mode));}',
  'assert!(RecursionMode::from_slug("up").is_some());'])assert.ok(!parsed(body).program,body);
});
test('source public module identity cannot be replaced by alias or quoted namespace syntax',()=>{
 for(const publicRoot of ['pub mod fixture; mod fixture{}','pub mod fixture; use foreign::Thing as fixture;',
  '"pub" mod fixture;','pub "mod" fixture;','pub mod "fixture";','#![no_implicit_prelude] pub mod fixture;']){
  assert.throws(()=>generateOptions(sourceFor(),'rust/src/fixture.rs','fixture',publicRoot));
 }
});

test('public imports and inner attributes cannot counterfeit builtin Option prelude',()=>{
 for(const changed of [sourceFor('_'),sourceFor('Toggle','_'),sourceFor('Toggle','label','_')])assert.throws(()=>compile(changed));
 assert.ok(!compile(sourceFor('Toggle','label','read_label','_')).programs.some(program=>program.nativeOptionQualified));

 for(const prefix of ['pub use custom::Option;','pub use custom::Choice as Some;','pub use custom::Choice as None;',
  '#[cfg(feature="unknown")] use custom::Option;','#![no_implicit_prelude]','#![cfg(feature="unknown")]']){
  assert.throws(()=>compile(prefix+sourceFor()),/Option import scope|owned import attribute|inner attribute/);
 }
 for(const prefix of ['pub use formal_ai::RecursionMode as Some;','pub use formal_ai::RecursionMode as None;',
  '#![no_implicit_prelude]','#![cfg(feature="unknown")]']){
  const source=prefix+' use formal_ai::meta_construction::RecursionMode; #[test] fn check(){assert_eq!(RecursionMode::from_slug("up"),Some(RecursionMode::Up));}';
  const result=typedProgramOf(testFunctions(tokenize(source)).find(item=>item.name==='check').body,{source,root});assert.ok(!result.program,prefix);
 }
});
