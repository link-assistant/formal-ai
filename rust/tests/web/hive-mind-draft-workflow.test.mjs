import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync, spawnSync} from 'node:child_process';
import YAML from 'yaml';
import {selectAttemptPullRequest, failureComment, usefulDraftChanges, inspectAttemptDraft, occupyingPullRequests} from '../../../scripts/hive-mind-draft-result.mjs';
import {shortOfRoom, main as freeDiskIfShort} from '../../../scripts/free-disk-if-short.mjs';
const workflow=readFileSync(new URL('../../../.github/workflows/hive-mind-draft.yml',import.meta.url),'utf8');
const context={branch:'issue-7-attempt',repository:'owner/repo',startedAt:'2026-10-16T00:00:00Z'};
const pr={number:9,headRefName:context.branch,headRepository:{nameWithOwner:context.repository},createdAt:'2026-10-16T00:01:00Z',state:'OPEN'};
test('dispatch selection requires unique exact recent same-repository PR',()=>{
 assert.equal(selectAttemptPullRequest([pr],context),pr);
 for(const changed of [{headRefName:'other'},{headRepository:{nameWithOwner:'foreign/repo'}},{createdAt:'2026-10-15T00:00:00Z'},{state:'CLOSED'}]) assert.equal(selectAttemptPullRequest([{...pr,...changed}],context),null);
 assert.equal(selectAttemptPullRequest([pr,{...pr,number:10}],context),null);
 assert.equal(selectAttemptPullRequest([pr],{...context,startedAt:'invalid'}),null);
});
test('no-PR failure comment always links run and available full session artifact',()=>{
 assert.match(failureComment({runUrl:'https://example/run',artifactUrl:'https://example/artifact'}),/Complete session evidence.*https:\/\/example\/artifact/);
 assert.match(failureComment({runUrl:'https://example/run'}),/https:\/\/example\/run/);
});
test('workflow contracts retain source identity, shared policy, evidence and default-token checks',()=>{
 assert.match(workflow,/types:\s*(?:\[opened\]|- opened)/); assert.match(workflow,/issue_number:/);
 assert.match(workflow,/cancel-in-progress: false/); assert.match(workflow,/group: hive-mind-draft-/);
 assert.doesNotMatch(workflow,/2083d6fd86ba0a7ee54e36110c8aebcfb5f2774d/);
 const mainCheckouts=YAML.parse(workflow).jobs.build.steps.filter(step=>step.uses?.startsWith('actions/checkout@')&&step.with?.path);
 assert.deepEqual(mainCheckouts.map(step=>[step.with.repository,step.with.ref]),
  [['link-assistant/hive-mind','refs/heads/main'],[undefined,'refs/heads/main']]);
 assert.match(workflow,/ref: \$\{\{ needs.build.outputs.hive_source_sha/);
 assert.match(workflow,/FORMAL_AI_DRAFT_IMAGE: \$\{\{ needs.build.outputs.hive_image/);
 assert.match(workflow,/HIVE_MIND_SOURCE_SHA: \$\{\{ needs.build.outputs.hive_source_sha/);
 assert.ok(workflow.indexOf('--decide') < workflow.indexOf('cargo build'));
 assert.match(workflow,/EXPECTED_SOURCE_SHA: \$\{\{ steps.source.outputs.sha/);
 assert.match(workflow,/test \"\$actual_sha\" = \"\$EXPECTED_SOURCE_SHA\"/);
 assert.match(workflow,/--version/); assert.match(workflow,/sha256sum/);
 assert.match(workflow,/FORMAL_AI_DRAFT_BINARY:.*formal-ai/);
 assert.match(workflow,/GH_TOKEN: \$\{\{ github.token \}\}/); assert.match(workflow,/actions\/dispatch-checks/);
 assert.match(workflow,/AUTOMATION_LAYER: default/); assert.doesNotMatch(workflow,/secrets\.AUTOMATION|steps\.credentials/); assert.match(workflow,/steps.exact-pr.outputs.pull_request_number/);
 assert.match(workflow,/always\(\).*steps.decide.outputs.should_run == 'true'.*failure\(\)/);
 assert.ok(workflow.indexOf('id: logs') < workflow.indexOf('node scripts/hive-mind-draft-result.mjs --failure'));
 assert.doesNotMatch(workflow,/github.event.issue.(body|title)|auto-merge|writer-lock/);
});

test('runner-only session path is initialized at step runtime',()=>{
 const jobEnv=workflow.slice(workflow.indexOf('    env:'),workflow.indexOf('    steps:'));
 assert.doesNotMatch(jobEnv,/runner\./);
 const setup=workflow.indexOf('FORMAL_AI_DRAFT_LOG_DIR=$RUNNER_TEMP/formal-ai-draft-logs');
 assert(setup>workflow.indexOf('    steps:')&&setup<workflow.indexOf('--decide'));
 assert.match(workflow,/FORMAL_AI_DRAFT_LOG_DIR=\$RUNNER_TEMP\/formal-ai-draft-logs" >> "\$GITHUB_ENV"/);
 assert.match(workflow,/path: \$\{\{ runner.temp \}\}\/formal-ai-draft-logs/);
});

import {workflowJobs} from '../../../scripts/lib/ci-speed-workflows.mjs';
import {capProblems} from '../../../scripts/check-ci-speed.mjs';
test('bounded build and attempt jobs share one exact-main binary and retain failure evidence',()=>{
 const jobs=workflowJobs(workflow);
 assert.deepEqual(jobs.map(job=>job.id),['build','draft']);
 assert.deepEqual(capProblems([{path:'.github/workflows/hive-mind-draft.yml',jobs}],{limitMinutes:30,exceptions:[]}),[]);
 assert.equal((workflow.match(/cargo build/g)||[]).length,1);
 assert.match(workflow,/run-with-session-evidence\.sh" 1320 build cargo build/);
 assert.match(workflow,/run-with-session-evidence\.sh" 1200 attempt node .hive-mind\/scripts\/formal-ai-draft.mjs/);
 assert.match(workflow,/needs: build/);
 assert.match(workflow,/needs.build.outputs.should_run == 'true'/);
 assert.match(workflow,/hive-mind-main-binary-/);
 assert.match(workflow,/hive-mind-main-evidence-/);
 assert.match(workflow,/needs.build.outputs.evidence_url/);
 assert.match(workflow,/needs.build.result != 'success'/);
 assert.match(workflow,/FORMAL_AI_DEADLINE_GRACE_SECONDS=30/);
});

test('deadline invocations remain portable with exact setup and kill-grace budgets',()=>{
 assert.doesNotMatch(workflow,/\btimeout --/);
 assert.equal((workflow.match(/run-with-deadline\.sh" 180 npm/g)||[]).length,2);
 assert.equal((workflow.match(/FORMAL_AI_DEADLINE_GRACE_SECONDS=15/g)||[]).length,3);
 assert.equal((workflow.match(/FORMAL_AI_DEADLINE_GRACE_SECONDS=30/g)||[]).length,2);
 assert.equal((workflow.match(/timeout-minutes: 30/g)||[]).length,2);
 assert.match(workflow,/bash "\$GITHUB_WORKSPACE\/scripts\/run-with-session-evidence\.sh" 1320 build cargo/);
 assert.match(workflow,/bash "\$GITHUB_WORKSPACE\/scripts\/run-with-session-evidence\.sh" 1200 attempt node/);
});

test('downloads use the raw adapter and concurrency distinguishes build from issue writer',()=>{
 const header=workflow.slice(0,workflow.indexOf('\njobs:\n'));
 assert.match(header,/\nconcurrency:\n/);
 assert.match(header,/cancel-in-progress: false/);
 const jobs=workflowJobs(workflow);
 assert.match(jobs.find(job=>job.id==='build').body,/hive-mind-draft-build-.*github.ref/);
 assert(jobs.find(job=>job.id==='build').body.includes("cancel-in-progress: ${{ github.event_name == 'pull_request' && github.event.action == 'synchronize' }}"));
 assert.doesNotMatch(workflow,/DEP0005|actions\/download-artifact@/);
 assert.equal((workflow.match(/uses: \.\/\.github\/actions\/download-artifact/g)||[]).length,3);
 assert.doesNotMatch(workflow,/--no-deprecation|--no-warnings|NODE_NO_WARNINGS/);
});

test('observed plan/log/bootstrap-only drafts are gaps; legitimate docs and removals remain candidates',()=>{
 const evidence=[{filename:'.gitkeep',status:'modified',changes:2},{filename:'dev/log/issue/session/solve.log',status:'added',changes:3862},
  {filename:'dev/log/issue/session/metadata.json',status:'added',changes:12},
  {filename:'.formal-ai/general-change-plan.lino',status:'added',changes:30}];
 assert.deepEqual(usefulDraftChanges(evidence),[]);
 assert.deepEqual(usefulDraftChanges([...evidence,{filename:'docs/reporting.md',status:'modified',changes:2}]),['docs/reporting.md']);
 assert.deepEqual(usefulDraftChanges([{filename:'js/broken.mjs',status:'removed',changes:3}]),['js/broken.mjs']);
 assert.deepEqual(usefulDraftChanges([{filename:'dev/log/old.mjs',previous_filename:'js/old.mjs',status:'renamed',changes:0}]),['dev/log/old.mjs']);
});
test('candidate inspection consumes the full census and refuses head movement or missing pages',()=>{
 const selected={...pr,headRefOid:'a'.repeat(40)};
 const detail={state:'open',head:{ref:context.branch,sha:selected.headRefOid,repo:{full_name:context.repository}},
  base:{sha:'b'.repeat(40)},changed_files:101};
 const files=Array.from({length:101},(_,i)=>({filename:'js/file-'+i+'.mjs',status:'modified',changes:1}));
 const run=(mode)=>{
  let reads=0;
  return inspectAttemptDraft(args=>{
   const url=args[1];
   if(url.includes('/files?'))return JSON.stringify(url.endsWith('page=1')?files.slice(0,100):mode==='missing'?[]:files.slice(100));
   const result=++reads===2&&mode==='moved'?{...detail,head:{...detail.head,sha:'c'.repeat(40)}}:detail;
   return JSON.stringify(result);
  },context.repository,selected);
 };
 const receipt=run('valid');assert.equal(receipt.eligibleForChecks,true);assert.equal(receipt.taskVerified,false);
 assert.equal(receipt.files.length,101);
 assert.throws(()=>run('moved'),/changed during inspection/);
 assert.throws(()=>run('missing'),/incomplete diff census/);
});

test('fresh selections are sealed across jobs and setup failures retain an issue response',()=>{
 assert(workflow.indexOf('id: hive-source')<workflow.indexOf('id: decide'));
 assert(workflow.indexOf('id: hive-version')>workflow.indexOf('id: decide'));
 assert.match(workflow,/hive_source_sha: \$\{\{ steps.hive-source.outputs.sha/);
 assert.match(workflow,/hive_image: \$\{\{ steps.hive-version.outputs.image/);
 assert.match(workflow,/image.endsWith\(':'\+version\)/);
 assert.match(workflow,/should_run:.*job.status == 'failure'.*'true'/);
 const outputs=workflow.slice(workflow.indexOf('    outputs:'),workflow.indexOf('    # The default token'));
 assert.doesNotMatch(outputs,/\b(?:failure|success|always|cancelled)\(\)/);
 assert.match(workflow,/issue_number:.*github.event.issue.number.*inputs.issue_number/);
 assert.match(workflow,/FORMAL_AI_DRAFT_ISSUE:.*needs.build.outputs.issue_number/);
 assert.match(workflow,/tee -a "\$FORMAL_AI_DRAFT_LOG_DIR\/source-identity.txt"/);
 const integration=readFileSync(new URL('../../../.github/workflows/agent-cli-e2e.yml',import.meta.url),'utf8');
 assert.match(integration,/@link-assistant\/hive-mind@latest/);
 assert.match(integration,/run: \|\n\s+bun add -g --ignore-scripts @link-assistant\/hive-mind@latest\n\s+bun pm ls -g/);
 assert.doesNotMatch(integration,/bun add -g[^\n]*&&/);
 assert.doesNotMatch(integration,/@link-assistant\/hive-mind@2\.15\.1/);
});

// Issue #1580, run 43813238887: the attempt began after another session opened #1581 for its issue, and Hive Mind
// continued that pull request -- three log commits and a `formal-ai-draft` label on a PR it did not open.
test('an issue another session already works on is left to that session',()=>{
 const census=[{number:1581,headRefName:'issue-1580-8d61a32a61e1',state:'OPEN'},{number:1579,headRefName:'issue-1162-100a9a565183',state:'OPEN'},
  {number:1500,headRefName:'issue-15800-aaaa',state:'OPEN'},{number:1490,headRefName:'issue-1580-old',state:'CLOSED'}];
 assert.deepEqual(occupyingPullRequests(census,'1580'),[1581]);
 assert.deepEqual(occupyingPullRequests(census,1162),[1579]);
 assert.deepEqual(occupyingPullRequests(census,'7'),[]);
 for(const bad of ['', '0', 'x', undefined]) assert.throws(()=>occupyingPullRequests(census,bad),/invalid occupancy census/);
 assert.throws(()=>occupyingPullRequests(null,'1580'),/invalid occupancy census/);
 const occupancy=workflow.indexOf('id: occupancy');
 assert(occupancy>workflow.indexOf('id: custody')&&occupancy<workflow.indexOf('id: attempt'));
 assert.match(workflow,/run: node scripts\/hive-mind-draft-result.mjs --occupied/);
 assert.match(workflow,/id: attempt\n[^\n]*\n\s+if: needs.build.result == 'success' && steps.occupancy.outputs.occupied_by == ''/);
 assert.match(workflow,/steps.custody.outcome == 'success' && steps.attempt.outcome != 'skipped'/);
 assert.match(workflow,/steps.exact-pr.outputs.pull_request_number == '' && steps.occupancy.outputs.occupied_by == ''/);
});

// Issue #1580, run 43738765561: 4 minutes of SDK deletion on a runner with 80 GB free, then a 5-minute image pull
// inside the attempt's 1200 s, left solve 15 of its 20 minutes.
test('the image is pulled before the attempt deadline and the disk cleanup runs only when room is short',()=>{
 const GIB=1024**3;
 assert.equal(shortOfRoom(80*GIB,40),false);
 assert.equal(shortOfRoom(40*GIB,40),false);
 assert.equal(shortOfRoom(40*GIB-1,40),true);
 for(const bad of [-1,1.5,NaN]) assert.throws(()=>shortOfRoom(bad,40),/available bytes/);
 assert.throws(()=>shortOfRoom(GIB,0),/required gigabytes/);
 const calls=[],lines=[],deps=available=>({measure:()=>available,run:(command,args)=>{calls.push([command,...args]);return {status:0};},log:line=>lines.push(line),env:{}});
 assert.equal(freeDiskIfShort(['40','node','clean.mjs'],deps(80*GIB)),0);
 assert.deepEqual(calls,[]);
 assert.match(lines.at(-1),/80\.0 GiB available on \/, at least the 40 GiB required; skipping the cleanup/);
 assert.equal(freeDiskIfShort(['40','node','clean.mjs'],deps(10*GIB)),0);
 assert.deepEqual(calls,[['node','clean.mjs']]);
 assert.throws(()=>freeDiskIfShort(['forty','node'],deps(GIB)),/usage/);
 assert.throws(()=>freeDiskIfShort(['40'],deps(GIB)),/usage/);
 assert.match(workflow,/run: node scripts\/free-disk-if-short.mjs 40 node .hive-mind\/scripts\/free-disk-space.mjs/);
 assert.doesNotMatch(workflow,/run: node .hive-mind\/scripts\/free-disk-space.mjs/);
 const pull=workflow.indexOf('id: image');
 assert(pull>workflow.indexOf('id: occupancy')&&pull<workflow.indexOf('id: attempt'));
 const step=workflow.slice(pull,workflow.indexOf('id: attempt'));
 assert.match(step,/if: needs.build.result == 'success' && steps.occupancy.outputs.occupied_by == ''/);
 assert.match(step,/continue-on-error: true/);
 assert.match(step,/run-with-deadline\.sh" 360 docker pull --quiet "\$FORMAL_AI_DRAFT_IMAGE"/);
});


for (const [id, checkoutPath, repository] of [
 ['hive-source', '.hive-mind', 'link-assistant/hive-mind'],
 ['source', '.formal-ai-source', undefined],
]) test(`literal main checkout seals ${id} before maintained code runs`, {timeout: 20000}, () => {
 const definition=YAML.parse(workflow), steps=definition.jobs.build.steps;
 const selected=steps.find(step=>step.uses?.startsWith('actions/checkout@')&&step.with?.path===checkoutPath);
 assert.equal(selected?.with.ref,'refs/heads/main','select the maintained branch directly before binding its actual commit');
 assert.equal(selected.with.repository,repository);
 assert.equal(selected.with['persist-credentials'],false);
 const binding=steps.find(step=>step.id===id);
 assert(steps.indexOf(selected)<steps.indexOf(binding));
 assert(steps.indexOf(binding)<steps.findIndex(step=>id==='hive-source'?
  step.name==='Install shared automation dependencies':step.id==='build-identity'));
 assert.doesNotMatch(binding.run,/gh api|github\.event|inputs\./u);
 assert.doesNotMatch(JSON.stringify(selected.with),/outputs\.|github\.event|inputs\./u);
 // Delivery still checks out the admitted Hive commit, never a later branch tip.
 assert.equal(definition.jobs.draft.steps.find(step=>step.with?.path==='.hive-mind').with.ref,
  '${{ needs.build.outputs.hive_source_sha }}');
 const directory=mkdtempSync(join(tmpdir(),'maintained-main-'));
 try {
  const source=join(directory,checkoutPath), output=join(directory,'output'), logs=join(directory,'logs');
  mkdirSync(source);mkdirSync(logs);
  const env={...process.env,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_DEFAULT_HASH:'sha1',
   GITHUB_WORKSPACE:directory,RUNNER_TEMP:directory,GITHUB_OUTPUT:output,FORMAL_AI_DRAFT_LOG_DIR:logs};
  const git=(...args)=>execFileSync('git',['-C',source,...args],{encoding:'utf8',env,timeout:5000}).trim();
  git('init','--quiet','--initial-branch=main');
  const commit=message=>git('-c','user.name=fixture','-c','user.email=fixture@example.invalid',
   '-c','core.hooksPath=/dev/null','commit','--quiet','--allow-empty','-m',message);
  commit('maintained main');
  const admitted=git('rev-parse','HEAD');git('update-ref','refs/remotes/origin/main',admitted);
  const run=()=>{
   writeFileSync(output,'');
   return spawnSync('bash',['-e','-o','pipefail','-c',binding.run],
    {cwd:directory,env,encoding:'utf8',timeout:5000});
  };
  const accepted=run();assert.equal(accepted.status,0,accepted.stdout+accepted.stderr);
  assert.equal(readFileSync(output,'utf8'),`sha=${admitted}\n`);
  const identity=readFileSync(join(logs,'source-identity.txt'),'utf8');assert(identity.includes(admitted));
  // A subsequent mutable ref/HEAD change cannot silently replace the admitted identity.
  commit('different main');assert.notEqual(git('rev-parse','HEAD'),admitted);
  assert.equal(readFileSync(output,'utf8'),`sha=${admitted}\n`);
  const changed=run();assert.notEqual(changed.status,0);
  assert.equal(readFileSync(output,'utf8'),'');
  assert.equal(readFileSync(join(logs,'source-identity.txt'),'utf8'),identity);
  git('reset','--quiet','--hard',admitted);git('update-ref','-d','refs/remotes/origin/main');
  const missing=run();assert.notEqual(missing.status,0);
  assert.equal(readFileSync(output,'utf8'),'');
  assert.equal(readFileSync(join(logs,'source-identity.txt'),'utf8'),identity);
 } finally {rmSync(directory,{recursive:true,force:true});}
});
