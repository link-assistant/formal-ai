import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
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
  fs.writeFileSync(verifier,'#!/bin/bash\nprintf "compile failed\\n" > "$1/.agent-ladder/verify.tsv"\nprintf "full native diagnostic\\n" > "$1/.agent-ladder/cargo-test-module.log"\nprintf "verdict\\n"\nprintf "stderr diagnostic\\n" >&2\nexit '+code+'\n');fs.chmodSync(verifier,0o700);
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
