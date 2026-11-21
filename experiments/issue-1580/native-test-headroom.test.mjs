// Run the maintained scanner against the native regression module owners.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
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
