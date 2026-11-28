import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';

const wrapper = fileURLToPath(new URL('../../scripts/run-with-deadline.sh', import.meta.url));
const probe = fileURLToPath(new URL('./probe-budget-process-lifetime.py', import.meta.url));
for (const [fixture, status] of [
  ['expired-parent-success', 124], ['expired-parent-failure', 124], ['term-resistant', 124], ['delayed-kill', 124],
  ['short-success', 0], ['short-failure', 7], ['inspection-unavailable', 125], ['inspection-unavailable-delayed-kill', 125],
]) {
  test(`deadline owns the live child after its parent exits: ${fixture}`, {
    skip: process.platform !== 'linux' && 'fixture needs the Linux child-subreaper API', timeout: 15000,
  }, () => {
    const directory = mkdtempSync(join(tmpdir(), 'deadline-child-regression-'));
    try {
      const adapter = join(directory, 'adapter.sh'), output = join(directory, 'observation.json');
      // Adapt the existing finite probe's budget/label arguments to the production deadline CLI.
      writeFileSync(adapter, 'budget=$1\nshift 2\nexport FORMAL_AI_DEADLINE_POLL_SECONDS=0.1\n' +
        'export FORMAL_AI_DEADLINE_GRACE_SECONDS=0\nexec bash "$DEADLINE_SOURCE" "$budget" "$@"\n');
      const observed = spawnSync('python3', [probe, adapter, output, '--case', fixture], {
        env: {...process.env, DEADLINE_SOURCE: wrapper}, encoding: 'utf8', timeout: 12000, maxBuffer: 65536,
      });
      assert.equal(observed.status, 0, observed.stderr);
      const result = JSON.parse(readFileSync(output, 'utf8'));
      assert.equal(result.exitCode, status, JSON.stringify(result));
      assert.equal(result.workerRunningWhenWrapperExited, false, JSON.stringify(result));
      assert.match(result.stdout, /fixture stdout/u);
      assert.match(result.stderr, /fixture stderr/u);
      if (status === 124) assert.ok(result.pipeClosedSeconds < 3.5, JSON.stringify(result));
      if (status === 125) assert.match(result.stderr, /Cannot inspect the process group/u);
    } finally {
      rmSync(directory, {recursive: true, force: true});
    }
  });
}
