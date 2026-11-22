import {mkdtempSync,writeFileSync,readFileSync,readdirSync,symlinkSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {recipeSourceCommand,recipeSourceMatches,recipeSourceUpdateCommand,recipeSourcesCommand,recipeSourcesResult,recipeSourcePreimage} from '../../../js/agentic/recipe_resume.mjs';
import {recipeProgressAfterLatestUser} from '../../../js/agentic/command_reroute.mjs';
import {Progress} from '../../../js/agentic/progress.mjs';
import {reportedExitCode} from '../../../js/agentic/tool_result.mjs';
import {preparedWorkPaths} from '../../../js/agentic/restart_feedback.mjs';
before(async()=>installNodeHost(new WorkerHost()));
const recipe={language:'python',path:'held-out.py',source:'# English Русский हिन्दी 中文\nprint(17)\n',supporting_files:[],commands:['python3 held-out.py']};
const request='Issue to solve: https://github.com/fixture/project/issues/1\nYour prepared Pull Request: https://github.com/fixture/project/pull/2';
const absent={schema:'workspace-source-v1',path:recipe.path,success:true,complete:true,absent:true,error_code:'ENOENT'};
function history(command,content,{tool='Bash',failed=false}={}){
 return [{role:'user',content:request},{role:'assistant',content:'',tool_calls:[{id:'source-probe',type:'function',
  function:{name:tool,arguments:JSON.stringify({[tool==='exec_command'?'cmd':'command']:command})}}]},
 {role:'tool',name:tool,tool_call_id:'source-probe',is_error:failed,content}];
}
const progress=messages=>recipeProgressAfterLatestUser(messages,'Write',recipe,recipe.commands,true);
test('raw Git porcelain leading status columns survive the same callback transport',()=>{
 const raw=' M held-out.py\n?? ci/test.yml\n',expected=['held-out.py','ci/test.yml'];
 assert.deepEqual(preparedWorkPaths(raw),expected);
 assert.deepEqual(preparedWorkPaths(JSON.stringify({exit_code:0,stdout:raw,complete:true})),expected);
 assert.equal(preparedWorkPaths(JSON.stringify({exit_code:1,stdout:raw})),null);
 assert.equal(preparedWorkPaths(JSON.stringify({exit_code:0,stdout:raw,truncated:true})),null);
 assert.equal(preparedWorkPaths('M held-out.py\n'),null);
});
test('actual Hive stdout-only absence callback permits the pending write without inventing process status',()=>{
 // Hive run 40124638032: Claude job 120963507697 and Agent job 120963507679
 // returned this complete schema without exit_code; paths are held out here.
 const output=JSON.stringify(absent);assert.equal(reportedExitCode(output),null);
 for(const tool of ['Bash','bash','exec_command']){
  const result=progress(history(recipeSourceCommand(recipe.path),output,{tool}));
  assert.equal(result.failure,null,tool);assert.equal(result.files_written,0);
  assert.equal(result.source_checked.has(recipe.path),true);
  assert.equal(result.source_preimages.get(recipe.path).absent,true);
 }
});
test('bound complete source bytes can verify an artifact with either raw or explicit-zero callbacks',()=>{
 const source={...absent,absent:false,source:recipe.source};delete source.error_code;
 for(const output of [JSON.stringify(source),JSON.stringify({exit_code:0,stdout:JSON.stringify(source),complete:true})]){
  const messages=history(recipeSourceCommand(recipe.path),output);
  assert.equal(recipeSourceMatches(Progress.scan(messages),recipe.path,recipe.source),true);
  assert.equal(progress(messages).files_written,1);
 }
});
test('explicit process errors, partial results and contradictory callback bindings still veto source evidence',()=>{
 for(const content of [JSON.stringify({exit_code:1,stdout:JSON.stringify(absent)}),
  JSON.stringify({...absent,complete:false}),JSON.stringify({...absent,timed_out:true}),
  JSON.stringify({...absent,truncated:true}),JSON.stringify({...absent,signal:'SIGTERM'}),
  JSON.stringify({...absent,path:'foreign.py'}),JSON.stringify({...absent,success:false}),
  JSON.stringify({...absent,error_code:'EACCES'}),'not a source receipt']){
  assert.notEqual(progress(history(recipeSourceCommand(recipe.path),content)).failure,null,content);
 }
 const output=JSON.stringify(absent);
 assert.notEqual(progress(history(recipeSourceCommand(recipe.path),output,{failed:true})).failure,null);
 const duplicate=history(recipeSourceCommand(recipe.path),output);duplicate.push({...duplicate.at(-1)});
 assert.equal(progress(duplicate).source_checked.size,0);
 assert.equal(progress(history('printf unrelated',output)).source_checked.size,0);
});
test('stdout-only atomic update receipt requires a new byte probe before any artifact credit',()=>{
 const prior={...absent,absent:false,source:'prior bytes\n',sha256:'a'.repeat(64),identity:['1','2','3','4','5']};delete prior.error_code;
 const messages=history(recipeSourceCommand(recipe.path),JSON.stringify(prior));
 const command=recipeSourceUpdateCommand(recipe.path,recipe.source,prior);assert.equal(typeof command,'string');
 const update=history(command,JSON.stringify({schema:'workspace-source-update-v1',path:recipe.path,success:true,complete:true})).slice(1);
 update[0].tool_calls[0].id='source-update';update[1].tool_call_id='source-update';messages.push(...update);
 const result=progress(messages);assert.equal(result.failure,null);
 assert.equal(result.source_checked.has(recipe.path),false);assert.equal(result.files_written,0);
});

test('supporting source batches bind all owners and retain independent mutation epochs',()=>{
 const supporting=[{path:'test/verify.sh',source:'#!/bin/sh\ntrue\n'},{path:'ci/check.yml',source:'name: held-out\n'}];
 const paths=supporting.map(file=>file.path),batch={schema:'workspace-sources-v1',success:true,complete:true,
  observations:supporting.map(file=>({...absent,path:file.path}))};
 const r={...recipe,supporting_files:supporting};
 function append(messages,tool,args,content){const id='batch-'+messages.length;messages.push(
  {role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:tool,arguments:JSON.stringify(args)}}]},
  {role:'tool',tool_call_id:id,name:tool,content:JSON.stringify(content)});}
 function transcript(value=batch){const messages=[{role:'user',content:request}];
  append(messages,'Write',{path:recipe.path,content:recipe.source},{success:true});
  append(messages,'Bash',{command:recipeSourcesCommand(paths)},value);return messages;}
 const state=messages=>recipeProgressAfterLatestUser(messages,'Write',r,r.commands,true);
 const messages=transcript();assert.equal(state(messages).failure,null);
 assert.deepEqual([...state(messages).source_checked],paths);
 assert.equal(state(messages).files_written,1);
 append(messages,'Write',{path:paths[0],content:supporting[0].source},{success:true});
 assert.equal(state(messages).source_invalidated,false);
 assert.equal(recipeSourcePreimage(Progress.scan(messages),paths[1],paths).absent,true);
 append(messages,'Edit',{path:paths[1],old_string:'old',new_string:'foreign'},{success:true});
 assert.equal(recipeSourcePreimage(Progress.scan(messages),paths[1],paths),null);
 assert.equal(state(messages).source_invalidated,true);
 const identical={...batch,observations:supporting.map(file=>({schema:'workspace-source-v1',path:file.path,success:true,complete:true,absent:false,source:file.source}))};
 assert.equal(state(transcript(identical)).files_written,3);
 assert.equal(state(transcript(identical)).source_invalidated,false);
 const mixed=structuredClone(identical);mixed.observations[0]=batch.observations[0];
 const remaining=transcript(mixed);assert.equal(state(remaining).files_written,1);
 append(remaining,'Write',{path:paths[0],content:supporting[0].source},{success:true});
 assert.equal(state(remaining).files_written,3);
 for(const change of [
  value=>value.observations.pop(),value=>value.observations.push(value.observations[0]),
  value=>value.observations.reverse(),value=>value.observations[1].path='foreign.yml',
  value=>value.observations[1]=value.observations[0],value=>value.observations[1].complete=false,
  value=>value.observations[1].truncated=true,value=>value.observations[1].timed_out=true,
  value=>value.observations[1].signal='SIGTERM',value=>value.observations[1].exit_code=1,
  value=>value.complete=false,value=>value.success=false,
 ]){const invalid=structuredClone(batch);change(invalid);
  assert.equal(recipeSourcesResult(JSON.stringify(invalid),paths),null);
  assert.notEqual(state(transcript(invalid)).failure,null);}
 assert.equal(recipeSourcesResult(JSON.stringify(batch),[paths[0],paths[0]]),null);
 const duplicate=transcript();duplicate.push({...duplicate.at(-1)});
  assert.deepEqual([...state(duplicate).source_checked],[]);
  assert.equal(state(duplicate).source_preimages.size,0);
  assert.equal(state(duplicate).source_invalidated,true);
  for(const path of paths) assert.equal(recipeSourcePreimage(Progress.scan(duplicate),path,paths),null);
});

test('the actual bounded batch probe reads existing and absent owners without modifying files',()=>{
 const root=mkdtempSync(join(tmpdir(),'recipe-source-batch-'));
 try{
  const text='English Русский हिन्दी 中文\n';writeFileSync(join(root,'held-out.txt'),text);
  const paths=['held-out.txt','missing.txt'];
  const execute=values=>spawnSync('sh',['-c',recipeSourcesCommand(values)],{cwd:root,encoding:'utf8',timeout:5000,maxBuffer:1024*1024});
  const observed=execute(paths);assert.equal(observed.status,0,observed.stderr);
  const values=recipeSourcesResult(observed.stdout,paths);
  assert.equal(values[0].source,text);assert.match(values[0].sha256,/^[a-f0-9]{64}$/u);
  assert.equal(values[0].identity.length,5);assert.equal(values[1].absent,true);
  assert.equal(readFileSync(join(root,'held-out.txt'),'utf8'),text);
  assert.deepEqual(readdirSync(root),['held-out.txt']);
  assert.notEqual(execute(['held-out.txt','held-out.txt']).status,0);
  assert.notEqual(execute([]).status,0);
  assert.notEqual(execute(['../foreign']).status,0);
  symlinkSync('held-out.txt',join(root,'symbolic.txt'));
  assert.notEqual(execute(['held-out.txt','symbolic.txt']).status,0);
 }finally{rmSync(root,{recursive:true,force:true});}
});
