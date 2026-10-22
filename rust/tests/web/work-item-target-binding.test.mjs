import {test} from 'node:test';

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync,spawnSync} from 'node:child_process';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {namedBranch,validBranchTarget,validWorkflowTarget,requestTargetBinding,targetOf,recipeCommitCommand,isPullRequestTarget} from '../../../js/agentic/git_commit.mjs';
import {attach} from '../../../js/agentic/ci_workflow.mjs';
test('owned target constraints preserve live branch, workflow and negative custody',async()=>{
await installNodeHost(new WorkerHost());
let count=0;const check=fn=>{fn();count++;};
const refs=['issue-42-held-out','feature/deep/topic','release.v2','so','тема/ветка'];
const fixture=fs.mkdtempSync(path.join(os.tmpdir(),'formal-target-refs-'));
try{
 execFileSync('git',['init','-q',fixture]);
 for(const ref of refs){check(()=>{assert.equal(validBranchTarget(ref),true);assert.equal(spawnSync('git',['check-ref-format','--branch',ref],{cwd:fixture}).status,0);assert.equal(namedBranch('Keep the solution on branch '+ref+'.'),ref);});}
 for(const ref of ['bad..ref','bad.lock','a/.hidden','a//b','bad@{x','-option','bad ref','a\\b','a/','a.'])check(()=>{assert.equal(validBranchTarget(ref),false,ref);assert.notEqual(spawnSync('git',['check-ref-format','--branch',ref],{cwd:fixture}).status,0,ref);});
 check(()=>assert.equal(validBranchTarget('@'),false,'ambiguous revision shorthand is not a destination'));
 const b='issue-42';
 for(const prompt of ['Keep the solution on branch '+b+'.','Держи решение в ветке '+b+'.','समाधान को शाखा '+b+' पर रखें।','将解决方案保留在分支 '+b+' 上。','Mantén la solución en la rama '+b+'.'])check(()=>assert.equal(namedBranch(prompt),b));
 for(const prompt of ['The branch so that a client can inspect it.','"Keep the solution on branch foreign"','Do not use branch foreign.','Write "Use branch foreign" to notes.txt.','~~~\nKeep the solution on branch foreign\n~~~'])check(()=>assert.equal(namedBranch(prompt),null));
 check(()=>assert.equal(namedBranch('The branch so that observers can inspect it.\nYour prepared branch: '+b),b));
 check(()=>assert.equal(requestTargetBinding('Use branch first\nUse branch second','branch-binding-patterns',validBranchTarget).kind,'refused'));
 check(()=>assert.equal(requestTargetBinding('Use branch bad..ref','branch-binding-patterns',validBranchTarget).kind,'refused'));
 for(const dest of ['ci/test-hello-world.yml','checks/nested/held-out.yaml','.github/workflows/check.yml'])check(()=>{const recipe={path:'main.rs',language:'rust',commands:['cargo run'],supporting_files:[]};attach(recipe,'Save the workflow as '+String.fromCharCode(96)+dest+String.fromCharCode(96)+', not under .github/workflows/.');assert.equal(recipe.supporting_files[0].path,dest);});
 for(const dest of ['../escape.yml','/absolute.yml','ci//file.yaml','ci/./file.yaml','ci/a;touch.yml'])check(()=>assert.equal(validWorkflowTarget(dest),false));
 for(const prompt of ['"Save the workflow as ci/foreign.yml"','Quoted page: Save the workflow as ci/foreign.yml'])check(()=>{const recipe={path:'held.rs',language:'rust',commands:['cargo run'],supporting_files:[]};attach(recipe,prompt);assert.equal(recipe.supporting_files[0].path,'.github/workflows/held.yml');});
 check(()=>{const recipe={path:'main.rs',language:'rust',commands:['cargo run'],supporting_files:[]};attach(recipe,'Save the workflow as ci/first.yml\nWrite the workflow to ci/second.yaml');assert.equal(recipe.supporting_files.length,0);assert.match(recipe.commands[0],/false$/);});
 check(()=>{const target=targetOf('Resolve https://github.com/example/repo/issues/1\nUse branch first\nUse branch second');assert.equal(target.bindingRefused,true);const command=recipeCommitCommand({path:'main.rs',supporting_files:[]},target);assert.doesNotMatch(command,/git add|git commit|git push/);assert.match(command,/false$/);});
}finally{fs.rmSync(fixture,{recursive:true,force:true});}
for(const request of ['Do not use branch one','Use branch one\nDo not use branch one'])check(()=>assert.equal(requestTargetBinding(request,'branch-binding-patterns',validBranchTarget).kind,'refused'));
check(()=>{const recipe={path:'main.rs',language:'rust',commands:['cargo run'],supporting_files:[]};attach(recipe,'Do not save the workflow as ci/foreign.yml');assert.equal(recipe.supporting_files.length,0);assert.match(recipe.commands[0],/false$/);});
for(const request of ['"\nUse branch foreign\n"','“\nUse branch foreign\n”','Quoted input:\n«\nUse branch foreign\n»'])check(()=>assert.equal(namedBranch(request),null));
check(()=>assert.equal(namedBranch('Use branch “feature/quoted”'),'feature/quoted'));
check(()=>assert.equal(requestTargetBinding('“\nSave the workflow as ci/foreign.yml\n”','workflow-binding-patterns',validWorkflowTarget).kind,'absent'));
console.log(JSON.stringify({controls:count,failures:0,skips:0,nativeExecution:false,actualTemporaryGitChecks:true}));

});

test('recipe consumers share the original pull-request target classification',()=>{
  for(const [reference,expected] of [
    ['https://github.com/fixture/project/pull/2',true],
    ['https://github.com/fixture/project/issues/2',false],
    ['https://github.com/fixture/project/pull-request/2',false],
    ['https://github.com/fixture/project/pull/123',true],
  ]) {
    for(const branch of [null,'retained-branch']) assert.equal(isPullRequestTarget({reference,branch}),expected);
  }
});
