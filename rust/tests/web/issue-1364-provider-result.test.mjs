import {before,test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {reportedExitCode,stepOutcome,StepOutcome,normalizedPayload,render} from '../../../js/agentic/tool_result.mjs';
const packet=JSON.parse(fs.readFileSync(new URL('../fixtures/issue-1364-agent-tool-use.json',import.meta.url)));
const original=packet.fixture;
before(async()=>{await installNodeHost(new WorkerHost());});
test('captured complete Agent result preserves actual nested exit127 and diagnostic',()=>{
 const raw=JSON.stringify(original);
 assert.equal(reportedExitCode(raw),127);
 assert.equal(stepOutcome(raw),StepOutcome.Failed);
 assert.equal(normalizedPayload(raw),null);
 assert.doesNotMatch(render('scalac Main.scala',raw,'Explain the observed result in English.'),/no exit code/iu);
});
test('typed metadata and labelled errors preserve general exit values and conflicts',()=>{
 for(const code of [1,13,42,127,-1]){
  const frame=structuredClone(original);frame.part.tool='exec';frame.part.state.error='The compiler is unavailable.';frame.part.state.metadata.exit=code;
  assert.equal(reportedExitCode(JSON.stringify(frame)),code);
  assert.equal(stepOutcome(JSON.stringify(frame)),StepOutcome.Failed);
 }
 const fallback=structuredClone(original);delete fallback.part.state.metadata.exit;
 assert.equal(reportedExitCode(JSON.stringify(fallback)),127);
 fallback.part.state.metadata.exit=0;
 assert.equal(reportedExitCode(JSON.stringify(fallback)),127);
});
function successful(){const frame=structuredClone(original);frame.part.state.status='completed';delete frame.part.state.error;frame.part.state.metadata.exit=0;frame.part.state.metadata.output='tool output';return frame;}
test('completed output survives and every authoritative outer failure still vetoes it',()=>{
 const frame=successful();assert.equal(stepOutcome(JSON.stringify(frame)),StepOutcome.Succeeded);assert.equal(normalizedPayload(JSON.stringify(frame)),'tool output');
 for(const extra of [{status:'error'},{outcome:'failed'},{status_code:500},{success:false},{ok:false},{isError:true},{is_error:true},{timed_out:true},{aborted:true},{signal:'SIGTERM'},{error:'transport failure'}]){
  assert.equal(stepOutcome(JSON.stringify({...successful(),...extra})),StepOutcome.Failed,JSON.stringify(extra));
 }
 for(const extra of [{timedOut:true},{aborted:true},{signal:'SIGTERM'}]){
  const changed=successful();Object.assign(changed.part.state.metadata,extra);assert.equal(stepOutcome(JSON.stringify(changed)),StepOutcome.Failed,JSON.stringify(extra));
 }
});
test('arbitrary source JSON and nonterminal or malformed parts grant no nested exit',()=>{
 for(const frame of [{state:original.part.state},{type:'note',part:original.part}, {...original,part:null},{...original,part:[]}, {...original,part:'untrusted'}])assert.equal(reportedExitCode(JSON.stringify(frame)),null);
 const pending=successful();pending.part.state.status='running';assert.equal(reportedExitCode(JSON.stringify(pending)),null);
 const stopped={...successful(),status:'pending'};assert.equal(reportedExitCode(JSON.stringify(stopped)),null);
});
