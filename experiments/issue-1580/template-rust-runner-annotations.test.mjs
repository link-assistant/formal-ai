import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import {fileURLToPath} from 'node:url';

const directory = new URL('./', import.meta.url);
const pin = JSON.parse(readFileSync(new URL('fixtures/rust-runner-recovery/pin.json', directory), 'utf8'));
const source = readFileSync(new URL('fixtures/rust-runner-recovery/unacquired-jobs.py', directory));
assert.equal(createHash('sha256').update(source).digest('hex'), pin.sha256);
const run = spawnSync('python3', [fileURLToPath(new URL('rust-runner-annotation-replay.py', directory))],
  {encoding: 'utf8', maxBuffer: 65536});
assert.equal(run.status, 0, run.stderr);
const rows = JSON.parse(run.stdout);
for (const name of ['first-annotation', 'default-page-boundary', 'second-default-page',
  'second-full-page', 'no-acquisition-message', 'actual-code-failure']) {
  test('Rust upstream runner classification: ' + name, {timeout: 30000}, () => {
    const baseline = rows.find(row => row.implementation === 'baseline' && row.case === name);
    const candidate = rows.find(row => row.implementation === 'candidate' && row.case === name);
    assert(baseline && candidate);
    const recognized = !['no-acquisition-message', 'actual-code-failure'].includes(name);
    const expected = recognized ? ['native-tests'] : [];
    const missing = ['second-default-page', 'second-full-page'].includes(name);
    assert.deepEqual(baseline.observed, missing ? [] : expected);
    assert.deepEqual(candidate.observed, expected);
    if (name === 'actual-code-failure') {
      assert.equal(candidate.calls.length, 1, 'running code failures must not fetch acquisition annotations');
    } else {
      const request = candidate.calls.find(args => args.at(-1).includes('/annotations'));
      assert(request.includes('--paginate') && request.includes('--slurp'));
    }
  });
}
