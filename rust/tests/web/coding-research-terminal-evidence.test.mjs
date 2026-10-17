// Conversations and receipts are synthetic; expected witnesses were observed from the production JS planner.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
const index=JSON.parse(fs.readFileSync(new URL('../fixtures/coding-research/terminal-evidence-index.json',import.meta.url)));
const fixture={rows:index.shards.flatMap(name=>JSON.parse(fs.readFileSync(new URL('../fixtures/coding-research/'+name,import.meta.url))).rows)};
await installNodeHost(new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}}));
test('actual production JS retains all captured held-out terminal, no-op, failed and incomplete witnesses',async()=>{
  for(const row of fixture.rows){
    const plan=await planChatStepResolved(row.messages,row.tools);
    assert.deepEqual(JSON.parse(JSON.stringify(plan)),row.expected,row.fixture+':'+row.scenario+':'+row.phase);
    if(row.phase==='tool')assert(plan.research && !plan.research.verified);
    else if(row.scenario==='incomplete'||row.scenario==='module-incomplete'){
      assert.equal(plan.result.disposition,'gap');assert.equal(plan.result.discovery.verified,false);
    }else{
      assert.equal(plan.research.verified,row.scenario!=='failed');
      if(row.scenario==='existing')assert.equal(plan.research.sourceAuthored,false);
    }
  }
});
