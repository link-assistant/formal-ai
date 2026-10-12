import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {generateRecords,TEMPORARY_RECORD_WORKAROUND} from '../../../scripts/lib/native-record-registry.mjs';
import {tokenize,testFunctions} from '../../../scripts/lib/rust-specification-cases.mjs';
import {typedProgramOf,executeTypedProgram} from '../../../scripts/lib/rust-specification-programs.mjs';
import {evaluate} from '../../../scripts/lib/rust-specification-values.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const root=resolve(import.meta.dirname,'../../..');
const primitiveSource=(name='Counter',method='ready')=>`#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)] pub struct ${name}{pub first:usize,pub second:usize} impl ${name}{#[must_use] pub const fn ${method}(&self)->bool{self.first > 0 && self.first == self.second}}`;
const compile=source=>generateRecords(source,'rust/src/fixture.rs','fixture','pub mod fixture;');
test('pure borrowed record method runs the entire unchanged original native case',async()=>{
 const file=root+'/rust/tests/unit/specification/selection_heuristics.rs',source=readFileSync(file,'utf8');
 const body=testFunctions(tokenize(source)).find(item=>item.name==='a_zero_zero_check_count_is_not_a_pass').body;
 const parsed=typedProgramOf(body,{source,file,root});assert.ok(parsed.program,parsed.reason);assert.equal(parsed.program.nativeAssertions,1);
 const result=await executeTypedProgram(new WorkerHost(),parsed.program);assert.equal(result.status,'passed',result.failure);assert.equal(result.assertions,1);
});
test('renamed struct fields and borrow AST retain generic primitive semantics',async()=>{
 for(const source of [primitiveSource(),primitiveSource('Renamed','works')]){
  const programs=compile(source).programs,constructor=programs.find(item=>item.kind==='nativeRecordConstructor');
  const method=programs.find(item=>item.kind==='nativeRecordBorrow');assert.ok(constructor&&method);
  for(const [first,second,expected]of [[0,0,false],[1,1,true],[1,2,false],[2,1,false]]){
   const owned={kind:'record',binding:constructor,entries:[['first',{kind:'literal',value:first}],['second',{kind:'literal',value:second}]]};
   const result=await evaluate({kind:'call',binding:method,args:[owned]},new Map(),{observations:[]});assert.equal(result,expected);
  }
 }
 const programs=compile(primitiveSource()).programs,defaults=programs.find(item=>item.kind==='nativeRecordDefault');
 const value=await evaluate({kind:'call',binding:defaults,args:[]},new Map(),{observations:[]});assert.deepEqual(value,{first:0,second:0});
});
test('unknown schema, traits, effects, default provenance and ownership refuse',async()=>{
 const source=primitiveSource();
 for(const changed of [source.replace('&self','self'),source.replace('self.first > 0','effect()'),source.replace('usize','Unknown'),source.replace('self.first > 0','self.first > 65536')])
  assert.ok(!compile(changed).programs.some(item=>item.kind==='nativeRecordBorrow'));
 for(const changed of [source+source,'use custom::Default; '+source,'use custom::Copy; '+source,
  source+' impl Default for Counter {}',source+' #[cfg(test)] impl Default for Counter {}',
  source+' impl Counter{fn default()->Self{effect()}}',source+' impl Counter{pub const fn ready(&self)->bool{true}}'])assert.throws(()=>compile(changed));
 assert.ok(!compile(source.replace(',Default','')).programs.some(item=>item.kind==='nativeRecordDefault'));
 const programs=compile(source).programs,constructor=programs.find(item=>item.kind==='nativeRecordConstructor'),method=programs.find(item=>item.kind==='nativeRecordBorrow');
 await assert.rejects(()=>evaluate({kind:'call',binding:method,args:[{kind:'literal',value:{first:0,second:0}}]},new Map(),{observations:[]}),/unowned/);
 await assert.rejects(()=>evaluate({kind:'record',binding:constructor,entries:[['first',{kind:'literal',value:65536}],['second',{kind:'literal',value:0}]]},new Map(),{observations:[]}),/range/);
 assert.equal(TEMPORARY_RECORD_WORKAROUND.upstreamIssue,null);assert.match(TEMPORARY_RECORD_WORKAROUND.blocker,/unsupported/);
});
test('actual imported record identity rejects aliases, moves, field escapes and custom equality',()=>{
 const prefix='use formal_ai::selection_heuristics::{CandidateScore,ActionCost};';
 const declaration='let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(0,0),cost:ActionCost::default()};';
 for(const body of ['let moved=score; assert!(!score.satisfies());','assert_eq!(score,score);',
  'let moved=score.candidate_id; assert!(!score.satisfies());','let CandidateScore=1; '+declaration+'assert!(!score.satisfies());']){
  const source=prefix+' #[test] fn refusal(){'+(body.includes('let CandidateScore')?'':declaration)+body+'}';
  const parsed=typedProgramOf(testFunctions(tokenize(source))[0].body,{source,root});assert.ok(!parsed.program);
 }
});
test('indented ASCII Rust string continuations match tokenizer decoding conservatively',async()=>{
 const {nativeStringContract}=await import('../../../scripts/lib/native-string-contract.mjs');
 const slash=String.fromCharCode(92);
 for(const whitespace of [' ','\t','\n  ',' \t\n\n  ']){
  const literal='"head'+slash+'\n'+whitespace+'tail"';
  nativeStringContract(literal);assert.deepEqual(tokenize(literal),[{kind:'string',text:'headtail'}]);
 }
 for(const literal of ['"head'+slash+'\n'+'tail"','"head'+slash+'\r\n  tail"',
  '"head'+slash+'\n  '+String.fromCodePoint(0xa0)+'tail"','"head'+slash+'\n  "',
  '"head'+slash+'\n  ','"head'+slash+'u{'])assert.throws(()=>nativeStringContract(literal));
 const escaped='"head'+slash+slash+'\n tail"';nativeStringContract(escaped);
 assert.deepEqual(tokenize(escaped),[{kind:'string',text:'head'+slash+'\n tail'}]);
});

test('unused imports retain full scope witnesses while executed filesystem and root shadows refuse',()=>{
 const prefix='use formal_ai::selection_heuristics::{CandidateScore,ActionCost};';
 const declaration='let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(0,0),cost:ActionCost::default()};';
 function parsed(header,body='assert!(!score.satisfies());'){
  const source=header+' '+prefix+' #[test] fn check(){'+declaration+body+'}';
  return typedProgramOf(testFunctions(tokenize(source)).find(item=>item.name==='check').body,{source,root});
 }
 const pure=parsed('use std::fs; use std::path::{Path,PathBuf};');assert.ok(pure.program,pure.reason);
 assert.ok(pure.program.fixtures.some(item=>item.source?.includes('use std::fs;')));
 for(const header of ['use std::fs as formal_ai;','use std::fs as std;','use std::path::*;',
  'mod std {} use std::fs;','static EFFECT:usize=effect();','include!("unknown.rs");',
  '#[unknown] fn initializer(){effect();}','use std::path::Path as CandidateScore;',
  'use custom::Thing;'])assert.ok(!parsed(header).program);
 assert.ok(!parsed('use std::fs;','let source=fs::read_to_string("unknown");assert!(!score.satisfies());').program);
});

test('borrowed primitive typing and record collection moves refuse without coercion',()=>{
 const sample=primitiveSource();
 for(const changed of [sample.replace('pub first:usize','pub first:u8'),
  sample.replaceAll('usize','u8').replace('self.first > 0','self.first > 256'),
  'use custom::usize; '+sample])assert.ok(!compile(changed).programs.some(item=>item.kind==='nativeRecordBorrow'));
 const prefix='use formal_ai::selection_heuristics::{CandidateScore,ActionCost};';
 const declaration='let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(0,0),cost:ActionCost::default()};';
 for(const collection of ['[score]','(score,score)']){
  const source=prefix+' #[test] fn refusal(){'+declaration+' let values='+collection+'; assert!(!score.satisfies());}';
  assert.ok(!typedProgramOf(testFunctions(tokenize(source))[0].body,{source,root}).program);
 }
});


test('owned record AST token kinds and logical precedence stay native',async()=>{
 const source=primitiveSource();
 for(const text of ['Copy','Clone','PartialEq','Default','usize','pub','struct','self','first','bool']){
  const changed=source.replace(text,JSON.stringify(text));
  assert.ok(!compile(changed).programs.some(item=>item.kind==='nativeRecordBorrow'));
 }
 for(const body of ['"true"',"'t'",'r#true','self.first > 9007199254740993','false == false == true']){
  const changed=source.replace('self.first > 0 && self.first == self.second',body);
  assert.ok(!compile(changed).programs.some(item=>item.kind==='nativeRecordBorrow'));
 }
 const programs=compile(source.replace('self.first > 0 && self.first == self.second','true || false && false')).programs;
 const constructor=programs.find(item=>item.kind==='nativeRecordConstructor'),method=programs.find(item=>item.kind==='nativeRecordBorrow');
 const owner={kind:'record',binding:constructor,entries:[['first',{kind:'literal',value:0}],['second',{kind:'literal',value:0}]]};
 assert.equal(await evaluate({kind:'call',binding:method,args:[owner]},new Map(),{observations:[]}),true);
 for(const value of [-0,NaN,9007199254740993])await assert.rejects(()=>evaluate({kind:'record',binding:constructor,
  entries:[['first',{kind:'literal',value}],['second',{kind:'literal',value:0}]]},new Map(),{observations:[]}));
});

test('unused imported nominal declarations are source proofs rather than trait behavior',()=>{
 const prefix='use formal_ai::selection_heuristics::{CandidateScore,ActionCost};';
 const declaration='let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(0,0),cost:ActionCost::default()};';
 function parsed(header,tail='assert!(!score.satisfies());'){
  const source=header+' '+prefix+' #[test] fn probe(){'+declaration+tail+'}';
  return typedProgramOf(testFunctions(tokenize(source)).find(item=>item.name==='probe').body,{source,root});
 }
 const header='use formal_ai::selection_heuristics::CandidateRanker as UnusedTrait;';
 const observed=parsed(header);assert.ok(observed.program,observed.reason);
 assert.ok(observed.program.fixtures.some(item=>item.source?.includes(header)));
 for(const unknown of ['use formal_ai::selection_heuristics::Missing;',
  'use formal_ai::missing::Thing;','use formal_ai::CandidateRanker;',
  'use formal_ai::selection_heuristics::CandidateRanker as CandidateScore;',
  'use formal_ai::selection_heuristics::*;',
  'use formal_ai::selection_heuristics::"CandidateRanker";'])assert.ok(!parsed(unknown).program);
 assert.ok(!parsed(header,'let ranker=UnusedTrait::default();assert!(!score.satisfies());').program);
});


test('unknown producer module attributes, initialization and builtin namespace aliases refuse',()=>{
 const source=primitiveSource();
 for(const prefix of ['#[proc] fn unrelated(){} ', '#[macro_use] mod child; ',
  'static UNKNOWN:usize=effect(); ', 'expanded!(); ', 'use custom::Thing as alloc; ',
  'extern crate custom as alloc; ', 'use custom::core; '])assert.throws(()=>compile(prefix+source));
});


test('native record fields with unsupported JavaScript prototype representation refuse',()=>{
 const source='#[derive(Clone,Copy,Default)] pub struct Counter{pub first:usize,pub __proto__:usize} impl Counter{pub const fn ready(&self)->bool{self.__proto__ == 0}}';
 assert.equal(compile(source).programs.length,0);
 for(const name of ['constructor','toString','__proto__']){
  assert.ok(!compile(primitiveSource().replace('self.first > 0','self.'+name+' > 0')).programs.some(item=>item.kind==='nativeRecordBorrow'));
  const caller='use formal_ai::selection_heuristics::{CandidateScore,ActionCost}; #[test] fn probe(){'+
   'let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(0,0),cost:ActionCost::default()}; assert_eq!(score.'+name+',0);}';
  assert.ok(!typedProgramOf(testFunctions(tokenize(caller))[0].body,{source:caller,root}).program);
 }
});


test('native record initializer ownership and integer types follow source AST provenance',async()=>{
 const prefix='use formal_ai::selection_heuristics::{CandidateScore,ActionCost}; use formal_ai::summarization::DEFAULT_MAX_STATEMENTS;';
 function parsed(declaration,check='assert!(!score.satisfies());',extra=''){
  const source=extra+prefix+' #[test] fn probe(){'+declaration+check+'}';
  return typedProgramOf(testFunctions(tokenize(source)).find(item=>item.name==='probe').body,{source,root});
 }
 const score=value=>'let score=CandidateScore{candidate_id:'+value+',checks:(0,0),cost:ActionCost::default()};';
 for(const value of ['"draft"','"draft".clone()'])assert.ok(!parsed(score(value)).program);
 for(const declaration of ['let label="draft".to_owned();'+score('label'),
  'let borrowed="draft";'+score('borrowed.to_owned()'),
  'let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(false,0),cost:ActionCost::default()};',
  'let cost=ActionCost{steps:DEFAULT_MAX_STATEMENTS,code_size:0,resource_units:0,leaf_count:0};',
  'let steps=0;let cost=ActionCost{steps:steps,code_size:0,resource_units:0,leaf_count:0};',
  'let cost=ActionCost{steps:65536,code_size:0,resource_units:0,leaf_count:0};'])assert.ok(!parsed(declaration,'assert!(true);').program);
 for(const extra of ['trait Foreign{fn to_owned(&self)->String;}',
  'impl Foreign for &str{fn to_owned(&self)->String{unknown()}}',
  'use formal_ai::selection_heuristics::CandidateRanker as ToOwned;'])assert.ok(!parsed(score('"draft".to_owned()'),undefined,extra).program);
 for(const value of ['"draft".to_owned()','"draft".to_string()']){
  const accepted=parsed(score(value));assert.ok(accepted.program,accepted.reason);
  assert.equal((await executeTypedProgram(new WorkerHost(),accepted.program)).status,'passed');
 }
 const exact=parsed('let cost=ActionCost{steps:0,code_size:DEFAULT_MAX_STATEMENTS,resource_units:0,leaf_count:0};','assert_eq!(cost.code_size,30);');
 assert.ok(exact.program,exact.reason);assert.equal((await executeTypedProgram(new WorkerHost(),exact.program)).status,'passed');
});


import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

test('imported trait dispatch closure refuses before any generated-source observation',()=>{
 const temporary=mkdtempSync(join(tmpdir(),'record-dispatch-'));
 try{
  mkdirSync(temporary+'/rust/src',{recursive:true});
  writeFileSync(temporary+'/rust/src/lib.rs',readFileSync(root+'/rust/src/lib.rs'));
  const producer=readFileSync(root+'/rust/src/selection_heuristics.rs','utf8');
  const source='use formal_ai::selection_heuristics::{CandidateScore,ActionCost,Bridge}; #[test] fn probe(){'+
   'let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(0,0),cost:ActionCost::default()};assert!(!score.satisfies());}';
  for(const trait of ['pub trait Bridge: Missing {}','pub trait Bridge<T>{}',
   'pub trait Bridge{fn to_owned(&self)->String;}','pub trait Bridge{fn to_string(&self)->String;}',
   'pub trait Bridge{expanded!();}','#[derive(Clone)] pub trait Bridge{}',
   'pub trait Bridge{#[proc] fn rank(&self);}','pub trait Bridge{"fn" rank(&self);}']){
   writeFileSync(temporary+'/rust/src/selection_heuristics.rs',producer+'\n'+trait);
   const parsed=typedProgramOf(testFunctions(tokenize(source))[0].body,{source,root:temporary});
   assert.equal(parsed.program,undefined);assert.match(parsed.reason,/unknown imported trait dispatch closure/);
  }
  const nested='use formal_ai::selection_heuristics::{CandidateScore,ActionCost}; fn unused(){impl Unknown for &str{}} '+
   '#[test] fn probe(){let score=CandidateScore{candidate_id:"draft".to_owned(),checks:(0,0),cost:ActionCost::default()};assert!(!score.satisfies());}';
  const parsed=typedProgramOf(testFunctions(tokenize(nested)).find(item=>item.name==='probe').body,{source:nested,root});
  assert.equal(parsed.program,undefined);assert.match(parsed.reason,/unknown caller trait or implementation dispatch/);
 }finally{rmSync(temporary,{recursive:true,force:true});}
});


test('native record public root namespace and alloc identities are unique lexical declarations',()=>{
 const source=readFileSync(root+'/rust/src/selection_heuristics.rs','utf8');
 const good='extern crate alloc; pub mod selection_heuristics;';
 function assertRefused(rootSource){
  let result;
  try{
   result=generateRecords(source,'rust/src/selection_heuristics.rs','selection_heuristics',rootSource);
  }catch(error){
   assert.match(error.message,/ambiguous or unknown root namespace binding|unsupported owned record token kind|unknown root macro expansion|unknown root attribute effect/u);
   return;
  }
  assert.equal(result.programs.length,0,rootSource);
  assert.ok(result.refusals.length>0,rootSource);
  const baseline=generateRecords(source,'rust/src/selection_heuristics.rs','selection_heuristics',good).refusals;
  assert.ok(result.refusals.some(item=>item.reason==='ambiguous or unknown root namespace binding'),rootSource);
  for(const item of result.refusals){
   if(item.reason==='ambiguous or unknown root namespace binding')continue;
   assert.ok(baseline.some(prior=>prior.name===item.name&&prior.reason===item.reason),JSON.stringify(item));
  }
  assert.equal(new Map(result.programs.map(item=>[item.path,item])).size,0);
  const temporary=mkdtempSync(join(tmpdir(),'record-root-refusal-'));
  try{
   mkdirSync(temporary+'/rust/src',{recursive:true});
   writeFileSync(temporary+'/rust/src/lib.rs',rootSource);
   writeFileSync(temporary+'/rust/src/selection_heuristics.rs',source);
   const file=root+'/rust/tests/unit/specification/selection_heuristics.rs';
   const original=readFileSync(file,'utf8');
   const body=testFunctions(tokenize(original)).find(item=>item.name==='a_zero_zero_check_count_is_not_a_pass').body;
   assert.ok(!typedProgramOf(body,{source:original,file,root:temporary}).program,rootSource);
  }finally{rmSync(temporary,{recursive:true,force:true});}

 }
 assert.ok(generateRecords(source,'rust/src/selection_heuristics.rs','selection_heuristics',good).programs.length);
 for(const shadow of ['pub mod selection_heuristics;', 'type selection_heuristics=usize;',
  'pub use external as selection_heuristics;', 'pub use external::{Other as selection_heuristics};',
  'extern crate custom as alloc;', 'mod alloc{}', 'pub use external as alloc;',
  '"pub" mod selection_heuristics;', 'pub "mod" selection_heuristics;',
  'pub mod "selection_heuristics";', 'expanded!();', '#[proc] mod unrelated;']){
  assertRefused(good+shadow);
 }
 for(const conditional of ['#[cfg(unknown)] pub mod selection_heuristics;', '#[cfg(unknown)] extern crate alloc;']){
  const changed=conditional.includes('selection_heuristics')?'extern crate alloc; '+conditional:'pub mod selection_heuristics; '+conditional;
  assertRefused(changed);
 }
 assert.ok(generateRecords(source,'rust/src/selection_heuristics.rs','selection_heuristics',good+'mod nested{mod alloc{} mod selection_heuristics{}}').programs.length);
});


test('producer imports refuse wildcards, duplicate bindings and primitive or derive aliases',()=>{
 const source=primitiveSource();
 for(const declaration of ['use other::*;', 'use other::{Nested,*};',
  'use other::First as Repeated; use another::Second as Repeated;',
  'use other::{First as Repeated,Second as Repeated};',
  'use other::Type as u32;', 'use other::Type as usize;',
  'use other::Type as String;', 'use other::Trait as Copy;', 'use other::Trait as Default;']){
  let result;
  try{result=compile(declaration+source);}catch{continue;}
  assert.equal(result.programs.length,0,declaration);
 }
 assert.ok(compile('use other::First as One; use another::Second as Two;'+source).programs.length);
});
