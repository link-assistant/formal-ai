// Isolate finite process observations from remote-shell startup files.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import test from 'node:test';

const probe = resolve('experiments/issue-1580/probe-budget-process-lifetime.py');
for (const kind of ['budget', 'deadline']) {
  test(kind + ' process fixture ignores a hostile remote shell startup', {
    skip: process.platform !== 'linux' && 'fixture needs Linux child subreaping',
    timeout: 12000,
  }, t => {
    const directory = mkdtempSync(join(tmpdir(), 'process-fixture-startup-'));
    t.after(() => rmSync(directory, {recursive: true, force: true}));
    writeFileSync(join(directory, '.bashrc'), 'printf host-startup-read >&2; exit 97\n');
    let wrapper = resolve('scripts/run-with-budget-warning.sh');
    if (kind === 'deadline') {
      wrapper = join(directory, 'adapter.sh');
      writeFileSync(wrapper, 'budget=$1\nshift 2\nexport FORMAL_AI_DEADLINE_POLL_SECONDS=0.1\n'
        + 'export FORMAL_AI_DEADLINE_GRACE_SECONDS=0\nexec bash "$DEADLINE_SOURCE" "$budget" "$@"\n');
    }
    const output = join(directory, 'observation.json');
    const observed = spawnSync('python3', [probe, wrapper, output, '--case', 'short-success'], {
      encoding: 'utf8', timeout: 10000, maxBuffer: 65536,
      env: {...process.env, HOME: directory, SSH_CLIENT: '127.0.0.1 49152 22',
        DEADLINE_SOURCE: resolve('scripts/run-with-deadline.sh')},
    });
    assert.equal(observed.error, undefined, String(observed.error));
    assert.equal(observed.status, 0, observed.stderr);
    const result = JSON.parse(readFileSync(output, 'utf8'));
    assert.equal(result.exitCode, 0, JSON.stringify(result));
    assert.equal(result.workerRunningWhenWrapperExited, false);
    assert.match(result.stdout, /fixture stdout/u);
    assert.doesNotMatch(result.stderr, /host-startup-read/u);
  });
}

test('a fixture that exits before its PID retains the actual child diagnostics', {
  skip: process.platform !== 'linux' && 'fixture needs Linux child subreaping',
  timeout: 12000,
}, t => {
  const directory = mkdtempSync(join(tmpdir(), 'process-fixture-diagnostics-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const wrapper = join(directory, 'failure.sh'), output = join(directory, 'observation.json');
  writeFileSync(wrapper, 'printf fixture-startup-failed >&2\nexit 42\n');
  const observed = spawnSync('python3', [probe, wrapper, output], {
    encoding: 'utf8', timeout: 10000, maxBuffer: 65536,
  });
  assert.equal(observed.error, undefined, String(observed.error));
  assert.equal(observed.status, 1);
  assert.match(observed.stderr, /Fixture never wrote its worker PID/u);
  assert.match(observed.stderr, /"exitCode": 42/u);
  assert.match(observed.stderr, /fixture-startup-failed/u);
});
