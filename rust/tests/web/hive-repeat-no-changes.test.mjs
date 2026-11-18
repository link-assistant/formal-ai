import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {adaptReplayEntrypoint, replayPassEnvironment, requireNoopTask} from '../../../scripts/lib/issue-1364-replay-contract.mjs';
import {requireHarnessDelivery} from '../../../scripts/lib/hive-delivery-evidence.mjs';

const captured = JSON.parse(fs.readFileSync(new URL('../fixtures/ci/hive-repeat-no-changes.json', import.meta.url)));
const upstream = await import('data:text/javascript;base64,' + Buffer.from(captured.upstream.exitCodeSource + '\n' + captured.upstream.dockerFunction).toString('base64'));
const gitBlob = text => createHash('sha1').update('blob ' + Buffer.byteLength(text) + '\0').update(text).digest('hex');
assert.equal(gitBlob(captured.upstream.exitCodeSource), captured.upstream.exitCodeBlob);

// Execute the adapted constructor and the unchanged upstream Docker argv builder.
// No Docker process, network request, native compiler or product is executed.
function containerEnvironment(environment, pass) {
  const host = replayPassEnvironment(environment, pass);
  const adapted = adaptReplayEntrypoint(captured.upstream.constructor,
    {pullRequestUrl: 'https://github.com/public/task/pull/2'}, '/tmp/owned-replay');
  const construct = new Function('buildE2eSolveArgv', 'buildDockerArgv', 'issueUrl', 'tool',
    'model', 'image', 'hostLogDir', 'modelKeys', adapted + '\nreturn dockerArgv;');
  const argv = construct(() => ['https://github.com/public/task/issues/1', '--attach-logs'],
    options => upstream.buildDockerArgv({...options, setEnv: {}}),
    'https://github.com/public/task/issues/1', 'codex', 'formal-ai', 'source-bound-image', '/tmp/logs', []);
  const forwarded = {};
  for (let index = 0; index < argv.length; index++) {
    if (argv[index] === '-e' && Object.hasOwn(host, argv[index + 1])) {
      forwarded[argv[index + 1]] = host[argv[index + 1]];
    }
  }
  return forwarded;
}

for (const record of captured.cases) {
  test(record.tool + ' captured repeat reproduces the original no-change failure', () => {
    requireHarnessDelivery(record.first, record.tool);
    assert.equal(record.repeat.solveExitCode, upstream.getSolveNoChangesExitCode({}));
    assert.deepEqual(record.repeat.evaluation.results.filter(row => !row.ok).map(row => row.name), ['solve completed']);
    assert.throws(() => requireHarnessDelivery(record.repeat, record.tool));
  });

  test(record.tool + ' only verified-repeat mode forwards upstream compatibility without relaxing the oracle', () => {
    const inherited = {HIVE_MIND_SOLVE_NO_CHANGES_EXIT_CODE: '0'};
    const firstExit = upstream.getSolveNoChangesExitCode(containerEnvironment(inherited, 1));
    const repeatExit = upstream.getSolveNoChangesExitCode(containerEnvironment({}, 2));
    assert.equal(firstExit, 5, 'an inherited override must not excuse a first-pass no-op');
    assert.equal(repeatExit, 0, 'the documented setting must reach the actual container');
    assert.throws(() => requireHarnessDelivery({...record.first, solveExitCode: firstExit}, record.tool));
    requireHarnessDelivery({...record.first, solveExitCode: repeatExit}, record.tool);
    for (const broken of [
      {...record.first, solveExitCode: 1},
      {...record.first, solveExitCode: 5},
      {...record.first, harness: [{ok: false}]},
      {...record.first, evaluation: {results: [{ok: false}]}},
    ]) assert.throws(() => requireHarnessDelivery(broken, record.tool));
  });
}

test('repeat compatibility still refuses additional passes and task mutations', () => {
  for (const pass of [0, 3, '2', null]) assert.throws(() => replayPassEnvironment({}, pass));
  const before = {head: 'a'.repeat(40), branch: 'issue-1-owned', status: '', commitRecords: 1,
    files: [{path: 'main.rs', sha256: 'b'.repeat(64), identity: ['same-file']} ]};
  requireNoopTask(before, structuredClone(before));
  for (const patch of [{head: 'c'.repeat(40)}, {commitRecords: 2}, {status: ' M main.rs'},
    {files: [{...before.files[0], identity: ['rewritten-file']}]}]) {
    assert.throws(() => requireNoopTask(before, {...before, ...patch}));
  }
});
