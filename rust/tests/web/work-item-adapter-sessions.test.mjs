// Issue #1483: the boundary-repair work item through the Claude (Anthropic
// Messages) and Codex (Responses) adapters, without an LLM. Each returned tool
// call is executed against a scratch workspace; the request carries no patch.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {adapterSession,boundaryWorkspace,boundaryTask,hivePrompts,ghShim} from './helpers/work-item-adapter-session.mjs';

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
// The ordinary Hive Mind prompts name only the issue address; the controller
// reads the issue itself. Its title restates the defect without a bound
// contract, so it stays an open requirement and the report is partial.
for(const protocol of ['claude','codex']){
  test(`${protocol} with the ordinary Hive prompts: the fetched issue ends with a checked edit, then a verified no-op`,{timeout:120000},async()=>{
    const workspace=boundaryWorkspace();
    try{
      const url='https://github.com/example/ceiling/issues/7';
      const gh=ghShim(workspace.root,{title:'Ceiling check excludes the maximum',body:boundaryTask});
      const prompts=hivePrompts(protocol,{issue_url:url,issue:7,pull_url:'https://github.com/example/ceiling/pull/8',pull:8,
        branch:'issue-7-0123456789ab',workspace:workspace.directory,workspace_tmp:join(workspace.root,'tmp'),owner:'example',repo:'ceiling'});
      assert.ok(!prompts.user.includes(boundaryTask)&&!prompts.system.includes(boundaryTask));
      const first=await adapterSession(protocol,workspace.directory,prompts.user,{system:prompts.system,env:gh.env});
      assert.match(JSON.stringify(first.trail[0].args),/gh issue view/u);
      assert.match(first.final,/^Partly done: https:\/\/github\.com\/example\/ceiling\/issues\/7\. The session stopped at `UsefulChangeVerified`/u,first.final);
      assert.ok(first.final.includes('Files changed: `src/ceiling.mjs`.')&&first.final.includes('`contract-derived-execution` passed'));
      assert.ok(first.final.includes('1. Ceiling check excludes the maximum'),first.final);
      assert.ok(first.trail.every(row=>row.status===0),JSON.stringify(first.trail));
      const after=readFileSync(join(workspace.directory,'src/ceiling.mjs'),'utf8');
      assert.ok(after.includes(repaired),after);
      const repeat=await adapterSession(protocol,workspace.directory,prompts.user,{system:prompts.system,env:gh.env});
      assert.match(repeat.final,/^Partly done already: .* The session stopped at `VerifiedNoOp`/u,repeat.final);
      assert.ok(!repeat.final.includes('Files changed'));
      assert.equal(readFileSync(join(workspace.directory,'src/ceiling.mjs'),'utf8'),after);
      assert.ok(gh.calls().every(call=>call.startsWith('issue view '+url+' --json ')),gh.calls().join('\n'));
    }finally{workspace.dispose();}
  });
}
