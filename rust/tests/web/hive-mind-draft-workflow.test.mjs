import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {selectAttemptPullRequest, failureComment} from '../../../scripts/hive-mind-draft-result.mjs';
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
 assert.match(workflow,/2083d6fd86ba0a7ee54e36110c8aebcfb5f2774d/);
 assert.ok(workflow.indexOf('--decide') < workflow.indexOf('cargo build'));
 assert.match(workflow,/commits\/main/); assert.match(workflow,/ref: \$\{\{ steps.source.outputs.sha/);
 assert.match(workflow,/--version/); assert.match(workflow,/sha256sum/);
 assert.match(workflow,/FORMAL_AI_DRAFT_BINARY:.*formal-ai/);
 assert.match(workflow,/actions\/automation-token/); assert.match(workflow,/actions\/dispatch-checks/);
 assert.match(workflow,/layer == 'default'/); assert.match(workflow,/steps.exact-pr.outputs.pull_request_number/);
 assert.match(workflow,/!cancelled\(\).*steps.decide.outputs.should_run == 'true'/);
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
 assert.match(workflow,/run-with-deadline\.sh" 1320 cargo build/);
 assert.match(workflow,/run-with-deadline\.sh" 1200 node .hive-mind\/scripts\/formal-ai-draft.mjs/);
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
 assert.match(workflow,/bash "\$GITHUB_WORKSPACE\/scripts\/run-with-deadline\.sh" 1320 cargo/);
 assert.match(workflow,/bash "\$GITHUB_WORKSPACE\/scripts\/run-with-deadline\.sh" 1200 node/);
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
