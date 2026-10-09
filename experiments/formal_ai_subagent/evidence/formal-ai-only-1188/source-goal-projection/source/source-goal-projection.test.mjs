import assert from'node:assert/strict';
import fs from'node:fs';
import test from'node:test';
import{createHash}from'node:crypto';
import{WorkerHost}from'/Users/konard/Code/Archive/link-assistant/formal-ai/js/server/worker-host.mjs';
import{installNodeHost}from'/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/node-host.mjs';
import{drive}from'/Users/konard/Code/Archive/link-assistant/formal-ai/experiments/js_dogfood/drive.mjs';
import{finalResult,projectPlan,canDeliverFinal}from'/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/final_result.mjs';
import{planFileReadStep,directTask,FileReadMode}from'/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/file_read.mjs';
import{planSourceProjectionGoal as plan}from'./source-goal-projection.mjs';
import{planGoalBound}from'./goal-bound-bridge.mjs';
await installNodeHost(new WorkerHost());
const hash=source=>createHash('sha256').update(source).digest('hex');
const prompt='Show the observed declaration in source.mjs.';
const goal=(name='pick',kind='export')=>({kind:'source-declaration',request:prompt,path:'source.mjs',selector:{kind,name}});
function messages(source,options={}){return[{role:'user',content:prompt},
 {role:'assistant',content:'',tool_calls:[{id:'read-now',type:'function',function:{name:'read',arguments:JSON.stringify({path:'source.mjs'})}}]},
 {role:'tool',name:'read',tool_call_id:'read-now',content:source,source_read:{path:'source.mjs',success:true,complete:true,format:'raw'},...options}];}
const literal='// 文 😀\nexport function pick(value) { return value; }';
test('exact declaration retains independent UTF8/UTF16 spans and hashes without module authoring',()=>{
 const answer=plan(goal(),messages(literal),['read']),finding=answer.result.projection;
 assert.equal(finalResult(answer).disposition,'finding');assert.equal(canDeliverFinal(answer),true);
 assert.equal(finding.declaration.source,'export function pick(value) { return value; }');
 assert.equal(finding.declaration.span.start,literal.indexOf('export function'));
 assert.equal(finding.declaration.span.byteStart,Buffer.from(literal).indexOf('export function'));
 assert.equal(finding.observed.moduleContentId,hash(literal));assert.equal(finding.declaration.identity.declarationContentId,hash(finding.declaration.source));
 assert.equal(finding.authored,false);assert.equal(finding.executed,false);assert.equal(finding.semanticImplementation,false);
});
test('export alias resolves actual local declaration',()=>{
 const source='function local(x) { return x; } export { local as exposed };';
 const result=plan(goal('exposed'),messages(source),['read']).result.projection;
 assert.equal(result.declaration.name,'local');assert.equal(result.declaration.source,'function local(x) { return x; }');
});
test('local declaration is available without fabricated export',()=>{
 const result=plan(goal('local','local'),messages('function local(x) { return x; }'),['read']);
 assert.equal(finalResult(result).disposition,'finding');assert.equal(result.result.projection.goal.selector.kind,'local');
});
for(const[label,options]of[['bare',{source_read:undefined}],['partial',{source_read:{path:'source.mjs',success:true,complete:false,format:'raw'}}],['wrong provider path',{source_read:{path:'other.mjs',success:true,complete:true,format:'raw'}}]])test(label+' cannot certify a projection',()=>{
 const result=plan(goal(),messages(literal,options),['read']);assert.equal(finalResult(result).disposition,'gap');assert.equal(canDeliverFinal(result),false);
});
test('outer actual failure owns Failure',()=>{
 const result=plan(goal(),messages('EACCES denied',{is_error:true}),['read']);assert.equal(finalResult(result).disposition,'failure');assert.equal(canDeliverFinal(result),false);
});
for(const change of['wrong call','wrong tool','new window','conflicting paths'])test(change+' requires fresh Read',()=>{
 const history=messages(literal);
 if(change==='wrong call')history[2].tool_call_id='other';
 if(change==='wrong tool')history[2].name='bash';
 if(change==='new window')history.push({role:'user',content:prompt});
 if(change==='conflicting paths')history[1].tool_calls[0].function.arguments=JSON.stringify({path:'source.mjs',filePath:'other.mjs'});
 const result=plan(goal(),history,['read']);assert.equal(result.kind,'tool_calls');assert.equal(JSON.parse(result.calls[0].arguments).path,'source.mjs');
});
test('source and request drift refuse stale query',()=>{
 assert.equal(plan({...goal(),contentId:hash(literal+'changed')},messages(literal),['read']).result.projection.reason,'SourceChanged');
 assert.equal(plan({...goal(),request:prompt+'changed'},messages(literal),['read']).result.projection.reason,'GoalRequestChanged');
});
test('arbitrary JSON remains data in both source-query Gap and complete plain Read',()=>{
 const source='{"success":false,"complete":false,"error":"failed","exit_code":9}';
 const result=plan(goal(),messages(source),['read']);assert.equal(finalResult(result).disposition,'gap');assert.notEqual(result.result.projection.reason,'SourceProviderFailed');
 const full=planFileReadStep(directTask('source.mjs',FileReadMode.Full),messages(source),['read']);assert.ok(full.answer.includes(source));assert.equal(finalResult(full).disposition,'finding');
});
test('member/index access never receives getter or proxy purity inference',()=>{
 const source='export function pick(input) { return input.values[input.position]; }';
 const finding=plan(goal(),messages(source),['read']).result.projection;
 assert.equal(finding.certification,'exact-lexical-source-region-only');assert.equal(finding.declaration.contract.status,'unknown');assert.equal(finding.declaration.contract.callEffects,'unknown');
 assert.equal(finding.declaration.contract.gap.reason,'MissingStructuralSchema');
 assert.equal(plan({...goal(),requiredConstraints:['pure-call-effects']},messages(source),['read']).result.projection.reason,'UnsupportedGoalConstraint');
});
test('import alias never manufactures unobserved dependency declaration',()=>{
 const source="import { pick as other } from './dependency.mjs'; export { other as pick };";
 const result=plan(goal(),messages(source),['read']);assert.equal(finalResult(result).disposition,'gap');assert.equal(result.result.projection.reason,'DeclarationUnobserved');
});
for(const source of['export function pick(x) { return x;','function pick(x){return x;} function pick(y){return y;}','export function pick(x){return x;} export { pick as pick };'])test('invalid or duplicate source refuses projection '+source.length,()=>{
 assert.equal(finalResult(plan(goal(),messages(source),['read'])).disposition,'gap');
});
test('unsupported design/composition/Run cannot masquerade as source declaration',()=>{
 for(const kind of['design','compose','verified-run'])assert.equal(finalResult(plan({...goal(),kind},messages(literal),['read'])).disposition,'gap');
});
test('independent complete full Read still delegates exactly',async()=>{
 const source='{"error":"failed","content":"whole file"}\n',history=messages(source);history[0].content='Read source.mjs.';
 const result=await planGoalBound(history,['read'],(m,t)=>planFileReadStep(directTask('source.mjs',FileReadMode.Full),m,t));
 assert.ok(result.answer.includes(source));assert.equal(finalResult(result).disposition,'finding');
});
const tick=String.fromCharCode(96);
const realCases=[
 ['solver_formalization.mjs','selectedCandidate',"export function selectedCandidate(selection) {\n  return selection.decision.kind === 'selected' ? selection.candidates[selection.decision.index] ?? null : null;\n}"],
 ['translation_formalization.mjs','candidateCompactSummary',"export function candidateCompactSummary(candidate) {\n  const parts = candidate.slots.map((slot) => "+tick+"${slot.role}=${slot.anchor.id}"+tick+");\n  if (candidate.unresolved_terms.length > 0) parts.push("+tick+"unresolved=${candidate.unresolved_terms.join('|')}"+tick+");\n  return parts.length === 0 ? 'empty' : parts.join(' ');\n}"]];
for(const[file,name,expected]of realCases)test('actual source '+name+' projects whole declaration with unknown effects',async()=>{
 const source=fs.readFileSync(new URL('./'+file,import.meta.url),'utf8'),request='Show actual declaration '+name+' in '+file+'.';
 const query={...goal(name),path:file,request,contentId:hash(source)},selected=[];
 const result=await drive((history,tools)=>{const resolved=plan(query,history,tools);selected.push(finalResult(resolved));return projectPlan(resolved);},new URL('.',import.meta.url).pathname,request,{tools:['read'],steps:6});
 assert.equal(result.transcript.length,1);assert.equal(result.transcript[0].result,source);assert.equal(selected.at(-1).disposition,'finding');
 const finding=JSON.parse(result.answer);assert.equal(finding.declaration.source,expected);assert.equal(finding.declaration.contract.callEffects,'unknown');
 assert.equal(finding.semanticImplementation,false);assert.equal(fs.readFileSync(new URL('./'+file,import.meta.url),'utf8'),source);
});

test('observed declaration is never a certified complete source module',()=>{
 const result=plan(goal(),messages(literal),['read']);
 assert.equal(canDeliverFinal(result,'source-module'),false);assert.equal(result.result.artifact,null);
 assert.equal(plan({...goal(),semanticImplementation:true},messages(literal),['read']).result.projection.reason,'UnsupportedTerminalGoal');
});
test('source containing getters remains unevaluated with unknown access effects',()=>{
 const source="Object.defineProperty(globalThis,'__source_projection_getter__',{get(){throw new Error('observed getter ran');}}); export function pick(input){return input.value;}";
 const before=Object.hasOwn(globalThis,'__source_projection_getter__');
 const result=plan(goal(),messages(source),['read']);
 assert.equal(Object.hasOwn(globalThis,'__source_projection_getter__'),before);
 assert.equal(result.result.projection.declaration.contract.callEffects,'unknown');
 assert.equal(result.result.projection.module.effects,'unknown');
});
test('source-only projection cannot certify requested syntax or purity constraints',()=>{
 for(const constraint of['validated-syntax','pure-call-effects','owned-import-initialization']){
  const result=plan({...goal(),requiredConstraints:[constraint]},messages(literal),['read']);
  assert.equal(finalResult(result).disposition,'gap');assert.equal(canDeliverFinal(result),false);
 }
});
test('latest true provider error revokes an earlier complete declaration receipt',()=>{
 const history=messages(literal),denied=messages('EACCES denied',{is_error:true}).slice(1);
 denied[0].tool_calls[0].id='later';denied[1].tool_call_id='later';history.push(...denied);
 assert.equal(finalResult(plan(goal(),history,['read'])).disposition,'failure');
});
test('unknown absence cannot become an empty declaration success',()=>{
 const result=plan(goal(),[{role:'user',content:prompt}],[]);
 assert.equal(result.result.projection.reason,'SourceProviderUnavailable');assert.equal(finalResult(result).disposition,'gap');
});
