import test from 'node:test';
import assert from 'node:assert/strict';
import {requireExactProgramOutput, requireHarnessDelivery, requireReadyPullRequest,
  requireProgramMember} from '../../../scripts/lib/hive-delivery-evidence.mjs';
const observation = text => ({exitCode: 0, signal: null,
  stdoutBase64: Buffer.from(text).toString('base64'), stderrBase64: ''});
test('independent original stdout oracle rejects double prints, prefixes and failed execution', () => {
  const expected = 'Hello, World!\n';
  requireExactProgramOutput(observation(expected), expected);
  for (const text of ['', 'Hello, World!', expected + expected, 'prefix ' + expected, expected + '\n']) {
    assert.throws(() => requireExactProgramOutput(observation(text), expected));
  }
  for (const changed of [{exitCode: 1}, {signal: 'SIGTERM'}, {stderrBase64: 'ZXJyb3I='},
    {stdoutBase64: observation(expected).stdoutBase64 + '!!'}]) {
    assert.throws(() => requireExactProgramOutput({...observation(expected), ...changed}, expected));
  }
});
test('actual solve and every original harness assertion are mandatory', () => {
  const evidence = {tool: 'codex', model: 'formal-ai', passed: true, solveExitCode: 0,
    harness: [{ok: true}], evaluation: {results: [{ok: true}]},
    verdict: {harness: true, model: true}, pullRequestUrl: 'https://github.com/owner/repo/pull/7'};
  assert.deepEqual(requireHarnessDelivery(evidence, 'codex'), {repository: 'owner/repo', number: 7});
  for (const patch of [{solveExitCode: null}, {passed: false}, {harness: []},
    {evaluation: {results: [{ok: false}]}}, {verdict: {harness: true, model: null}},
    {pullRequestUrl: 'https://evil.invalid/owner/repo/pull/7'}]) {
    assert.throws(() => requireHarnessDelivery({...evidence, ...patch}, 'codex'));
  }
});
test('a real ready isolated PR and one safe original-language source are required', () => {
  const value = {number: 7, base: {repo: {full_name: 'owner/repo'}, ref: 'e2e/hello-world/1/codex-formal-ai'},
    head: {sha: 'a'.repeat(40), ref: 'issue-1-aabbcc'}, state: 'open', draft: false};
  assert.equal(requireReadyPullRequest(value, 'owner/repo', 7), 'a'.repeat(40));
  for (const patch of [{draft: true}, {state: 'closed'}, {head: {sha: 'a'.repeat(40), ref: 'main'}}]) {
    assert.throws(() => requireReadyPullRequest({...value, ...patch}, 'owner/repo', 7));
  }
  assert.equal(requireProgramMember([{filename: 'src/main.rs'}], 'Rust'), 'src/main.rs');
  for (const files of [[], [{filename: '../main.rs'}], [{filename: 'a.rs'}, {filename: 'b.rs'}]]) {
    assert.throws(() => requireProgramMember(files, 'Rust'));
  }
});

import {readFileSync} from 'node:fs';
import YAML from 'yaml';
const {parse} = YAML;
test('all actual Hive evidence consumers receive one initialized shared directory', () => {
  const root = new URL('../../../', import.meta.url);
  const source = readFileSync(new URL('.github/workflows/hive-original-delivery.yml', root), 'utf8');
  const job = parse(source).jobs.delivery;
  assert.equal(job.env.E2E_LOG_DIR, undefined, 'runner context must not appear at job env scope');
  const initialize = job.steps.findIndex((step) => step.name === 'Initialize shared delivery evidence directory');
  assert.ok(initialize >= 0);
  assert.equal(job.steps[initialize].run, 'echo "E2E_LOG_DIR=$RUNNER_TEMP/hive-original-evidence" >> "$GITHUB_ENV"');
  let sourceConsumers = 0;
  for (const [index, step] of job.steps.entries()) {
    for (const match of (step.run || '').matchAll(/node (scripts\/[A-Za-z0-9.-]+\.mjs)/g)) {
      let script;
      try { script = readFileSync(new URL(match[1], root), 'utf8'); } catch (error) { if (error.code === 'ENOENT') continue; throw error; }
      if (script.includes('process.env.E2E_LOG_DIR')) {
        sourceConsumers += 1;
        assert.ok(initialize < index, match[1] + ' must inherit initialized evidence path');
        assert.equal(step.env?.E2E_LOG_DIR, undefined, 'a stale override must not split evidence');
      }
    }
  }
  assert.equal(sourceConsumers, 2, 'actual image producer and independent verifier both consume evidence');
  const solve = job.steps.findIndex((step) => (step.run || '').includes('scripts/e2e-hello-world.mjs'));
  assert.ok(initialize < solve);
  assert.equal(job.steps[solve].env.E2E_KEEP, 'true');
  const upload = job.steps.findIndex((step) => step.uses?.startsWith('actions/upload-artifact@'));
  assert.ok(initialize < upload);
  assert.equal(job.steps[upload].with.path, '${{ runner.temp }}/hive-original-evidence');
});
