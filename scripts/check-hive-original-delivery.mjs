#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { requireHistoricalTask } from './lib/hive-delivery-admission.mjs';
import { requireHarnessDelivery,
 requireReadyPullRequest,
 requireProgramMember,
 requireExactProgramOutput,
 requireHistoricalReadyPullRequest } from './lib/hive-delivery-evidence.mjs';
assert.equal(process.env.GITHUB_ACTIONS, 'true', 'hosted original delivery oracle only');
const logDirectory = process.env.E2E_LOG_DIR;
const inputs = JSON.parse(fs.readFileSync('data/benchmarks/hive-original-delivery-inputs.json'));
const evidence = JSON.parse(fs.readFileSync(path.join(logDirectory, 'result.json')));
const tool = process.env.E2E_TOOL;
const language = inputs.clients[tool];
assert(language);
const selected = requireHarnessDelivery(evidence, tool);
let historical = null;
if (process.env.HIVE_ISSUE1364_MODE === 'true') {
  const manifestBytes = fs.readFileSync('data/benchmarks/issue-1364-client-replay-inputs.json');
  historical = {
    manifest: JSON.parse(manifestBytes),
    manifestBytes,
    inputHash: process.env.HIVE_INPUT_SHA256,
    tool,
    evidence,
    repository: selected.repository
  };
  assert.match(historical.inputHash ?? '', /^[a-f0-9]{64}$/u);
} else {
  assert.equal(selected.repository, process.env.GITHUB_REPOSITORY, 'owned isolated task repository');
}
const api = endpoint => JSON.parse(execFileSync('gh', ['api', 'repos/' + selected.repository + '/' + endpoint], {
  encoding: 'utf8',
  timeout: 60000,
  maxBuffer: 2 * 1024 * 1024
}));
const pullRequest = api('pulls/' + selected.number);
const ready = value => {
  if (historical) {
    const entry = requireHistoricalTask({
      ...historical,
      branch: value.head.ref
    });
    return requireHistoricalReadyPullRequest(value, selected.repository, selected.number, entry.branch);
  }
  return requireReadyPullRequest(value, selected.repository, selected.number);
};
const head = ready(pullRequest);
if (historical) requireHistoricalTask({
  ...historical,
  branch: pullRequest.head.ref
});
const commits = api('pulls/' + selected.number + '/commits?per_page=100');
assert(commits.length > 0 && commits.length < 100, 'complete actual commit range required');
assert.equal(commits.at(-1).sha, head);
const files = api('pulls/' + selected.number + '/files?per_page=100');
assert.equal(files.length, pullRequest.changed_files, 'complete changed-file inventory');
const filename = requireProgramMember(files, language);
const tree = api('git/trees/' + head + '?recursive=1');
assert.equal(tree.truncated, false);
const members = tree.tree.filter(row => row.path === filename);
assert.equal(members.length, 1);
assert.equal(members[0].type, 'blob');
assert(['100644', '100755'].includes(members[0].mode));
const blob = api('git/blobs/' + members[0].sha);
assert.equal(blob.encoding, 'base64');
const program = Buffer.from(blob.content, 'base64');
assert.equal(blob.sha, members[0].sha);
assert.equal(createHash('sha1').update(Buffer.from('blob ' + program.length + '\0')).update(program).digest('hex'), blob.sha, 'actual Git blob byte identity');
assert(program.length > 0 && program.length <= 65536);
const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-hive-independent-'));
try {
  const source = path.join(workspace, path.basename(filename));
  fs.writeFileSync(source, program);
  const result = JSON.parse(execFileSync('python3', ['scripts/hive-original-delivery-oracle.py', workspace, source, language], {
    encoding: 'utf8',
    timeout: 960000,
    maxBuffer: 2 * 1024 * 1024
  }));
  requireExactProgramOutput(result, inputs.expectedStdout);
  assert.equal(ready(api('pulls/' + selected.number)), head, 'PR changed during independent verification');
  const identity = JSON.parse(fs.readFileSync(path.join(logDirectory, 'source-binary-identity.json')));
  assert.equal(identity.sameRunBinary, true);
  assert.equal(identity.binarySha256, identity.actualTaskImageBinarySha256);
  fs.writeFileSync(path.join(logDirectory, 'independent-delivery-receipt.json'), JSON.stringify({
    schema: 'formal-ai-original-hive-delivery-v1',
    tool,
    language,
    pullRequest: evidence.pullRequestUrl,
    head,
    commitRange: commits.map(row => row.sha),
    filename,
    programSha256: createHash('sha256').update(program).digest('hex'),
    binary: identity,
    result,
    originalHarnessPassed: true,
    ready: true,
    authorship: 'not-verified-by-this-oracle',
    autonomousAuthorshipCredit: 0
  }, null, 2) + '\n');
  console.log(JSON.stringify({
    tool,
    language,
    pullRequest: evidence.pullRequestUrl,
    head,
    independentExactlyOneLine: true,
    ready: true
  }));
} finally {
  fs.rmSync(workspace, {
    recursive: true,
    force: true
  });
}
