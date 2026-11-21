import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {mkdtempSync,writeFileSync,rmSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
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

test('budget warning starts a fresh line after partial stderr', {timeout: 12000}, t => {
  const directory = mkdtempSync(join(tmpdir(), 'annotation-boundary-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const ready = join(directory, 'ready');
  const environment = join(directory, 'environment.sh');
  // The warning clock may cross its first whole second before Node starts.
  // Hold only the fixture's initial liveness probe until stderr is flushed.
  writeFileSync(environment, `kill() {
    if [ "\${1:-}" = '-0' ]; then
      for ((attempt=0; attempt<100; attempt++)); do
        [ -f "$ANNOTATION_BOUNDARY_READY" ] && break
        sleep 0.01
      done
      [ -f "$ANNOTATION_BOUNDARY_READY" ] || exit 126
    fi
    builtin kill "$@"
  }\n`);
  const result = run('1',
    "process.stderr.write('partial child stderr',()=>require('node:fs').writeFileSync(process.env.ANNOTATION_BOUNDARY_READY,''));setTimeout(()=>{},2200)",
    {TEST_BUDGET_ENFORCE: 'false', BASH_ENV: environment, ANNOTATION_BOUNDARY_READY: ready});
  assert.equal(result.status, 0, result.stderr);
  assert.ok(existsSync(ready), 'partial stderr was flushed before observing the warning');
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
