import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {referenceEntry,PRIMITIVES} from '../../../experiments/agent_cli_e2e/issue-707-reference.mjs';
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

// Pull-request twins of two E2E harness contracts that only `main` exercised
// (issue #1639, main run 45636781871). The harnesses live under experiments/,
// whose tests never run before merge, so each break surfaced only after it
// landed. They live here, not in a new file, because the hosted JavaScript
// capacity cohort pins the set of rust/tests/web owners.
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');

// `computer-use --plan` and `computer-use --agent-mode` output for the first
// held-out case, captured from the CI binary of 7f664bb80 and trimmed to the
// fields the reference reads.
const explicitPlan='[{"arguments":{"confirmed":true,"content":"alpha\\nbeta\\ngamma\\n","path":"input/notes.txt"},"primitive":"fs.write"},'
 +'{"arguments":{"confirmed":true,"input":"input/notes.txt","operation":"count_lines","output":"reports/count_lines-notes.txt"},"primitive":"shell.run"},'
 +'{"arguments":{"archive":"out/notes.fai","confirmed":true,"paths":["reports/count_lines-notes.txt"]},"primitive":"archive.pack"},'
 +'{"arguments":{"path":"out/notes.fai"},"primitive":"fs.read"}]';
const prompt='Count the lines in the notes and archive the result';
const explicitText=prompt+'\n```computer-use\n'+explicitPlan+'\n```\n';
const outcome=()=>({
 verified:true,
 plan:{
  id:'synthesized-computer_use_resource_notes-computer_use_count_lines-computer_use_pack_archive',
  steps:['FsWrite','ShellRun','ArchivePack','FsRead'].map(primitive=>({primitive})),
 },
 steps:['fs.write','shell.run','archive.pack','fs.read'].map(primitive=>({primitive})),
});

test('issue #1639: the #707 reference compares the explicit plan in one naming scheme',()=>{
 const entry=referenceEntry({outcome:outcome(),caseId:'notes_count_and_pack',prompt,explicitText});
 assert.deepEqual(entry,{
  case_id:'notes_count_and_pack',
  prompt,
  plan_id:'computer-use-3c7f830591e4a0536b4e83bd',
  learned_plan_id:outcome().plan.id,
  // The verifier maps these serde variant names back through PRIMITIVES.
  steps:['FsWrite','ShellRun','ArchivePack','FsRead'],
 });
 for(const variant of entry.steps)assert.ok(PRIMITIVES[variant],variant);
});

test('the #707 reference still rejects a plan that differs from local synthesis',()=>{
 const reordered=outcome();reordered.steps.reverse();
 assert.throws(()=>referenceEntry({outcome:reordered,caseId:'c',prompt,explicitText}),/explicit plan differs/u);
 const drifted=outcome();drifted.plan.steps[1].primitive='HttpFetch';
 assert.throws(()=>referenceEntry({outcome:drifted,caseId:'c',prompt,explicitText}),/local plan and executed steps differ/u);
 const unverified={...outcome(),verified:false};
 assert.throws(()=>referenceEntry({outcome:unverified,caseId:'c',prompt,explicitText}),/did not verify/u);
 const recalled=outcome();recalled.plan.id='recalled-x';
 assert.throws(()=>referenceEntry({outcome:recalled,caseId:'c',prompt,explicitText}),/recalled, not synthesized/u);
});

test('the #707 harness derives its reference through the shared module',()=>{
 const harness=fs.readFileSync(path.join(root,'experiments/agent_cli_e2e/run_issue_707_generalization.sh'),'utf8');
 assert.match(harness,/experiments\/agent_cli_e2e\/issue-707-reference\.mjs/u);
 assert.doesNotMatch(harness,/node -e/u);
});

test('issue #1639: every harness launch that traces requests also enables the trace level',()=>{
 // The `[trace] POST` header is a trace-level event since #1592 (ec6ade5cd);
 // the default level is debug, so FORMAL_AI_TRACE_REQUESTS=1 alone logs no
 // request and every evidence projection counted zero posts.
 const skip=new Set(['.git','node_modules','target','dev']);
 const harnesses=[];
 const walk=relative=>{
  for(const entry of fs.readdirSync(path.join(root,relative),{withFileTypes:true})){
   if(skip.has(entry.name))continue;
   const next=relative?relative+'/'+entry.name:entry.name;
   if(entry.isDirectory())walk(next);else if(entry.name.endsWith('.sh'))harnesses.push(next);
  }
 };
 walk('');
 let launches=0;
 for(const harness of harnesses){
  const lines=fs.readFileSync(path.join(root,harness),'utf8').split('\n');
  lines.forEach((line,index)=>{
   if(/^\s*#/u.test(line)||!/\bFORMAL_AI_TRACE_REQUESTS=(1|true)\b/u.test(line))return;
   launches++;
   assert.match(lines.slice(index,index+4).join('\n'),/\bFORMAL_AI_LOG_LEVEL=trace\b/u,harness+':'+(index+1));
  });
 }
 assert.ok(launches>=60,'expected every harness launch in the repository to be scanned, saw '+launches);
 const authoring=fs.readFileSync(path.join(root,'rust/src/authoring_loop.rs'),'utf8');
 if(/"FORMAL_AI_TRACE_REQUESTS",\s*"1"/u.test(authoring))assert.match(authoring,/"FORMAL_AI_LOG_LEVEL",\s*"trace"/u);
});
