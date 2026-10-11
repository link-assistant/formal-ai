import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync,spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
export function requireFixtureRefusal(result) {
 assert.notEqual(result.status,0,'production guard must refuse fixture caller');
 assert.equal(result.signal,null,'refusal must be explicit, not killed');
 assert.match(result.stderr,/deployed caller differs from checked original projection|unique source-declared operation in current stage required/);
 assert.equal(result.stdout.trim(),'','no production receipt may be emitted');
}
export async function runFixture() {
 assert.equal(process.env.GITHUB_ACTIONS,'true');
 assert.equal(process.env.GITHUB_EVENT_NAME,'pull_request');
 assert.equal(process.env.GITHUB_JOB,'staged-authority-refusal');
 assert.match(process.env.GITHUB_SHA,/^[a-f0-9]{40}$/u);
 for(const field of ['GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT'])assert.match(process.env[field],/^[1-9][0-9]*$/u);
 const event=JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH,'utf8'));
 assert.equal(event.repository.full_name,process.env.GITHUB_REPOSITORY);
 const trusted=path.resolve('_protocol');
 assert.equal(execFileSync('git',['-C',trusted,'rev-parse','HEAD'],{encoding:'utf8',timeout:5000}).trim(),process.env.GITHUB_SHA);
 const adapter=await import(pathToFileURL(path.join(trusted,'scripts/maintained-staged-authority.mjs')));
 const inventory=adapter.maintainedStagedInventory(adapter.compileDeployedStagedAuthority());
 assert.equal(inventory.operations,52);assert.equal(inventory.bindings,104);
 assert.ok(process.env.RUNNER_TEMP && path.resolve(process.env.RUNNER_TEMP)!==process.cwd(),'actual private runner directory required');
 const states=['auto-release','manual-release'].map(caller=>path.join(process.env.RUNNER_TEMP,'formal-ai-release-freshness-'+process.env.GITHUB_RUN_ID+'-'+process.env.GITHUB_RUN_ATTEMPT+'-'+caller+'.json'));
 for(const state of states)assert.equal(fs.existsSync(state),false);
 const result=spawnSync(process.execPath,[path.join(trusted,'scripts/release-source-freshness.mjs'),'before-sync'],{cwd:process.cwd(),env:process.env,encoding:'utf8',timeout:45000,maxBuffer:65536});
 requireFixtureRefusal(result);for(const state of states)assert.equal(fs.existsSync(state),false);
 const bytes=fs.readFileSync(path.join(trusted,'scripts/release-source-freshness.mjs'));
 const receipt = {
   schema: 'GuardedStagedSubprocessFixtureRefusalV1',
   run: process.env.GITHUB_RUN_ID,
   attempt: process.env.GITHUB_RUN_ATTEMPT,
   head: process.env.GITHUB_SHA,
   repository: process.env.GITHUB_REPOSITORY,
   job: process.env.GITHUB_JOB,
   sourceSha256: createHash('sha256').update(bytes).digest('hex'),
   operations: 52,
   bindings: 104,
   status: result.status,
   explicitRefusal: true,
   productionAuthority: false,
   ancestorAuthority: 'Unknown',
   publication: false
 };
 fs.mkdirSync('.release-fixture',{recursive:true});fs.writeFileSync('.release-fixture/staged-authority-refusal.json',JSON.stringify(receipt,null,2)+String.fromCharCode(10));
}
if(process.argv[1] && import.meta.url===pathToFileURL(fs.realpathSync(process.argv[1])).href)runFixture().catch(error=>{console.error(error.message);process.exitCode=1;});
