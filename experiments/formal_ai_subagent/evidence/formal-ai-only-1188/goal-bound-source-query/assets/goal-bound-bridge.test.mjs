import test from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/server/worker-host.mjs';
import { installNodeHost } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const {planGoalBound,requestGraph}=await import('./goal-bound-bridge.mjs');
const {finalResult}=await import('/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/final_result.mjs');
const {planChatStepResolved}=await import('/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/planner.mjs');
const prompt='Read f.rs. Design a reusable interface from observed declarations.';
function history({raw='pub fn answer() {}',metadata={path:'f.rs',success:true,complete:true,format:'raw'},name='read',id='c',secondUser=null,error=false}={}) {
 const messages=[{role:'user',content:prompt},{role:'assistant',content:'',tool_calls:[{id:'c',type:'function',function:{name:'read',arguments:JSON.stringify({path:'f.rs'})}}]},
 {role:'tool',name,tool_call_id:id,content:raw,is_error:error,source_read:metadata}];
 if(secondUser)messages.push({role:'user',content:secondUser});
 return messages;
}
const none=()=>{throw Error('unsupported goal must not delegate to unsafe terminal route');};
test('complete inputs yield bounded typed Gap, not source-plan success',async()=>{
 const raw='pub fn answer() {}'+ 'x'.repeat(50000);
 const plan=await planGoalBound(history({raw}),['read'],none);
 assert.equal(finalResult(plan).disposition,'gap');assert.ok(plan.answer.length<2000);
 assert.ok(!plan.answer.includes(raw));assert.equal(plan.result.discovery.observations[0].utf8Bytes,Buffer.byteLength(raw));
 assert.equal(plan.result.discovery.observations[0].contentIdentity.length,64);assert.equal(plan.result.discovery.verified,false);
});
test('plain Read delegates unchanged and quoted goal text cannot open a goal',async()=>{
 const messages=[{role:'user',content:'Read f.rs.'}];const sentinel={kind:'final',answer:'all raw bytes'};
 assert.equal(await planGoalBound(messages,['read'],()=>sentinel),sentinel);
 assert.equal(requestGraph('Read «aDesign; Propose.rs».').goals.length,0);
});
test('real complete plain Read retains exact underlying planner answer',async()=>{
 const messages=history();messages[0].content='Read f.rs.';
 const baseline=await planChatStepResolved(messages,['read']);
 const bridged=await planGoalBound(messages,['read'],planChatStepResolved);
 assert.deepEqual(bridged,baseline);assert.ok(bridged.answer.includes('pub fn answer() {}'));
});
test('raw JSON failure keys are source data under actual provider metadata',async()=>{
 const raw=JSON.stringify({success:false,complete:false,error:'failed',signal:'SIGTERM'});
 const plan=await planGoalBound(history({raw}),['read'],none);assert.equal(finalResult(plan).disposition,'gap');
 assert.equal(plan.result.discovery.observations[0].status,'reported-success');assert.ok(!plan.answer.includes('SIGTERM'));
});
for(const [label,options] of [
 ['bare source',{metadata:null}],['partial source',{metadata:{path:'f.rs',success:true,complete:false,format:'raw'}}],
 ['wrong metadata path',{metadata:{path:'other.rs',success:true,complete:true,format:'raw'}}],
]) test(label+' cannot certify complete dependency',async()=>{
 const plan=await planGoalBound(history(options),['read'],none);assert.equal(finalResult(plan).disposition,'gap');
 assert.deepEqual(plan.result.discovery.missingContracts,['complete-bound-source-observation']);
});
for(const [label,options] of [
 ['wrong call ID',{id:'other'}],['wrong tool name',{name:'bash'}],['previous user window',{secondUser:prompt}],
]) test(label+' must require a fresh Read',async()=>{
 const plan=await planGoalBound(history(options),['read'],none);assert.equal(plan.kind,'tool_calls');
 assert.equal(plan.calls.length,1);assert.equal(plan.calls[0].tool,'read');
});
test('explicit provider error stays Failure',async()=>{
 const plan=await planGoalBound(history({raw:'denied',error:true}),['read'],none);
 assert.equal(finalResult(plan).disposition,'failure');assert.deepEqual(plan.result.discovery.missingContracts,['failed-source-provider']);
});
test('no read provider yields typed Gap without claiming observation',async()=>{
 const plan=await planGoalBound([{role:'user',content:prompt}],[],none);
 assert.equal(finalResult(plan).disposition,'gap');assert.equal(plan.result.discovery.observations[0].complete,false);
});
test('verified Run holds command exit expectation without execution or success claim',async()=>{
 const messages=history();messages[0].content='Read f.rs. Run cargo check.';
 const plan=await planGoalBound(messages,['read','bash'],none);assert.equal(plan.kind,'final');
 assert.equal(plan.result.discovery.goals[0].kind,'verified-run');
 assert.deepEqual(plan.result.discovery.goals[0].expectation,{kind:'command_exit',command:'cargo check',expected_exit:0});
 assert.equal(plan.result.discovery.verified,false);assert.equal(finalResult(plan).disposition,'gap');
});
test('multilingual owned read and Unicode UTF8 goal spans are retained',()=>{
 for(const prompt of ['Lee f.rs. Diseña una interfaz.','Прочитай f.rs. Реализуй интерфейс.','读取 f.rs。设计接口。']){
  const graph=requestGraph(prompt);assert.equal(graph.startsWithOwnedRead,true);assert.equal(graph.dependencies[0].path,'f.rs');
  assert.ok(graph.goals.length>0);assert.ok(graph.goals.every(({span})=>span[1]<=Buffer.byteLength(prompt)));
 }
});
