// The original native ladder requests, with actual filesystem/process readback.
import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {existsSync, mkdtempSync, readFileSync, rmSync, mkdirSync, writeFileSync} from 'node:fs';
import {dirname, join} from 'node:path';
import {composeGeneralChangePlan, planLinksNotation} from '../../../js/agentic/general_planner.mjs';
import {tmpdir} from 'node:os';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {canDeliverFinal, FinalPayloadRole} from '../../../js/agentic/plan.mjs';
import {planNoteCompositionStep} from '../../../js/agentic/note_composition.mjs';
import {observedPayload} from '../../../js/agentic/tool_result.mjs';
import {executeResult} from '../../../experiments/js_dogfood/drive.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
const atomic="Atomic task 9: Assemble an intake summary containing the applicant name, the referral source, and the interview date. Store it in `intake/monday.md`.";
const expected="Atomic task 9: Assemble an intake summary containing the applicant name, the referral source, and the interview date\n\nRequested parts:\n- the applicant name\n- the referral source\n- the interview date\n\nObserved in this session:\n- nothing: no tool result was recorded before this note.\n\nNo requested part above is backed by an observation from this session.\n";
const colon="Verify a single node can be selected by dotted binary path for focused debugging. Create `agent-ladder-effects/node.lino` with these exact field lines: `node_path=2.1.2.1.2`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by at least four words that state the task result actually observed in this checkout. Leave supporting evidence in `.agent-ladder/node-proof.md`. The first line must be exactly `node_path=2.1.2.1.2`.";
const marker='if depth == level and (not filt or node == filt):';
const proof='.agent-ladder/node-proof.md';
const effect='agent-ladder-effects/node.lino';
const tools=['bash','batch','codesearch','edit','glob','grep','list','read','task','todoread','todowrite','webfetch','websearch','write'];
function observedSearch(){return[{role:'assistant',content:'',tool_calls:[{id:'search-node-filter',type:'function',function:{name:'grep',arguments:'{"include":"experiments/**/*","pattern":"node == filt"}'}}]},{role:'tool',tool_call_id:'search-node-filter',name:'grep',content:'Line 148:         '+marker}];}
async function replay(prompt, mutate=null, options={}){
 const directory=mkdtempSync(join(tmpdir(),'formal-ai-ladder-delivery-'));
 const messages=[{role:'user',content:prompt},...(options.history??(prompt===colon?observedSearch():[]))];
 for(const[path,content]of Object.entries(options.initial??{})){mkdirSync(dirname(join(directory,path)),{recursive:true});writeFileSync(join(directory,path),content);}
 const calls=[];let final=null;
 try{
  for(let turn=0;turn<(options.turns??6);turn++){
   const plan=await planChatStep(messages,tools);assert.ok(plan);
   if(plan.kind==='final'){final=plan.answer;break;}
   for(const[index,call]of plan.calls.entries()){
    const id='delivery-'+turn+'-'+index;calls.push(call);
    messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]});
    const receipt=executeResult(directory,call);
    messages.push({role:'tool',tool_call_id:id,name:call.tool,...(mutate?.(call,receipt)??receipt)});
    if(options.continue)messages.push({role:'user',content:'Continue if you have next steps'});
   }
  }
  const paths=new Set([proof,effect,'intake/monday.md','.formal-ai/general-change-plan.lino',...calls.filter(call=>call.tool==='write').map(call=>{const args=JSON.parse(call.arguments);return args.filePath??args.path;})]);
  const files=new Map([...paths].filter(path=>existsSync(join(directory,path))).map(path=>[path,readFileSync(join(directory,path),'utf8')]));
  return{files,calls,final};
 }finally{rmSync(directory,{recursive:true,force:true});}
}
test('composed note supplies producer-owned audit provenance without changing any bytes',()=>{
 const plan=planNoteCompositionStep(atomic,[]);assert.equal(plan.answer,expected);
 assert.equal(plan.result.disposition,'finding');assert.equal(plan.result.origin,'note_composition');
 assert.equal(plan.result.payloadRole,FinalPayloadRole.AuditReport);assert.equal(canDeliverFinal(plan),true);
});
test('unchanged atomic intake request writes the full original expected note',async()=>{
 const result=await replay(atomic);assert.equal(result.files.get('intake/monday.md'),expected);
 assert.ok(result.calls.some(call=>call.tool==='bash'));assert.ok(result.final);
});
test('unchanged colon source request preserves all fields, destinations and exact source condition',async()=>{
 const result=await replay(colon);assert.ok(result.files.has(proof));assert.ok(result.files.has(effect));
 for(const path of[proof,effect]){const content=result.files.get(path);assert.ok(content.includes(marker),content);assert.ok(!content.trimEnd().endsWith(':'));}
 assert.deepEqual(result.files.get(effect).split('\n').slice(0,3),['node_path=2.1.2.1.2','node_depth=5','node_kind=leaf']);
 assert.ok(result.files.get(effect).split('\n').find(line=>line.startsWith('result=')).split(/\s+/u).length>=4);
});
for(const[mode,mutate]of[
 ['bare acknowledgement',(call,receipt)=>call.tool==='bash'?{content:'ok'}:receipt],
 ['wrong successful bytes',(call,receipt)=>call.tool==='bash'?{content:JSON.stringify({stdout:'wrong observed bytes',exit_code:0,complete:true})}:receipt],
 ['unknown status with matching bytes',(call,receipt)=>call.tool==='bash'?{content:observedPayload(receipt.content)}:receipt],
 ['incomplete matching bytes',(call,receipt)=>call.tool==='bash'?{content:JSON.stringify({stdout:observedPayload(receipt.content),exit_code:0,complete:false})}:receipt],
])test(mode+' cannot certify proof or discharge its dependent record',async()=>{
 const result=await replay(colon,mutate);assert.ok(result.files.has(proof));assert.equal(result.files.has(effect),false);
 assert.ok(result.final);assert.ok(/Verification failed|not verified|unverified/.test(result.final),result.final);
});

const vendor='Draft a vendor brief containing the contract owner, the renewal window, and the escalation path. Store it in `vendors/acme.md`.';
const vendorExpected='Draft a vendor brief containing the contract owner, the renewal window, and the escalation path\n\nRequested parts:\n- the contract owner\n- the renewal window\n- the escalation path\n\nObserved in this session:\n- nothing: no tool result was recorded before this note.\n\nNo requested part above is backed by an observation from this session.\n';
const retention='Create file `policy/retention.md` containing Logs are kept for ninety days; backups are kept for a year.';
const retentionBytes='Logs are kept for ninety days; backups are kept for a year.';
test('unchanged vendor brief delivers every original byte after observed append',async()=>{
 const result=await replay(vendor);assert.equal(result.files.get('vendors/acme.md'),vendorExpected);assert.ok(result.final);
});
test('unchanged semicolon payload retains both clauses with no invented newline',async()=>{
 const result=await replay(retention);assert.equal(result.files.get('policy/retention.md'),retentionBytes);assert.ok(result.final);
});
for(const[prompt,path,part]of[
 ['Create `list.txt` containing apples, bananas and cherries.','list.txt','apples, bananas and cherries'],
 ['Write `greeting.txt` with the text hello from the harness.','greeting.txt','hello from the harness'],
])test('spelled-out bytes reach physical '+path,async()=>{
 const result=await replay(prompt);assert.ok(result.files.get(path)?.includes(part));assert.ok(result.final);
});
test('unchanged continuation pings advance a literal plan without third repeated calls',async()=>{
 const result=await replay('Create the file notes/welcome.txt containing the single line hello wikiquote.',null,{turns:10,continue:true});
 for(const call of result.calls)assert.ok(result.calls.filter(other=>other.tool===call.tool&&other.arguments===call.arguments).length<3);
 assert.ok(result.files.get('notes/welcome.txt')?.includes('hello wikiquote'));
 assert.ok(/welcome.txt|wikiquote/i.test(result.final),result.final);
});
test('a genuine earlier plan remains before the new append and target bytes',async()=>{
 const prior=planLinksNotation(composeGeneralChangePlan('Create `prior.txt` containing preserved history.'));
 const result=await replay(retention,null,{initial:{'.formal-ai/general-change-plan.lino':prior}});
 assert.equal(result.files.get('policy/retention.md'),retentionBytes);
 const history=result.files.get('.formal-ai/general-change-plan.lino');assert.ok(history.startsWith(prior));assert.ok(history.length>prior.length);assert.ok(result.final);
});
test('unchanged terse source fact retains the actual sentinel in result and proof',async()=>{
 const prompt='Verify that a leaf without an observable completion contract is never treated as independently checkable. Create `agent-ladder-effects/node.lino` with these exact field lines: `node_path=1.1`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by at least four words that state the task result actually observed in this checkout. Leave supporting evidence in `.agent-ladder/node-proof.md`. The first line must be exactly `node_path=1.1`.';
 const history=[{role:'assistant',content:'',tool_calls:[{id:'search-contract',type:'function',function:{name:'grep',arguments:'{"include":"src/**/*","pattern":"unresolved_single_need"}'}}]},{role:'tool',tool_call_id:'search-contract',name:'grep',content:'src/task_decomposition.rs:385: "unresolved_single_need",'}];
 const result=await replay(prompt,null,{history});
 assert.equal(result.files.get(effect).split('\n')[0],'node_path=1.1');
 const observed=result.files.get(effect).split('\n').find(line=>line.startsWith('result=')).slice(7);
 assert.ok(observed.split(/\s+/u).length>=4);assert.ok(observed.includes('unresolved_single_need'));
 const body=result.files.get(proof).split('\n').slice(1).join(' ');assert.ok(body.split(/\s+/u).length>=4);assert.ok(body.includes('unresolved_single_need'));
});

for(const[polite,familiar,path]of[
 ['Создайте файл `hello.txt` с текстом привет.','Создай файл `hello.txt` с текстом привет.','hello.txt'],
 ['Сохраните файл `notes.txt` с текстом заметка.','Сохрани файл `notes.txt` с текстом заметка.','notes.txt'],
 ['Сделайте `report.md` с текстом итог квартала.','Сделай `report.md` с текстом итог квартала.','report.md'],
])test('unchanged polite and familiar requests deliver identical '+path,async()=>{
 const politeResult=await replay(polite),familiarResult=await replay(familiar);
 assert.ok(politeResult.files.has(path));assert.ok(familiarResult.files.has(path));
 assert.equal(politeResult.files.get(path),familiarResult.files.get(path));
 assert.equal(politeResult.calls.filter(call=>call.tool==='write').length,familiarResult.calls.filter(call=>call.tool==='write').length);
});

test('the unchanged local readiness observation delivers both nested artifacts before optional research',async()=>{
 const prompt='Review the existing readiness check and record the observable completion contract for workers. Create `audit-effects/readiness.lino` with these exact field lines: `subject=readiness`, `kind=inspection`, and `result=` followed by the observed result. Leave supporting evidence in `.audit/readiness-proof.md`. The first line must be exactly `proof_for=readiness`. Use web research when it materially improves factual accuracy.';
 const observed='Found 3 matches\n/tmp/work/src/work.rs:\n  Line 70:     pub completion_criterion: String,\n  Line 89:             && !self.completion_criterion.starts_with("unresolved_")\n  Line 130:                 self.completion_criterion.clone(),';
 const history=[{role:'assistant',content:'',tool_calls:[{id:'search-readiness',type:'function',function:{name:'grep',arguments:'{"pattern":"completion_criterion"}'}}]},{role:'tool',tool_call_id:'search-readiness',name:'grep',content:observed}];
 const result=await replay(prompt,null,{history});assert.equal(result.calls.some(call=>call.tool==='websearch'),false);
 const effectBytes=result.files.get('audit-effects/readiness.lino'),proofBytes=result.files.get('.audit/readiness-proof.md');
 assert.ok(effectBytes.includes('result=Line 89:'));assert.ok(effectBytes.includes('!self.completion_criterion.starts_with'));
 assert.ok(proofBytes.startsWith('proof_for=readiness\n'));assert.ok(proofBytes.includes('!self.completion_criterion.starts_with'));
 assert.equal(proofBytes.includes('Line 70:'),false);assert.equal(proofBytes.includes('Line 130:'),false);
});
