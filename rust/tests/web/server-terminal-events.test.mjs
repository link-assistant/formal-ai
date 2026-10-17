import assert from 'node:assert/strict';
import fs from 'node:fs';import os from 'node:os';import path from 'node:path';
import {test} from 'node:test';
import {drive,executeResult} from '../../../experiments/js_dogfood/drive.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {createDriveEventSink,observedPlanTerminal} from '../../../js/logging/terminal-events.mjs';
import {createLogger,decodeTyped} from '../../../js/server/structured-log.mjs';
import {agenticOutcome} from '../../../js/server/agentic.mjs';
import {parseChatRequest} from '../../../js/server/chat-request.mjs';
function fixture(t,{silent=false}={}){
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-terminal-events-'));t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
 const chunks=[],logger=createLogger({env:{},level:silent?'silent':'debug',stderrWrite:b=>(chunks.push(Buffer.from(b)),b.length)});t.after(()=>logger.close());
 return {directory,logger,onEvent:createDriveEventSink(logger,{dialogId:'terminal-test'}),records:()=>Buffer.concat(chunks).toString().split('\n').filter(Boolean).map(decodeTyped)};
}
const final=()=>({kind:'final',answer:'No execution claim'});
function terminal(f){const events=f.records().filter(x=>x.kind==='agent.terminal');assert.equal(events.length,1);return events[0].metadata;}
test('actual maintained literal creation reports success only with verified planner origin',async t=>{
 const f=fixture(t);const result=await drive(planChatStep,f.directory,'Create result.txt containing exactly "hello".',{steps:12,onEvent:f.onEvent});
 assert.equal(result.stop,'final');assert.equal(fs.readFileSync(path.join(f.directory,'result.txt'),'utf8'),'hello');
 assert.equal(terminal(f).reason,'success');assert.equal(terminal(f).origin,'general_change_verified');
});
test('actual step exhaustion records planned_not_executed with no filesystem effect',async t=>{
 const f=fixture(t);const result=await drive(planChatStep,f.directory,'Create result.txt containing exactly "hello".',{steps:0,onEvent:f.onEvent});
 assert.equal(result.stop,'steps');assert.equal(terminal(f).reason,'planned_not_executed');assert.deepEqual(fs.readdirSync(f.directory),[]);
});
test('maintained repeated-call guard retains its semantic reason through public plan projection',async t=>{
 const f=fixture(t);fs.writeFileSync(path.join(f.directory,'evidence.txt'),'evidence');
 // Bootstrap the maintained host, then give the planner an actual prior List observation.
 await drive(async()=>final(),f.directory,'diagnostic bootstrap',{steps:1});
 const messages=[{role:'user',content:'List files'}];const first=await planChatStep(messages,['list']);assert.equal(first?.kind,'tool_calls');
 const call=first.calls[0],receipt=executeResult(f.directory,call);
 messages.push({role:'assistant',content:'',tool_calls:[{id:'prior',type:'function',function:{name:call.tool,arguments:call.arguments}}]},
  {role:'tool',tool_call_id:'prior',name:call.tool,...receipt},{role:'user',content:'Continue'});
 const guarded=await planChatStep(messages,['list']);assert.deepEqual(observedPlanTerminal(guarded),{reason:'repeated_call_guard',origin:'repeated_call_guard'});
 assert.deepEqual(Object.keys(guarded).sort(),['answer','kind']);
 await drive(async()=>guarded,f.directory,'diagnostic guard result',{onEvent:f.onEvent});assert.equal(terminal(f).reason,'repeated_call_guard');
});
test('actual missing Read receipt becomes tool_failure rather than a successful final answer',async t=>{
 const f=fixture(t);const result=await drive(async messages=>messages.some(m=>m.role==='tool')?final():{kind:'tool_calls',calls:[{tool:'read',arguments:'{"path":"absent"}'}]},f.directory,'diagnostic task',{onEvent:f.onEvent});
 assert.equal(result.transcript[0].is_error,true);assert.equal(terminal(f).reason,'tool_failure');
 assert(f.records().some(x=>x.kind==='agent.tool_result'&&x.metadata.reason==='tool_failure'));
});
test('actual AbortController cancellation and AbortSignal deadline retain original exceptions and reasons',async t=>{
 for(const [reason,planner] of [
  ['cancellation',async()=>{const controller=new AbortController();controller.abort();controller.signal.throwIfAborted();}],
  ['timeout',async()=>{const signal=AbortSignal.timeout(1);await new Promise(resolve=>setTimeout(resolve,5));signal.throwIfAborted();}],
 ]){
  const f=fixture(t);await assert.rejects(drive(planner,f.directory,'diagnostic task',{onEvent:f.onEvent}),error=>error.name===(reason==='timeout'?'TimeoutError':'AbortError'));
  assert.equal(terminal(f).reason,reason);
 }
});
test('actual driver policy refusal and server agent-mode refusal emit explicit reasons',async t=>{
 const f=fixture(t);const result=await drive(async()=>final(),f.directory,'diagnostic task',{tools:['unadvertised'],onEvent:f.onEvent});
 assert.equal(result.stop,'invalid-tools');assert.equal(terminal(f).reason,'refusal');
 const server=fixture(t),request=parseChatRequest('{"messages":[{"role":"user","content":"Run printf hello"}],"tools":[{"type":"function","function":{"name":"bash"}}]}');
 const outcome=await agenticOutcome({agentMode:false,logger:server.logger},request,()=>({answer:'refused'}));
 assert.equal(outcome.kind,'refused');assert.equal(terminal(server).reason,'refusal');
});
test('unclassified no-write finals remain Unknown and silent sinks preserve exact result',async t=>{
 const f=fixture(t);await drive(async()=>final(),f.directory,'diagnostic task',{onEvent:f.onEvent});assert.equal(terminal(f).reason,'Unknown');assert.deepEqual(fs.readdirSync(f.directory),[]);
 const quiet=fixture(t,{silent:true});const result=await drive(async()=>final(),quiet.directory,'diagnostic task',{onEvent:quiet.onEvent});assert.equal(result.answer,'No execution claim');assert.deepEqual(quiet.records(),[]);
});
