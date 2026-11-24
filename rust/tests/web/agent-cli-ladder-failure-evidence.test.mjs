import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const runner=process.env.LADDER_RUNNER_SOURCE??new URL('../../../experiments/issue_1028_agent_cli_ladder/run.sh',import.meta.url);
const source=fs.readFileSync(runner,'utf8');
const from=source.indexOf('  effect="$work/agent-ladder-effects/node-');
const to=source.indexOf('  VERIFIED_EFFECTS["$id"]=',from);
assert.ok(from>=0&&to>from);
const fragment=source.slice(from,to);
for(const [code,gitFailure,missingEffect] of [[0,false],[1,false],[7,false],[7,true],[0,true],[0,false,true]])test('verifier evidence persists with status '+code+' and capture failure '+gitFailure,()=>{
 const temp=fs.mkdtempSync(path.join(os.tmpdir(),'ladder-evidence-'));
 try{
  const work=path.join(temp,'work'),session=path.join(temp,'session');
  fs.mkdirSync(path.join(work,'.agent-ladder'),{recursive:true});
  fs.mkdirSync(path.join(work,'agent-ladder-effects'));fs.mkdirSync(session);
  const proof=path.join(work,'.agent-ladder','proof.md');fs.writeFileSync(proof,'node_path=leaf\nresult\n');
  const effect=path.join(work,'agent-ladder-effects','node-leaf.lino');fs.writeFileSync(effect,'node_path=leaf\n');
  const verifier=path.join(temp,'verify.sh');
  fs.writeFileSync(verifier,'#!/bin/bash\nprintf "compile failed\\n" > "$LADDER_VERIFY_LOG_DIR/verify.tsv"\nprintf "full native diagnostic\\n" > "$LADDER_VERIFY_LOG_DIR/cargo-test-module.log"\nprintf "verdict\\n"\nprintf "stderr diagnostic\\n" >&2\nexit '+code+'\n');fs.chmodSync(verifier,0o700);
  if(missingEffect)fs.unlinkSync(effect);
  const shell='git(){ printf "owned source diff\\n"; return "$GIT_FAILURE"; }\nrun(){\n'+fragment+'return 0\n}\nrun\n';
  const result=spawnSync('bash',['-c',shell],{encoding:'utf8',env:{
    ...process.env,GIT_FAILURE:gitFailure?'42':'0',work,session_dir:session,proof,
    id:'leaf',depth:'5',OUT:temp,leaf_span:'1-1',child_diffs:'',VERIFY_NODE:verifier,left:'',right:'',criterion_path:'source.rs',
    criterion_marker:'marker',criterion_guard:'guard',RUN_LOG:path.join(temp,'run.tsv')}});
  assert.equal(result.status,code===0&&!gitFailure&&!missingEffect?0:1);
  assert.equal(fs.readFileSync(path.join(session,'verifier-stdout.log'),'utf8'),'verdict\n');
  assert.equal(fs.readFileSync(path.join(session,'verifier-stderr.log'),'utf8'),'stderr diagnostic\n');
  for(const file of ['proof.md',...missingEffect?[]:['effect.lino'],'verify.tsv','cargo-test-module.log','change.diff'])assert.ok(fs.statSync(path.join(session,file)).size>0);
  if(code===0&&(gitFailure||missingEffect))assert.match(fs.readFileSync(path.join(temp,'run.tsv'),'utf8'),/artifact_capture_failure/);
  if(code!==0)assert.match(fs.readFileSync(path.join(temp,'run.tsv'),'utf8'),/leaf\tFAIL\tverdict/);
 }finally{fs.rmSync(temp,{recursive:true,force:true});}
});

// Replays main run 45018538750 without building the native test harness.
test('the ladder deadline leaves the in-progress native verifier evidence in the upload directory', () => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'ladder-deadline-'));
  try {
    const work = path.join(temp, 'work');
    const session = path.join(temp, 'session');
    const proof = path.join(work, '.agent-ladder', 'proof.md');
    const effect = path.join(work, 'agent-ladder-effects', 'node-leaf.lino');
    fs.mkdirSync(path.dirname(proof), {recursive: true});
    fs.mkdirSync(path.dirname(effect));
    fs.mkdirSync(session);
    fs.writeFileSync(proof, 'node_path=leaf\nresult\n');
    fs.writeFileSync(effect, 'node_path=leaf\n');
    const verifier = path.join(temp, 'verify.sh');
    fs.writeFileSync(verifier, [
      '#!/usr/bin/env bash',
      'logs="${LADDER_VERIFY_LOG_DIR:-$1/.agent-ladder}"',
      'printf "compile\\tok\\n" > "$logs/verify.tsv"',
      'printf "running one exact test\\n" > "$logs/cargo-test-module.log"',
      'printf "started native verification\\n" >&2',
      'sleep 8',
    ].join('\n'), {mode: 0o700});
    const wrapper = fileURLToPath(new URL('../../../scripts/run-with-budget-warning.sh', import.meta.url));
    const shell = 'git() { printf "owned source diff\\n"; }\nrun() {\n' + fragment + 'return 0\n}\nrun\n';
    const result = spawnSync('bash', [wrapper, '1', 'Ladder verifier fixture', 'bash', '-c', shell], {
      encoding: 'utf8',
      env: {
        ...process.env,
        work, session_dir: session, proof, id: 'leaf', depth: '5',
        OUT: temp, leaf_span: '1-1', child_diffs: '', VERIFY_NODE: verifier,
        left: '', right: '', criterion_path: 'source.rs', criterion_marker: 'marker', criterion_guard: 'guard',
        RUN_LOG: path.join(temp, 'run.tsv'),
        TEST_BUDGET_ENFORCE: 'true', TEST_BUDGET_GRACE_SECONDS: '0',
      },
    });
    assert.equal(result.status, 124, result.stderr);
    assert.equal(fs.readFileSync(path.join(session, 'verify.tsv'), 'utf8'), 'compile\tok\n');
    assert.equal(fs.readFileSync(path.join(session, 'cargo-test-module.log'), 'utf8'), 'running one exact test\n');
    assert.equal(fs.readFileSync(path.join(session, 'verifier-stderr.log'), 'utf8'), 'started native verification\n');
    assert.equal(fs.readFileSync(path.join(session, 'proof.md'), 'utf8'), 'node_path=leaf\nresult\n');
    assert.equal(fs.readFileSync(path.join(session, 'effect.lino'), 'utf8'), 'node_path=leaf\n');
    assert.equal(fs.readFileSync(path.join(session, 'change.diff'), 'utf8'), 'owned source diff\n');
    assert.equal(fs.existsSync(path.join(temp, 'run.tsv')), false);
  } finally {
    fs.rmSync(temp, {recursive: true, force: true});
  }
});
