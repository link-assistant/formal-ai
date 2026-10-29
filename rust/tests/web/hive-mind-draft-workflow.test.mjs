import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {selectAttemptPullRequest, failureComment, usefulDraftChanges, inspectAttemptDraft} from '../../../scripts/hive-mind-draft-result.mjs';
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
 assert.match(workflow,/repos\/link-assistant\/hive-mind\/commits\/main/);
 assert.match(workflow,/ref: \$\{\{ steps.hive-source.outputs.sha/);
 assert.match(workflow,/ref: \$\{\{ needs.build.outputs.hive_source_sha/);
 assert.match(workflow,/FORMAL_AI_DRAFT_IMAGE: \$\{\{ needs.build.outputs.hive_image/);
 assert.match(workflow,/HIVE_MIND_SOURCE_SHA: \$\{\{ needs.build.outputs.hive_source_sha/);
 assert.ok(workflow.indexOf('--decide') < workflow.indexOf('cargo build'));
 assert.match(workflow,/commits\/main/); assert.match(workflow,/ref: \$\{\{ steps.source.outputs.sha/);
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
 assert.equal((workflow.match(/FORMAL_AI_DEADLINE_GRACE_SECONDS=15/g)||[]).length,2);
 assert.equal((workflow.match(/FORMAL_AI_DEADLINE_GRACE_SECONDS=30/g)||[]).length,2);
 assert.equal((workflow.match(/timeout-minutes: 30/g)||[]).length,2);
 assert.match(workflow,/bash "\$GITHUB_WORKSPACE\/scripts\/run-with-session-evidence\.sh" 1320 build cargo/);
 assert.match(workflow,/bash "\$GITHUB_WORKSPACE\/scripts\/run-with-session-evidence\.sh" 1200 attempt node/);
});

test('download warning suppression is narrow and concurrency distinguishes build from issue writer',()=>{
 const header=workflow.slice(0,workflow.indexOf('\njobs:\n'));
 assert.match(header,/\nconcurrency:\n/);
 assert.match(header,/cancel-in-progress: false/);
 const jobs=workflowJobs(workflow);
 assert.match(jobs.find(job=>job.id==='build').body,/hive-mind-draft-build-.*github.ref/);
 assert.match(jobs.find(job=>job.id==='build').body,/cancel-in-progress: true/);
 assert.equal((workflow.match(/NODE_OPTIONS: --disable-warning=DEP0005/g)||[]).length,2);
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
