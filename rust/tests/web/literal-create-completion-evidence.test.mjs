import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {renderSeededOutcome} from '../../../js/agentic/code_task.mjs';
import {completeOwnedSourceReadFrame} from '../../../js/agentic/tool_result.mjs';
import {chatCompletionSse} from '../../../js/server/openai.mjs';
import {verifyLiteralCreation} from '../../../experiments/agent_cli_e2e/verify-literal-creation.mjs';

await installNodeHost(new WorkerHost(),{sourceUsageRustGrammar:false});
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const task='Create a file named hello.txt with the content hello world';
const target='hello.txt',expectedContent='hello world';
const digest=value=>createHash('sha256').update(value).digest('hex');
const request=(row,change)=>{const body=JSON.parse(row.request_body);change(body);row.request_body=JSON.stringify(body);};
const response=(row,change)=>{const body=JSON.parse(row.response_body);change(body);row.response_body=JSON.stringify(body);};
function completion(message,finish,index){
 return {id:'chatcmpl-literal-'+index,object:'chat.completion',created:1792380000,model:'formal-ai',
  choices:[{index:0,message,finish_reason:finish}],usage:{prompt_tokens:1,completion_tokens:1,total_tokens:2}};
}
function fixture(){
 const workspace=fs.mkdtempSync(path.join(os.tmpdir(),'literal-completion-'));
 const previousHost=host();installHost({...previousHost,currentDirectory:()=>workspace});
 const write={id:'write-unique',type:'function',function:{name:'write',arguments:JSON.stringify({filePath:path.join(workspace,target),content:expectedContent})}};
 const read={id:'read-unique',type:'function',function:{name:'read',arguments:JSON.stringify({filePath:path.join(workspace,target)})}};
 const messages=[{role:'user',content:task}],records=[];
 for(const [index,call]of [write,read].entries()){
  records.push({timestamp_unix_ms:index+1,method:'POST',path:'/v1/chat/completions',
   request_body:JSON.stringify({messages}),response_body:JSON.stringify(completion({role:'assistant',content:null,tool_calls:[call]},'tool_calls',index))});
  messages.push({role:'assistant',content:null,tool_calls:[call]});
  messages.push({role:'tool',tool_call_id:call.id,name:call.function.name,is_error:false,
   content:index===0?'':completeOwnedSourceReadFrame(expectedContent)});
 }
 records.push({timestamp_unix_ms:3,method:'POST',path:'/v1/chat/completions',request_body:JSON.stringify({messages}),
  response_body:JSON.stringify(completion({role:'assistant',content:renderSeededOutcome('coding_workspace_effect_observed',task,target)},'stop',2))});
 fs.writeFileSync(path.join(workspace,target),expectedContent);
 return {records,workspace,task,target,expectedContent,cleanup(){installHost(previousHost);fs.rmSync(workspace,{recursive:true,force:true});}};
}
function eachRequest(context,change){for(const row of context.records)request(row,change);}
function toolResult(context,id,change){eachRequest(context,body=>{for(const message of body.messages)if(message.role==='tool'&&message.tool_call_id===id)change(message);});}

test('the observed three-exchange issue681 Write Read final proves exact completion despite legacy floor four',()=>{
 const context=fixture();
 try{
  const report=verifyLiteralCreation(context);
  assert.equal(report.exchanges,3);assert.equal(report.bytes,11);
  assert.equal(report.writeCallId,'write-unique');assert.equal(report.readCallId,'read-unique');
  assert.equal(report.method,'read-tool');assert.equal(report.terminalCompleted,true);assert.equal(report.verified,true);
  assert.equal(report.sha256,digest(Buffer.from(expectedContent)));
  assert.deepEqual(fs.readFileSync(path.join(context.workspace,target)),Buffer.from(expectedContent));
 }finally{context.cleanup();}
});
test('the actual server SSE encoder with usage trailers preserves the same three-exchange proof',()=>{
 const context=fixture();
 try{
  for(const row of context.records){const completion=JSON.parse(row.response_body);completion.choices[0].message.content??='';row.response_body=chatCompletionSse(completion,true).body;}
  assert.equal(verifyLiteralCreation(context).exchanges,3);
 }finally{context.cleanup();}
});
const negatives=[
 ['missing physical effect',context=>fs.rmSync(path.join(context.workspace,target))],
 ['foreign physical bytes',context=>fs.appendFileSync(path.join(context.workspace,target),'foreign')],
 ['no Read call or receipt',context=>{
  context.records.splice(1,1);
  request(context.records.at(-1),body=>{body.messages=body.messages.filter(message=>message.role!=='assistant'||!message.tool_calls?.some(call=>call.id==='read-unique')).filter(message=>message.tool_call_id!=='read-unique');});
 }],
 ['foreign Write receipt',context=>toolResult(context,'write-unique',message=>{message.tool_call_id='foreign-write';})],
 ['foreign Read receipt',context=>toolResult(context,'read-unique',message=>{message.tool_call_id='foreign-read';})],
 ['failed Write acknowledgement',context=>toolResult(context,'write-unique',message=>{message.is_error=true;message.content='Write transport unavailable';})],
 ['failed Read acknowledgement',context=>toolResult(context,'read-unique',message=>{message.is_error=true;})],
 ['untyped exact Read bytes',context=>toolResult(context,'read-unique',message=>{message.content=expectedContent;})],
 ['incomplete Read frame',context=>toolResult(context,'read-unique',message=>{message.content=message.content.slice(0,Math.floor(message.content.length/2));})],
 ['Read of another target',context=>{
  for(const row of context.records){
   request(row,body=>{for(const message of body.messages)for(const call of message.tool_calls??[])if(call.id==='read-unique')call.function.arguments=JSON.stringify({filePath:path.join(context.workspace,'other.txt')});});
   response(row,body=>{for(const call of body.choices[0].message.tool_calls??[])if(call.id==='read-unique')call.function.arguments=JSON.stringify({filePath:path.join(context.workspace,'other.txt')});});
  }
 }],
 ['wrong Read source bytes',context=>toolResult(context,'read-unique',message=>{message.content=completeOwnedSourceReadFrame('foreign');})],
 ['missing terminal completion',context=>context.records.pop()],
 ['unverified seeded terminal',context=>response(context.records.at(-1),body=>{body.choices[0].message.content=renderSeededOutcome('coding_workspace_written_unverified',task,target);})],
 ['explicit Gap terminal',context=>response(context.records.at(-1),body=>{body.choices[0].message.content=JSON.stringify({schema:'workspace-creation-gap/v1',verified:false,completed:false});})],
 ['truncated terminal',context=>response(context.records.at(-1),body=>{body.choices[0].finish_reason='length';})]
];
for(const [name,change]of negatives)test(name+' cannot substitute for literal completion',()=>{
 const context=fixture();
 try{change(context);assert.throws(()=>verifyLiteralCreation(context));}finally{context.cleanup();}
});
test('only explicit literal-creation evidence supersedes the unchanged issue681 count floor',()=>{
 const workflow=fs.readFileSync(path.join(root,'.github/workflows/agent-cli-e2e.yml'),'utf8');
 const start=workflow.indexOf('TASK: "'+task+'"');assert.ok(start>=0);
 const end=workflow.indexOf('\n      - name:',start);
 const recipe=workflow.slice(start,end<0?undefined:end);
 assert.match(recipe,/MIN_POSTS:\s*4/u);
 assert.match(recipe,/COMPLETION_EVIDENCE:\s*literal-creation/u);
 assert.match(recipe,/EXPECT_CONTENT:\s*(?:"hello world"|'hello world'|hello world)/u);
 const harness=fs.readFileSync(path.join(root,'experiments/agent_cli_e2e/run_agent_cli.sh'),'utf8');
 assert.ok(harness.includes('literal-creation)'));assert.ok(harness.includes('verify-literal-creation.mjs'));
 assert.ok(harness.includes('source-cst|minimum-posts)'));assert.ok(harness.includes('"$posts" -ge "$MIN_POSTS"'));
 const profile=JSON.parse(fs.readFileSync(path.join(root,'data/benchmarks/fast-pull-request-javascript-tests.json'),'utf8'));
 assert.equal(profile.files.filter(file=>file==='rust/tests/web/literal-create-completion-evidence.test.mjs').length,1);
});
