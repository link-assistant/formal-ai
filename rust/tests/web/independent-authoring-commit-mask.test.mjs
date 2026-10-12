import assert from 'node:assert/strict';
import {before,test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
before(async()=>installNodeHost(new WorkerHost()));
const tools=['bash','batch','codesearch','edit','glob','grep','list','read','task','todoread','todowrite','webfetch','websearch','write'];
for(const request of ['Implement a parser and haz commit.','Revisa haz committer y haz commit.','Revisa src/haz-committer.mjs y haz commit.','Append "haz commit" to notes.txt.']){
 test('original independent authoring never becomes a commit: '+request,async()=>{
  const step=await planChatStep([{role:'user',content:request}],tools);
  if(step.kind==='final'){
   assert.match(step.answer,/MissingContract/u);
   const receipt=JSON.parse(step.answer.slice(step.answer.indexOf('{')));
   assert.equal(receipt.reason,'MissingContract');assert.equal(receipt.goal,request);
   assert.equal(receipt.authored,false);assert.equal(receipt.verified,false);
   assert.ok(receipt.missingContracts.length>=2);
  }else{
   assert.equal(step.kind,'tool_calls');
   assert.ok(step.calls.every(call=>call.tool!=='bash'||!JSON.parse(call.arguments).command.includes('git commit')));
  }
 });
}
