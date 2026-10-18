import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {host} from '../../../js/agentic/host.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {decodeTyped} from '../../../js/logging/typed-codec.mjs';
import {verifyDriveEvidence,retainedEvidenceBytes} from '../../../js/provenance/verify-drive.mjs';
import {recomputePlannerEvidence} from '../../../js/provenance/replay-planner.mjs';
import {withEvidenceContext} from '../../../js/provenance/capture-context.mjs';
import {recordFor,toLino,answerDerivationId} from '../../../js/agentic/crate/derivation.mjs';
before(async()=>installNodeHost(new WorkerHost({WebAssembly:{compile(){throw Error('LocalProductWasmForbidden');},instantiate(){throw Error('LocalProductWasmForbidden');}}})));
test('actual planner EventLog and canonical derivation are retained and real decisions are recomputed',async t=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'engine-observation-'));
 t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
 const workspace=path.join(directory,'work');fs.mkdirSync(workspace);
 const result=await drive(planChatStep,workspace,'Create file trace.mjs containing export const unique = 43;',
  {steps:12,provenance:{directory:path.join(directory,'evidence'),environment:{}}});
 assert.equal(result.stop,'final');
 const records=fs.readFileSync(path.join(result.provenance.directory,'chain.lino'),'utf8').trimEnd().split('\n').map(decodeTyped);
 const thinking=records.filter(row=>row.kind==='planner_thinking');assert(thinking.length>0);
 for(const record of thinking){
  const value=JSON.parse(retainedEvidenceBytes(result.provenance.directory,record.payload.evidence));
  assert.equal(value.schema,'formal-ai-planner-observation-v1');
  assert.equal(value.eventLog[0].kind,'impulse');
  assert(value.eventLog.some(event=>event.kind==='agentic:resolved'));
  assert.equal(value.eventLog.at(-1).kind,'render:emit');
  assert.deepEqual(JSON.parse(value.eventLog.at(-1).payload),value.projected);
  assert.equal(value.derivation,toLino(recordFor(value.eventLog,answerDerivationId(value.eventLog.at(-1).payload))));
 }
 const checked=verifyDriveEvidence(result.provenance.directory,workspace,result.provenance.anchor);
 assert.equal(checked.plannerObservations,thinking.length);
 const replay=await recomputePlannerEvidence(result.provenance.directory,workspace,{expectedAnchor:result.provenance.anchor});
 assert.equal(replay.plannerComputation,'matched',JSON.stringify(replay));
 assert.equal(replay.comparisons.length,thinking.length);
 assert.equal(replay.fullReplay.verified,false);
 assert.equal(replay.externalEffects,'not-reexecuted');
 assert.equal(fs.readFileSync(path.join(workspace,'trace.mjs'),'utf8'),'export const unique = 43;');
});
test('actual worker solve retains its own published events and source-owned derivation',async()=>{
 const observed=[];
 const result=await withEvidenceContext({active:true,engineObservation:value=>observed.push(value)},()=>host().solve('What is 19 + 23?',[]));
 assert.equal(observed.length,1);
 assert.equal(observed[0].schema,'formal-ai-worker-observation-v1');
 assert.equal(observed[0].symbolic.answer,result.answer);
 assert(observed[0].eventLog.length>0);
 assert.deepEqual(observed[0].eventLog,result.solver_events);
 assert.equal(observed[0].derivation,toLino(recordFor(result.solver_events,answerDerivationId(result.answer))));
});
