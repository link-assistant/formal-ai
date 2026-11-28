import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';

const wrapper = fileURLToPath(new URL('../../scripts/run-with-budget-warning.sh', import.meta.url));
const probe = fileURLToPath(new URL('./probe-budget-clock.py', import.meta.url));
for (const fixture of ['within-budget', 'term-grace']) {
  test(`whole-second clock preserves the actual execution interval: ${fixture}`, {timeout: 15000}, () => {
    const directory = mkdtempSync(join(tmpdir(), 'budget-clock-boundary-'));
    try {
      const output = join(directory, 'observation.json');
      const process = spawnSync('python3', [probe, wrapper, fixture, output], {
        encoding: 'utf8', timeout: 12000, maxBuffer: 65536,
      });
      assert.equal(process.status, 0, process.stderr);
      const result = JSON.parse(readFileSync(output, 'utf8'));
      if (fixture === 'within-budget') {
        assert.equal(result.exitCode, 0, JSON.stringify(result));
        assert.match(result.stdout, /completed inside budget/u);
        assert.doesNotMatch(result.stderr, /exceeded its execution budget/u);
      } else {
        assert.equal(result.exitCode, 124, JSON.stringify(result));
        assert.match(result.stdout, /cooperative cleanup completed/u, JSON.stringify(result));
      }
    } finally {
      rmSync(directory, {recursive: true, force: true});
    }
  });
}
