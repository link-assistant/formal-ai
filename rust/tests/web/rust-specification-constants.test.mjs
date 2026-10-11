import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {generateBorrowedConstants,borrowedConstantTestImportContract} from '../../../scripts/lib/native-constant-registry.mjs';
import {tokenize,testFunctions} from '../../../scripts/lib/rust-specification-cases.mjs';
import {typedProgramOf,executeTypedProgram} from '../../../scripts/lib/rust-specification-programs.mjs';
import {generateRecords} from '../../../scripts/lib/native-record-registry.mjs';
import {expression,evaluate} from '../../../scripts/lib/rust-specification-values.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const root=resolve(import.meta.dirname,'../../..');
const source='pub const BORROWED_LABEL: &str = "plain";';
const library='pub mod fixture;pub use fixture::{BORROWED_LABEL as PUBLIC_LABEL};';
const compile=(native=source,publicRoot=library)=>generateBorrowedConstants(native,'rust/src/fixture.rs','fixture',publicRoot);
const refused=(native,publicRoot=library)=>{
  let result;try{result=compile(native,publicRoot);}catch(error){assert.match(error.message,/constant|namespace|str/);return;}
  assert.equal(result.programs.length,0);
};
test('unchanged complete native public constant specification executes its original assertion',async()=>{
  const file=root+'/rust/tests/unit/specification/openai_compatibility.rs',native=readFileSync(file,'utf8');
  const body=testFunctions(tokenize(native)).find(item=>item.name==='canonical_model_id_is_formal_ai').body;
  const parsed=typedProgramOf(body,{source:native,file,root});assert.ok(parsed.program,parsed.reason);
  assert.equal(parsed.program.nativeAssertions,1);
  const result=await executeTypedProgram(new WorkerHost(),parsed.program);assert.equal(result.status,'passed',result.failure);
  assert.equal(result.assertions,1);
});
test('maintained meta and JS constant emission follows arbitrary names and unique export aliases',async()=>{
  for(const [name,alias,value]of [['BORROWED_LABEL','PUBLIC_LABEL','plain'],['OTHER_TEXT','RENAMED_TEXT','\t\n\r value ']]){
    const native='pub const '+name+': &str = '+JSON.stringify(value)+';';
    const result=compile(native,'pub mod fixture;pub use fixture::{'+name+' as '+alias+'};');
    assert.equal(result.programs.length,2);
    for(const binding of result.programs){
      assert.equal(binding.nativeType,'&str');assert.equal(binding.value,value);
      assert.ok(binding.maintainedMeta.includes('(constant'));
      const emitted=await import('data:text/javascript,'+encodeURIComponent(binding.maintainedJavaScript));
      assert.equal(emitted[name],value);
      assert.equal(await evaluate({kind:'nativeValue',binding},new Map(),{observations:[]}),value);
    }
  }
});
test('borrowed constant lexical types, raw and byte strings, escapes and non-ASCII remain conservative',()=>{
  for(const native of ['pub "const" BORROWED_LABEL: &str = "plain";',
    'pub const BORROWED_LABEL: &"str" = "plain";','pub const BORROWED_LABEL: String = "plain";',
    'pub const BORROWED_LABEL: &\'static str = "plain";','#[cfg(feature="x")] '+source,
    'pub const BORROWED_LABEL: &str = r#"plain"#;','pub const BORROWED_LABEL: &str = br#"plain"#;',
    'pub const BORROWED_LABEL: &str = "é";','pub const BORROWED_LABEL: &str = "\\u{1f600}";',
    'pub const BORROWED_LABEL: &str = unknown();','pub const BORROWED_LABEL: &str = concat!("plain");',
    'pub const BORROWED_LABEL: &str = "\\u{4_1}";','pub const BORROWED_LABEL: &str = "\\xFF";'])refused(native);
});
test('constant source and export ownership rejects aliases, duplicates, wildcard and lexical counterfeits',()=>{
  for(const native of ['use custom::Type as str;'+source,'pub(crate) use custom::Type as str;'+source,
    'type str = Custom;'+source,source+source,'pub(crate) const BORROWED_LABEL: &str = "other";'+source])refused(native);
  for(const publicRoot of ['pub "mod" fixture;pub use fixture::BORROWED_LABEL;',
    'pub mod fixture;pub(crate) mod fixture;pub use fixture::BORROWED_LABEL;',
    'pub mod fixture;pub use fixture::*;','pub mod fixture;pub use fixture::{BORROWED_LABEL as PUBLIC_LABEL,BORROWED_LABEL as PUBLIC_LABEL};',
    'pub mod fixture;pub use fixture::BORROWED_LABEL as PUBLIC_LABEL;pub const PUBLIC_LABEL:u8=1;',
    'pub mod fixture;#[cfg(feature="x")]pub use fixture::BORROWED_LABEL;'])refused(source,publicRoot);
});
test('caller import and macro namespaces cannot counterfeit borrowed constants or native assertions',()=>{
  const binding=compile().programs[0];
  for(const caller of ['use formal_ai::fixture::BORROWED_LABEL as assert_eq;',
    'use formal_ai::fixture::BORROWED_LABEL as str;','#![no_implicit_prelude]use formal_ai::fixture::BORROWED_LABEL;',
    'macro_rules! assert_eq {()=>{}}','#[proc]fn helper(){}','pub(crate) mod formal_ai {}'])
    assert.throws(()=>borrowedConstantTestImportContract(caller,root,binding));
});
test('borrowed source constants never erase owned String field initialization authority',()=>{
  const constructor=generateRecords('use alloc::string::{String,ToString};pub struct Owned{pub text:String,pub flag:bool}impl Owned{pub const fn ready(&self)->bool{self.flag}}','rust/src/fixture.rs','fixture','extern crate alloc;pub mod fixture;')
    .programs.find(item=>item.kind==='nativeRecordConstructor');assert.ok(constructor);
  const binding=compile().programs[0],calls=new Map([['Owned',constructor],['BORROWED_LABEL',binding]]);
  assert.throws(()=>expression(tokenize('Owned{text:BORROWED_LABEL,flag:true}'),{calls,environment:new Map(),schemas:new Map([['Owned',constructor.fields]])}),/String|owned|initializer/);
});
