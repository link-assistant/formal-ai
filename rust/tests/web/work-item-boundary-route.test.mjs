// Intended owner: rust/tests/web/work-item-boundary-route.test.mjs.
// Actual maintained planner + filesystem driver; no patch or grader answer is
// included in the solver request. Hosted issue/client runs remain separate.
import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
before(async()=>installNodeHost(new WorkerHost({WebAssembly:{
  compile(){throw Error('LocalProductWasmForbidden');},instantiate(){throw Error('LocalProductWasmForbidden');},
}})));
const source=`// Keep this comment.
export function accepts(value,minimum,maximum) {
  return value >= minimum && value < maximum;
}
export function untouched(value) { return value * 3; }
`;
const task='Repair the upper-bound check in src/ceiling.mjs so the maximum is included. Preserve the lower bound, comments and other functions.';
const digest=value=>createHash('sha256').update(value).digest('hex');

test('production planner/drive performs a guarded structural leaf and retains external acceptance as pending',async()=>{
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-boundary-route-'));
  try{
    mkdirSync(join(directory,'src'));writeFileSync(join(directory,'src/ceiling.mjs'),source);
    const result=await drive(planChatStep,directory,task,{steps:12,tools:['read','edit','write'],allowedCommands:[]});
    assert.equal(result.stop,'final');
    assert.deepEqual(result.transcript.map(row=>row.tool),['read','edit','read']);
    const report=JSON.parse(result.answer);
    assert.equal(report.schema,'repository-work-item-controller/v1');
    assert.equal(report.taskIdentity,digest(task));assert.equal(report.workItem,null);
    assert.equal(report.physicalReadbackVerified,true);assert.equal(report.authored,true);
    assert.equal(report.verified,false);assert.equal(report.complete,false);assert.equal(report.independentlyAccepted,false);
    assert.ok(report.remainingWork.length>0);
    for(const requirement of report.requirements)assert.equal(Buffer.from(task).subarray(...requirement.span).toString(),requirement.text);
    // PRIVATE GRADER: only executes after solver interaction has ended.
    const artifact=await import(pathToFileURL(join(directory,'src/ceiling.mjs')).href);
    for(const [value,expected] of [[-1,false],[0,true],[3,true],[5,true],[6,false]])assert.equal(artifact.accepts(value,0,5),expected);
    assert.equal(artifact.untouched(7),21);
    const after=readFileSync(join(directory,'src/ceiling.mjs'),'utf8');
    assert.ok(after.includes('// Keep this comment.'));
    assert.ok(after.includes('export function untouched(value) { return value * 3; }'));
    const replay=await drive(planChatStep,directory,task,{steps:8,tools:['read','edit','write'],allowedCommands:[]});
    assert.equal(replay.stop,'final');assert.deepEqual(replay.transcript.map(row=>row.tool),['read']);
    const replayReport=JSON.parse(replay.answer);assert.equal(replayReport.noOp,true);assert.equal(replayReport.authored,false);
    assert.equal(readFileSync(join(directory,'src/ceiling.mjs'),'utf8'),after);
  }finally{rmSync(directory,{recursive:true,force:true});}
});

test('an unsupported expression stays an explicit gap with no physical edit',async()=>{
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-boundary-unsupported-'));
  const unsupported=source.replace('value < maximum','value < maximum + 1');
  try{
    mkdirSync(join(directory,'src'));writeFileSync(join(directory,'src/ceiling.mjs'),unsupported);
    const result=await drive(planChatStep,directory,task,{steps:8,tools:['read','edit','write'],allowedCommands:[]});
    assert.equal(result.stop,'final');assert.deepEqual(result.transcript.map(row=>row.tool),['read']);
    assert.equal(readFileSync(join(directory,'src/ceiling.mjs'),'utf8'),unsupported);
    const report=JSON.parse(result.answer);assert.equal(report.verified,false);assert.equal(report.authored,false);
    assert.equal(report.complete,false);assert.ok(report.remainingWork.length>0);
  }finally{rmSync(directory,{recursive:true,force:true});}
});
