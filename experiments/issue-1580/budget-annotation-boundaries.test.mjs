import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const wrapper = fileURLToPath(new URL('../../scripts/run-with-budget-warning.sh', import.meta.url));
function run(budget, code, extra = {}) {
  const result = spawnSync('bash', [wrapper, budget, 'boundary probe', process.execPath, '-e', code], {
    env: {...process.env, TEST_BUDGET_ENFORCE: 'true', TEST_BUDGET_GRACE_SECONDS: '0',
      TEST_BUDGET_POLL_SECONDS: '0.05', RUSTC_WRAPPER: '', ...extra},
    encoding: 'utf8', timeout: 10000, maxBuffer: 65536,
  });
  assert.equal(result.error, undefined);
  return result;
}

test('budget completion starts a fresh line after NUL-terminated stdout', {timeout: 12000}, () => {
  const result = run('10', "process.stdout.write('child output\\0')");
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /child output\0\nboundary probe took /u);
});

test('budget warning starts a fresh line after partial stderr', {timeout: 12000}, () => {
  const result = run('1', "process.stderr.write('partial child stderr');setTimeout(()=>{},2200)",
    {TEST_BUDGET_ENFORCE: 'false'});
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stderr, /partial child stderr\n::warning title=/u);
  assert.doesNotMatch(result.stderr, /::error/u);
});

test('budget deadline annotations start fresh lines after partial stderr', {timeout: 12000}, () => {
  const result = run('1', "process.stderr.write('partial child stderr');setInterval(()=>{},1000)",
    {TEST_WARN_RATIO_PERCENT: '1000'});
  assert.equal(result.status, 124, result.stderr);
  assert.doesNotMatch(result.stderr, /::warning/u);
  assert.match(result.stderr, /^::error title=.*exceeded its execution budget/mu);
});
