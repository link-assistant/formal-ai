import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, chmodSync, rmSync, readFileSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { executionBatches, executeBatches } from '../../../scripts/lib/coverage-execution.mjs';

function fakeHarness() {
  const directory = mkdtempSync(join(tmpdir(), 'coverage-queue-contract-'));
  const path = join(directory, 'harness.mjs');
  writeFileSync(path, `#!/usr/bin/env node
const args = process.argv.slice(2);
const names = args.slice(args.indexOf('--test-threads') + 2).sort();
const mode = process.env.COVERAGE_FAKE_MODE ?? '';
let passed = 0, failed = 0, ignored = 0;
for (const name of names) {
  if (mode === 'omit' && name.endsWith('omit')) continue;
  if (mode === 'silent-failure' && name.endsWith('fail')) continue;
  const status = name.endsWith('fail') ? 'FAILED' : name.endsWith('ignored') ? 'ignored' : 'ok';
  if (status === 'FAILED') failed++; else if (status === 'ignored') ignored++; else passed++;
  console.log('test ' + name + ' ... ' + status);
}
if (mode === 'duplicate') console.log('test ' + names[0] + ' ... ok');
if (mode === 'extra') console.log('test unselected ... ok');
console.log('test result: ' + (failed ? 'FAILED' : 'ok') + '. ' + passed + ' passed; ' + failed + ' failed; ' + ignored + ' ignored; 0 measured; 0 filtered out;');
process.exit(mode === 'false-success' ? 0 : failed || mode === 'silent-failure' ? 1 : 0);
`);
  chmodSync(path, 0o755);
  return { path, close: () => rmSync(directory, { recursive: true, force: true }) };
}
const item = (name) => `unit\t${name}`;
const planned = (names, seconds) => {
  const items = names.map(item);
  return executionBatches(items, items, new Map([['unit', 'fixture']]), (entry) => seconds[entry.split('\t')[1]] ?? 1);
};

test('actual process order starts weighted late-lexical case before early-lexical cases', async () => {
  const fixture = fakeHarness();
  try {
    const batches = planned(['a-fast', 'b-fast', 'z-slow'], { 'z-slow': 300 });
    assert.deepEqual(batches.map((batch) => batch.names), [['z-slow'], ['a-fast', 'b-fast']]);
    const launched = [];
    const result = await executeBatches(batches, new Map([['unit', fixture.path]]), {
      concurrency: 1, onStart: (batch) => launched.push(batch.names),
    });
    assert.deepEqual(launched, [['z-slow'], ['a-fast', 'b-fast']]);
    assert.equal(result.succeeded, true);
    assert.equal(result.completed, 3);
  } finally { fixture.close(); }
});

test('bounded pool never exceeds two real processes and still completes remaining batches after failure', async () => {
  const fixture = fakeHarness();
  try {
    const batches = planned(['first-fail', 'later-good', 'last-good'], {
      'first-fail': 30, 'later-good': 20, 'last-good': 10,
    });
    let active = 0, peak = 0;
    const finished = [];
    const result = await executeBatches(batches, new Map([['unit', fixture.path]]), {
      onStart: () => { active++; peak = Math.max(peak, active); },
      onFinish: (batch) => { active--; finished.push(batch.index); },
    });
    assert.equal(peak, 2);
    assert.equal(active, 0);
    assert.deepEqual(finished.sort(), [0, 1, 2]);
    assert.equal(result.succeeded, false);
    assert.equal(result.completed, 3);
    assert.equal(result.batches[0].exitCode, 1);
    assert.ok(result.batches.every((batch) => batch.elapsedSeconds >= 0));
  } finally { fixture.close(); }
});

test('successful exit cannot hide an omitted, repeated, or unselected test', async () => {
  const fixture = fakeHarness();
  try {
    for (const mode of ['omit', 'duplicate', 'extra']) {
      const result = await executeBatches(planned(['real-good', 'real-omit'], {}), new Map([['unit', fixture.path]]), {
        env: { ...process.env, COVERAGE_FAKE_MODE: mode },
      });
      assert.equal(result.batches[0].exitCode, 0);
      assert.equal(result.succeeded, false, mode);
      assert.ok(result.batches[0].problems.length > 0, mode);
    }
  } finally { fixture.close(); }
});

test('actual ignored-case completion remains distinct from passed cases', async () => {
  const fixture = fakeHarness();
  try {
    const result = await executeBatches(planned(['real-good', 'real-ignored'], {}), new Map([['unit', fixture.path]]));
    assert.equal(result.succeeded, true);
    assert.equal(result.selected, 2);
    assert.deepEqual(result.batches[0].summary, { passed: 1, failed: 0, ignored: 1 });
  } finally { fixture.close(); }
});

test('nonzero harness status remains failure even when printed cases look successful', async () => {
  const fixture = fakeHarness();
  try {
    const result = await executeBatches(planned(['real-good'], {}), new Map([['unit', fixture.path]]), {
      env: { ...process.env, COVERAGE_FAKE_MODE: 'silent-failure' },
    });
    assert.equal(result.completed, 1);
    assert.equal(result.batches[0].problems.length, 0);
    assert.equal(result.batches[0].exitCode, 1);
    assert.equal(result.succeeded, false);
  } finally { fixture.close(); }
});

test('invalid runtime selections and missing executable are refused before any execution', () => {
  const good = item('good');
  const executables = new Map([['unit', 'fixture']]);
  assert.throws(() => executionBatches([good, good], [good], executables, () => 1), /duplicate selected/);
  assert.throws(() => executionBatches([item('absent')], [good], executables, () => 1), /not listed/);
  assert.throws(() => executionBatches([good], [good, good], executables, () => 1), /duplicate runtime/);
  assert.throws(() => executionBatches([good], [good], new Map(), () => 1), /missing executable/);
});

test('equal weights share only a target and batches retain fixed size bounds', () => {
  const items = ['unit\ta', 'unit\tb', 'unit\tc', 'integration\td'];
  const batches = executionBatches(items, items, new Map([['unit', 'u'], ['integration', 'i']]), () => 1, 2);
  assert.ok(batches.every((batch) => batch.names.length <= 2));
  assert.deepEqual(batches.flatMap((batch) => batch.names.map((name) => `${batch.target}\t${name}`)).sort(), items.sort());
});


test('printed FAILED completion cannot be certified by an exit-zero harness', async () => {
  const fixture = fakeHarness();
  try {
    const result = await executeBatches(planned(['real-fail'], {}), new Map([['unit', fixture.path]]), {
      env: { ...process.env, COVERAGE_FAKE_MODE: 'false-success' },
    });
    assert.equal(result.completed, 1);
    assert.equal(result.batches[0].exitCode, 0);
    assert.equal(result.batches[0].problems.length, 0);
    assert.deepEqual(result.batches[0].summary, { passed: 0, failed: 1, ignored: 0 });
    assert.equal(result.succeeded, false);
  } finally { fixture.close(); }
});

test('actual shard shell retains missing-executable failure after an existing target succeeds', () => {
  const directory = mkdtempSync(join(tmpdir(), 'coverage-shell-contract-'));
  try {
    mkdirSync(join(directory, 'rust'));
    mkdirSync(join(directory, 'bin'));
    const executable = join(directory, 'existing-harness');
    writeFileSync(executable, `#!/usr/bin/env bash
if [[ " $* " == *" --list "* ]]; then
  printf 'unit_good: test\n'
else
  printf 'test unit_good ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n'
fi
`);
    chmodSync(executable, 0o755);
    const nodeShim = join(directory, 'bin', 'node');
    writeFileSync(nodeShim, `#!/usr/bin/env bash
case "$1" in
  scripts/plan-test-shards.mjs)
    if [[ " $* " == *" --check "* ]]; then
      printf '1 test(s), exact partition\n'
    else
      printf 'unit\tunit_good\n'
    fi
    ;;
  scripts/run-coverage-plan.mjs)
    ./existing-harness --exact unit_good
    ;;
  *) exit 72 ;;
esac
`);
    chmodSync(nodeShim, 0o755);
    const original = readFileSync(resolve(process.cwd(), 'scripts/run-coverage-shard.sh'), 'utf8');
    const shell = join(directory, 'run.sh');
    writeFileSync(shell, original);
    const manifest = join(directory, 'manifest.tsv');
    const env = { ...process.env, PATH: `${join(directory, 'bin')}:${process.env.PATH}`,
      SHARD_INDEX: '1', SHARD_TOTAL: '1', LLVM_PROFILE_FILE: '/tmp/unused-fake-profile-%p' };
    writeFileSync(manifest, 'unit\texisting-harness\n');
    const valid = spawnSync('bash', [shell, manifest], { cwd: directory, env, encoding: 'utf8' });
    assert.equal(valid.status, 0, valid.stderr);
    assert.match(valid.stdout, /test unit_good \.\.\. ok/);
    writeFileSync(manifest, 'unit\texisting-harness\nmissing\tmissing-harness\n');
    const mixed = spawnSync('bash', [shell, manifest], { cwd: directory, env, encoding: 'utf8' });
    assert.equal(mixed.status, 1);
    assert.match(mixed.stderr, /missing: missing-harness is not an executable/);
    assert.match(mixed.stdout, /test unit_good \.\.\. ok/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
