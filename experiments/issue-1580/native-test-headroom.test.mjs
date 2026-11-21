// Run the maintained scanner against the native regression module owners.
import assert from 'node:assert/strict';
import {existsSync, readFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import YAML from 'yaml';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url)).replace(/\/$/u, '');
for (const [owner, child] of [
  ['rust/tests/unit/agentic-coding/agentic_general_planner', 'command_capture'],
  ['rust/tests/unit/ci-cd/job_budget_fit', 'standalone_suites'],
  ['rust/tests/unit/ci-cd/pinned_tool_images', 'image_policy'],
]) {
  test(owner + ' retains file-size headroom and a reachable regression group', () => {
    const files = new Set([owner + '.rs', owner + '/' + child + '.rs']);
    const result = checkListedFiles(root, files);
    assert.deepEqual(result, {warnings: [], violations: [], embeddedDataViolations: []});
    assert.match(readFileSync(root + '/' + owner + '.rs', 'utf8'), new RegExp('^mod ' + child + ';$', 'mu'));
    assert.match(readFileSync(root + '/' + owner + '/' + child + '.rs', 'utf8'), /^#\[test\]$/mu);
  });
}

test('the hosted native targets execute every extracted regression owner', () => {
  for (const [target, owners] of [
    ['coding_contracts', ['agentic-coding/agentic_general_planner']],
    ['workflow_contracts', ['ci-cd/job_budget_fit', 'ci-cd/pinned_tool_images']],
  ]) {
    const source = readFileSync(root + '/rust/tests/unit/' + target + '.rs', 'utf8');
    for (const owner of owners) {
      assert(source.includes('#[path = "' + owner + '.rs"]'), target + ' omits ' + owner);
    }
    assert(readFileSync(root + '/scripts/ci-native-regressions.sh', 'utf8').includes('--test ' + target));
  }
});

test('both core checkout attempts retain the original upstream-report fixtures', () => {
  const workflow = YAML.parse(readFileSync(root + '/.github/workflows/native-type-check.yml', 'utf8'));
  const attempts = workflow.jobs.clippy.steps.filter(step => step.uses === 'actions/checkout@v7');
  assert.equal(attempts.length, 2);
  for (const step of attempts) {
    assert(step.with['sparse-checkout'].split('\n').includes('/dev/log/issues/1079/pulls/1080/upstream-reports/'));
  }
});

for (const [owner, child] of [
  ['rust/tests/unit/ci-cd/workflow_release', 'workflow_policy'],
  ['rust/tests/unit/data_files', 'text_rows'],
  ['rust/tests/unit/specification/benchmarks', 'evaluation'],
]) {
  test(owner + ' keeps native policy module headroom', () => {
    const result = checkListedFiles(root, new Set([owner + '.rs', owner + '/' + child + '.rs']));
    assert.deepEqual(result, {warnings: [], violations: [], embeddedDataViolations: []});
    assert.match(readFileSync(root + '/' + owner + '.rs', 'utf8'), new RegExp('^mod ' + child + ';$', 'mu'));
    assert.match(readFileSync(root + '/' + owner + '/' + child + '.rs', 'utf8'), /^(?:pub\(super\) )?fn /mu);
  });
}

test('the standalone data-files Cargo target resolves its extracted helper', () => {
  const cargo = readFileSync(root + '/rust/Cargo.toml', 'utf8');
  const target = cargo.split('[[test]]').find(block => /^name = "data_files"$/mu.test(block));
  assert(target, 'the data-files target must remain registered');
  const cratePath = /^path = "([^"]+)"$/mu.exec(target)?.[1];
  assert(cratePath, 'the target must identify its real Rust crate root');
  const crateFile = resolve(root, 'rust', cratePath);
  const declaration = /(?:#\[path = "([^"]+)"\]\n)?mod text_rows;/u.exec(readFileSync(crateFile, 'utf8'));
  assert(declaration, 'the crate root must retain its text-row helpers');
  const child = resolve(dirname(crateFile), declaration[1] ?? 'text_rows.rs');
  assert(existsSync(child), 'the crate-root module declaration resolves to a missing file: ' + child);
});
