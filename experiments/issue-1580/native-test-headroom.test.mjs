// Run the maintained scanner against the native regression module owners.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
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
