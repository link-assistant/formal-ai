import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {generateScalars,generateEnumPredicates,TEMPORARY_ENUM_WORKAROUND} from '../../../scripts/lib/native-scalar-registry.mjs';
import {GENERATED_NATIVE_PROGRAMS} from '../../../scripts/lib/generated-native-programs.mjs';
import {tokenize,testFunctions} from '../../../scripts/lib/rust-specification-cases.mjs';
import {typedProgramOf,executeTypedProgram} from '../../../scripts/lib/rust-specification-programs.mjs';
import {evaluate} from '../../../scripts/lib/rust-specification-values.mjs';
import {selfTranslate} from '../../../scripts/self-translation/envelope.mjs';
import {lex,topLevelItems} from '../../../scripts/self-translation/lexer.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const root=resolve(import.meta.dirname,'../../..'),host=new WorkerHost();
const sourceFor=(name='Switch',method='enabled',first='Quiet',second='Active')=>
 `#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)] pub enum ${name}{${first},#[default] ${second}} impl ${name}{#[must_use] pub const fn ${method}(self)->bool{matches!(self,Self::${second})}}`;
const compile=source=>generateScalars(source,'rust/src/fixture.rs','fixture','pub mod fixture;');
function inline(source){return typedProgramOf(testFunctions(tokenize(source))[0].body,{source,root});}
test('source enum defaults and predicates execute six unchanged original programs',async()=>{
 const cases=[['meta_construction','both_directions_are_the_default_depth_floor',5],
  ['meta_construction','up_emits_only_the_upward_direction_and_both_emits_both',4],
  ['selection','record_is_the_default_and_off_emits_no_artifact',3],['selection','record_emits_an_artifact',1],
  ['meta_self_improvement','propose_is_the_default_and_off_proposes_nothing',3],
  ['skill_ledger','accumulate_is_the_default_and_off_records_nothing',3]];
 for(const [file,name,count] of cases){const path='rust/tests/unit/specification/'+file+'.rs',source=readFileSync(root+'/'+path,'utf8');
  const body=testFunctions(tokenize(source)).find(item=>item.name===name).body;
  const parsed=typedProgramOf(body,{source,file:root+'/'+path,root});assert.ok(parsed.program,parsed.reason);
  assert.equal(parsed.program.nativeAssertions,count);const actual=await executeTypedProgram(host,parsed.program);
  assert.equal(actual.status,'passed',actual.failure);assert.equal(actual.assertions,count);
 }
});
test('renamed source AST supplies private defaults and exact matches disjunctions',async()=>{
 for(const source of [sourceFor(),sourceFor('Alternate','running','Stopped','Running')]){
  const generated=compile(source).programs,constructor=generated.find(item=>item.kind==='nativeEnumDefault');
  const method=generated.find(item=>item.kind==='nativeEnumScalarMatch');assert.ok(constructor&&method);
  const actual=await evaluate({kind:'call',binding:method,args:[{kind:'call',binding:constructor,args:[]}]},new Map(),{observations:[]});
  assert.equal(actual,true);
  await assert.rejects(()=>evaluate({kind:'call',binding:method,args:[{kind:'literal',value:{nativeEnum:constructor.enumPath,variant:constructor.variant}}]},new Map(),{observations:[]}),/unowned/);
 }
 const union=compile(sourceFor().replace('Self::Active)','Self::Quiet | Self::Active)')).programs;
 assert.deepEqual(union.find(item=>item.kind==='nativeEnumScalarMatch').cases.map(item=>item.value),[true,true]);
});
test('unknown default derivation and matches hygiene refuse conservatively',()=>{
 const source=sourceFor();
 for(const changed of [source.replace(',Default',''),source.replace('#[default]',''),source.replace('Quiet,','#[default] Quiet,'),
  source.replace('Default','CustomDefault'),source.replace('Quiet,','Quiet(u8),'),source.replace('Quiet,','Quiet = 3,'),
  source.replace('pub enum','#[custom] pub enum')])assert.ok(!compile(changed).programs.some(item=>item.kind==='nativeEnumDefault'));
 for(const changed of [source.replace('Self::Active)','Self::Unknown)'),source.replace('Self::Active)','_ )'),
  source.replace('Self::Active)','Self::Active | Self::Active)'),source.replace('matches!','unknown!'),
  source.replace('matches!','effect(); matches!'),source.replace('matches!(self','matches!(other')])
  assert.ok(!compile(changed).programs.some(item=>item.kind==='nativeEnumScalarMatch'));
 for(const changed of ['macro_rules! matches {()=>{true}} '+source,'use foreign::matches; '+source,
  'use foreign::Default; '+source,source+' impl Custom for Switch {}',source+' impl Switch {fn default()->Self{Self::Quiet}}',
  source+' #[cfg(test)] impl Switch {fn default()->Self{Self::Quiet}}'])assert.throws(()=>compile(changed));
});
test('source import DAG, aliases, shadows and fixture drift are enforced',async()=>{
 const prefix='use formal_ai::selection::SelectionMode as Gate; use formal_ai::meta_frame::WorkUnit;';
 const valid=inline(prefix+' #[test] fn renamed(){assert!(Gate::default().emits_artifact());}');assert.ok(valid.program,valid.reason);
 assert.ok(valid.program.fixtures.some(item=>item.file?.endsWith('/rust/src/meta_frame.rs')));
 assert.equal((await executeTypedProgram(host,valid.program)).status,'passed');
 for(const source of [prefix+' use unknown::Thing; #[test] fn x(){assert!(Gate::default().emits_artifact());}',
  prefix+' type Gate=Foreign; #[test] fn x(){assert!(Gate::default().emits_artifact());}',
  prefix+' #[test] fn x(){let Gate="shadow";assert!(Gate::default().emits_artifact());}',
  'use formal_ai::SelectionMode; #[test] fn x(){assert!(SelectionMode::default().emits_artifact());}',
  prefix+' macro_rules! custom {()=>{true}} #[test] fn x(){assert!(Gate::default().emits_artifact());}',
  prefix+' fn Gate()->bool{true} #[test] fn x(){assert!(Gate::default().emits_artifact());}',
  prefix+' const Gate:u32=1; #[test] fn x(){assert!(Gate::default().emits_artifact());}',
  prefix+' #[test] fn x(Gate:u32){assert!(Gate::default().emits_artifact());}',
  'mod formal_ai{} #[test] fn x(){assert!(formal_ai::selection::SelectionMode::default().emits_artifact());}',
  prefix+' use formal_ai::world_model_dialog::WorldModelMode as Gate; #[test] fn x(){assert!(Gate::default().emits_artifact());}'])assert.equal(inline(source).program,undefined);
 const drift=structuredClone(valid.program);drift.fixtures.find(item=>item.file?.endsWith('/rust/src/meta_frame.rs')).sha256='0'.repeat(64);
 const result=await executeTypedProgram(host,drift);assert.equal(result.status,'failed');assert.equal(result.observations.length,0);
 const wrong=inline(prefix+' #[test] fn x(){assert!(!Gate::default().emits_artifact());}');
 assert.equal((await executeTypedProgram(host,wrong.program)).status,'failed');
});
test('maintained translation blockers and deterministic source registry stay governed',()=>{
 const generated=generateEnumPredicates(root);assert.ok(generated.programs.some(item=>item.path==='formal_ai::selection::SelectionMode::default'));
 for(const namespace of ['meta_construction','selection','meta_self_improvement','skill_ledger']){
  const source=readFileSync(root+'/rust/src/'+namespace+'.rs','utf8');
  const compiled=generated.programs.filter(item=>item.path.startsWith('formal_ai::'+namespace+'::'));
  assert.ok(compiled.some(item=>item.kind==='nativeEnumDefault'));
  for(const binding of compiled){assert.ok(GENERATED_NATIVE_PROGRAMS.some(item=>JSON.stringify(item)===JSON.stringify(binding)));
   const actual=source.slice(binding.sourceSpan.start,binding.sourceSpan.end),translation=selfTranslate(actual,'Rust','JavaScript');
   assert.ok(JSON.stringify(translation).includes('a Rust item outside portable-pure-v1'));
  }
 }
 assert.equal(TEMPORARY_ENUM_WORKAROUND.upstreamIssue,null);
 assert.match(TEMPORARY_ENUM_WORKAROUND.retirement,/executable maintained twins/);
});

test('ambiguous owned enum and method declarations mint no programs',()=>{
 const source=sourceFor();
 for(const changed of [source+source,source+' impl Switch {pub const fn enabled(self)->bool{matches!(self,Self::Quiet)}}',
 '#[custom] pub enum Switch{Unknown} '+source,source+' impl Default for Switch{fn default()->Self{Self::Quiet}}'])
  assert.throws(()=>compile(changed));
 for(const changed of [source.replace('Default)','Default,Default)'),source.replace('#[default]','#[default] #[default]')])
  assert.ok(!compile(changed).programs.some(item=>item.kind==='nativeEnumDefault'));
});

test('non-Copy enum reuse and shadowed ownership derives remain refused',()=>{
 const source=sourceFor();
 for(const changed of [source.replace(',Copy',''),source.replace('Clone,',''),source.replace('PartialEq,',''),
  source.replace('Quiet,','Copy,' ).replace(',Copy','')]){
  const generated=compile(changed).programs;
  assert.ok(!generated.some(item=>item.kind==='nativeEnumVariant'||item.kind==='nativeEnumDefault'||item.kind==='nativeEnumScalarMatch'));
 }
 for(const name of ['Clone','Copy','PartialEq','Eq','Debug','Default'])assert.throws(()=>compile('use custom::'+name+'; '+source));
});


test('native derive names and boolean literals require identifier tokens',()=>{
 const source=sourceFor();
 for(const name of ['Debug','Clone','Copy','PartialEq','Eq','Default']){
  const generated=compile(source.replace(name,JSON.stringify(name)));
  assert.equal(generated.programs.length,0);
 }
 for(const literal of ['"true"','"false"','r#true','r#false','NaN','-0','9007199254740993']){
  assert.equal(compile('pub const FLAG:bool='+literal+';').programs.length,0);
 }
 for(const literal of ['"1"','NaN','-0','9007199254740993'])
  assert.equal(compile('pub const COUNT:usize='+literal+';').programs.length,0);
 assert.equal(compile('pub const FLAG:bool=true;').programs[0].value,true);
 assert.equal(compile('pub const FLAG:bool=false;').programs[0].value,false);
 assert.ok(compile(source).programs.length>0);
});
