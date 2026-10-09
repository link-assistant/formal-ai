import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { projectPlan, finalResult, canDeliverFinal, FinalDisposition } from '../../../js/agentic/final_result.mjs';
import { FileReadMode, directTask, directManyTask, listThenReadTask, planFileReadStep } from '../../../js/agentic/file_read.mjs';
import { planObservedCallableOutcome } from '../../../js/agentic/module_function/discovery.mjs';
await installNodeHost(new WorkerHost());
const raw = JSON.stringify({ success:false, complete:false, error:'failed', is_error:true,
  exit_code:9, signal:'SIGTERM', content:'exact authored source α 😀' },null,2)+'\n';
const metadata = (path) => ({path,success:true,complete:true,format:'raw'});
function receipt(path,content,options={}) {
 const call={id:'read:'+path,type:'function',function:{name:'read',arguments:JSON.stringify({path})}};
 return [{role:'assistant',content:'',tool_calls:[call]},
   {role:'tool',name:'read',tool_call_id:call.id,content,...options}];
}
const history=(content=raw,options={source_read:metadata('data.json')})=>[
 {role:'user',content:'Read data.json.'},...receipt('data.json',content,options)];
const direct=(messages)=>planFileReadStep(directTask('data.json',FileReadMode.Full),messages,['read']);
for(const body of [raw,JSON.stringify({schema:'source-read-receipt/v1',path:'data.json',success:true,complete:true,error:'failed',content:'not the file'})]) {
 test('complete provider Read retains every JSON source byte '+body.length,()=>{
  const plan=direct(history(body));assert.equal(finalResult(plan).disposition,FinalDisposition.Finding);
  assert.ok(plan.answer.includes(body));assert.ok(!plan.answer.includes('The command failed'));
 });
 test('bare same JSON stays available Unknown and cannot own transport role '+body.length,()=>{
  const plan=direct(history(body,{}));assert.equal(finalResult(plan).disposition,FinalDisposition.Unknown);
  assert.ok(plan.answer.includes(body));assert.equal(canDeliverFinal(plan),false);
 });
}
test('actual ordinary full planner and filesystem provider preserve exact JSON bytes',async()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'read-consumer-'));
 try{
  fs.writeFileSync(path.join(dir,'data.json'),raw);
  const results=[];const result=await drive(async(messages,tools)=>{const plan=await planChatStepResolved(messages,tools);results.push(finalResult(plan));return projectPlan(plan);},dir,'Read data.json.',{tools:['read'],steps:6});
  assert.equal(result.transcript.length,1);assert.equal(result.transcript[0].result,raw);
  assert.deepEqual(result.transcript[0].source_read,metadata('data.json'));
  assert.ok(result.answer.includes(raw));assert.equal(results.at(-1).disposition,FinalDisposition.Finding);
  assert.equal(fs.readFileSync(path.join(dir,'data.json'),'utf8'),raw);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});
test('explicit provider failure wins over successful Read metadata',()=>{
 const plan=direct(history('ENOENT no file',{is_error:true,source_read:metadata('data.json')}));
 assert.equal(finalResult(plan).disposition,FinalDisposition.Failure);assert.equal(canDeliverFinal(plan),false);
 assert.ok(plan.answer.includes('ENOENT'));
});
for(const [label,patch] of [
 ['partial',{complete:false}],['truncated',{truncated:true}],['wrong metadata path',{path:'other.json'}],
])test(label+' source does not gain complete Finding metadata',()=>{
 const plan=direct(history(raw,{source_read:{...metadata('data.json'),...patch}}));
 assert.equal(finalResult(plan).disposition,FinalDisposition.Unknown);assert.equal(canDeliverFinal(plan),false);
 assert.ok(plan.answer.includes(raw));
});
for(const [label,patch] of [['timed out',{timed_out:true}],['aborted',{aborted:true}],['signalled',{signal:'SIGTERM'}]])test(label+' actual provider metadata remains Failure',()=>{
 const plan=direct(history(raw,{source_read:{...metadata('data.json'),...patch}}));
 assert.equal(finalResult(plan).disposition,FinalDisposition.Failure);assert.equal(canDeliverFinal(plan),false);
 assert.ok(!plan.answer.startsWith('Contents of'));
});
for(const label of ['wrong call','wrong tool','new user window','conflicting paths'])test(label+' cannot bind the original file',()=>{
 const messages=history();
 if(label==='wrong call')messages[2].tool_call_id='other';
 if(label==='wrong tool')messages[2].name='bash';
 if(label==='new user window')messages.push({role:'user',content:'Read data.json.'});
 if(label==='conflicting paths')messages[1].tool_calls[0].function.arguments=JSON.stringify({path:'data.json',filePath:'other.json'});
 const plan=direct(messages);assert.equal(plan.kind,'tool_calls');assert.equal(plan.calls[0].tool,'read');
});
test('latest source observation supersedes older bytes and provider refusal',()=>{
 const messages=history('first',{source_read:metadata('data.json')});
 const later=receipt('data.json',raw,{source_read:metadata('data.json')});later[0].tool_calls[0].id='later';later[1].tool_call_id='later';messages.push(...later);
 const plan=direct(messages);assert.ok(plan.answer.includes(raw));assert.equal(finalResult(plan).disposition,FinalDisposition.Finding);
 const denied=receipt('data.json','denied',{is_error:true});denied[0].tool_calls[0].id='denied';denied[1].tool_call_id='denied';messages.push(...denied);
 assert.equal(finalResult(direct(messages)).disposition,FinalDisposition.Failure);
});
test('every multi-read source uses typed provider observations',()=>{
 const messages=[{role:'user',content:'Read data.json and other.json.'},...receipt('data.json',raw,{source_read:metadata('data.json')}),...receipt('other.json','other bytes\n',{source_read:metadata('other.json')})];
 const plan=planFileReadStep(directManyTask(['data.json','other.json'],FileReadMode.Full),messages,['read']);
 assert.ok(plan.answer.includes(raw));assert.ok(plan.answer.includes('other bytes\n'));assert.equal(finalResult(plan).disposition,FinalDisposition.Finding);
 messages.at(-1).source_read.complete=false;
 assert.equal(finalResult(planFileReadStep(directManyTask(['data.json','other.json'],FileReadMode.Full),messages,['read'])).disposition,FinalDisposition.Unknown);
});
test('partial multi-read JSON input does not become an error before the remaining Read',()=>{
 const messages=history();messages[0].content='Read data.json and other.json.';
 const plan=planFileReadStep(directManyTask(['data.json','other.json'],FileReadMode.Full),messages,['read']);
 assert.equal(plan.kind,'tool_calls');assert.equal(plan.calls.length,1);assert.equal(JSON.parse(plan.calls[0].arguments).path,'other.json');
});
test('list then read preserves JSON source and real shell listing receipt',()=>{
 const task=listThenReadTask('.','first',FileReadMode.Full),messages=[{role:'user',content:'List files then read the first file.'}];
 const first=planFileReadStep(task,messages,['read','bash']);
 const call={id:'ls',type:'function',function:{name:first.calls[0].tool,arguments:first.calls[0].arguments}};
 messages.push({role:'assistant',content:'',tool_calls:[call]},{role:'tool',name:'bash',tool_call_id:'ls',content:JSON.stringify({exit_code:0,stdout:'data.json\n'})});
 const next=planFileReadStep(task,messages,['read','bash']);assert.equal(next.calls[0].tool,'read');
 const readPath=JSON.parse(next.calls[0].arguments).path;
 messages.push(...receipt(readPath,raw,{source_read:metadata(readPath)}));
 const plan=planFileReadStep(task,messages,['read','bash']);assert.ok(plan.answer.includes(raw));assert.equal(finalResult(plan).disposition,FinalDisposition.Finding);
});
const request={name:'compose',parameters:['value'],destination:'out.mjs',inputs:['data.json'],acceptance:[],command:null};
function discoveryHistory(source=raw,options={source_read:metadata('data.json')}){
 return [{role:'user',content:'Add exported compose(value) to out.mjs. Read data.json before authoring.'},
 ...receipt('out.mjs','// destination\n',{source_read:metadata('out.mjs')}),...receipt('data.json',source,options)];
}
test('discovery hashes exact JSON source without treating its keys as a failed Read',()=>{
 const outcome=planObservedCallableOutcome(request,discoveryHistory(),['read','write']);
 assert.equal(outcome.disposition,FinalDisposition.Gap);assert.equal(outcome.witness.reason,'MissingContract');
 const observed=outcome.witness.observations[1];assert.equal(observed.contentId,createHash('sha256').update(raw).digest('hex'));
 assert.equal(observed.bytes,Buffer.byteLength(raw));assert.equal(observed.complete,true);assert.equal(observed.providerStatus,'reported-success');
 assert.equal(outcome.witness.authored,false);assert.equal(outcome.witness.verified,false);
});
for(const [label,options]of[['bare',{}],['partial',{source_read:{...metadata('data.json'),complete:false}}],['wrong metadata path',{source_read:metadata('other.json')}]] )test('discovery '+label+' input cannot certify a source contract',()=>{
 const outcome=planObservedCallableOutcome(request,discoveryHistory(raw,options),['read','write']);
 assert.equal(outcome.disposition,FinalDisposition.Gap);const observed=outcome.witness.observations[1];
 assert.equal(observed.complete,false);assert.equal(observed.contentId,createHash('sha256').update(raw).digest('hex'));
 assert.equal(Object.hasOwn(observed,'catalog'),false);assert.deepEqual(outcome.witness.detail.graphs,[]);
});
test('discovery genuine outer failure is Failure; authored ENOENT data is not absence',()=>{
 const outcome=planObservedCallableOutcome(request,discoveryHistory(raw,{is_error:true,source_read:metadata('data.json')}),['read']);
 assert.equal(outcome.disposition,FinalDisposition.Failure);assert.equal(outcome.witness.reason,'ReadFailed');
 const messages=discoveryHistory();messages[2].content='{"error":"ENOENT"}';
 const data=planObservedCallableOutcome(request,messages,['read']);
 assert.equal(data.witness.observations[0].state,'observed');assert.equal(data.witness.observations[0].complete,true);
});
