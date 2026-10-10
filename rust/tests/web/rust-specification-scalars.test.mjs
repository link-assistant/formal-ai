import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {generateScalars,TEMPORARY_SCALAR_WORKAROUND} from '../../../scripts/lib/native-scalar-registry.mjs';
import {GENERATED_NATIVE_PROGRAMS} from '../../../scripts/lib/generated-native-programs.mjs';
import {tokenize,testFunctions} from '../../../scripts/lib/rust-specification-cases.mjs';
import {typedProgramOf,executeTypedProgram} from '../../../scripts/lib/rust-specification-programs.mjs';
import {evaluate} from '../../../scripts/lib/rust-specification-values.mjs';
import {selfTranslate} from '../../../scripts/self-translation/envelope.mjs';
import {lex,topLevelItems} from '../../../scripts/self-translation/lexer.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const root=resolve(import.meta.dirname,'../../..');
const host=new WorkerHost();
function inline(source) {
 return typedProgramOf(testFunctions(tokenize(source))[0].body,{source,root});
}
function producer(type='Choice',method='amount',first='Primary',second='Secondary') {
 return `pub const BOUND: usize = 30; #[derive(Clone,Copy,PartialEq,Eq)] pub enum ${type} { ${first}, ${second} } impl ${type} { pub const fn ${method}(self)->u32 { match self { Self::${first} => 7, Self::${second} => 11, } } }`;
}
const compile=source=>generateScalars(source,'rust/src/arbitrary/mod.rs','arbitrary','pub mod arbitrary;');
test('actual source constants and enum assertions execute unchanged',async()=> {
 const path='rust/tests/unit/specification/summarization_pipeline.rs',source=readFileSync(root+'/'+path,'utf8');
 for(const [name,assertions] of [['default_max_statements_is_thirty',1],['summarization_mode_target_percent_matches_vision',5]]) {
  const body=testFunctions(tokenize(source)).find(item=>item.name===name).body;
  const parsed=typedProgramOf(body,{source,file:root+'/'+path,root});
  assert.ok(parsed.program,parsed.reason);
  assert.equal(parsed.program.nativeAssertions,assertions);
  const actual=await executeTypedProgram(host,parsed.program);
  assert.equal(actual.status,'passed',actual.failure);
  assert.equal(actual.assertions,assertions);
 }
});
test('generic scalar compiler follows renamed types and methods with owned enum values',async()=> {
 for(const [source,variant,method,expected] of [[producer(),'Primary','amount',7],[producer('Unrelated','measurement','Initial','Final'),'Initial','measurement',7]]) {
  const generated=compile(source).programs;
  const binding=generated.find(item=>item.kind==='nativeEnumVariant'&&item.variant===variant);
  const callable=generated.find(item=>item.kind==='nativeEnumScalarMatch'&&item.name===method);
  assert.ok(binding&&callable);
  const actual=await evaluate({kind:'call',binding:callable,args:[{kind:'nativeValue',binding}]},new Map(),{observations:[]});
  assert.equal(actual,expected);
  await assert.rejects(()=>evaluate({kind:'call',binding:callable,args:[{kind:'literal',value:{nativeEnum:binding.enumPath,variant}}]},new Map(),{observations:[]}),/unowned/);
  const other=compile(producer('Foreign')).programs.find(item=>item.kind==='nativeEnumVariant');
  await assert.rejects(()=>evaluate({kind:'call',binding:callable,args:[{kind:'nativeValue',binding:other}]},new Map(),{observations:[]}),/foreign/);
 }
 const boolean=compile(producer().replace('->u32','->bool').replace('=> 7','=> true').replace('=> 11','=> false')).programs;
 const variant=boolean.find(item=>item.kind==='nativeEnumVariant'&&item.variant==='Secondary');
 const method=boolean.find(item=>item.kind==='nativeEnumScalarMatch');
 assert.equal(await evaluate({kind:'call',binding:method,args:[{kind:'nativeValue',binding:variant}]},new Map(),{observations:[]}),false);
});
test('scalar AST compiler refuses unproved ranges, variants, dispatch and effects',()=> {
 for(const source of [producer().replace('= 30','= 65536'),producer().replace('= 30','= 9007199254740993'),producer().replace('usize = 30','Custom = 30')])assert.ok(!compile(source).programs.some(item=>item.kind==='nativeConstant'));
 for(const source of [producer().replace('Primary, Secondary','Primary, Primary'),producer().replace('Primary, Secondary','Primary(u32), Secondary'),producer().replace('Primary, Secondary','Primary = 8, Secondary')])assert.ok(!compile(source).programs.some(item=>item.kind==='nativeEnumVariant'));
 for(const source of [producer().replace('Self::Secondary => 11,',
''),
producer().replace('Self::Secondary => 11',
'Self::Primary => 11'),
producer().replace('Self::Secondary => 11',
'_ => 11'),
producer().replace('Self::Secondary => 11',
'Self::Unknown => 11'),
producer().replace('Primary, Secondary',
'Primary, Secondary, Extra'),
producer().replace('match self {',
'effect(); match self {'),
producer().replace('=> 11',
'=> unknown()'),
producer().replace('=> 11',
'=> 4294967296')])assert.ok(!compile(source).programs.some(item=>item.kind==='nativeEnumScalarMatch'));

 for(const source of ['trait Custom {} '+producer(),'use unknown::Trait; '+producer(),producer()+' impl Custom for Choice {}'])assert.throws(()=>compile(source));
 assert.ok(!compile(producer().replace('derive(Clone,Copy,PartialEq,Eq)','derive(Custom)')).programs.some(item=>item.kind==='nativeEnumVariant'));
 assert.ok(!compile(producer().replace('pub const BOUND','#[custom] pub const BOUND')).programs.some(item=>item.kind==='nativeConstant'));
});
test('scalar imports, aliases, shadows and source drift remain guarded',async()=> {
 const imported='use formal_ai::summarization::{SummarizationMode as Selected, DEFAULT_MAX_STATEMENTS as Cap};';
 const valid=inline(imported+' #[test] fn arbitrary(){assert_eq!(Selected::Short.target_percent(),20);assert_eq!(Cap,30);}');
 assert.ok(valid.program,valid.reason);
 assert.equal((await executeTypedProgram(host,valid.program)).status,'passed');
 for(const source of [imported+' type Selected=Custom; #[test] fn arbitrary(){assert_eq!(Selected::Short.target_percent(),20);}',imported+' #[test] fn arbitrary(){let Selected="shadow";assert_eq!(Selected::Short.target_percent(),20);}',imported+' #[test] fn arbitrary(Cap:u32){assert_eq!(Cap,30);}',imported+' #[test] fn arbitrary(){assert_eq!(Cap("x"),30);}',imported+' use unknown::Thing; #[test] fn arbitrary(){assert_eq!(Cap,30);}',imported+' use unknown::*; #[test] fn arbitrary(){assert_eq!(Cap,30);}',imported+' mod formal_ai{} #[test] fn arbitrary(){assert_eq!(Cap,30);}','use formal_ai::SummarizationMode as Selected; #[test] fn arbitrary(){assert_eq!(Selected::Short.target_percent(),20);}'])assert.equal(inline(source).program,undefined);
 const wrong=inline(imported+' #[test] fn arbitrary(){assert_eq!(Selected::Short.target_percent(),99);}');
 assert.equal((await executeTypedProgram(host,wrong.program)).status,'failed');
 const stale=structuredClone(valid.program);stale.fixtures[1].sha256='0'.repeat(64);
 const refused=await executeTypedProgram(host,stale);
 assert.equal(refused.status,'failed');assert.equal(refused.observations.length,0);
 const corrupted=structuredClone(valid.program);corrupted.steps[0].left.binding.cases[0].value=999;
 const guarded=await executeTypedProgram(host,corrupted);
 assert.equal(guarded.status,'failed');assert.equal(guarded.observations.length,0);
});
test('temporary scalar compiler records actual maintained translation blockers',()=> {
 const path='rust/src/summarization/mod.rs',source=readFileSync(root+'/'+path,'utf8');
 const programs=generateScalars(source).programs;
 for(const kind of ['nativeConstant','nativeEnumVariant','nativeEnumScalarMatch']) {
  const binding=programs.find(item=>item.kind===kind);
  assert.ok(binding);
  const actual=source.slice(binding.sourceSpan.start,binding.sourceSpan.end);
  const result=selfTranslate(actual,'Rust','JavaScript');
  assert.ok(result.items.some(item=>item.status==='carried'&&TEMPORARY_SCALAR_WORKAROUND.blockers.includes(item.reason)));
  assert.equal(binding.sourceWitnesses[0].path,path);
  assert.equal(binding.sourceWitnesses[0].sha256.length,64);
 }
 assert.equal(TEMPORARY_SCALAR_WORKAROUND.upstreamIssue,null);
 assert.ok(GENERATED_NATIVE_PROGRAMS.some(item=>item.kind==='nativeEnumScalarMatch'));
});
