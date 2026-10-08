// Issue #1180 R3, R10 and R11 in the JavaScript root: the github-logs
// importer twin (js/agentic/crate/history_github.mjs) formalizes a body into
// requirement statements and the capture into dated lifecycle transitions,
// and the lineage route twin (js/agentic/crate/history_lineage.mjs) claims
// and answers exactly as rust/tests/unit/memory/issue_1180_history_context.rs pins
// for the native route.

import assert from 'node:assert/strict';
import { before, test } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

let history;
let github;
let lineage;
let rules;

before(async () => {
  await installNodeHost(new WorkerHost());
  history = await import('../../../js/agentic/crate/history_context.mjs');
  github = await import('../../../js/agentic/crate/history_github.mjs');
  lineage = await import('../../../js/agentic/crate/history_lineage.mjs');
  rules = history.historyRules();
});

test('issue bodies formalize into requirement statements and dated transitions', () => {
  const pull = {
    number: 1188,
    title: 'bulk fixes',
    body: 'This cites R1180-3 in prose.\n\n- **R1** A prompt routes by its formalization.\nR2. Every answer carries a derivation id.\n* R3: Retries are bounded',
    state: 'MERGED',
    labels: [],
    createdAt: '2026-09-30T00:00:00Z',
    closedAt: '2026-10-07T10:00:00Z',
    mergedAt: '2026-10-07T10:00:00Z',
    updatedAt: '2026-10-07T10:00:00Z',
  };
  const events = github.importIssuesAndPulls([{ name: 'pr-1188.json', text: JSON.stringify(pull) }], null, rules);
  assert.equal(events.length, 1);
  assert.equal(events[0].id, 'pull:1188');
  assert.equal(events[0].intent, 'MERGED');
  assert.deepEqual(events[0].evidence, [
    'requirement:R1 A prompt routes by its formalization.',
    'requirement:R2 Every answer carries a derivation id.',
    'requirement:R3 Retries are bounded',
    'state:opened@2026-09-30T00:00:00Z',
    'state:closed@2026-10-07T10:00:00Z',
    'state:merged@2026-10-07T10:00:00Z',
  ]);
});

test('comments, reviews and CI runs import as the Rust importer maps them', () => {
  const files = [
    { name: 'issue-1014.json', text: JSON.stringify({ number: 1014, title: 'release gate keeps failing', body: '', state: 'closed', labels: [{ name: 'bug' }], createdAt: '2026-09-07T00:00:00Z', updatedAt: '2026-09-07T00:00:00Z' }) },
    { name: 'pr-1015-reviews.json', text: JSON.stringify([{ id: 777, user: { login: 'konard' }, state: 'APPROVED', body: '', submittedAt: '2026-09-08T01:00:00Z' }]) },
  ];
  const events = github.importIssuesAndPulls(files, null, rules);
  assert.deepEqual(events.map((event) => [event.id, event.kind, event.intent, event.content, event.conversationId, event.evidence]), [
    ['issue:1014', 'issue', 'closed', 'release gate keeps failing', 'issue-1014', ['label:bug', 'state:opened@2026-09-07T00:00:00Z']],
    ['review:pr-1015-review_decision-777', 'review', 'review_decision', 'konard: APPROVED', 'pr-1015', ['pr:1015']],
  ]);
  const runs = github.importCiRuns([{ name: 'run-42.json', text: JSON.stringify({ databaseId: 42, workflowName: 'CI', conclusion: 'failure', event: 'push', headSha: 'abc', createdAt: '2026-09-08T00:00:00Z', jobs: [{ name: 'test', conclusion: 'failure', steps: [{ name: 'cargo test', conclusion: 'failure' }] }] }) }], null, rules);
  assert.deepEqual(runs.map((run) => [run.id, run.content, run.evidence]), [['ci_run:42', 'CI: failure\ntest/cargo test', ['commit:abc']]]);
});

test('lineage subjects need a cue and a path', () => {
  assert.deepEqual(lineage.lineageSubjects('Which issue introduced `scripts/check-self-development-release.rs`?', rules), ['scripts/check-self-development-release.rs']);
  assert.deepEqual(lineage.lineageSubjects('scripts/gate.rs的历史是什么？', rules), ['scripts/gate.rs']);
  assert.deepEqual(lineage.lineageSubjects('Какая задача добавила rust/src/solver.rs?', rules), ['rust/src/solver.rs']);
  assert.deepEqual(lineage.lineageSubjects('Explain scripts/gate.rs', rules), []);
  assert.deepEqual(lineage.lineageSubjects('Which issue asked for faster builds?', rules), []);
  assert.deepEqual(lineage.lineageSubjects('What is the history of /etc/passwd?', rules), []);
});

test('the lineage answer names the commits, the issue and an unrecorded pull request', () => {
  const RS = '\u001e';
  const US = '\u001f';
  const record = (sha, date, subject, body) => [RS, sha, US, 'Ada', US, date, US, subject, US, body, US, '\nscripts/gate.rs\n'].join('');
  const raws = history.parseLogOutput(
    record('2222222222b', '2026-10-02T00:00:00Z', 'tighten the gate', '')
      + record('1111111111a', '2026-10-01T00:00:00Z', 'add the gate', 'Refs #1014'),
  );
  const events = raws.map((raw) => history.formalizeCommit(raw, [], null, rules));
  assert.equal(
    lineage.lineageAnswer('en', 'scripts/gate.rs', events, rules),
    "scripts/gate.rs has 2 commits in this repository's history, oldest first:\n"
      + '- 111111111 2026-10-01 add the gate\n'
      + '- 222222222 2026-10-02 tighten the gate\n\n'
      + 'The introducing commit 111111111 was made for issue #1014 and delivered by pull request '
      + '(not recorded in the history). The lineage is read from `git log --follow`; the issue comes '
      + "from the commit's trailer and the pull request from the merge that delivered it.",
  );
  const merged = raws.map((raw, index) => history.formalizeCommit(raw, [], index === 0 ? 'Merge pull request #1015 from x/y' : null, rules));
  assert.match(lineage.lineageAnswer('en', 'scripts/gate.rs', merged, rules), /issue #1014 and delivered by pull request #1015\./u);
  assert.equal(lineage.lineageAnswer('en', 'scripts/gate.rs', [], rules), null);
});
