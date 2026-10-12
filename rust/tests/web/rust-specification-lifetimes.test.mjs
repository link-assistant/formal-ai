import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {test} from 'node:test';
import {typedProgramOf,executeTypedProgram} from '../../../scripts/lib/rust-specification-programs.mjs';
import {testFunctions,tokenize} from '../../../scripts/lib/rust-specification-cases.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {REPO_ROOT} from '../../../js/server/lino.mjs';
const root=REPO_ROOT;
const original=join(root,'rust/tests/unit/specification/conversation_history.rs');
const name='previous_user_question_recall_skips_meta_turns_in_supported_languages';
function inline(source) {
 const native=testFunctions(tokenize(source))[0];assert.ok(native);
 return typedProgramOf(native.body,{source,root});
}
function originalProgram(file=original) {
 const source=readFileSync(file,'utf8');const native=testFunctions(tokenize(source)).find(item=>item.name===name);assert.ok(native);
 return {native,parsed:typedProgramOf(native.body,{source,file,root})};
}
test('unchanged source-owned lifetime recall case executes every original assertion in every language',async()=>{
 const {native,parsed}=originalProgram();assert.ok(parsed.program,parsed.reason);
 const macros=native.body.filter((token,index)=>['assert','assert_eq','assert_ne'].includes(token.text)&&native.body[index+1]?.text==='!').length;
 assert.equal(macros,6);assert.equal(parsed.program.nativeAssertions,macros);
 assert.deepEqual(parsed.program.schemaWitnesses,[{name:'RecallCase',declaredLifetimes:['a'],fields:
  ['language','request','unknown','first_prompt','followup_prompt'].map(name=>({name,type:'string',borrowed:true,lifetime:'a'}))}]);
 const result=await executeTypedProgram(new WorkerHost(),parsed.program);
 assert.equal(result.status,'passed',result.failure);assert.equal(result.assertions,24);
 assert.deepEqual(result.iterations,[{parameter:'case',length:4}]);assert.equal(result.observations.length,8);
 assert.ok(result.observations.every(item=>item.history.length===2||item.history.length===4));
});
test('arbitrary local record names and multiple declared string lifetimes have structural schema witnesses',async()=>{
 const parsed=inline(`use formal_ai::compile_natural_language_skill as compile;
 #[test] fn heldout() {
 struct Portable<'input, 'output> { given: &'input str, want: &'output str }
 let records=[Portable {given:"When \`quartz 738\` then \`a heldout reply\`",want:"a heldout reply"},
 Portable {given:"When \`feldspar 927\` then \`another reply\`",want:"another reply"}];
 for record in records { let package=compile(record.given).expect("compile");assert_eq!(package.response,record.want);assert!(package.id.starts_with("compiled_skill_")); }
 }`);
 assert.ok(parsed.program,parsed.reason);assert.deepEqual(parsed.program.schemaWitnesses,[{name:'Portable',declaredLifetimes:['input','output'],fields:[
 {name:'given',type:'string',borrowed:true,lifetime:'input'},{name:'want',type:'string',borrowed:true,lifetime:'output'}]}]);
 const result=await executeTypedProgram(new WorkerHost(),parsed.program);assert.equal(result.status,'passed',result.failure);assert.equal(result.assertions,4);
});
test('existing plain and implicit static borrowed string records retain supported semantics',async()=>{
 for(const type of ['&str',"&'static str"]) {
  const parsed=inline(`#[test] fn heldout() { struct Borrow { text: ${type} } let row=Borrow {text:"actual value"};assert_eq!(row.text,"actual value"); }`);
  assert.ok(parsed.program,parsed.reason);assert.deepEqual(parsed.program.schemaWitnesses[0].declaredLifetimes,[]);
  const result=await executeTypedProgram(new WorkerHost(),parsed.program);assert.equal(result.status,'passed',result.failure);assert.equal(result.assertions,1);
 }
});
test('undeclared, unused, duplicate, bounded and type generic lifetime forms reject complete cases',()=>{
 for(const declaration of ["struct Borrow<'a> { text: &'missing str }","struct Borrow { text: &'missing str }",
  "struct Borrow<T> { text: T }","struct Borrow<'a,T> { text: &'a str }","struct Borrow<'a:'b> { text: &'a str }",
  "struct Borrow<'a,'a> { text: &'a str }","struct Borrow<'static> { text: &'static str }",
  "struct Borrow<'_> { text: &'_ str }","struct Borrow<'unused> { text: &str }",
  "struct Borrow<'a> { text: &'a mut str }","struct Borrow<,'a> { text: &'a str }","struct Borrow<'a,,> { text: &'a str }"]) {
  const parsed=inline(`#[test] fn heldout() { ${declaration} let row=Borrow {text:"x"};assert_eq!(row.text,"x"); }`);
  assert.equal(parsed.program,undefined,declaration);assert.ok(parsed.reason,declaration);
 }
});
test('declared lifetime parsing never permits effects or substitutes unknown engine profiles',()=>{
 for(const operation of ['mutate_repository();','let solver=UniversalSolver::offline();']) {
  const parsed=inline(`use formal_ai::UniversalSolver; #[test] fn heldout() { struct Borrow<'a> { text: &'a str } let row=Borrow {text:"x"};assert_eq!(row.text,"x");${operation} }`);
  assert.equal(parsed.program,undefined);assert.ok(parsed.reason);
 }
});
test('source identity remains enforced before lifetime history replay',async t=>{
 const directory=mkdtempSync(join(tmpdir(),'native-lifetime-'));t.after(()=>rmSync(directory,{recursive:true,force:true}));
 const file=join(directory,'source.rs');writeFileSync(file,readFileSync(original));const {parsed}=originalProgram(file);assert.ok(parsed.program,parsed.reason);
 writeFileSync(file,readFileSync(file,'utf8')+'\n');const result=await executeTypedProgram(new WorkerHost(),parsed.program);
 assert.equal(result.status,'failed');assert.equal(result.failure,'stale native fixture identity');assert.equal(result.assertions,0);assert.deepEqual(result.observations,[]);
});
test('wrong lifetime-record assertions stop before later genuine solver calls',async()=>{
 const parsed=inline(`use formal_ai::UniversalSolver; #[test] fn heldout() { struct Borrow<'a> { text: &'a str } let row=Borrow {text:"actual"};assert_eq!(row.text,"wrong");let solver=UniversalSolver::default();let later=solver.solve("Hi");assert_eq!(later.intent,"greeting"); }`);
 assert.ok(parsed.program,parsed.reason);assert.equal(parsed.program.nativeAssertions,2);
 const result=await executeTypedProgram(new WorkerHost(),parsed.program);assert.equal(result.status,'failed');assert.equal(result.assertions,1);assert.deepEqual(result.observations,[]);
});
