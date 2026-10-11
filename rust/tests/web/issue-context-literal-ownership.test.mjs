import assert from 'node:assert/strict';
import {test,before} from 'node:test';
import {readFileSync,mkdtempSync,existsSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {goalLedger} from '../../../js/agentic/planner/owned_goals.mjs';

const original='For existing issue-730, create file issue-730-finding.md containing provenance paths are portable';

before(async()=>{
  await installNodeHost(new WorkerHost());
});

async function run(request,path,expected){
  const directory=mkdtempSync(join(tmpdir(),'issue-context-original-'));
  try{
    const out=await drive(planChatStep,directory,request,{steps:8});
    assert.equal(readFileSync(join(directory,path),'utf8'),expected);
    assert.equal(out.stop,'final');
    assert.ok(out.transcript.some(item=>
      item.tool==='bash'&&JSON.parse(item.arguments).command==='cat '+path
    ));
    return out;
  }finally{
    rmSync(directory,{recursive:true,force:true});
  }
}

test('unchanged original native CLI contract writes exact bytes and obtains independent readback',async()=>{
  await run(original,'issue-730-finding.md','provenance paths are portable');
  assert.equal(goalLedger(original),null);
});

for(const [prefix,path,payload] of [
  ['For existing issue-9821,','another.txt','different payload'],
  ['FOR EXISTING ISSUE #42,','finding.md','rename X to Y suffix'],
  ['\tFor existing issue-9,\t','unicode.txt','томаты α']
])test('variable issue context '+prefix,async()=>{
  await run(prefix+' create file '+path+' containing '+payload,path,payload);
});

for(const prefix of [
  'Delete old.txt,',
  'For invented issue-730,',
  'For existing issue-730 delete old.txt,',
  'According to the source,',
  'For existing issue-X,',
  'For existing issue-730\u00a0,'
])test('unconsumed or attributed prefix stays refused '+prefix,()=>{
  const goals=goalLedger(prefix+' create file refused.txt containing alpha');
  assert.ok(goals?.some(goal=>goal.kind==='unsupported'));
});

test('unresolved read prerequisite remains refused before write',async()=>{
  const directory=mkdtempSync(join(tmpdir(),'issue-context-read-'));
  try{
    const out=await drive(
      planChatStep,
      directory,
      'For existing issue-73, read missing.txt before creating file refused.txt containing alpha',
      {steps:5}
    );
    assert.equal(existsSync(join(directory,'refused.txt')),false);
    assert.equal(out.transcript.some(item=>item.tool==='write'),false);
  }finally{
    rmSync(directory,{recursive:true,force:true});
  }
});

for(const request of [
  'Do not write. For existing issue-730, create file finding.md containing alpha',
  'Do not write. Create file finding.md containing alpha',
  'Resolve an unspecified prerequisite. For existing issue-811, create file finding.md containing alpha',
  'Deploy the unknown service. Create file finding.md containing alpha'
])test('whole request retains leading unsupported prerequisite before effects '+request,async()=>{
  const directory=mkdtempSync(join(tmpdir(),'issue-context-policy-'));
  try{
    const out=await drive(planChatStep,directory,request,{steps:8});
    assert.equal(existsSync(join(directory,'finding.md')),false);
    assert.equal(out.transcript.length,0);
    assert.equal(out.stop,'final');
    assert.match(out.answer,/no_artifact_in_clause/);
    assert.doesNotMatch(out.answer,/Created and verified/);
  }finally{
    rmSync(directory,{recursive:true,force:true});
  }
});
