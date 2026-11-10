// Issue #1548: what we do most wrong, that blocks the vision, and the plan.
//
// The issue asked for documentation and for issues created and organized, not
// for a change of the source. This file pins that outcome as data: the verbatim
// architect note, the documents that were corrected, the case study with every
// requirement, the snapshot of the issue graph the plan was published into, and
// the requirement rows that must not claim more than was delivered.
//
// It reads committed files only; it does not call GitHub.

import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const read = (path) => readFileSync(`${REPO_ROOT}/${path}`, 'utf8');
const issueUrl = (number) => `https://github.com/link-assistant/formal-ai/issues/${number}`;

const CASE = 'docs/case-studies/issue-1548';
const PARENT = 1561;
const CREATED = [1550, 1551, 1552, 1553, 1554, 1555, 1556, 1557, 1558, 1559, 1560, PARENT, 1562];
const REQUIREMENT_IDS = Array.from({ length: 19 }, (_, index) => `R1548-${index + 1}`);
// Rows that name a capability of the source. This issue delivers their plan,
// so their status may never open with "Implemented".
const CAPABILITY_ROWS = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 16].map((number) => `R1548-${number}`);

/** The cells of the row of `text` whose first cell is `id`. */
function row(text, id) {
  const line = text.split('\n').find((candidate) => candidate.startsWith(`| ${id} |`));
  assert.ok(line, `${id} has a row`);
  return line.split(' | ').map((cell) => cell.replace(/^\| | \|$/g, '').trim());
}

/** The body of the `## <heading>` section of a document. */
function section(text, heading) {
  const start = text.indexOf(`\n## ${heading}`);
  assert.notEqual(start, -1, `section "${heading}" exists`);
  const end = text.indexOf('\n## ', start + 1);
  return text.slice(start, end === -1 ? text.length : end);
}

// The snapshot is the answer of the GitHub GraphQL API: one record per open
// issue of this repository, taken after the last write of this issue.
const HERE = 'link-assistant/formal-ai';
const GRAPH = JSON.parse(read(`${CASE}/raw-data/github/issue-graph-after.json`)).map((issue) => ({
  number: issue.number,
  type: issue.issueType?.name ?? null,
  labels: issue.labels.nodes.map((label) => label.name),
  subIssues: issue.subIssues.nodes.map((sub) => sub.number),
  // Blockers of this repository only; a blocker in another repository cannot close a cycle here.
  blockedBy: issue.blockedBy.nodes.filter((blocker) => blocker.repository.nameWithOwner === HERE).map((blocker) => blocker.number),
}));
const ISSUES = new Map(GRAPH.map((issue) => [issue.number, issue]));

test('R1548-12: the issue is kept verbatim as an architect note and indexed', () => {
  const note = read('docs/architect-notes/2026-11-10-what-we-do-most-wrong.md');
  assert.ok(note.includes('We scope of this issue is documentation and issues planing'));
  assert.ok(note.includes('https://github.com/link-foundation/relative-meta-logic'));
  assert.ok(read('docs/architect-notes/README.md').includes('2026-11-10-what-we-do-most-wrong.md'));
});

test('R1548-12: VISION, GOALS and the guideline carry what the issue details', () => {
  const vision = section(read('VISION.md'), 'Requirements are axioms, options are scored, the plan is executed');
  for (const number of [1550, 1551, 1552, 1553, 1554, 1556]) {
    assert.ok(vision.includes(issueUrl(number)), `VISION names #${number}`);
  }
  assert.ok(vision.includes('deformalization (naturalization)'));
  const goals = read('GOALS.md');
  for (const number of [1550, 1551, 1552, 1553, 1554, 1556, 1483]) {
    assert.ok(goals.includes(issueUrl(number)), `GOALS names #${number}`);
  }
  assert.ok(existsSync(`${REPO_ROOT}/docs/guidelines/issue-planning.md`));
  assert.ok(read('CONTRIBUTING.md').includes('docs/guidelines/issue-planning.md'));
});

test('R1548-3/R1548-11: status sentences say what was measured', () => {
  const roadmap = section(read('ROADMAP.md'), '2026-11-10 Vision-Blocker Audit (issue #1548)');
  assert.ok(roadmap.includes('where a row below disagrees with an\nearlier row, the row below is current'));
  assert.ok(roadmap.includes('| The universal problem-solving loop of the vision picture produces every answer | Partial:'));
  assert.ok(roadmap.includes('23 of 130'));
  assert.ok(roadmap.includes(issueUrl(PARENT)));
  const readme = read('README.md');
  assert.ok(readme.includes(`${CASE}/README.md`));
  assert.ok(readme.includes(issueUrl(PARENT)));
  assert.ok(read('docs/meta-algorithm.md').includes('**What this page is and is not.**'));
  const doctrine = read('docs/requirements/doctrine-standing-doctrine-recursive-meta-algorithm-2026-10-06.md');
  assert.ok(row(doctrine, 'R1001')[2].includes('returned only when its status is `solved`'));
  assert.ok(row(doctrine, 'R1010')[2].includes('431 on 2026-11-10'));
});

test('R1548-11: every solver switch is documented with what it changes today', () => {
  const solver = read('docs/configuration/solver.md');
  for (const name of ['FORMAL_AI_SELECTION_MODE', 'FORMAL_AI_RECURSION_MODE', 'FORMAL_AI_LIVE_FETCH']) {
    assert.ok(solver.includes(name), name);
  }
  assert.ok(solver.includes(issueUrl(1556)));
  assert.ok(read('docs/configuration/README.md').includes('solver.md'));
});

test('R1548-18: the case study lists every requirement with a plan', () => {
  const requirements = read(`${CASE}/requirements.md`);
  const plan = read(`${CASE}/solution-plan.md`);
  for (const id of REQUIREMENT_IDS) {
    assert.ok(row(requirements, id).length >= 5, `${id} has every column`);
    assert.ok(plan.includes(id), `${id} has a plan`);
  }
  const study = read(`${CASE}/README.md`);
  for (const heading of [
    '1. Collected data',
    '3. What we do most wrong',
    '4. The three Hello World test repositories',
    '5. Failed attempts on real GitHub issues',
    '6. Orchestrator-side defects reported to Hive Mind',
    '8. Verification',
  ]) {
    section(study, heading);
  }
  for (const path of study.matchAll(/`(raw-data\/[^`*]+\.(?:md|json|tsv))`/g)) {
    assert.ok(existsSync(`${REPO_ROOT}/${CASE}/${path[1]}`), `${path[1]} exists`);
  }
});

test('R1548-13/R1548-14/R1548-15: test repositories, failed attempts and reports are recorded', () => {
  const study = read(`${CASE}/README.md`);
  const repositories = section(study, '4. The three Hello World test repositories');
  for (const client of ['--tool agent', '--tool codex', '--tool claude']) assert.ok(repositories.includes(client), client);
  const attempts = section(study, '5. Failed attempts on real GitHub issues');
  assert.ok(attempts.includes('### 5.1 link-assistant/formal-ai'));
  assert.ok(attempts.includes('### 5.2 link-assistant/hive-mind'));
  for (const number of [1483, 1557, 1558, 1560, 1562, 1189, 1158]) assert.ok(attempts.includes(issueUrl(number)), `#${number} advances a class`);
  const reports = section(study, '6. Orchestrator-side defects reported to Hive Mind');
  for (const number of [4031, 4032, 4033, 4034, 4035]) {
    assert.ok(reports.includes(`https://github.com/link-assistant/hive-mind/issues/${number}`), `hive-mind #${number}`);
  }
  assert.ok(existsSync(`${REPO_ROOT}/${CASE}/raw-data/attempts/real-issue-attempts.md`));
  assert.ok(existsSync(`${REPO_ROOT}/${CASE}/raw-data/attempts/hive-mind-attempts.md`));
  assert.ok(existsSync(`${REPO_ROOT}/${CASE}/raw-data/hive-mind/reports.md`));
});

test('R1548-17: every open issue of the snapshot has a type and the matching label', () => {
  const labels = { Bug: ['bug'], Feature: ['enhancement'], Task: ['enhancement', 'documentation'] };
  assert.equal(GRAPH.length, 109);
  for (const issue of GRAPH) {
    assert.ok(issue.type in labels, `#${issue.number} has a type`);
    assert.ok(labels[issue.type].some((label) => issue.labels.includes(label)), `#${issue.number} carries the label of its type`);
  }
});

test('R1548-1: one parent holds the blockers, and the published dependencies have no cycle', () => {
  const parent = ISSUES.get(PARENT);
  assert.ok(parent, 'the parent issue is open');
  for (const number of CREATED.filter((created) => created !== PARENT)) {
    assert.ok(parent.subIssues.includes(number), `#${number} is a sub-issue of the parent`);
  }
  assert.equal(parent.subIssues.length, 31);
  assert.equal(parent.blockedBy.length, 17);
  // Depth-first search over the open `blocked by` relations of this repository.
  const state = new Map();
  const visit = (number, path) => {
    if (state.get(number) === 'done') return;
    assert.notEqual(state.get(number), 'open', `cycle: ${[...path, number].join(' -> ')}`);
    state.set(number, 'open');
    for (const blocker of ISSUES.get(number).blockedBy) {
      if (ISSUES.has(blocker)) visit(blocker, [...path, number]);
    }
    state.set(number, 'done');
  };
  for (const number of ISSUES.keys()) visit(number, []);
});

test('R1548-19: a capability row never claims more than its plan', () => {
  const shard = read('docs/requirements/issue-1548-vision-blockers-and-plan.md');
  for (const id of REQUIREMENT_IDS) {
    const status = row(shard, id)[2];
    if (CAPABILITY_ROWS.includes(id)) {
      assert.ok(status.startsWith('Partial: '), `${id} is partial`);
      assert.match(status, /[Oo]wned by \[#\d+\]/, `${id} names the issue that owns the capability`);
    } else {
      assert.ok(status.startsWith('Implemented: '), `${id} is implemented`);
      assert.ok(status.includes('rust/tests/web/issue-1548-vision-blockers.test.mjs'), `${id} cites this test`);
    }
  }
});
