import {test} from 'node:test';
import {mkdtempSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {executeResult} from '../../../experiments/js_dogfood/drive.mjs';
import assert from 'node:assert/strict';
import {installNativeCaptureObservationHost} from '../../../scripts/lib/native-capture-observation.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {planEditReadVerification} from '../../../js/agentic/workspace_change/digest_verification.mjs';
await installNativeCaptureObservationHost();
const ask=(m,t)=>planChatStepResolved(m,t);
const exchange=(messages,call,result,source_read,is_error=false)=>{
 const id='actual-'+messages.length;
 messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]});
 messages.push({role:'tool',name:call.tool,tool_call_id:id,content:result,...(source_read?{source_read}:{}),is_error});
};
test('Hindi authoritative literal owns only its payload',async()=>{
 const payload='prefix rename X to Y suffix',task='फ़ाइल owned.txt ठीक इसी सामग्री के साथ बनाओ:\n'+payload;
 const p=await ask([{role:'user',content:task}],['write','read','edit','bash']);
 assert.equal(p.kind,'tool_calls');assert.equal(JSON.parse(p.calls[0].arguments).content,payload);
});
test('complete observed source retains the concrete next mutation',async()=>{
 const task='In the file src/queue.rs, replace "pending" with "waiting". Change only that file and keep it valid Rust.';
 for(const tools of [['read','grep','write','bash','websearch'],['read','grep','write','bash','websearch','edit']]) {
 const m=[{role:'user',content:task}];
 const p=await ask(m,tools);exchange(m,p.calls[0],'fn update() { let state = "pending"; }\n',{path:'src/queue.rs',success:true,complete:true,format:'raw'});
 const next=await ask(m,tools);assert.equal(next.kind,'tool_calls',JSON.stringify(next));
 assert.equal(next.calls[0].tool,tools.includes('edit')?'edit':'write');
 for(const suffix of [[task],[task,task],['continue',task]]){const continued=[...m,...suffix.map(content=>({role:'user',content}))],p=await ask(continued,tools);assert.equal(p.kind,'tool_calls',JSON.stringify({suffix,p}));assert.deepEqual(p.calls,next.calls);}
 }
});
test('matched carried Edit failure stays failure without Run capability',async()=>{
 const task='Create file a.txt containing «old». Then in a.txt replace «old» with «new».',tools=['read','write','edit'],m=[{role:'user',content:task}];
 let p=await ask(m,tools);exchange(m,p.calls[0],'wrote 3 byte(s) to a.txt');
 p=await ask(m,tools);assert.equal(p.calls[0].tool,'read');
 exchange(m,p.calls[0],'old',{path:'a.txt',success:true,complete:true,format:'raw'});
 p=await ask(m,tools);assert.equal(p.calls[0].tool,'edit');
 exchange(m,p.calls[0],'Error: Edit anchor missing or ambiguous',undefined,true);
 const end=await ask(m,tools);assert.equal(end.result.disposition,'failure',JSON.stringify(end));
 assert.match(end.answer,/Edit anchor missing or ambiguous/);
});

test('postposed header action cannot consume the same action word in literal bodies',async()=>{
 for(const payload of ['बनाओ:\nkeep this verb','prefix बनाओ: suffix','Error: बनाओ:\nkeep']) {
  for(const task of ['फ़ाइल scoped.txt ठीक इसी सामग्री के साथ बनाओ:\n'+payload,
    'Create file scoped.txt with exactly this content:\n'+payload]) {
   const p=await ask([{role:'user',content:task}],['write','read','edit']);
   assert.equal(JSON.parse(p.calls[0].arguments).content,payload);
  }
 }
});
function editedSession(){
 const task='In note.txt replace «old» with «new».',tools=['read','edit'];
 const args=JSON.stringify({path:'note.txt',filePath:'note.txt',file_path:'note.txt',oldString:'old',newString:'new',old_string:'old',new_string:'new'});
 return {task,tools,args,m:[{role:'user',content:task}],change:{target:'note.txt',expected:'new',intent:'coding-text-replaced',slots:[['{old}','old'],['{new}','new']]}};
}
test('inferred verification requests a real Read and checks its exact complete bytes',async()=>{
 const task='In note.txt replace «old» with «new».',tools=['read','edit'],m=[{role:'user',content:task}];
 let p=await ask(m,tools);exchange(m,p.calls[0],'old',{path:'note.txt',success:true,complete:true,format:'raw'});
 p=await ask(m,tools);assert.equal(p.calls[0].tool,'edit');exchange(m,p.calls[0],'');
 p=await ask(m,tools);assert.equal(p.calls[0].tool,'read');exchange(m,p.calls[0],'new',{path:'note.txt',success:true,complete:true,format:'raw'});
 p=await ask(m,tools);assert.equal(p.result.disposition,'finding',JSON.stringify(p));
});
test('inferred Read rejects stale foreign incomplete mismatched and later-mutated observations',async()=>{
 const task='In note.txt replace «old» with «new».',tools=['read','edit'],base=[{role:'user',content:task}];
 let p=await ask(base,tools);exchange(base,p.calls[0],'old',{path:'note.txt',success:true,complete:true,format:'raw'});
 p=await ask(base,tools);exchange(base,p.calls[0],'');
 const next=await ask(base,tools);assert.equal(next.calls[0].tool,'read');
 for(const variant of ['foreign','incomplete','mismatch','failed','laterWrite','laterRun','pendingWrite','pendingRun']) {
  const m=structuredClone(base),path=variant==='foreign'?'else.txt':'note.txt';
  exchange(m,{...next.calls[0],arguments:JSON.stringify({path})},variant==='mismatch'?'wrong':'new',
   {path,success:variant!=='failed',complete:variant!=='incomplete',format:'raw'},variant==='failed');
  if(variant==='laterWrite')exchange(m,{tool:'write',arguments:JSON.stringify({path:'else.txt',content:'foreign'})},'');
  if(variant==='laterRun')exchange(m,{tool:'bash',arguments:JSON.stringify({command:'unknown-effect'})},'Output: ok\nExit Code: 0');
  if(variant==='pendingWrite'||variant==='pendingRun')m.push({role:'assistant',content:'',tool_calls:[{id:'pending-effect',type:'function',function:{name:variant==='pendingWrite'?'write':'bash',arguments:JSON.stringify(variant==='pendingWrite'?{path:'else.txt',content:'foreign'}:{command:'unknown-effect'})}}]});
  const end=await ask(m,tools);assert.notEqual(end?.result?.disposition,'finding',variant);
 }
});
test('complete ownership does not turn explicit execution or preservation tails into verified tests',()=>{
 const {task,tools,args,m,change}=editedSession();
 exchange(m,{tool:'edit',arguments:args},'');
 exchange(m,{tool:'read',arguments:JSON.stringify({path:'note.txt'})},'new',{path:'note.txt',success:true,complete:true,format:'raw'});
 for(const original of [task+' Then run tests.',task+' Run the command true.']) {
  assert.equal(planEditReadVerification(original,m,tools,change,args),null);
 }
 const natural='In note.txt, replace «old» with «new». Preserve all other content.';
 const result=planEditReadVerification(natural,m,tools,change,args);
 assert.equal(result,null,'unowned preservation tail remains an honest verification gap');
});

test('postimage containing old text is not reapplied after verification or repeated reads',async()=>{
 const task='In grow.txt replace «a» with «aa».',tools=['read','edit'],m=[{role:'user',content:task}];
 let p=await ask(m,tools);exchange(m,p.calls[0],'a',{path:'grow.txt',success:true,complete:true,format:'raw'});
 p=await ask(m,tools);assert.equal(p.calls[0].tool,'edit');exchange(m,p.calls[0],'');
 p=await ask(m,tools);assert.equal(p.calls[0].tool,'read');
 const call=p.calls[0];exchange(m,call,'aa',{path:'grow.txt',success:true,complete:true,format:'raw'});
 p=await ask(m,tools);assert.equal(p.result.disposition,'finding',JSON.stringify(p));
 exchange(m,call,'aa',{path:'grow.txt',success:true,complete:true,format:'raw'});
 p=await ask(m,tools);assert.equal(p.result.disposition,'failure',JSON.stringify(p));
 assert.equal(p.result.origin,'repeated_action_budget');
 assert.equal(p.kind,'final');
 for(const pending of [false,true]) {
  const bad=structuredClone(m.slice(0,3)),call={tool:'write',arguments:JSON.stringify({path:'foreign.txt',content:'changed'})};
  if(pending)bad.push({role:'assistant',content:'',tool_calls:[{id:'unobserved-before-edit',type:'function',function:{name:call.tool,arguments:call.arguments}}]});
  else exchange(bad,call,'');
  bad.push(...structuredClone(m.slice(3,7)));
  const refused=await ask(bad,tools);assert.equal(refused.kind,'final',JSON.stringify(refused));
  assert.equal(refused.result.disposition,'gap');
  assert.equal(JSON.parse(refused.answer).reason,'SourcePreimageUnavailable');
 }
});

test('typed source Read keeps error-looking physical bytes distinct from provider errors',async()=>{
 const dir=mkdtempSync(join(tmpdir(),'formal-ai-error-source-'));
 try {
  const path=join(dir,'proof.txt');
  writeFileSync(path,'Error: failed');
  const tools=['read','edit'],task='In proof.txt replace «failed» with «recovered».';
  const messages=[{role:'user',content:task}];
  const actual=call=>{
   assert.ok(['read','edit'].includes(call.tool));
   const receipt=executeResult(dir,call);
   exchange(messages,call,receipt.content,receipt.source_read,receipt.is_error);
   return receipt;
  };
  let plan=await ask(messages,tools);
  const before=actual(plan.calls[0]);
  assert.equal(before.content,'Error: failed');
  assert.equal(before.source_read.success,true);
  assert.equal(before.source_read.complete,true);
  plan=await ask(messages,tools);assert.equal(plan.calls[0].tool,'edit');actual(plan.calls[0]);
  assert.equal(readFileSync(path,'utf8'),'Error: recovered');
  const edited=structuredClone(messages);
  plan=await ask(messages,tools);assert.equal(plan.kind,'tool_calls',JSON.stringify(plan));
  assert.equal(plan.calls[0].tool,'read');
  const readCall=plan.calls[0],receipt=actual(readCall);
  assert.equal(receipt.content,'Error: recovered');
  assert.equal(receipt.source_read.complete,true);
  const finished=await ask(messages,tools);
  assert.equal(finished.result.disposition,'finding',JSON.stringify(finished));
  for(const error of ['provider','incomplete']) {
   const refused=structuredClone(edited);
   exchange(refused,readCall,receipt.content,{...receipt.source_read,complete:error!=='incomplete'},error==='provider');
   const result=await ask(refused,tools);
   assert.notEqual(result.result?.disposition,'finding',error);
   if(error==='provider')assert.equal(result.result.disposition,'failure');
  }
 }finally{rmSync(dir,{recursive:true,force:true});}
});
