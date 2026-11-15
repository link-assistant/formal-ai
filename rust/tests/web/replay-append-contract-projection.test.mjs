import test from 'node:test';
import assert from 'node:assert/strict';
import {projectDriverHistory} from '../../../experiments/js_dogfood/drive.mjs';
import {appendRecordStep,APPEND_MODE} from '../../../js/agentic/append_contract.mjs';
import {driverAppendContracts,boundDriverAppendGrant,literalJournalCallsMatch} from '../../../js/provenance/literal-journal-authority.mjs';

import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
const forbidden=()=>{throw Error('NativeExecutionForbidden');};
await installNodeHost(new WorkerHost({WebAssembly:{compile:forbidden,instantiate:forbidden,instantiateStreaming:forbidden}}),{sourceUsageRustGrammar:false});

const tools=['write','read'];
const messages=[{role:'user',content:'Write the selected source file.'}];
const pending=history=>appendRecordStep(history,tools,'plan.lino','general_change_plan\n  target "retained.mjs"\n','record-17');

test('replay preserves the recorded append grant when independently checking canonical journal calls',()=>{
 const recorded=projectDriverHistory(messages,driverAppendContracts(tools,true));
 const before=JSON.stringify(recorded),canonical=pending(recorded).plan;
 assert.equal(JSON.parse(canonical.calls[0].arguments).append_mode,APPEND_MODE);
 const lost=pending(projectDriverHistory(recorded,[]));
 assert.equal(lost,null,'the previous replay projection discarded the actual provider contract');
 const enabled=boundDriverAppendGrant(true,true);
 const replay=pending(projectDriverHistory(recorded,driverAppendContracts(tools,enabled))).plan;
 assert.deepEqual(replay,canonical);
 assert.equal(JSON.stringify(recorded),before);
 assert.equal(literalJournalCallsMatch({target:'retained.mjs'},canonical.calls,replay),true);
 const supplied=JSON.parse(canonical.calls[0].arguments);
 const forged={...canonical.calls[0],arguments:JSON.stringify({...supplied,append_request_id:'foreign'})};
 assert.equal(literalJournalCallsMatch({target:'retained.mjs'},[forged],replay),false);
});

test('recorded metadata cannot grant append without the matching configured provider authority',()=>{
 for(const [recorded,configured] of [[true,false],[false,true],['true',true],[1,1],[{},{}]]){
  assert.throws(()=>boundDriverAppendGrant(recorded,configured),/FreshReplayRuntimeContextMismatch/u);
 }
 assert.equal(boundDriverAppendGrant(undefined,false),false);
 assert.equal(boundDriverAppendGrant(false,undefined),false);
 const forged=projectDriverHistory(messages,driverAppendContracts(tools,true));
 for(const grant of [false,'true',1]){
  assert.equal(pending(projectDriverHistory(forged,driverAppendContracts(tools,grant))),null);
 }
 assert.deepEqual(driverAppendContracts(['read'],true),[]);
});
