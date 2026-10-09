// The original native ladder requests, with actual filesystem/process readback.
import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {existsSync, mkdtempSync, readFileSync, rmSync} from 'node:fs';
import {join} from 'node:path';
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
async function replay(prompt, mutate=null){
 const directory=mkdtempSync(join(tmpdir(),'formal-ai-ladder-delivery-'));
 const messages=[{role:'user',content:prompt},...(prompt===colon?observedSearch():[])];
 const calls=[];let final=null;
 try{
  for(let turn=0;turn<6;turn++){
   const plan=await planChatStep(messages,tools);assert.ok(plan);
   if(plan.kind==='final'){final=plan.answer;break;}
   for(const[index,call]of plan.calls.entries()){
    const id='delivery-'+turn+'-'+index;calls.push(call);
    messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]});
    const receipt=executeResult(directory,call);
    messages.push({role:'tool',tool_call_id:id,name:call.tool,...(mutate?.(call,receipt)??receipt)});
   }
  }
  const files=new Map([proof,effect,'intake/monday.md'].filter(path=>existsSync(join(directory,path))).map(path=>[path,readFileSync(join(directory,path),'utf8')]));
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
