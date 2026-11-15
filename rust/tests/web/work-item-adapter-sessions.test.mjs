// Issue #1483: the boundary-repair work item through the Claude (Anthropic
// Messages) and Codex (Responses) adapters, without an LLM. Each returned tool
// call is executed against a scratch workspace; the request carries no patch.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {adapterSession,boundaryWorkspace,boundaryTask} from './helpers/work-item-adapter-session.mjs';

const repaired='  return value >= minimum && value <= maximum;\n';
for(const protocol of ['claude','codex']){
  test(`${protocol}: the work item ends with an edit and a passing independent check; a repeat is a verified no-op`,{timeout:120000},async()=>{
    const workspace=boundaryWorkspace();
    try{
      const first=await adapterSession(protocol,workspace.directory,boundaryTask);
      assert.match(first.final,/^Done: the request\. The session stopped at `UsefulChangeVerified`/u,first.final);
      assert.ok(first.final.includes('Files changed: `src/ceiling.mjs`.'));
      assert.ok(first.final.includes('`contract-derived-execution` passed'));
      assert.ok(first.trail.every(row=>row.status===0),JSON.stringify(first.trail));
      const after=readFileSync(join(workspace.directory,'src/ceiling.mjs'),'utf8');
      assert.ok(after.includes(repaired),after);
      assert.ok(after.startsWith('// Keep this comment.\n')&&after.includes('export function untouched(value) { return value * 3; }'));
      const repeat=await adapterSession(protocol,workspace.directory,boundaryTask);
      assert.match(repeat.final,/^Already done: the request\. The session stopped at `VerifiedNoOp`/u,repeat.final);
      assert.ok(!repeat.final.includes('Files changed'));
      assert.equal(readFileSync(join(workspace.directory,'src/ceiling.mjs'),'utf8'),after);
    }finally{workspace.dispose();}
  });
}
test('claude: a Read listing with no completeness proof is read again through the Run transport',{timeout:120000},async()=>{
  const workspace=boundaryWorkspace();
  try{
    const session=await adapterSession('claude',workspace.directory,boundaryTask);
    assert.deepEqual(session.trail.slice(0,3).map(row=>row.tool),['Read','Bash','Edit']);
    assert.match(session.trail[1].args.command,/^node -e /u);
  }finally{workspace.dispose();}
});
test('claude without a Run tool: an uncertified Read stays an explicit gap and nothing is written',{timeout:120000},async()=>{
  const workspace=boundaryWorkspace();
  try{
    const before=readFileSync(join(workspace.directory,'src/ceiling.mjs'),'utf8');
    const session=await adapterSession('claude',workspace.directory,boundaryTask,{tools:['Read','Edit','Write']});
    assert.deepEqual(session.trail.map(row=>row.tool),['Read']);
    assert.match(session.final,/^Not completed: the request\. The session stopped at `IncompleteSourceObservation`/u,session.final);
    assert.equal(readFileSync(join(workspace.directory,'src/ceiling.mjs'),'utf8'),before);
  }finally{workspace.dispose();}
});
