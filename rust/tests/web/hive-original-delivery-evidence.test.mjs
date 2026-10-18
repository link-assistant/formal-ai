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
