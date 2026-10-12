// Shell digest receipts verify exact authored bytes only after successful execution.
// Native twin: rust/tests/unit/pull_request_1188_line_operations.rs.
import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
let matches, plan;
before(async()=>{
 await installNodeHost(new WorkerHost());
 ({observedDigestMatches:matches}=await import('../../../js/agentic/tool_result.mjs'));
 ({planWorkspaceChangeStep:plan}=await import('../../../js/agentic/workspace_change.mjs'));
});
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const successReceipts=[output=>output,output=>JSON.stringify({stdout:output,exit_code:0}),output=>'Output: '+output+'\nExit Code: 0'];
test('digest normalization preserves Unicode and line-ending byte identities',()=>{
 for(const bytes of ['ascii','报告\n','Привет\r\n','no final newline']){
  const hash=digest(bytes);
  for(const receipt of successReceipts){assert.equal(matches(receipt(hash+'  f.txt\n'),hash),true);assert.equal(matches(receipt(hash+'  f.txt\n'),digest(bytes+'!')),false);}
 }
});
test('failed receipts cannot verify even when stdout contains the exact digest',()=>{
 const hash=digest('actual'),output=hash+'  f.txt\n';
 for(const receipt of [JSON.stringify({stdout:output,exit_code:1}),JSON.stringify({stdout:output,exit_code:0,is_error:true,error:'failed'}),'Output: '+output+'\nExit Code: 1'])assert.equal(matches(receipt,hash),false);
});
test('grounded workspace edit verifies receipt payload, preserving actual failed status',()=>{
 const prompt='In f.txt replace «old» with «new»',source='header\n报告 old\nfooter\n';
 for(const receipt of [...successReceipts,output=>JSON.stringify({stdout:output,exit_code:1})]){
  let bytes=source;const messages=[{role:'user',content:prompt}];const calls=[];let answer;
  for(let step=0;step<5;step++){
   const next=plan(prompt,messages,['read','edit','bash']);
   if(next?.kind==='final'){answer=next.answer;break;}
   assert.equal(next?.kind,'tool_calls');const call=next.calls[0],args=JSON.parse(call.arguments);calls.push(call.tool);
   let output='';if(call.tool==='read')output=bytes;
   if(call.tool==='edit')bytes=bytes.replace(args.oldString,()=>args.newString);
   if(call.tool==='bash')output=receipt(digest(bytes)+'  f.txt\n');
   const id='call_'+step;messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]},{role:'tool',tool_call_id:id,content:output});
  }
  assert.equal(bytes,'header\n报告 new\nfooter\n');assert.deepEqual(calls,['read','edit','bash']);
  assert.equal(answer.includes('Verification failed'),!successReceipts.includes(receipt));
 }
});
