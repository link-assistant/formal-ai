import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {recipeSourceCommand,recipeSourceMatches,recipeSourceUpdateCommand} from '../../../js/agentic/recipe_resume.mjs';
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
