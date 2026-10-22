import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {contentDigest, runtimeImageDigest, requireReplayTask, adaptReplayEntrypoint, taskSnapshot,
  requireNoopTask, requireActualTaskCi, requireTaskObligations, canonicalCapturedMessages,
  auditCapturedActions} from '../../../scripts/lib/issue-1364-replay-contract.mjs';
await installNodeHost(new WorkerHost());
const call = (id='one', name='Bash', args='{"command":"inspect public task"}') =>
  ({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name,arguments:args}}]});
const receipt = (id='one', failed=false) => ({role:'tool',tool_call_id:id,
  name:'Bash',content:failed?'Error: observed fixture failure':'ok',is_error:failed});
function context(family) {
  const bodies = {
    chat:[{role:'user',content:'public task'},call(),receipt(),{role:'assistant',content:'done'}],
    anthropic:[{role:'user',content:'public task'},
      {role:'assistant',content:[{type:'tool_use',id:'one',name:'Bash',input:{command:'inspect public task'}}]},
      {role:'user',content:[{type:'tool_result',tool_use_id:'one',content:'ok'}]},
      {role:'assistant',content:'done'}],
    responses:[{role:'user',content:'public task'},
      {type:'function_call',call_id:'one',name:'Bash',arguments:'{"command":"inspect public task"}'},
      {type:'function_call_output',call_id:'one',output:'ok'},
      {type:'message',role:'assistant',content:'done'}],
  };
  const suffix = {chat:'/chat/completions',anthropic:'/messages',responses:'/responses'}[family];
  return {metadata:{exchange_count:2},messages:bodies[family],server_logs:[
    {method:'POST',path:'/v1'+suffix,request_body:'{}',response_tool_calls:[{name:'Bash',arguments:{command:'inspect public task'}}]},
    {method:'POST',path:'/v1'+suffix,request_body:'{}',response_tool_calls:[]}]};
}
for(const family of ['chat','anthropic','responses']) {
  test(family+' capture uses production normalization and binds exact tool identities',()=>{
    const value=auditCapturedActions(canonicalCapturedMessages(context(family)));
    assert.equal(value.calls,1);assert.equal(value.completed,1);assert.equal(value.writes,0);
  });
  for(const [label,change] of [
    ['lost exchange',value=>{value.server_logs.pop();}],
    ['lost tool history',value=>{value.messages=[{role:'user',content:'public task'}];}],
    ['unfinished tool response',value=>{value.server_logs.at(-1).response_tool_calls=[{name:'Bash',arguments:{}}];}],
    ['mixed protocol',value=>{value.server_logs.at(-1).path='/unknown';}],
  ])test(family+' refuses '+label,()=>{const value=context(family);change(value);assert.throws(()=>canonicalCapturedMessages(value));});
}
test('unresolved, duplicate and wrong-owner results cannot be called a complete tool trace',()=>{
  for(const messages of [[call()],[call(),call(),receipt()],
    [receipt()],[call(),receipt(),receipt()],
    [call(),{...receipt(),name:'Read'}]])assert.throws(()=>auditCapturedActions(messages));
});
test('reused sequential IDs are valid and failed identical actions remain bounded across sessions',()=>{
  const failures=new Map();
  assert.equal(auditCapturedActions([call(),receipt('one',true),call(),receipt('one',true)],failures)
    .maximumIdenticalFailures,2);
  assert.throws(()=>auditCapturedActions([call(),receipt('one',true)],failures),/two identical/u);
});
test('canonical argument order cannot reopen a failed action budget',()=>{
  const failures=new Map();
  for(const args of ['{"a":1,"b":2}','{"b":2,"a":1}'])
    auditCapturedActions([call('one','Bash',args),receipt('one',true)],failures);
  assert.throws(()=>auditCapturedActions([call('one','Bash','{"a":1,"b":2}'),receipt('one',true)],failures));
});
test('source mutation tools are explicitly counted rather than hidden in a successful receipt',()=>{
  const messages=[call('write','Write','{"file_path":"main.rs","content":"public"}'),
    {role:'tool',tool_call_id:'write',name:'Write',content:'ok'}];
  assert.equal(auditCapturedActions(messages).writes,1);
});
test('actual task CI must be nonempty and terminal green',()=>{
  requireActualTaskCi([{__typename:'CheckRun',status:'COMPLETED',conclusion:'SUCCESS'}]);
  for(const checks of [[],[{status:'IN_PROGRESS',conclusion:null}],
    [{status:'COMPLETED',conclusion:'SKIPPED'}],
    [{__typename:'StatusContext',state:'PENDING'}]])assert.throws(()=>requireActualTaskCi(checks));
});
test('historical task binding rejects edited requirements, another PR and another branch',()=>{
  const body='Exact public task\n';
  const inputs={schema:'formal-ai-issue-1364-client-replay-inputs/v1',clients:{codex:{body,
    bodySha256:contentDigest(body),issueUrl:'https://github.com/public/task/issues/1',
    pullRequestUrl:'https://github.com/public/task/pull/2',branch:'issue-1-original'}}};
  const issue={html_url:inputs.clients.codex.issueUrl,body};
  const pr={html_url:inputs.clients.codex.pullRequestUrl,state:'open',head:{ref:'issue-1-original',sha:'a'.repeat(40)},base:{ref:'main'}};
  requireReplayTask(inputs,'codex',issue,pr);
  assert.throws(()=>requireReplayTask(inputs,'codex',{...issue,body:body+'changed'},pr));
  for(const change of [value=>{value.head.ref='another';},value=>{value.base.ref='another';},
    value=>{value.html_url='https://github.com/public/task/pull/3';}]){
    const value=structuredClone(pr);change(value);assert.throws(()=>requireReplayTask(inputs,'codex',issue,value));
  }
});
test('unchanged Git bytes alone cannot hide a second-pass source write or commit',()=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-1364-source-observation-'));
  try{
    const file=path.join(directory,'main.rs');fs.writeFileSync(file,'// public UTF-8 Привет\n');
    const before={head:'a'.repeat(40),branch:'issue-1-original',status:'',commitRecords:0,files:taskSnapshot(directory,['main.rs'])};
    requireNoopTask(before,{...before,files:taskSnapshot(directory,['main.rs'])});
    assert.throws(()=>requireNoopTask(before,{...before,commitRecords:1}));
    const original=fs.readFileSync(file);fs.unlinkSync(file);fs.writeFileSync(file,original);
    assert.throws(()=>requireNoopTask(before,{...before,files:taskSnapshot(directory,['main.rs'])}));
    fs.writeFileSync(path.join(directory,'binary'),Buffer.from([0,1]));assert.throws(()=>taskSnapshot(directory,['binary']));
    fs.symlinkSync(file,path.join(directory,'alias'));assert.throws(()=>taskSnapshot(directory,['alias']));
    assert.throws(()=>taskSnapshot(directory,['../escape']));
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
test('workflow and explanatory source obligations remain separate from the stdout oracle',()=>{
  const value={source:'// Print one greeting followed by a newline.\n// Build: rustc main.rs -o hello; run ./hello\n',
    programPath:'main.rs',workflow:'on: [push, pull_request]\njobs:\n  verify:\n    steps:\n      - run: bash tests/verify-output.sh\n',
    verifier:'#!/bin/sh\nset -eu\ncmp expected actual\n'};
  requireTaskObligations(value);
  for(const change of [{source:'fn main() {}'},{workflow:'on: push\n'},
    {verifier:'echo always-success\n'}])assert.throws(()=>requireTaskObligations({...value,...change}));
});
test('transport adaptation preserves every original harness assertion and treats shell punctuation as data',()=>{
  const original="const dockerArgv = buildDockerArgv({ solveArgv: buildE2eSolveArgv({ issueUrl, tool, model, baseBranch, logDir: '/home/box/logs' }), solveCommand: '/opt/hive-e2e/src/solve.mjs', image, hostLogDir, forwardEnv: ['GH_TOKEN', ...modelKeys, 'FORMAL_AI_INSTALL_GRANT', 'GITHUB_ACTIONS'] });";
  const source='const oracle = "unchanged";\n'+original+'\nindependentOriginalOracle();';
  const adapted=adaptReplayEntrypoint(source,{pullRequestUrl:'https://github.com/public/task/pull/2'},'/tmp/owned-$&-$`-$\'-$$');
  assert(adapted.startsWith('const oracle = "unchanged";\n'));
  assert(adapted.endsWith('\nindependentOriginalOracle();'));
  assert(adapted.includes('extraMounts:'));
  assert(adapted.includes('pinned attachment flag changed'));
  assert.throws(()=>adaptReplayEntrypoint(source+original,{},'/tmp/owned'));
  assert.throws(()=>adaptReplayEntrypoint(source,{},'/tmp/unsafe:path'));
});

test('released references and actual candidate content IDs bind the same bare receipt digest',()=>{
  const digest='sha256:'+'f'.repeat(64);
  assert.equal(runtimeImageDigest(digest),digest);
  assert.equal(runtimeImageDigest('ghcr.io/link-assistant/formal-ai@'+digest),digest);
  for(const reference of ['ghcr.io/link-assistant/formal-ai:latest','other.example/formal-ai@'+digest,
    'sha256:short'])assert.throws(()=>runtimeImageDigest(reference));
});

test('the real effect runner refuses a local context before any API, Docker or filesystem effect',async()=>{
  const {runIssue1364Replay}=await import('../../../scripts/run-issue-1364-client-replay.mjs');
  await assert.rejects(runIssue1364Replay({},{}),/hosted-only/u);
});


test('task replay keeps optional Hive attribution bookkeeping outside the task commit without bypassing repository hooks', () => {
  const original = "const dockerArgv = buildDockerArgv({ solveArgv: buildE2eSolveArgv({ issueUrl, tool, model, baseBranch, logDir: '/home/box/logs' }), solveCommand: '/opt/hive-e2e/src/solve.mjs', image, hostLogDir, forwardEnv: ['GH_TOKEN', ...modelKeys, 'FORMAL_AI_INSTALL_GRANT', 'GITHUB_ACTIONS'] });";
  const adapted = adaptReplayEntrypoint(original, {pullRequestUrl: 'https://github.com/public/task/pull/2'}, '/tmp/owned');
  const construct = new Function('buildE2eSolveArgv', 'buildDockerArgv', 'issueUrl', 'tool',
    'model', 'image', 'hostLogDir', 'modelKeys', adapted + '\nreturn dockerArgv;');
  const invoke = extra => construct(() => ['https://github.com/public/task/issues/1', '--attach-logs', '--keep', ...extra],
    value => value, 'https://github.com/public/task/issues/1', 'codex', 'formal-ai', 'task-image', '/tmp/logs', []);
  const result = invoke([]);
  const position = result.solveArgv.indexOf('--attribution');
  assert.notEqual(position, -1, 'explicit supported attribution policy required');
  assert.equal(result.solveArgv[position + 1], 'none');
  assert.equal(result.solveArgv.filter(value => value === '--attribution').length, 1);
  assert.equal(result.solveArgv[0], 'https://github.com/public/task/pull/2');
  assert(result.solveArgv.includes('--keep'));
  assert(!result.solveArgv.includes('--attach-logs'));
  assert(!result.solveArgv.some(value => /no-verify|no-hooks|hooksPath/u.test(value)));
  for (const extra of [['--attribution', 'auto'], ['--attribution=auto']]) {
    assert.throws(() => invoke(extra), /pinned attribution policy changed/u);
  }
});


test('tracked inventory refuses current-task internal client evidence while preserving other-task and ordinary source', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-task-evidence-control-'));
  const owner = 'dev/log/self-authored/issue-17/evidence/';
  const other = 'dev/log/self-authored/issue-18/evidence/session-id.txt';
  try {
    fs.mkdirSync(path.join(directory, owner), {recursive: true});
    fs.mkdirSync(path.dirname(path.join(directory, other)), {recursive: true});
    fs.writeFileSync(path.join(directory, 'main.rs'), 'fn main() {}\n');
    fs.writeFileSync(path.join(directory, other), 'preserve other-task evidence\n');
    for (const name of ['agent-stream.jsonl', 'claude-stream.jsonl', 'codex-stream.jsonl', 'session-id.txt']) {
      const relative = owner + name;
      fs.writeFileSync(path.join(directory, relative), name === 'session-id.txt' ? 'formal-ai session synthetic\nformal-ai model formal-ai/0.0.0\n' : '');
      assert.throws(() => taskSnapshot(directory, [relative], 'https://github.com/public/task/issues/17'),
        /internal task evidence belongs outside the delivered commit/u);
    }
    assert.equal(taskSnapshot(directory, ['main.rs', other], 'https://github.com/public/task/issues/17').length, 2);
    assert.throws(() => taskSnapshot(directory, ['main.rs'], 'https://github.com/public/task/pull/17'), /canonical issue owner/u);
  } finally {
    fs.rmSync(directory, {recursive: true, force: true});
  }
});

test('hash-only task metadata still requires the exact freshly observed public issue body', () => {
  const body = 'Independent original public task.\n';
  const task = {issueUrl:'https://github.com/public/task/issues/1',
    pullRequestUrl:'https://github.com/public/task/pull/2',branch:'issue-1-original',bodySha256:contentDigest(body)};
  const inputs = {schema:'formal-ai-issue-1364-client-replay-inputs/v1',clients:{codex:task}};
  const issue = {html_url:task.issueUrl,body};
  const pr = {html_url:task.pullRequestUrl,state:'open',head:{ref:task.branch,sha:'a'.repeat(40)},base:{ref:'main'}};
  assert.equal(requireReplayTask(inputs,'codex',issue,pr),task);
  assert.throws(() => requireReplayTask(inputs,'codex',{...issue,body:body+'changed'},pr), {code:'ERR_ASSERTION'});
  assert.throws(() => requireReplayTask(inputs,'codex',{...issue,body:''},pr), {code:'ERR_ASSERTION'});
  assert.throws(() => requireReplayTask(inputs,'codex',{...issue,html_url:task.issueUrl+'0'},pr), {code:'ERR_ASSERTION'});
});
