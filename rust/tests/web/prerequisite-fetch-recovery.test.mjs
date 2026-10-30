import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

test('CI replays captured installer connection failures without network or processes', {
  skip: process.env.GITHUB_ACTIONS !== 'true' ? 'Python controls run only in CI' : false,
}, () => {
  const result = spawnSync('python3', [
    'scripts/test-prerequisite-installer-hosted.py', 'DiscoveryConnectionControls',
  ], { encoding: 'utf8', timeout: 15_000, maxBuffer: 1024 * 1024 });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stderr, /Ran 4 tests/u);
  assert.match(result.stderr, /\bOK\b/u);
});
