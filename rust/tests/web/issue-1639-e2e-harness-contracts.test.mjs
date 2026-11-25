// Pull-request twins of two E2E harness contracts that only `main` exercised
// (issue #1639, main run 45636781871). The harnesses live under experiments/,
// whose tests never run before merge, so each break surfaced only after it
// landed.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {referenceEntry,PRIMITIVES} from '../../../experiments/agent_cli_e2e/issue-707-reference.mjs';

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
