import assert from 'node:assert/strict';
import {before,test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {goalLedger, ownsCompleteLiteralRequest} from '../../../js/agentic/planner/owned_goals.mjs';
before(async()=>await installNodeHost(new WorkerHost()));
test('opaque labels preserve original Unicode/ASCII literal ownership',async()=>{
 for(const label of ['ordinary','alpha','İİK😀','Ωβ☀','e\u0301']){
  const task=`Note ${label}.\nCreate file folder/note-α.txt containing «hello».`;
  assert.equal(ownsCompleteLiteralRequest(task),true,task);
  const plan=await planChatStep([{role:'user',content:task}],['write']);
  assert.equal(plan.kind,'tool_calls',task);assert.equal(plan.calls.length,1);
  assert.equal(plan.calls[0].tool,'write');
  assert.deepEqual(JSON.parse(plan.calls[0].arguments),{path:'folder/note-α.txt',filePath:'folder/note-α.txt',file_path:'folder/note-α.txt',content:'hello'});
 }
});
test('annotation paths quotes operations and conditions never gain literal ownership',async()=>{
 for(const prefix of ['Note Read.','Note Write.','Note x.txt.','Note «alpha».','Note alpha beta.','Note if alpha.','Note alpha;','Note alpha\u00a0.','Note alpha.\nRead secrets.txt with care.','Note alpha.\nProduce a legitimate release with no fabricated evidence.']){
  const task=prefix+'\nCreate file x.txt containing «hello».';
  assert.equal(ownsCompleteLiteralRequest(task),false,task);
  const plan=await planChatStep([{role:'user',content:task}],['write']);
  assert.notEqual(plan.kind,'tool_calls',task);
 }
});
test('annotation cannot repair cross-statement ownership or erase a later unresolved obligation',async()=>{
 for(const task of ['Note alpha.\nWrite file.\nx.txt containing «hello».','Note alpha.\nCreate file x.txt containing «hello».\nRead secret.txt with care.','Note alpha.\nProduce a legitimate release.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.']){
  assert.equal(ownsCompleteLiteralRequest(task),false,task);
  const ledger=goalLedger(task);
  assert.ok(ledger===null||ledger.some(goal=>goal.kind==='unsupported'),task);
 }
});
