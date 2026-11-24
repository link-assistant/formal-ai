// Issue #1486: the maintained mapping from changed components and captured
// failure families to the focused offline tests that run before any Rust
// compilation, Docker image or Hive solve.
import assert from 'node:assert/strict';
import test from 'node:test';
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { delimiter, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { selectFocused, validateFocusedTestMap } from '../../../scripts/select-focused-tests.mjs';

const map = JSON.parse(readFileSync(new URL('../../../data/benchmarks/focused-test-map.json', import.meta.url), 'utf8'));

test('the focused test map names only existing tests, checks and families', () => {
  assert.deepEqual(validateFocusedTestMap(map), []);
});

test('a broken map fails closed', () => {
  const broken = structuredClone(map);
  broken.families[0].tests.push('rust/tests/web/does-not-exist.test.mjs');
  broken.components.push({ prefixes: ['x/'], families: ['no-such-family'] });
  broken.families[1].reasons.push(broken.families[0].reasons[0]);
  const problems = validateFocusedTestMap(broken);
  assert.ok(problems.some(problem => problem.includes('missing test')), problems.join('\n'));
  assert.ok(problems.some(problem => problem.includes('unknown family')), problems.join('\n'));
  assert.ok(problems.some(problem => problem.startsWith('reason ')), problems.join('\n'));
});

test('a planner change selects the captured replay before any expensive work', () => {
  const selection = selectFocused(map, { changed: ['js/agentic/work_item_evidence.mjs'] });
  assert.ok(selection.tests.includes('rust/tests/web/draft-request-replay.test.mjs'));
  assert.ok(selection.tests.includes('rust/tests/web/work-item-report-outcome.test.mjs'));
  assert.deepEqual(selection.checks, []);
});

test('a captured reason selects its family and an unmapped reason is an error', () => {
  assert.deepEqual(selectFocused(map, { reasons: ['ReadPolicyRefused'] }).families, ['work-item-read-policy']);
  assert.throws(() => selectFocused(map, { reasons: ['NeverSeen'] }), /UnmappedFailureReason/);
  assert.throws(() => selectFocused(map, { families: ['no-such-family'] }), /UnknownFailureFamily/);
});

test('workflow changes select the adapter, custody, transport and policy checks', () => {
  const draft = selectFocused(map, { changed: ['.github/workflows/hive-mind-draft.yml'] });
  assert.deepEqual(draft.families, ['hive-draft-adapter', 'shared-build-custody', 'ci-phase-timing', 'workflow-policy']);
  assert.deepEqual(draft.checks, [['node', 'scripts/check-ci-speed.mjs']]);
  const delivery = selectFocused(map, { changed: ['.github/workflows/hive-original-delivery.yml'] });
  assert.ok(delivery.families.includes('hive-image-transport'));
  assert.ok(delivery.tests.includes('rust/tests/web/hive-image-delta.test.mjs'));
  assert.deepEqual(selectFocused(map, { changed: ['README.md'] }).tests, []);
});

// Issue #1580: changed evidence paths exceeded execFileSync's default 1 MiB.
for (const option of ['--base', '--since']) {
  test('the CLI selects from the complete large Git diff with ' + option, {timeout: 15000, skip: process.platform === 'win32' ? 'The Git CLI fixture uses a POSIX executable.' : false}, t => {
    const directory = mkdtempSync(join(tmpdir(), 'formal-focused-diff-'));
    t.after(() => rmSync(directory, {recursive: true, force: true}));
    const git = join(directory, 'git');
    writeFileSync(git, '#!' + process.execPath + '\n' + `
      const paths = Array.from({length: 12000}, (_, index) => 'experiments/' + 'x'.repeat(96) + index);
      paths.push('irrelevant\\n.github/workflows/hive-mind-draft.yml');
      paths.push('js/agentic/work_item_evidence.mjs');
      process.stdout.write(paths.join(process.argv.includes('-z') ? '\\0' : '\\n'));
    `);
    chmodSync(git, 0o755);
    const output = execFileSync(process.execPath, [
      fileURLToPath(new URL('../../../scripts/select-focused-tests.mjs', import.meta.url)),
      option, 'captured-base', '--json',
    ], {encoding: 'utf8', env: {...process.env, PATH: directory + delimiter + process.env.PATH}, timeout: 10000, stdio: ['ignore', 'pipe', 'pipe']});
    assert.deepEqual(JSON.parse(output), selectFocused(map, {changed: ['js/agentic/work_item_evidence.mjs']}));
  });
}

test('a failed Git diff cannot select tests from its partial output', {timeout: 15000, skip: process.platform === 'win32' ? 'The Git CLI fixture uses a POSIX executable.' : false}, t => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-focused-diff-failure-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const git = join(directory, 'git');
  writeFileSync(git, '#!' + process.execPath + '\n' + `
    process.stdout.write('js/agentic/work_item_evidence.mjs\\0');
    process.exitCode = 7;
  `);
  chmodSync(git, 0o755);
  assert.throws(() => execFileSync(process.execPath, [
    fileURLToPath(new URL('../../../scripts/select-focused-tests.mjs', import.meta.url)),
    '--since', 'missing-base', '--json',
  ], {encoding: 'utf8', env: {...process.env, PATH: directory + delimiter + process.env.PATH}, timeout: 10000, stdio: ['ignore', 'pipe', 'pipe']}),
  error => error.status !== 0 && error.stdout === '' && /Command failed: git diff/u.test(error.stderr));
});
