// Finite offline planner reproduction; no compiler, external IO or artifact writes.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {host, installHost} from '../../js/agentic/host.mjs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {attach as attachWorkflow} from '../../js/agentic/ci_workflow.mjs';
import {baselineVersionSet, resolveVersionSet} from '../../js/agentic/crate/version_resolution.mjs';
import {planSymbolicCommandReroute} from '../../js/agentic/command_reroute.mjs';
import {recipeSourceCommand, recipeSourcesCommand} from '../../js/agentic/recipe_resume.mjs';

await installNodeHost(new WorkerHost(),{sourceUsageRustGrammar:false});
const originalHost = host();
const baseline = baselineVersionSet();
const pins = Object.entries(baseline).filter(([key]) => key !== 'captures').map(([, pin]) => pin);
const source = 'object Main { def main(args: Array[String]): Unit = println("Hello, World!") }\n';
const support = {path:'tests/verify-output.sh', source:'#!/bin/sh\nexit 127\n'};
const recipe = {language:'scala',path:'Main.scala',source,supporting_files:[support],commands:['sh tests/verify-output.sh']};
const objective = 'Add a GitHub Actions workflow to run Main.scala.\nIssue to solve: https://github.com/fixture/project/issues/1\nYour prepared Pull Request: https://github.com/fixture/project/pull/2';
const fetches = [];
function fixtureFetch(cached, generation) {
  return url => {
    let body;
    if (url === 'https://api.adoptium.net/v3/info/available_releases') {
      body = {most_recent_lts:Number(baseline.java_lts.tag)};
    } else {
      const release = pins.find(pin => pin.source_url === url);
      const commit = pins.find(pin => pin.source_url?.replace(/releases\/latest$/u, 'commits/' + pin.tag) === url);
      if (release) body = {tag_name:release.tag};
      else if (commit) body = {sha:commit.sha};
      else throw new Error('Undeclared fixture source: ' + url);
    }
    const text = JSON.stringify(body);
    fetches.push({generation,url,cached});
    return {source_url:url,text,fetched_at:'2026-11-22T00:00:00Z',sha256:crypto.createHash('sha256').update(text).digest('hex'),cached};
  };
}
const firstVersions = resolveVersionSet(fixtureFetch(false, 0));
const cachedVersions=resolveVersionSet(fixtureFetch(true,-1));
for (const key of Object.keys(firstVersions).filter(key=>key!=='captures')) assert.deepEqual({...firstVersions[key],origin:'cache'},cachedVersions[key]);
installHost({...originalHost,sourceFetch:()=>fixtureFetch(false,0)});
const prepared=structuredClone(recipe);
attachWorkflow(prepared,objective);
const files=prepared.supporting_files;
assert.equal(files.length,2);
const messages = [{role:'user',content:objective}];
let sequence=0;
function observe(name,args,content) {
  const id='owned-epoch-'+(++sequence);
  messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name,arguments:JSON.stringify(args)}}]});
  messages.push({role:'tool',tool_call_id:id,name,content});
}
observe('bash',{command:recipeSourceCommand(recipe.path)},JSON.stringify({schema:'workspace-source-v1',path:recipe.path,source,success:true,complete:true,absent:false}));
const paths=files.map(file=>file.path);
observe('bash',{command:recipeSourcesCommand(paths)},JSON.stringify({schema:'workspace-sources-v1',success:true,complete:true,observations:paths.map(path=>({schema:'workspace-source-v1',path,success:true,complete:true,absent:true,error_code:'ENOENT'}))}));
for (const file of [{path:recipe.path,source},...files]) observe('write',{file_path:file.path,content:file.source},'');
const answer={intent:'program_synthesis',answer:'',confidence:1,evidence_links:[],thinking_steps:[],links_notation:'',execution_recipe:recipe};
let generations=0;
function run(cached) {
  generations=0;
  installHost({...originalHost,sourceFetch:()=>fixtureFetch(cached,++generations)});
  const plan=planSymbolicCommandReroute(messages,['bash','write'],structuredClone(answer),objective);
  return {plan,generations};
}
try {
  const stable=run(false);
  const replayed=run(true);
  console.log(JSON.stringify({stable,replayed,pinsUnchanged:true,fixtureCaptureCount:fetches.length,capturedSources:[...new Set(fetches.map(item=>item.url))],independentAuthorshipCredit:0,hostedRootCause:'unverified; controlled production-planner reproduction'},null,2));
  assert.equal(stable.generations,1);
  assert.equal(replayed.generations,1);
  assert.deepEqual(stable.plan,{kind:'tool_calls',calls:[{tool:'bash',arguments:JSON.stringify({command:recipe.commands[0]})}]});
  assert.equal(replayed.plan.kind,'final');
  assert.equal(replayed.plan.result.origin,'recipe-source-epoch-invalidated');
} finally {
  installHost(originalHost);
}
