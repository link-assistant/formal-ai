// Issue #914: the coding-first vision plan, pinned as data.
//
// Issue #914 asked for a plan, not for a feature: sync the documentation, then
// open every issue the vision needs, coding first, foundation blockers first.
// rust/tests/unit/docs_requirements/issue_914.rs holds the case study and the
// opened-issue list. This file holds the plan's *structure* to the rows that
// rest on it: the binding design rules, the foundation epic, and an "Existing
// components" inventory in every epic, so generalization never drops what is
// already supported.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const PLAN = readFileSync(`${REPO_ROOT}/docs/case-studies/issue-914/proposed-issues.md`, 'utf8');
const EPICS = ['E69', 'E70', 'E71', 'E72', 'E73', 'E74', 'E75', 'E76', 'E77'];

/** The body of one `## E<n>` section. */
function epic(name) {
  const start = PLAN.indexOf(`\n## ${name} `);
  assert.notEqual(start, -1, `${name} has a section`);
  const end = PLAN.indexOf('\n## ', start + 1);
  return PLAN.slice(start, end === -1 ? PLAN.length : end);
}

/** The numbered design rules that bind every epic, by number. */
function designRules() {
  const start = PLAN.indexOf('## Design rules that bind every epic');
  assert.notEqual(start, -1);
  const section = PLAN.slice(start, PLAN.indexOf('\n## E69', start));
  return Object.fromEntries([...section.matchAll(/^(\d+)\. \*\*(.+?)\*\*/gm)].map((match) => [match[1], match[2]]));
}

test('R914-4/R914-12: every epic of the batch was opened, numbered from #916', () => {
  EPICS.forEach((name, index) => {
    assert.ok(PLAN.includes(`- ${name}: <https://github.com/link-assistant/formal-ai/issues/${916 + index}>`), name);
  });
});

test('R914-14: the foundation blocker comes first and binds every capability epic', () => {
  assert.ok(epic('E69').includes('FOUNDATION, BLOCKER'));
  assert.equal(designRules()['1'], 'Foundation first.');
  assert.ok(PLAN.includes('E69 is\nthe foundation blocker (R914-14'));
});

test('R914-13: the regression floor binds every epic, and every epic inventories what it generalizes', () => {
  assert.equal(designRules()['2'], 'Keep the regression floor.');
  for (const name of EPICS) {
    assert.ok(epic(name).includes('**Existing components.**'), `${name} names the existing components it builds on`);
  }
});

test('R914-7: the plan binds the no-neural-networks rule to every epic', () => {
  assert.equal(designRules()['3'], 'No neural networks in reasoning.');
  assert.ok(epic('E76').includes('No neural inference enters the dependency tree'));
});
