// Issue #1191 (requirement 9): every runner label is pinned, and
// scripts/check-runner-labels.mjs rejects a moving `-latest` label wherever a
// job can name one -- directly, in a list, or through a matrix row -- and
// fails closed on a `runs-on` it cannot read.

import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkRepository, checkSource, isMovingLabel } from '../../../scripts/check-runner-labels.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const SCRIPT = join(ROOT, 'scripts/check-runner-labels.mjs');

function run(root) {
  return spawnSync(process.execPath, [SCRIPT, root], { encoding: 'utf8', timeout: 60_000 });
}

/** A scratch repository holding one workflow, checked through the CLI. */
function checkFixture(workflow, action) {
  const root = mkdtempSync(join(tmpdir(), 'runner-labels-'));
  try {
    mkdirSync(join(root, '.github/workflows'), { recursive: true });
    writeFileSync(join(root, '.github/workflows/fixture.yml'), workflow);
    if (action !== undefined) {
      mkdirSync(join(root, '.github/actions/local'), { recursive: true });
      writeFileSync(join(root, '.github/actions/local/action.yml'), action);
    }
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

const job = body => 'on: push\njobs:\n  build:\n' + body + '    steps:\n      - run: echo ok\n';

test('every workflow and action of this repository names only pinned runners', () => {
  const result = run(ROOT);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /all pinned: /u);
  const { files, labels, problems } = checkRepository(ROOT);
  assert.deepEqual(problems, []);
  assert.ok(files.some(file => file === '.github/workflows/release.yml'));
  assert.ok(files.some(file => file.startsWith('.github/actions/')), 'composite actions are read too');
  // The matrix-driven jobs are resolved, not skipped.
  const native = labels.filter(entry => entry.file === '.github/workflows/desktop-release.yml' && entry.job === 'native');
  assert.ok(native.some(entry => entry.label === 'windows-2025' && /matrix\.include/u.test(entry.via)));
  assert.ok(labels.every(entry => !isMovingLabel(entry.label)));
});

test('a literal moving label is rejected with its file, job and value', () => {
  const result = checkFixture(job('    runs-on: ubuntu-latest\n'));
  assert.equal(result.status, 1);
  assert.match(result.stderr, /\.github\/workflows\/fixture\.yml: job `build`: runs-on is "ubuntu-latest", a moving label/u);
});

test('a moving label reached through a matrix include row is rejected', () => {
  const result = checkFixture(job([
    '    runs-on: ${{ matrix.os }}',
    '    strategy:',
    '      matrix:',
    '        include:',
    '          - { os: ubuntu-24.04, target: linux }',
    '          - { os: windows-latest, target: windows }',
    ''
  ].join('\n')));
  assert.equal(result.status, 1);
  assert.match(result.stderr, /job `build`: runs-on <- matrix\.include\[1\]\.os is "windows-latest", a moving label/u);
  assert.doesNotMatch(result.stderr, /ubuntu-24\.04"/u, 'the pinned row is accepted');
});

test('a moving label in a matrix dimension list is rejected', () => {
  const result = checkFixture(job('    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n        os: [ubuntu-24.04, macos-latest]\n'));
  assert.equal(result.status, 1);
  assert.match(result.stderr, /runs-on <- matrix\.os\[1\] is "macos-latest"/u);
});

test('a moving label inside a runs-on list or runner group is rejected', () => {
  const list = checkFixture(job('    runs-on: [self-hosted, ubuntu-latest]\n'));
  assert.equal(list.status, 1);
  assert.match(list.stderr, /runs-on\[1\] is "ubuntu-latest", a moving label/u);
  const group = checkFixture(job('    runs-on:\n      group: large\n      labels: [ubuntu-latest]\n'));
  assert.equal(group.status, 1);
  assert.match(group.stderr, /runs-on\.labels\[0\] is "ubuntu-latest"/u);
});

test('a runs-on the check cannot resolve fails closed', () => {
  for (const [body, reason] of [
    ['    runs-on: ${{ inputs.runner }}\n', /expression this check cannot resolve/u],
    ['    runs-on: ubuntu-${{ matrix.version }}\n    strategy:\n      matrix:\n        version: [24.04]\n', /expression this check cannot resolve/u],
    ['    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix: ${{ fromJSON(needs.plan.outputs.matrix) }}\n', /strategy\.matrix is computed/u],
    ['    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n        os: ${{ fromJSON(inputs.oses) }}\n', /matrix\.os is computed/u],
    ['    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n        target: [a, b]\n', /no matrix row defines `os`/u],
    ['    runs-on: ${{ matrix.os }}\n', /the job has no strategy\.matrix/u],
    ['    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n        include:\n          - { os: "${{ inputs.os }}" }\n', /matrix\.include\[0\]\.os is an expression/u]
  ]) {
    const result = checkFixture(job(body));
    assert.equal(result.status, 1, body);
    assert.match(result.stderr, reason, body);
    assert.match(result.stderr, /fail closed/u, body);
  }
});

test('only a listed, reasoned job may keep an unresolvable runs-on, and its labels are still read', () => {
  const source = job('    runs-on: ["${{ inputs.runner }}", ubuntu-24.04]\n');
  assert.equal(checkSource('w.yml', source).problems.length, 1);
  const allowed = new Map([['w.yml#build', 'fixture: the caller passes a pinned runner']]);
  const checked = checkSource('w.yml', source, allowed);
  assert.deepEqual(checked.problems, []);
  assert.deepEqual(checked.labels.map(entry => entry.label), ['ubuntu-24.04']);
  const moving = checkSource('w.yml', job('    runs-on: ["${{ inputs.runner }}", ubuntu-latest]\n'), allowed);
  assert.equal(moving.problems.length, 1, 'the allowance covers the expression, never a moving label');
});

test('reusable-workflow calls, invalid YAML and runner names in actions are judged', () => {
  const call = checkFixture('on: push\njobs:\n  call:\n    uses: ./.github/workflows/other.yml\n');
  assert.equal(call.status, 0, call.stderr);
  const neither = checkFixture('on: push\njobs:\n  broken:\n    steps: [{run: echo}]\n');
  assert.equal(neither.status, 1);
  assert.match(neither.stderr, /has neither `runs-on` nor `uses`/u);
  const invalid = checkFixture('jobs: [\n');
  assert.equal(invalid.status, 1);
  assert.match(invalid.stderr, /is not valid YAML/u);
  const action = checkFixture(job('    runs-on: ubuntu-24.04\n'), 'name: a\nruns:\n  using: composite\n  runs-on: ubuntu-latest\n  steps: []\n');
  assert.equal(action.status, 1);
  assert.match(action.stderr, /\.github\/actions\/local\/action\.yml: runs\.runs-on names a runner in a file that is not a workflow/u);
});
