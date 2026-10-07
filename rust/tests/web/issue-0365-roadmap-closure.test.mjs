// Issue #349 roadmap (R265): the child issues (#355-#364) and their final
// status must remain auditable from repository documentation. The primary
// evidence is docs/case-studies/issue-365/README.md (the epic closure report)
// and the Issue #349 section in ROADMAP.md.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const CLOSURE = readFileSync(join(REPO_ROOT, 'docs/case-studies/issue-365/README.md'), 'utf8');
const ROADMAP = readFileSync(join(REPO_ROOT, 'ROADMAP.md'), 'utf8');

test('R265: docs/case-studies/issue-365/README.md lists all child issues #355-#364 with their closing PRs', () => {
  assert.ok(CLOSURE.trim().length > 0, 'issue-365/README.md is empty');
  // Every child issue from the roadmap must appear in the closure report.
  for (let issue = 355; issue <= 364; issue++) {
    assert.ok(CLOSURE.includes(`#${issue}`), `closure report is missing issue #${issue}`);
  }
  // The ten closing PRs must also be referenced (PR #368 is not used; the
  // sequence is #366, #367, #369-#376).
  for (const pr of [366, 367, 369, 370, 371, 372, 373, 374, 375, 376]) {
    assert.ok(CLOSURE.includes(`#${pr}`), `closure report is missing PR #${pr}`);
  }
});

test('R265: ROADMAP.md has an Issue #349 section recording the roadmap as closed and merged', () => {
  assert.ok(
    ROADMAP.includes('Issue #349') || ROADMAP.includes('#349'),
    'ROADMAP.md does not reference the #349 reverse-sort roadmap',
  );
  assert.ok(
    ROADMAP.includes('docs/case-studies/issue-365'),
    'ROADMAP.md does not link to the issue-365 closure report',
  );
});
