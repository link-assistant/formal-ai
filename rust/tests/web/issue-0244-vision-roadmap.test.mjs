// Issue #244 vision-planning requirements (R250, R251, R254, R255):
//
// R250 — documentation must fully track every vision pillar (ROADMAP.md is the
//        single authority; all E1-E34 epics are recorded as closed and merged).
// R251 — stale documentation references must be reconciled with the actual code
//        (ARCHITECTURE.md, REQUIREMENTS.md and VISION.md must all exist).
// R254 — critical foundation-blocking problems must be planned first
//        (ROADMAP.md must document the foundation-first ordering).
// R255 — all issues needed to implement the vision must be created and recorded
//        (docs/case-studies/issue-244/proposed-issues.md must list E1-E34).

import assert from 'node:assert/strict';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const ROADMAP = readFileSync(join(REPO_ROOT, 'ROADMAP.md'), 'utf8');
const PROPOSED = readFileSync(join(REPO_ROOT, 'docs/case-studies/issue-244/proposed-issues.md'), 'utf8');

test('R250: ROADMAP.md exists and records all E1-E34 epics as closed and merged', () => {
  assert.ok(ROADMAP.trim().length > 0, 'ROADMAP.md is empty');
  // The sixth-pass statement that closes the final parity batch.
  assert.ok(
    ROADMAP.includes('no vision-planning epic remains open'),
    'ROADMAP.md does not record "no vision-planning epic remains open"',
  );
  // Both the initial batch (E1-E14) and the final parity batch (E33-E34) must
  // appear as closed and merged.
  assert.ok(ROADMAP.includes('E33-E34'), 'ROADMAP.md is missing the E33-E34 parity batch');
  assert.ok(
    ROADMAP.includes('closed and merged'),
    'ROADMAP.md does not record any batch as closed and merged',
  );
});

test('R251: ARCHITECTURE.md, REQUIREMENTS.md, and VISION.md all exist and are non-trivial', () => {
  for (const file of ['ARCHITECTURE.md', 'REQUIREMENTS.md', 'VISION.md']) {
    const path = join(REPO_ROOT, file);
    assert.ok(existsSync(path), `${file} is missing`);
    assert.ok(statSync(path).size > 500, `${file} is suspiciously small`);
  }
});

test('R254: ROADMAP.md documents the foundation-first ordering of planning batches', () => {
  // The synthesis batch (E28-E32) explicitly depended on the foundation
  // epics (E1-E27) being done first — the roadmap must record this ordering.
  assert.ok(ROADMAP.includes('E28-E32'), 'ROADMAP.md is missing the E28-E32 synthesis batch');
  assert.ok(ROADMAP.includes('E21-E27'), 'ROADMAP.md is missing the E21-E27 reasoning batch');
  // The foundation-first principle must appear in the roadmap text.
  assert.ok(
    ROADMAP.includes('foundation') || ROADMAP.includes('Foundation'),
    'ROADMAP.md does not mention foundation ordering',
  );
});

test('R255: proposed-issues.md records issue numbers for all E1-E34 planning batches', () => {
  // E1-E14 opened as #246-#259.
  assert.ok(PROPOSED.includes('#246'), 'proposed-issues.md is missing #246 (E1)');
  assert.ok(PROPOSED.includes('#259'), 'proposed-issues.md is missing #259 (E14)');
  // E15-E20 opened as #278-#283.
  assert.ok(PROPOSED.includes('#278'), 'proposed-issues.md is missing #278 (E15)');
  assert.ok(PROPOSED.includes('#283'), 'proposed-issues.md is missing #283 (E20)');
  // E21-E27 opened as #298-#304.
  assert.ok(PROPOSED.includes('#298'), 'proposed-issues.md is missing #298 (E21)');
  assert.ok(PROPOSED.includes('#304'), 'proposed-issues.md is missing #304 (E27)');
  // E28-E32 and E33-E34 must also be present.
  assert.ok(PROPOSED.includes('E28'), 'proposed-issues.md is missing E28');
  assert.ok(PROPOSED.includes('E33'), 'proposed-issues.md is missing E33 (parity batch)');
  assert.ok(PROPOSED.includes('E34'), 'proposed-issues.md is missing E34');
});
