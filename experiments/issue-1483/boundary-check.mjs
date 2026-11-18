// Drives the planner (no LLM) on a boundary work item with a Run tool, so the
// contract-derived check executes, then prints transcript, reason and checks.
// Usage: KIND=repair|regression [BROKEN=1] [REPLAY=1] node experiments/issue-1483/boundary-check.mjs
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {planChatStep} from '../../js/agentic/planner.mjs';
import {drive} from '../js_dogfood/drive.mjs';
import {workItemRecordOf} from '../../js/agentic/work_item_report.mjs';
await installNodeHost(new WorkerHost({}));
const kind=process.env.KIND??'repair';
const directory=mkdtempSync(join(tmpdir(),'formal-ai-boundary-check-'));
const comparison=process.env.BROKEN?'<':'<=';
const source=`// Keep this comment.\nexport function accepts(value,minimum,maximum) {\n  return value >= minimum && value ${kind==='repair'?'<':comparison} maximum;\n}\nexport function untouched(value) { return value * 3; }\n`;
mkdirSync(join(directory,'src'));mkdirSync(join(directory,'test'));writeFileSync(join(directory,'src/ceiling.mjs'),source);
const task=kind==='repair'
  ?'Repair the upper-bound check in src/ceiling.mjs so the maximum is included. Preserve the lower bound, comments and other functions.'
  :'Add a regression test in test/ceiling.test.mjs for the upper bound of function accepts in src/ceiling.mjs: the maximum is inclusive.';
const show=async label=>{
  const result=await drive(planChatStep,directory,task,{steps:12,tools:['read','edit','write','bash']});
  const record=result.record??workItemRecordOf(result);
  console.log(label,result.stop,result.transcript.map(row=>row.tool));
  console.log(String(result.answer).split('\n').slice(0,12).join('\n'));
  console.log(JSON.stringify({reason:record?.reason,verified:record?.verified,complete:record?.complete,noOp:record?.noOp,authored:record?.authored,
    requirements:record?.requirements?.map(row=>row.status),checks:record?.checks?.map(row=>[row.kind,row.status])}));
};
try{await show('first');if(process.env.REPLAY)await show('replay');}finally{rmSync(directory,{recursive:true,force:true});}
