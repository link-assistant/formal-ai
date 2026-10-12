import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
await installNodeHost(new WorkerHost());
const source='export const tokens = [];\n';
const fixtures=[
 ['original report','Read input.mjs. Write a report about source in report.any.',true],
 ['repeated complete Read','Read input.mjs. Read input.mjs. Write a report about source in report.any.',true],
 ['unknown header action','Read input.mjs. Write a report and deploy production about source in report.any.',false],
 ['unknown header prerequisite','Read input.mjs. Write a report after unsupported approval about source in report.any.',false],
 ['unknown header permission','Read input.mjs. Write a report without permission about source in report.any.',false],
 ['embedded NoRead','Read input.mjs. Write a report do not read about source in report.any.',false],
 ['embedded NoWrite','Read input.mjs. Write a report do not write about source in report.any.',false],
 ['quoted topic data','Read input.mjs. Write a report about "do not write deploy production" in report.any.',true],
 ['technical topic atom','Read input.mjs. Write a report about source-v2 in report.any.',true],
 ['NoWrite report','Do not write. Read input.mjs. Write a report about source in report.any.',false],
 ['NoRead report','Do not read. Read input.mjs. Write a report about source in report.any.',false],
 ['unknown prefix','Perform an unsupported obligation. Read input.mjs. Write a report about source in report.any.',false],
 ['unowned create tail','Read input.mjs. Create something.',false],
 ['unknown report tail','Read input.mjs. Write a report about source in report.any and perform unsupported obligation.',false],
 ['unknown read suffix','Read input.mjs and perform unsupported obligation. Write a report about source in report.any.',false],
 ['source artifact exclusion','Add a source module to destination.any. Read input.mjs and report missing primitives.',false],
];
for(const [name,request,expected] of fixtures)test('read-led current source ownership: '+name,async()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'read-led-report-'));
 try{
  fs.writeFileSync(path.join(dir,'input.mjs'),source);
  const result=await drive(planChatStep,dir,request,{steps:12});
  assert.equal(fs.readFileSync(path.join(dir,'input.mjs'),'utf8'),source);
  assert.equal(fs.existsSync(path.join(dir,'report.any')),expected);
  assert.equal(fs.existsSync(path.join(dir,'destination.any')),false);
  if(expected){assert.ok(fs.readFileSync(path.join(dir,'report.any'),'utf8').includes(source.trim()));assert.ok(result.answer);}
  else assert.equal(result.transcript.some(call=>call.tool==='write'),false);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});
test('read-led current source ownership rejects caller invented report bytes',async()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'read-led-foreign-'));
 try{
  fs.writeFileSync(path.join(dir,'input.mjs'),source);
  const result=await drive(()=>({kind:'tool_calls',calls:[{tool:'write',arguments:JSON.stringify({file_path:'report.any',content:'invented bytes'})}]}),dir,fixtures[0][1],{steps:12});
  assert.equal(result.transcript.length,0);
  assert.equal(fs.existsSync(path.join(dir,'report.any')),false);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});
