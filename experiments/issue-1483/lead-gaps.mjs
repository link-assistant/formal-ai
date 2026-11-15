// Drives planWorkItemEvidence on the frozen 1395/1396/1438/1364 issue bodies with
// every fetch answered as unavailable and every read as missing; prints each step.
import {readFileSync} from 'node:fs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {planWorkItemEvidence} from '../../js/agentic/work_item_evidence.mjs';
import {workItemRecordOf} from '../../js/agentic/work_item_report.mjs';
await installNodeHost(new WorkerHost({}));
const issues=JSON.parse(readFileSync('data/benchmarks/fixtures/work-item-1483/issues.json','utf8')).issues;
for(const n of process.argv.slice(2)){
  const task=issues[n].title+'\n\n'+issues[n].body,messages=[{role:'user',content:task}];
  for(let step=0;step<12;step++){
    const plan=planWorkItemEvidence(task,messages,['read','grep','glob','webfetch','write'],issues[n].url);
    if(!plan){console.log(n,'null');break;}
    if(plan.kind==='final'){const r=plan.record??workItemRecordOf(plan);console.log(n,r.reason,r.gap,JSON.stringify(r.unresolved?.map(u=>[u.kind,u.reason])));break;}
    const call=plan.calls[0],id='c'+step;console.log('  ',call.tool,call.arguments.slice(0,140));
    messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]},
      {role:'tool',tool_call_id:id,name:call.tool,content:call.tool==='webfetch'?'Error: network access is not available':'Error: File not found'});
  }
}
