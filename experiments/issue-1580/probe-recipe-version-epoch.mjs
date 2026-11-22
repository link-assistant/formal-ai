// Finite source-level reproduction; no compiler or network; installs the maintained worker host.
import assert from 'node:assert/strict';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {renderWith} from '../../js/agentic/ci_workflow.mjs';
import {baselineVersionSet} from '../../js/agentic/crate/version_resolution.mjs';
import {recipeSourceCommand, recipeSourcesCommand} from '../../js/agentic/recipe_resume.mjs';
import {recipeProgressAfterLatestUser} from '../../js/agentic/command_reroute.mjs';
await installNodeHost(new WorkerHost());
const source = 'object Main { def main(args: Array[String]): Unit = println("Hello, World!") }\n';
const main = {path:'Main.scala', source};
const support = {path:'tests/verify-output.sh',source:'#!/bin/sh\nexit 127\n'};
const versions = baselineVersionSet();
const live = structuredClone(versions), replayed = structuredClone(versions);
for (const pin of Object.keys(versions).filter(key => key !== 'captures')) {
  live[pin].origin = 'live'; replayed[pin].origin = 'cache';
}
const base = {...main, language:'scala',supporting_files:[support],commands:['sh tests/verify-output.sh']};
const workflowPath = '.github/workflows/test-hello-world.yml';
const first = {...base,supporting_files:[support,{path:workflowPath,source:renderWith(base,live)}]};
const second = {...base,supporting_files:[support,{path:workflowPath,source:renderWith(base,replayed)}]};
const messages = [{role:'user',content:'Issue to solve: https://github.com/fixture/project/issues/1\nYour prepared Pull Request: https://github.com/fixture/project/pull/2'}];
let sequence = 0;
function observe(name,args,content) {
  const id='epoch-'+(++sequence);
  messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name,arguments:JSON.stringify(args)}}]});
  messages.push({role:'tool',tool_call_id:id,name,content});
}
observe('bash',{command:recipeSourceCommand(main.path)},JSON.stringify({schema:'workspace-source-v1',path:main.path,source:main.source,success:true,complete:true,absent:false}));
const paths=first.supporting_files.map(file=>file.path);
observe('bash',{command:recipeSourcesCommand(paths)},JSON.stringify({schema:'workspace-sources-v1',success:true,complete:true,observations:paths.map(path=>({schema:'workspace-source-v1',path,success:true,complete:true,absent:true,error_code:'ENOENT'}))}));
for (const file of first.supporting_files) observe('write',{file_path:file.path,content:file.source},'');
const stable=recipeProgressAfterLatestUser(messages,'write',first,first.commands,true);
const changed=recipeProgressAfterLatestUser(messages,'write',second,second.commands,true);
assert.equal(stable.source_invalidated,false);
assert.equal(stable.files_written,3);
assert.equal(changed.source_invalidated,true);
console.log(JSON.stringify({firstSource: first.supporting_files[1].source,cachedSource:second.supporting_files[1].source,stable:{written:stable.files_written,invalidated:stable.source_invalidated},changed:{written:changed.files_written,invalidated:changed.source_invalidated},hostedRootCause:'candidate, not yet demonstrated by retained hosted write arguments',independentAuthorshipCredit:0},null,2));
