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
 assert.match(workflow,/types: \[opened\]/); assert.match(workflow,/issue_number:/);
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
