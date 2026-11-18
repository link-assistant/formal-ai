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
import {spawnSync} from 'node:child_process';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {planWorkItemBoundaryStep} from '../../../js/agentic/work_item_boundary_candidate.mjs';
import {workItemEvidencePaths} from '../../../js/agentic/work_item_evidence.mjs';
import { workItemRecordOf, parsesAsJsonObject } from '../../../js/agentic/work_item_report.mjs';
function recorded(plan) {
  assert.equal(parsesAsJsonObject(plan.answer), false, 'a work-item answer is a report, not a JSON object');
  const record = plan.record ?? workItemRecordOf(plan);
  assert.ok(record, 'a work-item final keeps its record');
  return record;
}
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
    const report=recorded(result);
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
    const replayReport=recorded(replay);assert.equal(replayReport.noOp,true);assert.equal(replayReport.authored,false);
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
    const report=recorded(result);assert.equal(report.verified,false);assert.equal(report.authored,false);
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
    const report=recorded(plan);
    assert.equal(report.reason,expected,variant);assert.equal(report.authored,false);
    assert.equal(report.physicalReadbackVerified,false);assert.equal(report.complete,false);
    assert.equal(report.detail.budget.observed,variant==='totalBudget'?2:variant==='ambiguousRead'?0:1);
  }
});

// Issue #1483: with a Run tool the readback-verified artifact is executed
// against samples derived from the requested bound (not from the candidate);
// only that executed check verifies the bound requirement.
test('a Run tool executes the contract-derived check; a replay is a verified no-op',async()=>{
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-boundary-check-'));
  try{
    mkdirSync(join(directory,'src'));writeFileSync(join(directory,'src/ceiling.mjs'),source);
    const result=await drive(planChatStep,directory,task,{steps:12,tools:['read','edit','write','bash']});
    assert.equal(result.stop,'final');
    assert.deepEqual(result.transcript.map(row=>row.tool),['read','edit','read','bash']);
    const report=recorded(result);
    assert.equal(report.reason,'UsefulChangeVerified');
    assert.equal(report.verified,true);assert.equal(report.authored,true);assert.equal(report.complete,true);
    const check=report.checks.find(row=>row.kind==='contract-derived-execution');
    assert.equal(check.status,'passed');assert.equal(check.declaration,'accepts');
    assert.ok(report.requirements.every(row=>row.status==='verified'));
    assert.deepEqual(report.remainingWork,[]);
    // The localized report states the accepted change, its file and every check outcome.
    assert.match(result.answer,/^Done: the request\. The session stopped at `UsefulChangeVerified`/u);
    assert.ok(result.answer.includes('Files changed: `src/ceiling.mjs`.'));
    assert.ok(result.answer.includes('`contract-derived-execution` passed'));
    assert.ok(result.answer.includes('All requirements were verified (1).'));
    assert.ok(!result.answer.includes('not yet verified'));
    const replay=await drive(planChatStep,directory,task,{steps:8,tools:['read','edit','write','bash']});
    assert.deepEqual(replay.transcript.map(row=>row.tool),['read','bash']);
    const replayed=recorded(replay);
    assert.equal(replayed.reason,'VerifiedNoOp');assert.equal(replayed.noOp,true);
    assert.equal(replayed.authored,false);assert.equal(replayed.verified,true);
    assert.match(replay.answer,/^Already done: the request\./u);assert.ok(!replay.answer.includes('Files changed'));
  }finally{rmSync(directory,{recursive:true,force:true});}
});

test('a regression test over a source that meets the requested bound is written, passes on its own and replays as a no-op',async()=>{
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-boundary-regression-pass-'));
  const regression='Add a regression test in test/ceiling.test.mjs for the upper bound of function accepts in src/ceiling.mjs: the maximum is inclusive.';
  const inclusive=source.replace('value < maximum','value <= maximum');
  try{
    mkdirSync(join(directory,'src'));mkdirSync(join(directory,'test'));writeFileSync(join(directory,'src/ceiling.mjs'),inclusive);
    const result=await drive(planChatStep,directory,regression,{steps:12,tools:['read','edit','write','bash']});
    assert.equal(result.stop,'final');
    const report=recorded(result);
    assert.equal(report.reason,'UsefulChangeVerified');
    assert.equal(report.verified,true);assert.equal(report.authored,true);assert.equal(report.complete,true);
    assert.equal(report.checks.find(row=>row.kind==='contract-derived-execution').status,'passed');
    assert.ok(report.requirements.every(row=>row.status==='verified'));
    assert.match(result.answer,/^Done: the request\. The session stopped at `UsefulChangeVerified`/u);
    assert.ok(result.answer.includes('Files changed: `test/ceiling.test.mjs`.'));
    assert.equal(readFileSync(join(directory,'src/ceiling.mjs'),'utf8'),inclusive,'the source under test is not touched');
    // The written test is run by plain node, outside the controller.
    const run=spawnSync(process.execPath,['--test',join(directory,'test/ceiling.test.mjs')],{encoding:'utf8',timeout:60000});
    assert.equal(run.status,0,run.stdout+run.stderr);
    const replay=await drive(planChatStep,directory,regression,{steps:8,tools:['read','edit','write','bash']});
    const replayed=recorded(replay);
    assert.equal(replayed.reason,'VerifiedNoOp');assert.equal(replayed.noOp,true);assert.equal(replayed.authored,false);
    assert.ok(!replay.answer.includes('Files changed'));
  }finally{rmSync(directory,{recursive:true,force:true});}
});

test('a regression test over a source that contradicts the requested bound fails its check',async()=>{
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-boundary-regression-check-'));
  const regression='Add a regression test in test/ceiling.test.mjs for the upper bound of function accepts in src/ceiling.mjs: the maximum is inclusive.';
  try{
    mkdirSync(join(directory,'src'));mkdirSync(join(directory,'test'));writeFileSync(join(directory,'src/ceiling.mjs'),source);
    const result=await drive(planChatStep,directory,regression,{steps:12,tools:['read','edit','write','bash']});
    assert.equal(result.stop,'final');
    const report=recorded(result);
    assert.equal(report.reason,'IndependentCheckFailed');
    assert.equal(report.verified,false);assert.equal(report.complete,false);
    assert.equal(report.checks.find(row=>row.kind==='contract-derived-execution').status,'failed');
    assert.ok(report.requirements.every(row=>row.status!=='verified'));
    assert.match(result.answer,/^Not verified: the request\. The session stopped at `IndependentCheckFailed`/u);
    assert.ok(result.answer.includes('`contract-derived-execution` failed'));
    assert.ok(result.answer.includes('Requirements not yet verified (1):\n1. '));
  }finally{rmSync(directory,{recursive:true,force:true});}
});
