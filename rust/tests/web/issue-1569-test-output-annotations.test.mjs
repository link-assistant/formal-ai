// Issue #1569: a passing js tier carried six `##[error]` and one `##[notice]`
// annotations on main (run 43692237186, groups 1 and 3) and on PR #1570. They
// were not failures: the hive draft-attempt and pre-effect deferral tests drive
// the real CLI paths with fixture digests (aaaa..., dddd...), and those paths
// print `::error`/`::notice` workflow commands to stdout, which the runner turns
// into annotations on whichever job runs the test. A reader of the run summary
// saw red "identity-mismatch" errors on a green job.
//
// Both scripts now take an injectable log sink and the tests assert the
// commands they provoke. This guard runs the two test files the way the js tier
// does and refuses any workflow command in their output.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {test} from 'node:test';

const FILES = [
  'rust/tests/web/hive-draft-attempt-identity.test.mjs',
  'rust/tests/web/hive-pre-effect-deferral.test.mjs',
];

/** Lines GitHub Actions would parse as an annotation command. */
export function workflowCommands(output) {
  return output.split(/\r?\n/u).filter(line => /^::(error|warning|notice)( [^:]*)?::/u.test(line.trimStart()));
}

test('the command matcher finds what the runner would annotate', () => {
  assert.deepEqual(workflowCommands('ok\n::error title=x::boom\n  ::notice::n\n✔ alerts with ::error only\n'),
    ['::error title=x::boom', '  ::notice::n']);
});

test('the hive attempt tests pass without printing workflow commands', {timeout: 120000}, () => {
  // Without NODE_TEST_CONTEXT the nested runner prints test stdout as the js tier does,
  // instead of refusing to recurse inside this test.
  const {NODE_TEST_CONTEXT, ...env} = process.env;
  const run = spawnSync(process.execPath, ['--test', '--test-reporter=spec', ...FILES],
    {encoding: 'utf8', timeout: 110000, env: {...env, GITHUB_ACTIONS: 'true'}});
  assert.match(run.stdout, /hive|attempt/iu, 'the nested runner reported the test files');
  assert.equal(run.status, 0, run.stdout + run.stderr);
  assert.deepEqual(workflowCommands(run.stdout + run.stderr), []);
});
