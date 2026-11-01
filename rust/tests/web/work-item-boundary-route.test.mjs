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
import {planWorkItemBoundaryStep} from '../../../js/agentic/work_item_boundary_candidate.mjs';
import {workItemEvidencePaths} from '../../../js/agentic/work_item_evidence.mjs';
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

// These deliberately inconsistent protocol fixtures test refusal and custody;
// they do not claim a real mutation or a held-out production repair.
function receipt(messages,tool,argumentsValue,content,sourceRead=null) {
  const id='boundary-guard-'+messages.length;
  messages.push({role:'assistant',tool_calls:[{id,type:'function',function:{name:tool,arguments:JSON.stringify(argumentsValue)}}]},
    {role:'tool',tool_call_id:id,name:tool,content,...(sourceRead?{source_read:sourceRead}:{})});
}
test('ambiguous reads or edits and unrelated mutations terminate before another effect',()=>{
  for(const [variant,expected] of [
    ['ambiguousRead','BoundaryAmbiguousReadPathAttempt'],
    ['ambiguousEdit','BoundaryUnrelatedOrAmbiguousMutationAttempt'],
    ['unrelatedEdit','BoundaryUnrelatedOrAmbiguousMutationAttempt'],
    ['totalBudget','BoundaryEffectAttemptBudgetExceeded'],
  ]) {
    const messages=[{role:'user',content:task}];
    receipt(messages,'read',{filePath:'src/ceiling.mjs'},source,
      {path:'src/ceiling.mjs',success:true,complete:true,format:'raw'});
    if(variant==='ambiguousRead')receipt(messages,'read',{filePath:'src/ceiling.mjs',path:'other.mjs'},source,
      {path:'src/ceiling.mjs',success:true,complete:true,format:'raw'});
    else {
      const argumentsValue={filePath:variant==='unrelatedEdit'||variant==='totalBudget'?'other.mjs':'src/ceiling.mjs',
        oldString:source,newString:source};
      if(variant==='ambiguousEdit')argumentsValue.path='other.mjs';
      receipt(messages,'edit',argumentsValue,'File edited successfully');
      if(variant==='totalBudget')receipt(messages,'edit',{filePath:'src/ceiling.mjs',oldString:source,newString:source},'File edited successfully');
    }
    const plan=planWorkItemBoundaryStep(task,messages,['read','edit','write'],null,workItemEvidencePaths);
    assert.equal(plan.kind,'final',variant);
    const report=JSON.parse(plan.answer);
    assert.equal(report.reason,expected,variant);assert.equal(report.authored,false);
    assert.equal(report.physicalReadbackVerified,false);assert.equal(report.complete,false);
    assert.equal(report.detail.budget.observed,variant==='totalBudget'?2:variant==='ambiguousRead'?0:1);
  }
});
